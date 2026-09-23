use crate::harness::{dsh_entry, managed_path};
use crate::model::Instance;
use crate::paths::AppPaths;
use crate::registry::{PackageManifest, Registry};
use crate::runtime::NodeRuntime;
use anyhow::{Context, Result, bail};
use node_semver::{Range, Version};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Clone, Debug, Deserialize)]
pub struct CatalogItem {
    #[serde(rename = "fullName")]
    pub full_name: String,
    #[serde(rename = "repoUrl")]
    pub repo_url: String,
    #[serde(rename = "npmPackage")]
    pub npm_package: Option<String>,
    pub summary: Option<String>,
    #[serde(rename = "summaryZh")]
    pub summary_zh: Option<String>,
    pub stars: Option<u64>,
    pub installable: bool,
}

#[derive(Deserialize)]
struct CatalogPage {
    results: Vec<CatalogItem>,
}

#[derive(Deserialize)]
struct Curated {
    schema: u32,
    repositories: Vec<CuratedRepo>,
}
#[derive(Deserialize)]
struct CuratedRepo {
    repo: String,
    package: String,
    verified: Vec<Verified>,
}
#[derive(Deserialize)]
struct Verified {
    plugin_version: String,
    dsh_versions: Vec<String>,
    evidence: String,
    checked_at: String,
}

pub fn search_catalog(registry: &Registry, query: &str) -> Result<Vec<CatalogItem>> {
    let response: CatalogPage = registry
        .client()
        .get("https://dshmarketplace.dev/api/v1/plugins")
        .query(&[("q", query), ("limit", "30")])
        .send()?
        .error_for_status()?
        .json()?;
    Ok(response
        .results
        .into_iter()
        .filter(|item| {
            item.installable
                && item.npm_package.is_some()
                && item.repo_url.starts_with("https://github.com/")
        })
        .collect())
}

pub fn curated_catalog() -> Result<Vec<CatalogItem>> {
    let c: Curated = serde_json::from_str(include_str!("../catalog/curated.json"))?;
    if c.schema != 1 {
        bail!("人工目录格式不受支持");
    }
    Ok(c.repositories
        .into_iter()
        .map(|r| CatalogItem {
            full_name: r.repo.clone(),
            repo_url: format!("https://github.com/{}", r.repo),
            npm_package: Some(r.package),
            summary: None,
            summary_zh: None,
            stars: None,
            installable: true,
        })
        .collect())
}

pub fn installed_core_versions(
    paths: &AppPaths,
    instance: &Instance,
) -> Result<BTreeMap<String, String>> {
    let root = instance.dsh(paths)?;
    let dsh = fs::canonicalize(root.join("node_modules/@deepseek-ai/dsh"))?;
    let manifest: serde_json::Value = serde_json::from_slice(&fs::read(dsh.join("package.json"))?)?;
    let mut names: Vec<String> = manifest
        .get("dependencies")
        .and_then(|v| v.as_object())
        .into_iter()
        .flat_map(|o| o.keys().filter(|k| k.starts_with("@deepseek-ai/")).cloned())
        .collect();
    names.push("@deepseek-ai/dsh".into());
    let mut versions = BTreeMap::new();
    for name in names {
        if let Some(version) = resolve_package_version(&dsh, &name)? {
            versions.insert(name, version);
        }
    }
    Ok(versions)
}

fn resolve_package_version(from: &Path, name: &str) -> Result<Option<String>> {
    for ancestor in from.ancestors() {
        if ancestor.file_name().is_some_and(|n| n == "node_modules") {
            let manifest = ancestor.join(name).join("package.json");
            if manifest.is_file() {
                let parsed: serde_json::Value = serde_json::from_slice(&fs::read(manifest)?)?;
                return Ok(parsed
                    .get("version")
                    .and_then(|v| v.as_str())
                    .map(str::to_owned));
            }
        }
    }
    Ok(None)
}

