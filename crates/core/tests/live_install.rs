use hdsl_core::credentials::{redact_log_line, save_deepseek_api};
use hdsl_core::harness::{dsh_entry, install_dsh, managed_path, run_dsh};
use hdsl_core::plugin::{
    curated_catalog, install_plugin, installed_core_versions, installed_plugins, latest_compatible,
    remove_plugin,
};
use hdsl_core::registry::Registry;
use hdsl_core::runtime::{ensure_node24, ensure_pnpm};
use hdsl_core::{AppPaths, Instance};
use std::fs::File;
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

struct ManagedChild(Child);
impl Drop for ManagedChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn start_web(
    paths: &AppPaths,
    instance: &Instance,
    node: &hdsl_core::runtime::NodeRuntime,
    log: &std::path::Path,
) -> ManagedChild {
    let entry = dsh_entry(&instance.dsh(paths).unwrap()).unwrap();
    let child = Command::new(node.node())
        .arg(entry)
        .args([
            "--profile",
            "web",
            "--no-open",
            "--port",
            &instance.port.to_string(),
        ])
        .current_dir(&instance.workspace)
        .env("DSH_HOME", instance.home(paths).unwrap())
        .env("PATH", managed_path(paths, node).unwrap())
        .stdout(Stdio::from(File::create(log).unwrap()))
        .stderr(Stdio::from(
            File::create(log.with_extension("err")).unwrap(),
        ))
        .spawn()
        .unwrap();
    ManagedChild(child)
}

fn wait_for_web(child: &mut ManagedChild, port: u16, log: &std::path::Path) {
    let deadline = Instant::now() + Duration::from_secs(45);
    while Instant::now() < deadline {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return;
        }
        if let Some(exit) = child.0.try_wait().unwrap() {
            panic!(
                "web exited {exit}: {}",
                std::fs::read_to_string(log.with_extension("err")).unwrap_or_default()
            );
        }
        thread::sleep(Duration::from_millis(250));
    }
    panic!(
        "web did not listen on {port}: {}",
        std::fs::read_to_string(log.with_extension("err")).unwrap_or_default()
    );
}

#[test]
#[ignore = "downloads Node.js, pnpm and Harness from official registries"]
fn installs_exact_harness_in_isolated_home() {
    let temp = tempfile::tempdir().unwrap();
    let paths = AppPaths::with_root(temp.path().join("data"));
    paths.ensure().unwrap();
    let registry = Registry::new().unwrap();
    let node = ensure_node24(&paths, &registry).unwrap();
    let pnpm = ensure_pnpm(&paths, &node).unwrap();
    let instance = Instance::new(
        "smoke".into(),
        "0.1.5-rc.3".into(),
        temp.path().to_path_buf(),
        free_port(),
    )
    .unwrap();
    install_dsh(&paths, &instance, &node, &pnpm, &registry, |_review| {
        Ok(true)
    })
    .unwrap();
    let version = run_dsh(&paths, &instance, &node, &["--version"]).unwrap();
    assert!(version.status.success());
    assert!(String::from_utf8_lossy(&version.stdout).contains("0.1.5-rc.3"));
    let config = run_dsh(
        &paths,
        &instance,
        &node,
        &["--profile", "web", "--dump-config"],
    )
    .unwrap();
    assert!(
        config.status.success(),
        "{}",
        String::from_utf8_lossy(&config.stderr)
    );
    assert!(
        instance
            .dsh(&paths)
            .unwrap()
            .join("node_modules/@deepseek-ai/dsh/package.json")
            .is_file()
    );
    assert_ne!(
        instance.dsh(&paths).unwrap(),
        instance.home(&paths).unwrap()
    );
    let installed = installed_core_versions(&paths, &instance).unwrap();
    assert_eq!(
        installed.get("@deepseek-ai/dsh").map(String::as_str),
        Some("0.1.5-rc.3")
    );
    assert!(installed.contains_key("@deepseek-ai/dsh-settings"));
    let mut matches = Vec::new();
    let catalog = curated_catalog().unwrap();
    for item in &catalog {
        if let Some(manifest) = latest_compatible(&registry, item, &instance, &paths).unwrap() {
            println!("compatible: {} {}", manifest.name, manifest.version);
            matches.push(manifest.name);
        }
    }
    assert!(
        !matches.is_empty(),
        "curated catalog has no compatible versions"
    );
    let openviking = catalog
        .iter()
        .find(|item| item.full_name == "volcengine/OpenViking")
        .unwrap();
    let selected = latest_compatible(&registry, openviking, &instance, &paths)
        .unwrap()
        .unwrap();
    install_plugin(
        &paths,
        &instance,
        &node,
        &registry,
        openviking,
        &selected.version,
        |_review| Ok(true),
    )
    .unwrap();
    assert!(
        installed_plugins(&paths, &instance)
            .unwrap()
            .contains(&selected.name)
    );
    remove_plugin(&paths, &instance, &node, &selected.name).unwrap();
    assert!(
        !installed_plugins(&paths, &instance)
            .unwrap()
            .contains(&selected.name)
    );
    let other_workspace = temp.path().join("other-workspace");
    std::fs::create_dir(&other_workspace).unwrap();
    let other_port = loop {
        let port = free_port();
        if port != instance.port {
            break port;
        }
    };
    let other = Instance::new(
        "older".into(),
        "0.1.5-rc.2".into(),
        other_workspace,
        other_port,
    )
    .unwrap();
    install_dsh(&paths, &other, &node, &pnpm, &registry, |_review| Ok(true)).unwrap();
    save_deepseek_api(
        &instance.home(&paths).unwrap(),
        Some("sk-integration-secret-12345"),
        Some("https://api.deepseek.com/anthropic"),
        Some("deepseek-flash"),
    )
    .unwrap();
    assert_ne!(instance.dsh(&paths).unwrap(), other.dsh(&paths).unwrap());
    assert_ne!(instance.home(&paths).unwrap(), other.home(&paths).unwrap());
    let log_a = temp.path().join("web-a.log");
    let log_b = temp.path().join("web-b.log");
    let mut web_a = start_web(&paths, &instance, &node, &log_a);
    let mut web_b = start_web(&paths, &other, &node, &log_b);
    wait_for_web(&mut web_a, instance.port, &log_a);
    wait_for_web(&mut web_b, other.port, &log_b);
    for log in [log_a.clone(), log_a.with_extension("err")] {
        let line = std::fs::read_to_string(log).unwrap_or_default();
        assert!(
            !redact_log_line(&instance.home(&paths).unwrap(), &line)
                .contains("sk-integration-secret-12345")
        );
    }
    web_a.0.kill().unwrap();
    web_a.0.wait().unwrap();
    assert!(web_b.0.try_wait().unwrap().is_none());
    wait_for_web(&mut web_b, other.port, &log_b);
}
