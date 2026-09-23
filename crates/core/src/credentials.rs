use crate::paths::replace_file;
use anyhow::{Context, Result, bail};
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
    std::fs::create_dir_all(home)?;
    let file = home.join(".credentials.yaml");
    replace_file(&file, serde_yaml::to_string(&root)?.as_bytes())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
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
        let parsed = url::Url::parse(url)?;
        if parsed.scheme() != "https" && parsed.scheme() != "http" {
            bail!("API 端点须为 HTTP(S) URL");
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
    std::fs::create_dir_all(home)?;
    replace_file(&path, serde_yaml::to_string(&root)?.as_bytes())
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
}
