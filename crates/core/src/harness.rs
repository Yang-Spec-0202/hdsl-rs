use crate::model::Instance;
use crate::paths::AppPaths;
use crate::runtime::NodeRuntime;
use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

pub fn managed_path(paths: &AppPaths, runtime: &NodeRuntime) -> Result<OsString> {
    let mut dirs = vec![
        runtime.bin_dir(),
        paths.pnpm_tool().join("node_modules/.bin"),
    ];
    dirs.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    Ok(std::env::join_paths(dirs)?)
}

pub fn install_dsh(
    paths: &AppPaths,
    instance: &Instance,
    runtime: &NodeRuntime,
    pnpm: &Path,
) -> Result<()> {
    let instance_dir = instance.dir(paths)?;
    fs::create_dir_all(&instance_dir)?;
    let stage = tempfile::tempdir_in(&instance_dir)?;
    let manifest = json!({"name":format!("hdsl-instance-{}", instance.id),"private":true,
        "dependencies":{"@deepseek-ai/dsh":instance.version}});
    fs::write(
        stage.path().join("package.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    fs::write(
        stage.path().join("pnpm-workspace.yaml"),
        format!(
            "overrides:\n  '@deepseek-ai/dsh-app-boot': '{}'\n",
            instance.version
        ),
    )?;
    let output = Command::new(runtime.node())
        .arg(pnpm)
        .arg("--dir")
        .arg(stage.path())
        .args(["install", "--reporter=append-only"])
        .env("PATH", managed_path(paths, runtime)?)
        .output()
        .context("无法运行 pnpm 安装 Harness")?;
    if !output.status.success() {
        bail!(
            "Harness 安装失败: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let entry = dsh_entry(stage.path())?;
    let version_output = Command::new(runtime.node())
        .arg(entry)
        .arg("--version")
        .env("DSH_HOME", stage.path().join("probe-home"))
        .env("PATH", managed_path(paths, runtime)?)
        .output()
        .context("无法验证 Harness 入口")?;
    if !version_output.status.success() {
        bail!(
            "Harness 入口无法运行: {}",
            String::from_utf8_lossy(&version_output.stderr)
        );
    }
    let target = instance.dsh(paths)?;
    if target.exists() {
        bail!("目标实例已安装 Harness");
    }
    fs::rename(stage.path(), target)?;
    Ok(())
}

pub fn dsh_entry(install: &Path) -> Result<PathBuf> {
    let package = install.join("node_modules/@deepseek-ai/dsh");
    let manifest: Value = serde_json::from_slice(&fs::read(package.join("package.json"))?)?;
    let bin = manifest.get("bin").context("Harness 包没有 bin 声明")?;
    let relative = bin
        .as_str()
        .or_else(|| bin.get("dsh").and_then(Value::as_str))
        .context("Harness 包没有 dsh 入口")?;
    if relative
        .split(['/', '\\'])
        .any(|part| part == ".." || part.is_empty())
    {
        bail!("Harness 包入口路径不安全");
    }
    let entry = package.join(relative);
    if !entry.is_file() {
        bail!("Harness 包入口不存在");
    }
    Ok(entry)
}

pub fn run_dsh(
    paths: &AppPaths,
    instance: &Instance,
    runtime: &NodeRuntime,
    args: &[&str],
) -> Result<std::process::Output> {
    let entry = dsh_entry(&instance.dsh(paths)?)?;
    let output = Command::new(runtime.node())
        .arg(entry)
        .args(args)
        .current_dir(&instance.workspace)
        .env("DSH_HOME", instance.home(paths)?)
        .env("PATH", managed_path(paths, runtime)?)
        .output()?;
    Ok(output)
}

pub fn launch_web(paths: &AppPaths, instance: &Instance, runtime: &NodeRuntime) -> Result<Child> {
    let entry = dsh_entry(&instance.dsh(paths)?)?;
    fs::create_dir_all(instance.home(paths)?)?;
    let help = run_dsh(paths, instance, runtime, &["--profile", "web", "--help"])?;
    let supports_no_open = String::from_utf8_lossy(&help.stdout).contains("--no-open");
    let mut command = Command::new(runtime.node());
    command.arg(entry).args(["--profile", "web"]);
    if supports_no_open {
        command.arg("--no-open");
    }
    command
        .arg("--port")
        .arg(instance.port.to_string())
        .current_dir(&instance.workspace)
        .env("DSH_HOME", instance.home(paths)?)
        .env("PATH", managed_path(paths, runtime)?)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    Ok(command.spawn()?)
}
