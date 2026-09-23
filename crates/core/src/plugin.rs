use crate::harness::{dsh_entry, managed_path};
use crate::model::Instance;
use crate::paths::AppPaths;
use crate::registry::{PackageManifest, Registry};
use crate::runtime::NodeRuntime;
use anyhow::{Context, Result, bail};
use node_semver::{Range, Version};
use regex::Regex;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
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
    let dsh_file = root.join("node_modules/@deepseek-ai/dsh/package.json");
    let main: serde_json::Value = serde_json::from_slice(&fs::read(dsh_file)?)?;
    let main_version = main
        .get("version")
        .and_then(|v| v.as_str())
        .context("Harness 包缺少版本")?;
    if main_version != instance.version {
        bail!("实例记录与安装的 Harness 版本不一致");
    }
    let mut candidates: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    candidates
        .entry("@deepseek-ai/dsh".into())
        .or_default()
        .insert(main_version.into());
    let pnpm = root.join("node_modules/.pnpm");
    for entry in fs::read_dir(pnpm)? {
        let scope = entry?.path().join("node_modules/@deepseek-ai");
        if !scope.is_dir() {
            continue;
        }
        for package in fs::read_dir(scope)? {
            let manifest = package?.path().join("package.json");
            if !manifest.is_file() {
                continue;
            }
            let value: serde_json::Value = serde_json::from_slice(&fs::read(manifest)?)?;
            if let (Some(name), Some(version)) = (
                value.get("name").and_then(|v| v.as_str()),
                value.get("version").and_then(|v| v.as_str()),
            ) && name.starts_with("@deepseek-ai/")
                && version.parse::<Version>().is_ok()
            {
                candidates
                    .entry(name.into())
                    .or_default()
                    .insert(version.into());
            }
        }
    }
    Ok(candidates
        .into_iter()
        .filter_map(|(name, values)| {
            (values.len() == 1).then(|| (name, values.into_iter().next().unwrap()))
        })
        .collect())
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
        .unwrap_or_default();
    if github_identity(&repo_text).as_deref() != Some(&repo.to_ascii_lowercase()) {
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

fn github_identity(value: &str) -> Option<String> {
    let source = value.strip_prefix("git+").unwrap_or(value);
    let url = url::Url::parse(source).ok()?;
    if url.host_str()? != "github.com" {
        return None;
    }
    let path = url.path().trim_matches('/').trim_end_matches(".git");
    let mut parts = path.split('/');
    let owner = parts.next()?;
    let repo = parts.next()?;
    if owner.is_empty() || repo.is_empty() || parts.next().is_some() {
        return None;
    }
    Some(format!("{owner}/{repo}").to_ascii_lowercase())
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

#[derive(Clone, Debug)]
pub struct BuildScriptReview {
    pub package: String,
    pub version: String,
    pub scripts: BTreeMap<String, String>,
}

pub fn install_plugin<F>(
    paths: &AppPaths,
    instance: &Instance,
    runtime: &NodeRuntime,
    registry: &Registry,
    item: &CatalogItem,
    version: &str,
    mut approve: F,
) -> Result<()>
where
    F: FnMut(&BuildScriptReview) -> Result<bool>,
{
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
    let transaction = ProfileTransaction::begin(&profile)?;
    let outcome = install_candidate(
        paths,
        instance,
        runtime,
        registry,
        package,
        version,
        &mut approve,
    );
    match outcome {
        Ok(()) => {
            transaction.commit()?;
            Ok(())
        }
        Err(problem) => {
            transaction
                .rollback()
                .with_context(|| format!("插件安装失败（{problem:#}），且原 profile 恢复失败"))?;
            Err(problem)
        }
    }
}

fn install_candidate<F>(
    paths: &AppPaths,
    instance: &Instance,
    runtime: &NodeRuntime,
    registry: &Registry,
    package: &str,
    version: &str,
    approve: &mut F,
) -> Result<()>
where
    F: FnMut(&BuildScriptReview) -> Result<bool>,
{
    let profile = instance.home(paths)?.join("profiles/web");
    let entry = dsh_entry(&instance.dsh(paths)?)?;
    let mut approved = BTreeSet::new();
    for _ in 0..8 {
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
        let report = format!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let blocked = ignored_builds(&report)?;
        if !blocked.is_empty() {
            for (name, script_version) in blocked {
                let spec = format!("{name}@{script_version}");
                if approved.contains(&spec) {
                    bail!("已批准的 {spec} 构建脚本仍被拦截");
                }
                let review = review_build(registry, name, script_version)?;
                if review.scripts.is_empty() {
                    bail!("{spec} 未提供可核验的构建脚本");
                }
                if !approve(&review)? {
                    bail!("用户未批准 {spec} 的构建脚本");
                }
                allow_build(&profile, &spec)?;
                approved.insert(spec);
            }
            continue;
        }
        if !output.status.success() {
            bail!(
                "官方 dsh plugin 安装失败（退出码 {:?}）",
                output.status.code()
            );
        }
        let check = Command::new(runtime.node())
            .arg(&entry)
            .args(["--profile", "web", "--dump-config"])
            .current_dir(&instance.workspace)
            .env("DSH_HOME", instance.home(paths)?)
            .env("PATH", managed_path(paths, runtime)?)
            .output()?;
        if !check.status.success() {
            bail!("插件安装后 Harness 配置检查失败");
        }
        return Ok(());
    }
    bail!("需要确认的构建脚本超过 8 轮，安装已中止")
}

pub(crate) fn ignored_builds(report: &str) -> Result<Vec<(String, String)>> {
    let line = Regex::new(r"(?i)Ignored build scripts:\s*([^\r\n]+)")?;
    let Some(capture) = line.captures(report) else {
        return Ok(Vec::new());
    };
    let mut blocked = Vec::new();
    for raw in capture[1].trim_end_matches('.').split(',') {
        let spec = raw.trim().trim_matches('`');
        let (name, version) = spec.rsplit_once('@').context("无法识别被拦截的构建包")?;
        if name.is_empty() || version.parse::<Version>().is_err() {
            bail!("构建脚本报告缺少精确包版本");
        }
        blocked.push((name.into(), version.into()));
    }
    Ok(blocked)
}

pub(crate) fn review_build(
    registry: &Registry,
    package: String,
    version: String,
) -> Result<BuildScriptReview> {
    let metadata = registry.manifest(&package, &version)?;
    let scripts = metadata
        .scripts
        .into_iter()
        .filter(|(key, _)| {
            matches!(
                key.as_str(),
                "preinstall" | "install" | "postinstall" | "prepare"
            )
        })
        .collect();
    Ok(BuildScriptReview {
        package,
        version,
        scripts,
    })
}

pub(crate) fn allow_build(profile: &Path, spec: &str) -> Result<()> {
    let file = profile.join("pnpm-workspace.yaml");
    let mut document = if file.is_file() {
        serde_yaml::from_slice::<serde_yaml::Value>(&fs::read(&file)?)?
    } else {
        serde_yaml::Value::Mapping(serde_yaml::Mapping::new())
    };
    let root = document
        .as_mapping_mut()
        .context("pnpm-workspace.yaml 格式无效")?;
    let entries = root
        .entry(serde_yaml::Value::String("allowBuilds".into()))
        .or_insert_with(|| serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));
    entries
        .as_mapping_mut()
        .context("allowBuilds 格式无效")?
        .insert(
            serde_yaml::Value::String(spec.into()),
            serde_yaml::Value::Bool(true),
        );
    crate::paths::replace_file(&file, serde_yaml::to_string(&document)?.as_bytes())
}

struct ProfileTransaction {
    profile: std::path::PathBuf,
    backup: std::path::PathBuf,
    existed: bool,
    active: bool,
}
impl ProfileTransaction {
    fn begin(profile: &Path) -> Result<Self> {
        let parent = profile.parent().context("profile 缺少父目录")?;
        fs::create_dir_all(parent)?;
        let backup = tempfile::tempdir_in(parent)?.keep();
        let existed = profile.is_dir();
        let mut tx = Self {
            profile: profile.into(),
            backup,
            existed,
            active: false,
        };
        if existed {
            fs::rename(profile, tx.original())?;
            tx.active = true;
            if let Err(problem) = copy_profile_content(&tx.original(), profile, true) {
                tx.rollback().context("候选 profile 创建失败且恢复失败")?;
                return Err(problem);
            }
        }
        tx.active = true;
        Ok(tx)
    }
    fn original(&self) -> std::path::PathBuf {
        self.backup.join("original")
    }
    fn commit(mut self) -> Result<()> {
        self.active = false;
        fs::remove_dir_all(&self.backup)?;
        Ok(())
    }
    fn rollback(mut self) -> Result<()> {
        self.restore()
    }
    fn restore(&mut self) -> Result<()> {
        if !self.active {
            return Ok(());
        }
        if self.profile.exists() {
            fs::remove_dir_all(&self.profile)?;
        }
        if self.existed {
            fs::rename(self.original(), &self.profile)?;
        }
        self.active = false;
        fs::remove_dir_all(&self.backup)?;
        Ok(())
    }
}
impl Drop for ProfileTransaction {
    fn drop(&mut self) {
        if self.active
            && let Err(error) = self.restore()
        {
            eprintln!("HDSL profile 恢复失败：{error:#}");
        }
    }
}

fn copy_profile_content(source: &Path, dest: &Path, root: bool) -> Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        if root && entry.file_name() == "node_modules" {
            continue;
        }
        let target = dest.join(entry.file_name());
        let kind = entry.file_type()?;
        if kind.is_symlink() {
            let link = fs::read_link(entry.path())?;
            #[cfg(unix)]
            std::os::unix::fs::symlink(link, target)?;
            #[cfg(windows)]
            {
                if fs::metadata(entry.path())?.is_dir() {
                    std::os::windows::fs::symlink_dir(link, target)?;
                } else {
                    std::os::windows::fs::symlink_file(link, target)?;
                }
            }
        } else if kind.is_dir() {
            copy_profile_content(&entry.path(), &target, false)?;
        } else if kind.is_file() {
            fs::copy(entry.path(), target)?;
        } else {
            bail!("profile 含不支持的文件类型");
        }
    }
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
            scripts: BTreeMap::new(),
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
        core.insert("@deepseek-ai/dsh-tools".into(), "0.1.5-rc.3".into());
        m.repository = serde_json::json!({"url":"https://github.com/unknown/repository"});
        assert!(!compatible(&m, "unknown/repo", "0.1.5-rc.3", &core).unwrap());
    }

    #[test]
    fn parses_scoped_ignored_builds() {
        let report =
            "ERR_PNPM_IGNORED_BUILDS Ignored build scripts: node-pty@1.1.0, @scope/native@2.0.1.\n";
        assert_eq!(
            ignored_builds(report).unwrap(),
            vec![
                ("node-pty".into(), "1.1.0".into()),
                ("@scope/native".into(), "2.0.1".into())
            ]
        );
    }

    #[test]
    fn rollback_restores_original_profile_bytes() {
        let root = tempfile::tempdir().unwrap();
        let profile = root.path().join("profiles/web");
        fs::create_dir_all(profile.join("node_modules")).unwrap();
        fs::write(profile.join("custom.txt"), b"original").unwrap();
        fs::write(profile.join("node_modules/existing"), b"package").unwrap();
        let transaction = ProfileTransaction::begin(&profile).unwrap();
        fs::write(profile.join("custom.txt"), b"changed").unwrap();
        fs::write(profile.join("new.txt"), b"new").unwrap();
        transaction.rollback().unwrap();
        assert_eq!(fs::read(profile.join("custom.txt")).unwrap(), b"original");
        assert_eq!(
            fs::read(profile.join("node_modules/existing")).unwrap(),
            b"package"
        );
        assert!(!profile.join("new.txt").exists());
    }

    #[test]
    fn rollback_removes_new_profile() {
        let root = tempfile::tempdir().unwrap();
        let profile = root.path().join("profiles/web");
        let transaction = ProfileTransaction::begin(&profile).unwrap();
        fs::create_dir_all(&profile).unwrap();
        fs::write(profile.join("new.txt"), b"new").unwrap();
        transaction.rollback().unwrap();
        assert!(!profile.exists());
    }

    #[test]
    fn ambiguous_installed_core_version_is_not_admitted() {
        let temp = tempfile::tempdir().unwrap();
        let paths = AppPaths::with_root(temp.path().join("data"));
        let instance = Instance::new(
            "test".into(),
            "0.1.5-rc.3".into(),
            temp.path().into(),
            43080,
        )
        .unwrap();
        let root = instance.dsh(&paths).unwrap().join("node_modules");
        let dsh = root.join("@deepseek-ai/dsh");
        fs::create_dir_all(&dsh).unwrap();
        fs::write(
            dsh.join("package.json"),
            r#"{"name":"@deepseek-ai/dsh","version":"0.1.5-rc.3"}"#,
        )
        .unwrap();
        for (store, version) in [("a", "1.0.0"), ("b", "2.0.0")] {
            let package = root.join(format!(".pnpm/{store}/node_modules/@deepseek-ai/dsh-tools"));
            fs::create_dir_all(&package).unwrap();
            fs::write(
                package.join("package.json"),
                format!(r#"{{"name":"@deepseek-ai/dsh-tools","version":"{version}"}}"#),
            )
            .unwrap();
        }
        let versions = installed_core_versions(&paths, &instance).unwrap();
        assert_eq!(
            versions.get("@deepseek-ai/dsh").map(String::as_str),
            Some("0.1.5-rc.3")
        );
        assert!(!versions.contains_key("@deepseek-ai/dsh-tools"));
    }
}
