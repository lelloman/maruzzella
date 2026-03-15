use std::path::PathBuf;

use maruzzella::{
    default_product_spec, plugin_tab, run, MaruzzellaConfig, TabGroupSpec, WorkbenchNodeSpec,
};

fn main() {
    let plugin_path = example_plugin_path();
    if !plugin_path.exists() {
        eprintln!(
            "example plugin not found at {}\nbuild it first with: cargo build -p example_plugin",
            plugin_path.display()
        );
        return;
    }

    let mut product = default_product_spec();
    product.branding.title = "Plugin View Demo".to_string();
    product.branding.search_placeholder = "Search plugin demo".to_string();
    product.branding.status_text = "Plugin-backed GTK view mounted into Maruzzella".to_string();

    product.layout.workbench = WorkbenchNodeSpec::Group(TabGroupSpec::new(
        "workbench-plugin-demo",
        Some("plugin-welcome"),
        vec![
            plugin_tab(
                "plugin-welcome",
                "workbench-plugin-demo",
                "Plugin Welcome",
                "com.example.hello.welcome",
                "The example plugin view could not be created.",
                false,
            ),
            maruzzella::text_tab(
                "notes",
                "workbench-plugin-demo",
                "Notes",
                "This second tab remains host-owned placeholder content.",
                true,
            ),
        ],
    ));

    let config = MaruzzellaConfig::new("com.example.maruzzella.plugin-view")
        .with_persistence_id("plugin-view-demo")
        .with_product(product)
        .with_plugin_path(plugin_path);

    run(config);
}

fn example_plugin_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("debug");
    path.push(format!(
        "{}example_plugin{}",
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_SUFFIX
    ));
    path
}