pub fn compatible(
    manifest: &PackageManifest,
    repo: &str,
    dsh_version: &str,
    core: &BTreeMap<String, String>,
) -> Result<bool> {
    if manifest
        .dsh
        .pointer("/bundle/patch")
        .and_then(|v| v.as_str())
        .is_none()
    {
        return Ok(false);
    }
    if manifest
        .dist
        .as_ref()
        .and_then(|d| d.integrity.as_deref())
        .is_none()
    {
        return Ok(false);
    }
    let repo_text = manifest
        .repository
        .as_str()
        .map(str::to_owned)
        .or_else(|| {
            manifest
                .repository
                .get("url")
                .and_then(|v| v.as_str())
                .map(str::to_owned)
        })
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !repo_text.contains(&format!("github.com/{}", repo.to_ascii_lowercase())) {
        return Ok(false);
    }
    let peers: Vec<_> = manifest
        .peer_dependencies
        .iter()
        .filter(|(name, _)| name.starts_with("@deepseek-ai/"))
        .collect();
    if !peers.is_empty() {
        for (name, range) in peers {
            let Some(actual) = core.get(name) else {
                return Ok(false);
            };
            let Ok(version) = actual.parse::<Version>() else {
                return Ok(false);
            };
            let Ok(range) = range.parse::<Range>() else {
                return Ok(false);
            };
            if !version.satisfies(&range) {
                return Ok(false);
            }
        }
        return Ok(true);
    }
    let c: Curated = serde_json::from_str(include_str!("../catalog/curated.json"))?;
    Ok(c.repositories
        .iter()
        .filter(|item| item.repo.eq_ignore_ascii_case(repo) && item.package == manifest.name)
        .flat_map(|item| &item.verified)
        .any(|v| {
            v.plugin_version == manifest.version
                && v.dsh_versions.iter().any(|d| d == dsh_version)
                && !v.evidence.is_empty()
                && !v.checked_at.is_empty()
        }))
}

pub fn latest_compatible(
    registry: &Registry,
    item: &CatalogItem,
    instance: &Instance,
    paths: &AppPaths,
) -> Result<Option<PackageManifest>> {
    let Some(package) = &item.npm_package else {
        return Ok(None);
    };
    let repo = item
        .repo_url
        .strip_prefix("https://github.com/")
        .context("不是 GitHub 仓库")?
        .trim_end_matches('/');
    let core = installed_core_versions(paths, instance)?;
    for manifest in registry.manifests(package)? {
        if compatible(&manifest, repo, &instance.version, &core)? {
            return Ok(Some(manifest));
        }
    }
    Ok(None)
}

pub fn installed_plugins(paths: &AppPaths, instance: &Instance) -> Result<Vec<String>> {
    let file = instance.home(paths)?.join("profiles/web/package.json");
    if !file.exists() {
        return Ok(Vec::new());
    }
    let value: serde_json::Value = serde_json::from_slice(&fs::read(file)?)?;
    let mut names: Vec<String> = value
        .get("dependencies")
        .and_then(|v| v.as_object())
        .into_iter()
        .flat_map(|o| o.keys().cloned())
        .collect();
    names.sort();
    Ok(names)
}

