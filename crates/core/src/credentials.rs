use anyhow::{Context, Result};
use serde_yaml::{Mapping, Value};
use std::path::Path;

fn read_document(home: &Path) -> Result<Mapping> {
    let file = home.join(".credentials.yaml");
    if !file.exists() {
        return Ok(Mapping::new());
    }
    let root: Value = serde_yaml::from_slice(&std::fs::read(file)?)?;
    Ok(root.as_mapping().context("凭据文件格式不受支持")?.clone())
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn redacts_configured_key_and_url_token() {
        let home = tempfile::tempdir().unwrap();
        std::fs::write(
            home.path().join(".credentials.yaml"),
            "version: 1\nrefs:\n  DEEPSEEK_API_KEY: secret-12345\n",
        )
        .unwrap();
        let text = redact_log_line(
            home.path(),
            "key=secret-12345 http://127.0.0.1/?token=abc123",
        );
        assert!(!text.contains("secret-12345"));
        assert!(!text.contains("abc123"));
    }

    #[test]
    fn redacts_sk_keys_without_credentials_file() {
        let home = tempfile::tempdir().unwrap();
        let text = redact_log_line(home.path(), "using sk-abcdefgh12345678 now");
        assert!(!text.contains("sk-abcdefgh12345678"));
    }
}
