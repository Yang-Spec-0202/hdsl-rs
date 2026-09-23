use crate::paths::replace_file;
use anyhow::{Context, Result, anyhow, bail};
use serde_yaml::{Mapping, Value};
use std::path::Path;

fn read_document(home: &Path) -> Result<Mapping> {
    let file = home.join(".credentials.yaml");
    if !file.exists() {
        let mut root = Mapping::new();
        root.insert(Value::String("version".into()), Value::Number(1.into()));
        root.insert(Value::String("refs".into()), Value::Mapping(Mapping::new()));
        return Ok(root);
    }
    let root: Value = serde_yaml::from_slice(&std::fs::read(file)?)?;
    let map = root.as_mapping().context("凭据文件格式不受支持")?.clone();
    let version = map
        .get(Value::String("version".into()))
        .and_then(Value::as_i64);
    if version != Some(1) {
        bail!("凭据文件版本不受支持，请在 Harness 原生设置中管理");
    }
    Ok(map)
}

pub fn is_configured(home: &Path, key: &str) -> Result<bool> {
    let root = read_document(home)?;
    Ok(root
        .get(Value::String("refs".into()))
        .and_then(Value::as_mapping)
        .and_then(|refs| refs.get(Value::String(key.into())))
        .and_then(Value::as_str)
        .is_some_and(|v| !v.is_empty()))
}

pub fn redact_log_line(home: &Path, line: &str) -> String {
    let mut safe = line.to_owned();
    if let Ok(root) = read_document(home)
        && let Some(refs) = root
            .get(Value::String("refs".into()))
            .and_then(Value::as_mapping)
    {
        for secret in refs
            .values()
            .filter_map(Value::as_str)
            .filter(|s| !s.is_empty())
        {
            safe = safe.replace(secret, "[已隐藏密钥]");
        }
    }
    let token = regex::Regex::new(r"(?i)([?&]token=)[^\s&]+").expect("token regex");
    safe = token.replace_all(&safe, "${1}[已隐藏]").to_string();
    let api = regex::Regex::new(r"\bsk-[A-Za-z0-9_-]{8,}\b").expect("api key regex");
    api.replace_all(&safe, "[已隐藏密钥]").to_string()
}