pub fn install_plugin(
    paths: &AppPaths,
    instance: &Instance,
    runtime: &NodeRuntime,
    registry: &Registry,
    item: &CatalogItem,
    version: &str,
) -> Result<()> {
    let package = item.npm_package.as_deref().context("插件没有 npm 安装包")?;
    let manifest = registry.manifest(package, version)?;
    let repo = item
        .repo_url
        .strip_prefix("https://github.com/")
        .context("不是 GitHub 仓库")?
        .trim_end_matches('/');
    let core = installed_core_versions(paths, instance)?;
    if !compatible(&manifest, repo, &instance.version, &core)? {
        bail!("该插件版本没有当前 Harness 的兼容证据");
    }
    let profile = instance.home(paths)?.join("profiles/web");
    let backup = tempfile::tempdir_in(instance.dir(paths)?)?;
    let existed = profile.is_dir();
    if existed {
        backup_profile(&profile, backup.path())?;
    }
    let entry = dsh_entry(&instance.dsh(paths)?)?;
    let output = Command::new(runtime.node())
        .arg(&entry)
        .args([
            "plugin",
            "--profile",
            "web",
            "add",
            &format!("{package}@{version}"),
        ])
        .current_dir(&instance.workspace)
        .env("DSH_HOME", instance.home(paths)?)
        .env("PATH", managed_path(paths, runtime)?)
        .output()?;
    let passed = output.status.success()
        && Command::new(runtime.node())
            .arg(&entry)
            .args(["--profile", "web", "--dump-config"])
            .current_dir(&instance.workspace)
            .env("DSH_HOME", instance.home(paths)?)
            .env("PATH", managed_path(paths, runtime)?)
            .output()?
            .status
            .success();
    if passed {
        return Ok(());
    }
    rollback_profile(paths, instance, runtime, &profile, backup.path(), existed)?;
    bail!(
        "插件安装或配置检查失败: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn backup_profile(profile: &Path, dest: &Path) -> Result<()> {
    for name in [
        "package.json",
        "pnpm-lock.yaml",
        "pnpm-workspace.yaml",
        "cordis.patch.yml",
    ] {
        let source = profile.join(name);
        if source.is_file() {
            fs::copy(source, dest.join(name))?;
        }
    }
    Ok(())
}

fn rollback_profile(
    paths: &AppPaths,
    instance: &Instance,
    runtime: &NodeRuntime,
    profile: &Path,
    backup: &Path,
    existed: bool,
) -> Result<()> {
    if !existed {
        if profile.exists() {
            fs::remove_dir_all(profile)?;
        }
        return Ok(());
    }
    for name in [
        "package.json",
        "pnpm-lock.yaml",
        "pnpm-workspace.yaml",
        "cordis.patch.yml",
    ] {
        let original = backup.join(name);
        if original.is_file() {
            fs::copy(original, profile.join(name))?;
        } else {
            let _ = fs::remove_file(profile.join(name));
        }
    }
    let pnpm = paths.pnpm_tool().join("node_modules/pnpm/bin/pnpm.cjs");
    let output = Command::new(runtime.node())
        .arg(pnpm)
        .arg("--dir")
        .arg(profile)
        .args(["install", "--frozen-lockfile", "--reporter=append-only"])
        .env("PATH", managed_path(paths, runtime)?)
        .output()?;
    if !output.status.success() {
        bail!(
            "插件回滚时 pnpm 恢复失败: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let _ = instance;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn strict_compatibility_needs_evidence() {
        let mut m = PackageManifest {
            name: "x".into(),
            version: "1.0.0".into(),
            bin: serde_json::Value::Null,
            dependencies: BTreeMap::new(),
            peer_dependencies: BTreeMap::new(),
            engines: BTreeMap::new(),
            dsh: serde_json::json!({"bundle":{"patch":"./cordis.patch.yml"}}),
            repository: serde_json::json!({"url":"https://github.com/unknown/repo"}),
            dist: Some(crate::registry::Dist {
                integrity: Some("sha512-test".into()),
                tarball: None,
            }),
        };
        let mut core = BTreeMap::new();
        core.insert("@deepseek-ai/dsh-tools".into(), "0.1.5-rc.3".into());
        assert!(!compatible(&m, "unknown/repo", "0.1.5-rc.3", &core).unwrap());
        m.peer_dependencies
            .insert("@deepseek-ai/dsh-tools".into(), "^0.1.5-rc.3".into());
        assert!(compatible(&m, "unknown/repo", "0.1.5-rc.3", &core).unwrap());
        core.insert("@deepseek-ai/dsh-tools".into(), "0.1.4".into());
        assert!(!compatible(&m, "unknown/repo", "0.1.5-rc.3", &core).unwrap());
    }
}
