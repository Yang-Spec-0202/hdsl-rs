use anyhow::{Context, Result, bail};
use hdsl_core::credentials;
use hdsl_core::harness;
use hdsl_core::plugin::{self, BuildScriptReview, CatalogItem};
use hdsl_core::registry::Registry;
use hdsl_core::runtime;
use hdsl_core::{AppPaths, Instance, InstanceStore};
use hdsl_ui::{AppWindow, InstanceRow, PluginRow, VersionRow};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::Child;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;

struct Controller {
    paths: AppPaths,
    store: InstanceStore,
    registry: Registry,
    selected: Mutex<Option<Uuid>>,
    processes: Mutex<HashMap<Uuid, Child>>,
    web_urls: Mutex<HashMap<Uuid, String>>,
    pending_approval: Mutex<Option<mpsc::Sender<bool>>>,
    plugin_operation: Mutex<()>,
}

fn main() -> Result<()> {
    let paths = AppPaths::discover()?;
    paths.ensure()?;
    let state = Arc::new(Controller {
        store: InstanceStore::new(paths.clone()),
        paths,
        registry: Registry::new()?,
        selected: Mutex::new(None),
        processes: Mutex::new(HashMap::new()),
        web_urls: Mutex::new(HashMap::new()),
        pending_approval: Mutex::new(None),
        plugin_operation: Mutex::new(()),
    });
    let ui = AppWindow::new()?;
    let cwd = std::env::current_dir()?.to_string_lossy().to_string();
    ui.set_default_workspace(cwd.clone().into());
    ui.set_form_workspace(cwd.into());
    refresh_instances(&state, ui.as_weak());
    wire_callbacks(&ui, &state);
    ui.run()?;
    stop_all(&state);
    Ok(())
}