pub fn set_key(home: &Path, key: &str, secret: Option<&str>) -> Result<()> {
    let bytes = key_document(home, key, secret)?;
    std::fs::create_dir_all(home)?;
    let file = home.join(".credentials.yaml");
    replace_file(&file, &bytes)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

fn key_document(home: &Path, key: &str, secret: Option<&str>) -> Result<Vec<u8>> {
    if !valid_ref(key) {
        bail!("无效的 API 凭据名称");
    }
    if secret.is_some_and(|s| s.contains('\n') || s.contains('\r')) {
        bail!("API Key 不能包含换行");
    }
    let mut root = read_document(home)?;
    let refs_key = Value::String("refs".into());
    let refs = root
        .entry(refs_key)
        .or_insert_with(|| Value::Mapping(Mapping::new()));
    let refs = refs.as_mapping_mut().context("凭据 refs 格式不受支持")?;
    if let Some(value) = secret.filter(|v| !v.is_empty()) {
        refs.insert(Value::String(key.into()), Value::String(value.into()));
    } else {
        refs.remove(Value::String(key.into()));
    }
    Ok(serde_yaml::to_string(&root)?.into_bytes())
}

fn valid_ref(name: &str) -> bool {
    let mut bytes = name.bytes();
    matches!(bytes.next(), Some(b'A'..=b'Z' | b'_'))
        && bytes.all(|b| matches!(b, b'A'..=b'Z' | b'0'..=b'9' | b'_'))
}

pub fn set_deepseek_settings(
    home: &Path,
    base_url: Option<&str>,
    model: Option<&str>,
) -> Result<()> {
    let bytes = settings_document(home, base_url, model)?;
    std::fs::create_dir_all(home)?;
    replace_file(&home.join("settings.yaml"), &bytes)
}

fn settings_document(home: &Path, base_url: Option<&str>, model: Option<&str>) -> Result<Vec<u8>> {
    let path = home.join("settings.yaml");
    let mut root = if path.exists() {
        serde_yaml::from_slice::<Value>(&std::fs::read(&path)?)?
            .as_mapping()
            .context("settings.yaml 格式不受支持")?
            .clone()
    } else {
        Mapping::new()
    };
    if let Some(url) = base_url.filter(|v| !v.trim().is_empty()) {
        let parsed = url::Url::parse(url).map_err(|_| anyhow!("API 端点不是有效 URL"))?;
        if !matches!(parsed.scheme(), "https" | "http")
            || parsed.host().is_none()
            || !parsed.username().is_empty()
            || parsed.password().is_some()
            || parsed.query().is_some()
            || parsed.fragment().is_some()
        {
            bail!("API 端点须为无凭据、查询和片段的 HTTP(S) URL");
        }
        let key = Value::String("llm-deepseek".into());
        let section = root
            .entry(key)
            .or_insert_with(|| Value::Mapping(Mapping::new()));
        section
            .as_mapping_mut()
            .context("llm-deepseek 设置格式不受支持")?
            .insert(Value::String("baseURL".into()), Value::String(url.into()));
    }
    if let Some(model) = model.filter(|v| !v.trim().is_empty()) {
        if model.len() > 256 || model.chars().any(char::is_control) {
            bail!("默认模型名称无效");
        }
        let key = Value::String("agent-default-model".into());
        let section = root
            .entry(key)
            .or_insert_with(|| Value::Mapping(Mapping::new()));
        let map = section
            .as_mapping_mut()
            .context("默认模型设置格式不受支持")?;
        map.insert(
            Value::String("provider".into()),
            Value::String("deepseek-official".into()),
        );
        map.insert(Value::String("model".into()), Value::String(model.into()));
    }
    Ok(serde_yaml::to_string(&root)?.into_bytes())
}

pub fn save_deepseek_api(
    home: &Path,
    key: Option<&str>,
    base_url: Option<&str>,
    model: Option<&str>,
) -> Result<()> {
    let settings = if base_url.is_some() || model.is_some() {
        Some(settings_document(home, base_url, model)?)
    } else {
        None
    };
    let credentials = key
        .map(|value| key_document(home, "DEEPSEEK_API_KEY", Some(value)))
        .transpose()?;
    if settings.is_none() && credentials.is_none() {
        return Ok(());
    }
    let settings_path = home.join("settings.yaml");
    let credentials_path = home.join(".credentials.yaml");
    let previous_settings = settings
        .as_ref()
        .map(|_| read_optional(&settings_path))
        .transpose()?;
    let previous_credentials = credentials
        .as_ref()
        .map(|_| read_optional(&credentials_path))
        .transpose()?;
    std::fs::create_dir_all(home)?;
    if let Some(bytes) = &settings {
        replace_file(&settings_path, bytes)?;
    }
    if let Some(bytes) = &credentials {
        let write = (|| -> Result<()> {
            replace_file(&credentials_path, bytes)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(
                    &credentials_path,
                    std::fs::Permissions::from_mode(0o600),
                )?;
            }
            Ok(())
        })();
        if let Err(problem) = write {
            let mut rollback_errors = Vec::new();
            if let Some(previous) = previous_credentials
                && let Err(error) = restore_file(&credentials_path, previous)
            {
                rollback_errors.push(error.to_string());
            }
            if let Some(previous) = previous_settings
                && let Err(error) = restore_file(&settings_path, previous)
            {
                rollback_errors.push(error.to_string());
            }
            if !rollback_errors.is_empty() {
                bail!(
                    "API 设置保存失败（{problem:#}），恢复旧设置也失败（{}）",
                    rollback_errors.join("；")
                );
            }
            return Err(problem);
        }
    }
    Ok(())
}

