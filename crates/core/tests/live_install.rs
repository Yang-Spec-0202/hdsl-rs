use hdsl_core::harness::{install_dsh, run_dsh};
use hdsl_core::plugin::{
    curated_catalog, install_plugin, installed_core_versions, installed_plugins, latest_compatible,
    remove_plugin,
};
use hdsl_core::registry::Registry;
use hdsl_core::runtime::{ensure_node24, ensure_pnpm};
use hdsl_core::{AppPaths, Instance};

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
        43080,
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
}