fn post<F>(weak: slint::Weak<AppWindow>, action: F)
where
    F: FnOnce(AppWindow) + Send + 'static,
{
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = weak.upgrade() {
            action(ui);
        }
    });
}
fn status(weak: slint::Weak<AppWindow>, message: impl Into<String>) {
    let message = message.into();
    post(weak, move |ui| ui.set_status_text(message.into()));
}
fn job<F>(weak: slint::Weak<AppWindow>, label: &'static str, action: F)
where
    F: FnOnce() -> Result<()> + Send + 'static,
{
    status(weak.clone(), format!("{label}…"));
    std::thread::spawn(move || match action() {
        Ok(()) => status(weak, format!("{label}完成")),
        Err(e) => status(weak, format!("{label}失败：{e:#}")),
    });
}

fn selected_instance(state: &Controller) -> Result<Instance> {
    let id = (*state.selected.lock().expect("selected lock")).context("请先选择实例")?;
    state.store.load(id)
}

fn request_build_approval(
    state: &Controller,
    weak: slint::Weak<AppWindow>,
    review: &BuildScriptReview,
) -> Result<bool> {
    let (sender, receiver) = mpsc::channel();
    {
        let mut pending = state.pending_approval.lock().expect("approval lock");
        if pending.is_some() {
            bail!("另一个构建脚本正在等待确认");
        }
        *pending = Some(sender);
    }
    let title = format!("批准 {}@{} 的脚本？", review.package, review.version);
    let body = review
        .scripts
        .iter()
        .map(|(name, command)| format!("{name}: {command}"))
        .collect::<Vec<_>>()
        .join("\n\n");
    post(weak.clone(), move |ui| {
        ui.set_approval_title(title.into());
        ui.set_approval_body(body.into());
        ui.set_approval_pending(true);
    });
    let decision = receiver.recv_timeout(Duration::from_secs(300));
    state.pending_approval.lock().expect("approval lock").take();
    post(weak, |ui| ui.set_approval_pending(false));
    Ok(decision.unwrap_or(false))
}

fn refresh_instances(state: &Arc<Controller>, weak: slint::Weak<AppWindow>) {
    let result = state.store.list();
    let Ok(items) = result else {
        status(weak, format!("读取实例失败：{:#}", result.unwrap_err()));
        return;
    };
    let selected = {
        let mut guard = state.selected.lock().expect("selected lock");
        if guard.is_none() || !items.iter().any(|i| Some(i.id) == *guard) {
            *guard = items.first().map(|i| i.id);
        }
        *guard
    };
    let active = items.iter().find(|i| Some(i.id) == selected).cloned();
    let configured = active
        .as_ref()
        .and_then(|i| i.home(&state.paths).ok())
        .and_then(|h| credentials::is_configured(&h, "DEEPSEEK_API_KEY").ok())
        .unwrap_or(false);
    let installed: Vec<slint::SharedString> = active
        .as_ref()
        .and_then(|i| plugin::installed_plugins(&state.paths, i).ok())
        .unwrap_or_default()
        .into_iter()
        .map(Into::into)
        .collect();
    let rows: Vec<InstanceRow> = items
        .into_iter()
        .map(|i| InstanceRow {
            id: i.id.to_string().into(),
            name: i.name.into(),
            version: i.version.into(),
            workspace: i.workspace.to_string_lossy().to_string().into(),
            port: i.port.to_string().into(),
        })
        .collect();
    post(weak, move |ui| {
        ui.set_instances(ModelRc::new(VecModel::from(rows)));
        ui.set_installed_plugins(ModelRc::new(VecModel::from(installed)));
        ui.set_api_key_configured(configured);
        if let Some(i) = active {
            ui.set_selected_id(i.id.to_string().into());
            ui.set_active_instance_name(i.name.into());
            ui.set_active_instance_version(i.version.into());
            ui.set_active_instance_port(i.port.to_string().into());
        } else {
            ui.set_selected_id("".into());
            ui.set_active_instance_name("未选择实例".into());
            ui.set_active_instance_version("".into());
            ui.set_active_instance_port("".into());
        }
    });
}

fn wire_callbacks(ui: &AppWindow, state: &Arc<Controller>) {
    let weak = ui.as_weak();
    let controller = state.clone();
    ui.on_select_instance(move |raw| match Uuid::parse_str(&raw) {
        Ok(id) => {
            *controller.selected.lock().expect("selected lock") = Some(id);
            refresh_instances(&controller, weak.clone());
        }
        Err(_) => status(weak.clone(), "无效的实例 ID"),
    });

    let weak = ui.as_weak();
    let controller = state.clone();
    ui.on_refresh_versions(move || {
        let state = controller.clone();
        let update = weak.clone();
        job(weak.clone(), "读取版本目录", move || {
            let versions = state.registry.versions("@deepseek-ai/dsh")?;
            let latest = versions.first().context("npm 暂无 Harness 版本")?.clone();
            let count = versions.len() as i32;
            let rows = versions
                .into_iter()
                .map(|version| VersionRow {
                    label: version_label(&version).into(),
                    version: version.into(),
                })
                .collect::<Vec<_>>();
            post(update, move |ui| {
                ui.set_available_versions(ModelRc::new(VecModel::from(rows)));
                ui.set_available_version_count(count);
                ui.set_latest_version(latest.clone().into());
                if ui.get_form_version().is_empty() {
                    ui.set_form_version(latest.into());
                }
            });
            Ok(())
        });
    });

    let weak = ui.as_weak();
    let controller = state.clone();
    ui.on_create_instance(move |name, version, workspace, port| {
        let state = controller.clone();
        let update = weak.clone();
        let (name, version, workspace, port) = (
            name.to_string(),
            version.to_string(),
            PathBuf::from(workspace.as_str()),
            port.to_string(),
        );
        job(weak.clone(), "创建实例", move || {
            let instance = Instance::new(
                name,
                version,
                workspace,
                port.parse().context("端口须为 1–65535")?,
            )?;
            let node = runtime::ensure_node24(&state.paths, &state.registry)?;
            let pnpm = runtime::ensure_pnpm(&state.paths, &node)?;
            harness::install_dsh(
                &state.paths,
                &instance,
                &node,
                &pnpm,
                &state.registry,
                |review| request_build_approval(&state, update.clone(), review),
            )?;
            state.store.save(&instance)?;
            *state.selected.lock().expect("selected lock") = Some(instance.id);
            refresh_instances(&state, update.clone());
            post(update, |ui| ui.set_page(0));
            Ok(())
        });
    });

    let weak = ui.as_weak();
    let controller = state.clone();
    ui.on_launch_instance(move || {
        let state = controller.clone();
        let update = weak.clone();
        job(weak.clone(), "启动 Harness", move || {
            launch_selected(&state, update)
        });
    });
    let weak = ui.as_weak();
    let controller = state.clone();
    ui.on_stop_instance(move || {
        let state = controller.clone();
        job(weak.clone(), "停止 Harness", move || {
            let i = selected_instance(&state)?;
            stop_one(&state, i.id)
        });
    });
    let weak = ui.as_weak();
    let controller = state.clone();
    ui.on_clone_instance(move |raw| {
        let item = Uuid::parse_str(&raw)
            .ok()
            .and_then(|id| controller.store.load(id).ok());
        if let (Some(i), Some(ui)) = (item, weak.upgrade()) {
            ui.set_form_name(format!("{} 新版", i.name).into());
            ui.set_form_workspace(i.workspace.to_string_lossy().to_string().into());
            ui.set_form_version("".into());
            ui.set_page(1);
            ui.invoke_refresh_versions();
            ui.set_status_text("原实例保留；请选择目标版本创建独立实例".into());
        }
    });
    let weak = ui.as_weak();
    let controller = state.clone();
    ui.on_remove_instance(move |raw| {
        let state = controller.clone();
        let update = weak.clone();
        let raw = raw.to_string();
        job(weak.clone(), "删除实例", move || {
            let id = Uuid::parse_str(&raw)?;
            stop_one(&state, id)?;
            let item = state.store.load(id)?;
            state.store.remove(&item)?;
            *state.selected.lock().expect("selected lock") = None;
            refresh_instances(&state, update);
            Ok(())
        });
    });

    let weak = ui.as_weak();
    let controller = state.clone();
    ui.on_refresh_plugins(move |query| {
        let state = controller.clone();
        let update = weak.clone();
        let query = query.to_string();
        job(weak.clone(), "筛选兼容插件", move || {
            let instance = selected_instance(&state)?;
            let candidates = if query.trim().is_empty() {
                plugin::curated_catalog()?
            } else {
                plugin::search_catalog(&state.registry, &query)?
            };
            let mut rows = Vec::new();
            for item in candidates.into_iter().take(20) {
                if let Ok(Some(m)) =
                    plugin::latest_compatible(&state.registry, &item, &instance, &state.paths)
                {
                    rows.push(PluginRow {
                        name: item.full_name.into(),
                        package: m.name.into(),
                        version: m.version.into(),
                        summary: item.summary_zh.or(item.summary).unwrap_or_default().into(),
                        repo: item.repo_url.into(),
                    });
                }
            }
            post(update, move |ui| {
                ui.set_plugins(ModelRc::new(VecModel::from(rows)))
            });
            Ok(())
        });
    });
    let weak = ui.as_weak();
    let controller = state.clone();
    ui.on_install_plugin(move |package, version, repo| {
        let state = controller.clone();
        let update = weak.clone();
        let version = version.to_string();
        let item = CatalogItem {
            full_name: repo.to_string(),
            repo_url: repo.to_string(),
            npm_package: Some(package.to_string()),
            summary: None,
            summary_zh: None,
            stars: None,
            installable: true,
        };
        job(weak.clone(), "安装插件", move || {
            let _operation = state
                .plugin_operation
                .lock()
                .expect("plugin operation lock");
            let instance = selected_instance(&state)?;
            stop_one(&state, instance.id)?;
            let node = runtime::ensure_node24(&state.paths, &state.registry)?;
            runtime::ensure_pnpm(&state.paths, &node)?;
            plugin::install_plugin(
                &state.paths,
                &instance,
                &node,
                &state.registry,
                &item,
                &version,
                |review| request_build_approval(&state, update.clone(), review),
            )?;
            refresh_instances(&state, update);
            Ok(())
        });
    });
    let weak = ui.as_weak();
    let controller = state.clone();
    ui.on_remove_plugin(move |package| {
        let state = controller.clone();
        let update = weak.clone();
        let package = package.to_string();
        job(weak.clone(), "卸载插件", move || {
            let _operation = state
                .plugin_operation
                .lock()
                .expect("plugin operation lock");
            let instance = selected_instance(&state)?;
            stop_one(&state, instance.id)?;
            let node = runtime::ensure_node24(&state.paths, &state.registry)?;
            plugin::remove_plugin(&state.paths, &instance, &node, &package)?;
            refresh_instances(&state, update);
            Ok(())
        });
    });
    let controller = state.clone();
    ui.on_approve_build(move |approved| {
        if let Some(sender) = controller
            .pending_approval
            .lock()
            .expect("approval lock")
            .take()
        {
            let _ = sender.send(approved);
        }
    });

    let weak = ui.as_weak();
    let controller = state.clone();
    ui.on_save_api(move |key, endpoint, model| {
        let state = controller.clone();
        let update = weak.clone();
        let (key, endpoint, model) = (key.to_string(), endpoint.to_string(), model.to_string());
        job(weak.clone(), "保存 API 设置", move || {
            let home = selected_instance(&state)?.home(&state.paths)?;
            credentials::save_deepseek_api(
                &home,
                (!key.is_empty()).then_some(key.as_str()),
                (!endpoint.is_empty()).then_some(endpoint.as_str()),
                (!model.is_empty()).then_some(model.as_str()),
            )?;
            refresh_instances(&state, update);
            Ok(())
        });
    });
    let weak = ui.as_weak();
    let controller = state.clone();
    ui.on_remove_api_key(move || {
        let state = controller.clone();
        let update = weak.clone();
        job(weak.clone(), "移除 API Key", move || {
            let home = selected_instance(&state)?.home(&state.paths)?;
            credentials::set_key(&home, "DEEPSEEK_API_KEY", None)?;
            refresh_instances(&state, update);
            Ok(())
        });
    });
    let weak = ui.as_weak();
    let controller = state.clone();
    ui.on_open_web_settings(move || {
        let id = *controller.selected.lock().expect("selected lock");
        let url = id.and_then(|id| {
            controller
                .web_urls
                .lock()
                .expect("urls lock")
                .get(&id)
                .cloned()
        });
        if let Some(url) = url {
            if let Err(e) = webbrowser::open(&url) {
                status(weak.clone(), e.to_string());
            }
        } else {
            status(
                weak.clone(),
                "请先启动实例，再在 Harness 网页中打开设置 → 模型",
            );
        }
    });
    let weak = ui.as_weak();
    ui.on_open_help(move || {
        if let Err(e) = open_help() {
            status(weak.clone(), format!("无法打开帮助：{e:#}"));
        }
    });
    let weak = ui.as_weak();
    ui.on_open_upstream_report(move || {
        if let Err(e) =
            webbrowser::open("https://github.com/deepseek-ai/deepseek-harness/discussions/7593")
        {
            status(weak.clone(), format!("无法打开上游报告：{e}"));
        }
    });
}

fn version_label(version: &str) -> &'static str {
    match version
        .split_once('-')
        .map(|(_, suffix)| suffix.split('.').next().unwrap_or_default())
    {
        Some("alpha") => "Alpha 预览",
        Some("beta") => "Beta 预览",
        Some("rc") => "RC 预览",
        Some(_) => "预览版",
        None => "正式版",
    }
}