fn read_optional(path: &Path) -> Result<Option<Vec<u8>>> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn restore_file(path: &Path, previous: Option<Vec<u8>>) -> Result<()> {
    match previous {
        Some(bytes) => replace_file(path, &bytes),
        None => std::fs::remove_file(path).map_err(Into::into),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn key_is_write_only_and_removable() {
        let home = tempfile::tempdir().unwrap();
        set_key(home.path(), "DEEPSEEK_API_KEY", Some("sk-test")).unwrap();
        assert!(is_configured(home.path(), "DEEPSEEK_API_KEY").unwrap());
        set_key(home.path(), "DEEPSEEK_API_KEY", None).unwrap();
        assert!(!is_configured(home.path(), "DEEPSEEK_API_KEY").unwrap());
    }
    #[test]
    fn redacts_configured_key_and_url_token() {
        let home = tempfile::tempdir().unwrap();
        set_key(home.path(), "DEEPSEEK_API_KEY", Some("secret-12345")).unwrap();
        let text = redact_log_line(
            home.path(),
            "key=secret-12345 http://127.0.0.1/?token=abc123",
        );
        assert!(!text.contains("secret-12345"));
        assert!(!text.contains("abc123"));
    }

    #[test]
    fn invalid_endpoint_does_not_write_key_or_settings() {
        let home = tempfile::tempdir().unwrap();
        set_key(home.path(), "DEEPSEEK_API_KEY", Some("old-secret")).unwrap();
        std::fs::write(
            home.path().join("settings.yaml"),
            "custom:\n  enabled: true\n",
        )
        .unwrap();
        let before = std::fs::read(home.path().join(".credentials.yaml")).unwrap();
        assert!(
            save_deepseek_api(
                home.path(),
                Some("new-secret"),
                Some("https://user:pass@example.com/?token=secret"),
                Some("deepseek-flash")
            )
            .is_err()
        );
        assert_eq!(
            std::fs::read(home.path().join(".credentials.yaml")).unwrap(),
            before
        );
        assert_eq!(
            std::fs::read_to_string(home.path().join("settings.yaml")).unwrap(),
            "custom:\n  enabled: true\n"
        );
    }

    #[test]
    fn saves_verified_fields_and_preserves_unknown_settings() {
        let home = tempfile::tempdir().unwrap();
        std::fs::write(
            home.path().join("settings.yaml"),
            "custom:\n  enabled: true\n",
        )
        .unwrap();
        save_deepseek_api(
            home.path(),
            Some("test-secret"),
            Some("https://api.deepseek.com/anthropic"),
            Some("deepseek-flash"),
        )
        .unwrap();
        assert!(is_configured(home.path(), "DEEPSEEK_API_KEY").unwrap());
        let settings = std::fs::read_to_string(home.path().join("settings.yaml")).unwrap();
        assert!(settings.contains("custom:"));
        assert!(settings.contains("deepseek-official"));
        assert!(settings.contains("baseURL: https://api.deepseek.com/anthropic"));
        set_key(home.path(), "DEEPSEEK_API_KEY", None).unwrap();
        assert!(!is_configured(home.path(), "DEEPSEEK_API_KEY").unwrap());
    }

    #[test]
    fn malformed_settings_leave_existing_key_untouched() {
        let home = tempfile::tempdir().unwrap();
        set_key(home.path(), "DEEPSEEK_API_KEY", Some("old-secret")).unwrap();
        std::fs::write(home.path().join("settings.yaml"), "broken: [\n").unwrap();
        let before = std::fs::read(home.path().join(".credentials.yaml")).unwrap();
        assert!(
            save_deepseek_api(
                home.path(),
                Some("replacement-secret"),
                Some("https://api.deepseek.com/anthropic"),
                None,
            )
            .is_err()
        );
        assert_eq!(
            std::fs::read(home.path().join(".credentials.yaml")).unwrap(),
            before
        );
        assert_eq!(
            std::fs::read_to_string(home.path().join("settings.yaml")).unwrap(),
            "broken: [\n"
        );
    }
}
