use anyhow::{Context, Result, bail};
use node_semver::Version;
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::time::Duration;

#[derive(Clone)]
pub struct Registry {
    client: Client,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Dist {
    pub integrity: Option<String>,
    pub tarball: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PackageManifest {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub bin: Value,
    #[serde(default)]
    pub dependencies: BTreeMap<String, String>,
    #[serde(rename = "peerDependencies", default)]
    pub peer_dependencies: BTreeMap<String, String>,
    #[serde(default)]
    pub engines: BTreeMap<String, String>,
    #[serde(default)]
    pub scripts: BTreeMap<String, String>,
    #[serde(default)]
    pub dsh: Value,
    #[serde(default)]
    pub repository: Value,
    pub dist: Option<Dist>,
}

#[derive(Deserialize)]
struct PackageIndex {
    versions: BTreeMap<String, Value>,
}

impl Registry {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .user_agent("HDSL/0.1 (+https://github.com/deepseek-ai/deepseek-harness)")
                .build()?,
        })
    }
    pub fn client(&self) -> &Client {
        &self.client
    }
    pub fn versions(&self, name: &str) -> Result<Vec<String>> {
        let path = encode_package(name);
        let index: PackageIndex = self
            .client
            .get(format!("https://registry.npmjs.org/{path}"))
            .header("Accept", "application/vnd.npm.install-v1+json")
            .send()?
            .error_for_status()?
            .json()
            .context("npm 版本目录解析失败")?;
        let mut versions: Vec<Version> = index
            .versions
            .keys()
            .filter_map(|v| v.parse().ok())
            .collect();
        versions.sort_by(|a, b| b.cmp(a));
        Ok(versions.into_iter().map(|v| v.to_string()).collect())
    }
    pub fn manifests(&self, name: &str) -> Result<Vec<PackageManifest>> {
        let path = encode_package(name);
        let index: PackageIndex = self
            .client
            .get(format!("https://registry.npmjs.org/{path}"))
            .header("Accept", "application/json")
            .send()?
            .error_for_status()?
            .json()
            .context("npm 版本目录解析失败")?;
        let mut manifests: Vec<(Version, PackageManifest)> = index
            .versions
            .into_iter()
            .filter_map(|(version, value)| {
                let parsed = version.parse::<Version>().ok()?;
                let manifest = serde_json::from_value::<PackageManifest>(value).ok()?;
                (manifest.name == name && manifest.version == version).then_some((parsed, manifest))
            })
            .collect();
        manifests.sort_by(|a, b| b.0.cmp(&a.0));
        Ok(manifests
            .into_iter()
            .map(|(_, manifest)| manifest)
            .collect())
    }
    pub fn manifest(&self, name: &str, version: &str) -> Result<PackageManifest> {
        if version.is_empty() || version.contains('/') || version.contains('\\') {
            bail!("无效的包版本");
        }
        let url = format!(
            "https://registry.npmjs.org/{}/{}",
            encode_package(name),
            version
        );
        let manifest: PackageManifest = self.client.get(url).send()?.error_for_status()?.json()?;
        if manifest.name != name || manifest.version != version {
            bail!("npm 包元数据不匹配");
        }
        Ok(manifest)
    }
}

fn encode_package(name: &str) -> String {
    url::form_urlencoded::byte_serialize(name.as_bytes()).collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn encodes_scoped_package() {
        assert_eq!(encode_package("@deepseek-ai/dsh"), "%40deepseek-ai%2Fdsh");
    }
    #[test]
    #[ignore = "queries live npm metadata"]
    fn lists_harness_versions_newest_first() {
        let registry = Registry::new().unwrap();
        let versions = registry.versions("@deepseek-ai/dsh").unwrap();
        assert!(
            versions.len() > 3,
            "npm 目录应返回全部已发布版本，实际为 {versions:?}"
        );
        let mut sorted = versions.clone();
        sorted.sort_by(|a, b| {
            b.parse::<Version>()
                .unwrap()
                .cmp(&a.parse::<Version>().unwrap())
        });
        assert_eq!(versions, sorted);
    }

    #[test]
    #[ignore = "queries live npm metadata"]
    fn full_metadata_retains_bundle_and_repository() {
        let registry = Registry::new().unwrap();
        let manifest = registry
            .manifests("dsh-better-sidebar")
            .unwrap()
            .into_iter()
            .find(|m| m.version == "0.19.1")
            .unwrap();
        assert!(manifest.dsh.pointer("/bundle/patch").is_some());
        assert!(manifest.repository.to_string().contains("github.com"));
    }
}