fn launch_selected(state: &Arc<Controller>, weak: slint::Weak<AppWindow>) -> Result<()> {
    let instance = selected_instance(state)?;
    {
        let mut map = state.processes.lock().expect("processes lock");
        if let Some(child) = map.get_mut(&instance.id) {
            if child.try_wait()?.is_none() {
                bail!("该实例正在运行");
            }
            map.remove(&instance.id);
        }
    }
    let node = runtime::ensure_node24(&state.paths, &state.registry)?;
    let mut child = harness::launch_web(&state.paths, &instance, &node)?;
    let stdout = child.stdout.take().context("无法读取 Harness stdout")?;
    let stderr = child.stderr.take().context("无法读取 Harness stderr")?;
    state
        .processes
        .lock()
        .expect("processes lock")
        .insert(instance.id, child);
    state
        .web_urls
        .lock()
        .expect("urls lock")
        .insert(instance.id, format!("http://127.0.0.1:{}", instance.port));
    let home = instance.home(&state.paths)?;
    pipe_logs(home.clone(), weak.clone(), stdout, "");
    pipe_logs(home, weak, stderr, "[stderr] ");
    Ok(())
}

fn pipe_logs<R: std::io::Read + Send + 'static>(
    home: PathBuf,
    weak: slint::Weak<AppWindow>,
    source: R,
    prefix: &'static str,
) {
    std::thread::spawn(move || {
        for line in BufReader::new(source).lines().map_while(|r| r.ok()) {
            let safe = credentials::redact_log_line(&home, &line);
            let message = format!("{prefix}{safe}\n");
            post(weak.clone(), move |ui| {
                let mut logs = ui.get_log_text().to_string();
                logs.push_str(&message);
                if logs.chars().count() > 100_000 {
                    logs = logs
                        .chars()
                        .rev()
                        .take(100_000)
                        .collect::<String>()
                        .chars()
                        .rev()
                        .collect();
                }
                ui.set_log_text(logs.into());
            });
        }
    });
}

