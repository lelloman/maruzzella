use std::path::PathBuf;

use maruzzella::{
    default_product_spec, plugin_tab, run, MaruzzellaConfig, TabGroupSpec, ThemeSpec,
    WorkbenchNodeSpec,
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
        .with_theme(plugin_demo_theme())
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

fn plugin_demo_theme() -> ThemeSpec {
    let mut theme = ThemeSpec::default();
    theme.typography.font_family = "\"Space Grotesk\", \"Noto Sans\", sans-serif".to_string();
    theme.typography.mono_font_family = "\"JetBrains Mono\", monospace".to_string();
    theme.palette.bg_0 = "#0f1318".to_string();
    theme.palette.bg_1 = "#18202a".to_string();
    theme.palette.workbench = "#0c1015".to_string();
    theme.palette.panel_left = "#131920".to_string();
    theme.palette.panel_right = "#131a22".to_string();
    theme.palette.panel_bottom = "#0a0d12".to_string();
    theme.palette.border = "#27303c".to_string();
    theme.palette.border_strong = "#3d4959".to_string();
    theme.palette.text_0 = "#e3edf7".to_string();
    theme.palette.text_1 = "#acb9c8".to_string();
    theme.palette.text_2 = "#738197".to_string();
    theme.palette.accent = "#36c2a3".to_string();
    theme.palette.accent_strong = "#72f0cf".to_string();
    theme.density.radius_medium = 10;
    theme.density.radius_large = 14;
    theme.density.toolbar_height = 42;
    theme.density.tab_height = 30;
    theme
        .overrides
        .insert("color_app_shell".to_string(), "#0c1015".to_string());
    theme
        .overrides
        .insert("color_topbar".to_string(), "#151c24".to_string());
    theme
        .overrides
        .insert("color_menu_bg".to_string(), "#0f151d".to_string());
    theme
        .overrides
        .insert("color_toolbar_bg".to_string(), "#121922".to_string());
    theme
        .overrides
        .insert("color_toolbar_group".to_string(), "#18212c".to_string());
    theme
        .overrides
        .insert("color_toolbar_group_subtle".to_string(), "#141b23".to_string());
    theme
        .overrides
        .insert("color_accent_action_bg".to_string(), "#1f3940".to_string());
    theme
        .overrides
        .insert("color_accent_action_text".to_string(), "#dffcf6".to_string());
    theme
        .overrides
        .insert("color_notebook_tab_bg".to_string(), "#121922".to_string());
    theme
        .overrides
        .insert("color_notebook_tab_hover".to_string(), "#1a2430".to_string());
    theme
        .overrides
        .insert("color_notebook_tab_active".to_string(), "#1c2733".to_string());
    theme
        .overrides
        .insert("color_workbench_tab_bg".to_string(), "#111821".to_string());
    theme
        .overrides
        .insert("color_workbench_tab_hover".to_string(), "#182330".to_string());
    theme
        .overrides
        .insert("color_workbench_tab_active".to_string(), "#1e2a36".to_string());
    theme
        .overrides
        .insert("color_drag_preview_bg".to_string(), "#22303c".to_string());
    theme
        .overrides
        .insert("color_status_bar_bg".to_string(), "#10161d".to_string());
    theme
}
