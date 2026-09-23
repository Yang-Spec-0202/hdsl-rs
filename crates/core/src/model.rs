use crate::paths::{AppPaths, replace_file};
use anyhow::{Context, Result, bail};
use node_semver::Version;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Instance {
    pub schema: u32,
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub workspace: PathBuf,
    pub port: u16,
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(default)]
    pub icon: Option<String>,
}

impl Instance {
    pub fn new(name: String, version: String, workspace: PathBuf, port: u16) -> Result<Self> {
        if name.trim().is_empty() || name.len() > 80 {
            bail!("实例名称须为 1–80 个字符");
        }
        if version.parse::<Version>().is_err() {
            bail!("Harness 版本必须是精确的语义版本号");
        }
        if !workspace.is_absolute() || !workspace.is_dir() {
            bail!("工作目录不存在");
        }
        if port == 0 {
            bail!("端口不能为 0");
        }
        Ok(Self {
            schema: 1,
            id: Uuid::new_v4(),
            name,
            version,
            workspace: workspace.canonicalize()?,
            port,
            created_at: chrono::Utc::now(),
            icon: None,
        })
    }
    /// Returns the icon asset base name, defaulting to the grass icon.
    pub fn icon_name(&self) -> &str {
        self.icon.as_deref().unwrap_or("grass")
    }
    pub fn dir(&self, paths: &AppPaths) -> Result<PathBuf> {
        paths.instance(&self.id.to_string())
    }
    pub fn home(&self, paths: &AppPaths) -> Result<PathBuf> {
        Ok(self.dir(paths)?.join("home"))
    }
    pub fn dsh(&self, paths: &AppPaths) -> Result<PathBuf> {
        Ok(self.dir(paths)?.join("dsh"))
    }
}

#[derive(Clone)]
pub struct InstanceStore {
    pub paths: AppPaths,
}
impl InstanceStore {
    pub fn new(paths: AppPaths) -> Self {
        Self { paths }
    }
    pub fn list(&self) -> Result<Vec<Instance>> {
        self.paths.ensure()?;
        let mut items = Vec::new();
        for entry in std::fs::read_dir(self.paths.instances())? {
            let entry = entry?;
            let file = entry.path().join("instance.json");
            if !file.is_file() {
                continue;
            }
            let instance: Instance = serde_json::from_slice(&std::fs::read(&file)?)
                .with_context(|| format!("无法读取 {}", file.display()))?;
            if instance.schema != 1 {
                bail!("实例配置格式不受支持: {}", file.display());
            }
            items.push(instance);
        }
        items.sort_by_key(|a| a.created_at);
        Ok(items)
    }
    pub fn save(&self, instance: &Instance) -> Result<()> {
        if instance.schema != 1 {
            bail!("实例配置格式不受支持");
        }
        let dir = instance.dir(&self.paths)?;
        std::fs::create_dir_all(dir.join("home"))?;
        replace_file(
            &dir.join("instance.json"),
            &serde_json::to_vec_pretty(instance)?,
        )
    }
    pub fn load(&self, id: Uuid) -> Result<Instance> {
        let path = self.paths.instance(&id.to_string())?.join("instance.json");
        let instance: Instance = serde_json::from_slice(&std::fs::read(path)?)?;
        if instance.schema != 1 {
            bail!("实例配置格式不受支持");
        }
        Ok(instance)
    }
    pub fn remove(&self, instance: &Instance) -> Result<()> {
        let dir = instance.dir(&self.paths)?;
        if dir.exists() {
            std::fs::remove_dir_all(dir)?;
        }
        Ok(())
    }
}

pub fn copy_dir(source: &Path, target: &Path) -> Result<()> {
    std::fs::create_dir_all(target)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let dest = target.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &dest)?;
        } else {
            std::fs::copy(entry.path(), dest)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn keeps_instance_home_separate() {
        let root = tempfile::tempdir().unwrap();
        let paths = AppPaths::with_root(root.path().to_path_buf());
        let a = Instance::new("A".into(), "0.1.0".into(), root.path().into(), 3080).unwrap();
        let b = Instance::new("B".into(), "0.1.1".into(), root.path().into(), 3081).unwrap();
        assert_ne!(a.home(&paths).unwrap(), b.home(&paths).unwrap());
        let store = InstanceStore::new(paths);
        store.save(&a).unwrap();
        store.save(&b).unwrap();
        assert_eq!(store.list().unwrap().len(), 2);
    }

    #[test]
    fn rejects_ranges_and_relative_workspace() {
        let root = tempfile::tempdir().unwrap();
        assert!(Instance::new("A".into(), "^0.1.0".into(), root.path().into(), 3080).is_err());
        assert!(Instance::new("A".into(), "0.1.0".into(), PathBuf::from("."), 3080).is_err());
    }
}
