use crate::paths::AppPaths;
use crate::registry::Registry;
use anyhow::{Context, Result, bail};
use flate2::read::GzDecoder;
use node_semver::Version;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use zip::ZipArchive;

const PNPM_VERSION: &str = "11.7.0";

#[derive(Clone, Debug)]
pub struct NodeRuntime {
    pub version: String,
    pub root: PathBuf,
}

impl NodeRuntime {
    pub fn node(&self) -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            self.root.join("node.exe")
        }
        #[cfg(not(target_os = "windows"))]
        {
            self.root.join("bin/node")
        }
    }
    pub fn npm_cli(&self) -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            self.root.join("node_modules/npm/bin/npm-cli.js")
        }
        #[cfg(not(target_os = "windows"))]
        {
            self.root.join("lib/node_modules/npm/bin/npm-cli.js")
        }
    }
    pub fn bin_dir(&self) -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            self.root.clone()
        }
        #[cfg(not(target_os = "windows"))]
        {
            self.root.join("bin")
        }
    }
}

#[derive(Deserialize)]
struct NodeRelease {
    version: String,
    lts: serde_json::Value,
}

pub fn ensure_node24(paths: &AppPaths, registry: &Registry) -> Result<NodeRuntime> {
    if let Some(runtime) = cached_node24(paths)? {
        return Ok(runtime);
    }
    let releases: Vec<NodeRelease> = registry
        .client()
        .get("https://nodejs.org/dist/index.json")
        .send()?
        .error_for_status()?
        .json()?;
    let release = releases
        .into_iter()
        .find(|r| r.version.starts_with("v24.") && r.lts != false)
        .context("Node.js 24 LTS 不在官方发行索引中")?;
    let version = release.version.trim_start_matches('v').to_string();
    let root = paths.node_runtime(&version)?;
    let runtime = NodeRuntime { version, root };
    if runtime.node().is_file() && runtime.npm_cli().is_file() {
        return Ok(runtime);
    }

    #[cfg(target_os = "windows")]
    let platform = "win-x64.zip";
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    let platform = "linux-x64.tar.gz";
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    let platform = "linux-arm64.tar.gz";
    #[cfg(not(any(
        target_os = "windows",
        all(
            target_os = "linux",
            any(target_arch = "x86_64", target_arch = "aarch64")
        )
    )))]
    compile_error!("HDSL 首版仅支持 Windows x64 与 Linux x64/arm64");

    let archive_name = format!("node-{}-{platform}", release.version);
    let url = format!("https://nodejs.org/dist/{}/{archive_name}", release.version);
    let checksums = registry
        .client()
        .get(format!(
            "https://nodejs.org/dist/{}/SHASUMS256.txt",
            release.version
        ))
        .send()?
        .error_for_status()?
        .text()?;
    let expected = checksum_for(&checksums, &archive_name)?;
    let parent = runtime.root.parent().context("Node 目录无父目录")?;
    fs::create_dir_all(parent)?;
    let stage = tempfile::tempdir_in(parent)?;
    let archive_path = stage.path().join(&archive_name);
    download(registry, &url, &archive_path)?;
    verify_sha256(&archive_path, &expected)?;
    let extracted = stage.path().join(format!(
        "node-{}-{}",
        release.version,
        platform
            .trim_end_matches(".zip")
            .trim_end_matches(".tar.gz")
    ));
    if platform.ends_with(".zip") {
        extract_zip(&archive_path, stage.path())?;
    } else {
        extract_tar_gz(&archive_path, stage.path())?;
    }
    if !extracted.is_dir() {
        bail!("Node 发行包结构不正确");
    }
    if runtime.root.exists() {
        fs::remove_dir_all(&runtime.root)?;
    }
    fs::rename(&extracted, &runtime.root)?;
    if !runtime.node().is_file() || !runtime.npm_cli().is_file() {
        bail!("Node 发行包缺少 node 或 npm");
    }
    let output = Command::new(runtime.node()).arg("--version").output()?;
    if !output.status.success() {
        bail!("Node 运行时验证失败");
    }
    Ok(runtime)
}

fn cached_node24(paths: &AppPaths) -> Result<Option<NodeRuntime>> {
    let root = paths.root.join("runtimes/node");
    if !root.is_dir() {
        return Ok(None);
    }
    let mut candidates = Vec::new();
    for entry in fs::read_dir(root)? {
        let name = entry?.file_name().to_string_lossy().to_string();
        let Ok(version) = name.parse::<Version>() else {
            continue;
        };
        if !name.starts_with("24.") || version.to_string() != name {
            continue;
        }
        let runtime = NodeRuntime {
            version: name,
            root: paths.node_runtime(&version.to_string())?,
        };
        if runtime.node().is_file() && runtime.npm_cli().is_file() {
            candidates.push((version, runtime));
        }
    }
    candidates.sort_by(|a, b| b.0.cmp(&a.0));
    Ok(candidates.into_iter().next().map(|(_, runtime)| runtime))
}