fn stop_one(state: &Controller, id: Uuid) -> Result<()> {
    if let Some(mut child) = state.processes.lock().expect("processes lock").remove(&id)
        && child.try_wait()?.is_none()
    {
        child.kill()?;
        child.wait()?;
    }
    state.web_urls.lock().expect("urls lock").remove(&id);
    Ok(())
}
fn stop_all(state: &Controller) {
    let ids: Vec<_> = state
        .processes
        .lock()
        .expect("processes lock")
        .keys()
        .copied()
        .collect();
    for id in ids {
        let _ = stop_one(state, id);
    }
}
fn open_help() -> Result<()> {
    let exe = std::env::current_exe()?;
    let local = exe
        .parent()
        .context("无法定位启动器目录")?
        .join("help/index.html");
    #[cfg(target_os = "linux")]
    let system = PathBuf::from("/usr/share/hdsl/help/index.html");
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/user/book/index.html");
    let path = if local.is_file() {
        local
    } else {
        #[cfg(target_os = "linux")]
        if system.is_file() {
            return webbrowser::open(system.to_str().context("帮助路径不是 UTF-8")?)
                .map_err(Into::into);
        }
        source
    };
    if !path.is_file() {
        bail!("尚未生成离线用户手册");
    }
    webbrowser::open(path.to_str().context("帮助路径不是 UTF-8")?)?;
    Ok(())
}

#[cfg(test)]
mod version_tests {
    use super::version_label;

    #[test]
    fn labels_preview_channels_without_treating_them_as_stable() {
        assert_eq!(version_label("0.1.7-alpha.2"), "Alpha 预览");
        assert_eq!(version_label("0.1.7-rc.1"), "RC 预览");
        assert_eq!(version_label("0.1.7"), "正式版");
    }
}
