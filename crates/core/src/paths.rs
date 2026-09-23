use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct AppPaths {
    pub root: PathBuf,
}

impl AppPaths {
    pub fn discover() -> Result<Self> {
        let exe = std::env::current_exe().context("无法定位启动器")?;
        let exe_dir = exe.parent().context("启动器路径无父目录")?;
        if exe_dir.join("portable.flag").exists() {
            return Ok(Self {
                root: exe_dir.join("data"),
            });
        }
        #[cfg(target_os = "windows")]
        let base = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .ok_or_else(|| anyhow::anyhow!("LOCALAPPDATA 未设置"))?;
        #[cfg(not(target_os = "windows"))]
        let base = std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
            .ok_or_else(|| anyhow::anyhow!("无法定位用户数据目录"))?;
        Ok(Self {
            root: base.join("hdsl-rs"),
        })
    }

    pub fn with_root(root: PathBuf) -> Self {
        Self { root }
    }
    pub fn instances(&self) -> PathBuf {
        self.root.join("instances")
    }
    pub fn instance(&self, id: &str) -> Result<PathBuf> {
        if !is_safe_segment(id) {
            bail!("无效的实例 ID");
        }
        Ok(self.instances().join(id))
    }
    pub fn node_runtime(&self, version: &str) -> Result<PathBuf> {
        if !is_safe_segment(version) {
            bail!("无效的 Node 版本");
        }
        Ok(self.root.join("runtimes/node").join(version))
    }
    pub fn pnpm_tool(&self) -> PathBuf {
        self.root.join("tools/pnpm/11.7.0")
    }
    pub fn catalog(&self) -> PathBuf {
        self.root.join("catalog")
    }
    pub fn ensure(&self) -> Result<()> {
        std::fs::create_dir_all(self.instances())?;
        std::fs::create_dir_all(self.catalog())?;
        Ok(())
    }
}

pub fn is_safe_segment(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
        && value != "."
        && value != ".."
        && !value.ends_with('.')
}

pub fn replace_file(path: &Path, contents: &[u8]) -> Result<()> {
    let parent = path.parent().context("目标文件无父目录")?;
    std::fs::create_dir_all(parent)?;
    let mut temp = tempfile::NamedTempFile::new_in(parent)?;
    use std::io::Write;
    temp.write_all(contents)?;
    temp.as_file().sync_all()?;
    temp.persist(path).map_err(|e| e.error)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_path_segments() {
        for bad in ["", "..", "../x", "a/b", "a\\b", "x:", "x."] {
            assert!(!is_safe_segment(bad));
        }
        assert!(is_safe_segment("0.1.5-rc.1"));
    }
}