pub fn ensure_pnpm(paths: &AppPaths, runtime: &NodeRuntime) -> Result<PathBuf> {
    let root = paths.pnpm_tool();
    let script = root.join("node_modules/pnpm/bin/pnpm.cjs");
    if script.is_file() {
        return Ok(script);
    }
    let parent = root.parent().context("pnpm 目录无父目录")?;
    fs::create_dir_all(parent)?;
    let stage = tempfile::tempdir_in(parent)?;
    let output = Command::new(runtime.node())
        .arg(runtime.npm_cli())
        .args(["install", "--prefix"])
        .arg(stage.path())
        .args([
            "--no-save",
            "--ignore-scripts",
            "--no-audit",
            "--no-fund",
            &format!("pnpm@{PNPM_VERSION}"),
        ])
        .output()
        .context("无法运行 npm 安装 pnpm")?;
    if !output.status.success() {
        bail!("pnpm 安装失败: {}", String::from_utf8_lossy(&output.stderr));
    }
    let staged = stage.path().join("node_modules/pnpm/bin/pnpm.cjs");
    if !staged.is_file() {
        bail!("pnpm 安装包缺少入口");
    }
    if root.exists() {
        fs::remove_dir_all(&root)?;
    }
    fs::rename(stage.path(), &root)?;
    Ok(script)
}

fn download(registry: &Registry, url: &str, dest: &Path) -> Result<()> {
    let mut response = registry.client().get(url).send()?.error_for_status()?;
    let mut file = File::create(dest)?;
    std::io::copy(&mut response, &mut file)?;
    file.sync_all()?;
    Ok(())
}

fn checksum_for(contents: &str, archive: &str) -> Result<String> {
    for line in contents.lines() {
        let mut fields = line.split_whitespace();
        if let (Some(sum), Some(file)) = (fields.next(), fields.next())
            && file == archive
            && sum.len() == 64
            && sum.bytes().all(|c| c.is_ascii_hexdigit())
        {
            return Ok(sum.into());
        }
    }
    bail!("官方校验列表中找不到发行包")
}

fn verify_sha256(path: &Path, expected: &str) -> Result<()> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    if format!("{:x}", hasher.finalize()) != expected.to_ascii_lowercase() {
        bail!("Node 下载文件 SHA256 不匹配");
    }
    Ok(())
}

fn extract_zip(path: &Path, dest: &Path) -> Result<()> {
    let mut zip = ZipArchive::new(File::open(path)?)?;
    for index in 0..zip.len() {
        let mut entry = zip.by_index(index)?;
        let relative = entry
            .enclosed_name()
            .context("ZIP 包含非法路径")?
            .to_owned();
        let target = dest.join(relative);
        if entry.is_dir() {
            fs::create_dir_all(&target)?;
        } else {
            fs::create_dir_all(target.parent().context("ZIP 路径无父目录")?)?;
            let mut out = File::create(target)?;
            std::io::copy(&mut entry, &mut out)?;
            out.flush()?;
        }
    }
    Ok(())
}

fn extract_tar_gz(path: &Path, dest: &Path) -> Result<()> {
    let archive = GzDecoder::new(File::open(path)?);
    let mut tar = tar::Archive::new(archive);
    for entry in tar.entries()? {
        let mut entry = entry?;
        if !entry.unpack_in(dest)? {
            bail!("TAR 包含非法路径");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn finds_exact_checksum() {
        let sum = "a".repeat(64);
        let line = format!("{sum}  node-v24.1.0-win-x64.zip\n");
        assert_eq!(
            checksum_for(&line, "node-v24.1.0-win-x64.zip").unwrap(),
            sum
        );
        assert!(checksum_for(&line, "node-v24.2.0-win-x64.zip").is_err());
    }
    #[test]
    fn reuses_latest_cached_node_without_registry() {
        let root = tempfile::tempdir().unwrap();
        let paths = AppPaths::with_root(root.path().into());
        for version in ["24.8.0", "24.12.1", "25.0.0"] {
            let runtime = NodeRuntime {
                version: version.into(),
                root: paths.node_runtime(version).unwrap(),
            };
            fs::create_dir_all(runtime.node().parent().unwrap()).unwrap();
            fs::create_dir_all(runtime.npm_cli().parent().unwrap()).unwrap();
            fs::write(runtime.node(), b"node").unwrap();
            fs::write(runtime.npm_cli(), b"npm").unwrap();
        }
        assert_eq!(cached_node24(&paths).unwrap().unwrap().version, "24.12.1");
    }
}
