use crate::model::Instance;
use crate::paths::AppPaths;
use crate::plugin::{BuildScriptReview, allow_build, ignored_builds, review_build};
use crate::registry::Registry;
use crate::runtime::NodeRuntime;
use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use std::collections::BTreeSet;
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

pub fn install_dsh<F>(
    paths: &AppPaths,
    instance: &Instance,
    runtime: &NodeRuntime,
    pnpm: &Path,
    registry: &Registry,
    mut approve: F,
) -> Result<()>
where
    F: FnMut(&BuildScriptReview) -> Result<bool>,
{
    let instance_dir = instance.dir(paths)?;
    fs::create_dir_all(&instance_dir)?;
    let target = instance.dsh(paths)?;
    if target.exists() {
        bail!("目标实例已安装 Harness");
    }
    fs::create_dir(&target)?;
    let result = install_dsh_to(
        paths,
        instance,
        runtime,
        pnpm,
        registry,
        &target,
        &mut approve,
    );
    if let Err(problem) = result {
        if let Err(cleanup) = fs::remove_dir_all(&target) {
            bail!("Harness 安装失败（{problem:#}），且目录清理失败（{cleanup}）");
        }
        return Err(problem);
    }
    Ok(())
}

fn install_dsh_to<F>(
    paths: &AppPaths,
    instance: &Instance,
    runtime: &NodeRuntime,
    pnpm: &Path,
    registry: &Registry,
    target: &Path,
    approve: &mut F,
) -> Result<()>
where
    F: FnMut(&BuildScriptReview) -> Result<bool>,
{
    let manifest = json!({"name":format!("hdsl-instance-{}", instance.id),"private":true,
        "dependencies":{"@deepseek-ai/dsh":instance.version}});
    fs::write(
        target.join("package.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    let mut approved = BTreeSet::new();
    let mut installed = false;
    for _ in 0..8 {
        let output = Command::new(runtime.node())
            .arg(pnpm)
            .arg("--dir")
            .arg(target)
            .args(["install", "--reporter=append-only"])
            .env("PATH", managed_path(paths, runtime)?)
            .output()
            .context("无法运行 pnpm 安装 Harness")?;
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
                allow_build(target, &spec)?;
                approved.insert(spec);
            }
            continue;
        }
        if !output.status.success() {
            bail!("Harness 安装失败（pnpm 退出码 {:?}）", output.status.code());
        }
        installed = true;
        break;
    }
    if !installed {
        bail!("Harness 安装需要确认的构建脚本超过 8 轮");
    }
    let entry = dsh_entry(target)?;
    let version_output = Command::new(runtime.node())
        .arg(entry)
        .arg("--version")
        .env("DSH_HOME", instance.home(paths)?)
        .env("PATH", managed_path(paths, runtime)?)
        .output()
        .context("无法验证 Harness 入口")?;
    if !version_output.status.success() {
        bail!(
            "Harness 入口无法运行: {}",
            String::from_utf8_lossy(&version_output.stderr)
        );
    }
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
    let mut command = Command::new(runtime.node());
    command.arg(entry).args(["--profile", "web"]);
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
