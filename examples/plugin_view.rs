use std::path::PathBuf;

use maruzzella::{
    default_product_spec, plugin_tab, run, text_tab, BottomPanelLayout, ButtonAppearance,
    ButtonStyle, MaruzzellaConfig, ShellChrome, SurfaceAppearance, SurfaceLevel, TabGroupSpec,
    TabStripAppearance, TabStripStyle, TextRole, ThemeSpec, Tone, ToolbarPlacement,
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
    product.branding.status_text = "Plugin-backed GTK view with semantic shell styling".to_string();

    product.layout.left_panel = TabGroupSpec::new(
        "panel-left",
        Some("navigation"),
        vec![text_tab(
            "navigation",
            "panel-left",
            "Navigation",
            "Clicking this left-panel tab changes focus, but the right context observer should keep showing the active workbench tab.",
            false,
        )],
    )
    .with_panel_appearance("demo-side")
    .with_panel_header_appearance("demo-header")
    .with_tab_strip_appearance("demo-tabs");

    product.layout.right_panel = TabGroupSpec::new(
        "panel-right",
        Some("context-observer"),
        vec![plugin_tab(
            "context-observer",
            "panel-right",
            "Context",
            "com.example.hello.context-observer",
            "The active context observer could not be created.",
            false,
        )],
    )
    .with_panel_appearance("demo-side")
    .with_panel_header_appearance("demo-header")
    .with_tab_strip_appearance("demo-tabs");

    product.layout.bottom_panel = TabGroupSpec::new(
        "panel-bottom",
        Some("console"),
        vec![text_tab(
            "console",
            "panel-bottom",
            "Console",
            "Clicking this bottom panel changes focus but does not replace the active workbench context.",
            false,
        )
        .with_text_appearance("code")],
    )
    .with_panel_appearance("demo-console")
    .with_panel_header_appearance("demo-header")
    .with_tab_strip_appearance("demo-tabs")
    .with_text_appearance("code");
    product.layout.bottom_panel_layout = BottomPanelLayout::FullWidth;

    product.layout.workbench = WorkbenchNodeSpec::Group(
        TabGroupSpec::new(
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
                text_tab(
                    "notes",
                    "workbench-plugin-demo",
                    "Notes",
                    "Selecting this workbench tab should update the right-panel active context observer.",
                    true,
                ),
            ],
        )
        .with_panel_appearance("demo-workbench")
        .with_panel_header_appearance("demo-header")
        .with_tab_strip_appearance("demo-tabs"),
    );

    let config = MaruzzellaConfig::new("com.example.maruzzella.plugin-view")
        .with_persistence_id("plugin-view-context-demo")
        .with_workspace_chrome(
            ShellChrome::workspace_default().with_toolbar_placement(ToolbarPlacement::Unified),
        )
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
    ThemeSpec::default()
        .with_surface_appearance(
            "topbar",
            SurfaceAppearance::new(Tone::Primary, SurfaceLevel::Flat, TextRole::BodyStrong),
        )
        .with_surface_appearance(
            "toolbar",
            SurfaceAppearance::new(Tone::Primary, SurfaceLevel::Raised, TextRole::Body),
        )
        .with_surface_appearance(
            "demo-workbench",
            SurfaceAppearance::new(Tone::Neutral, SurfaceLevel::Sunken, TextRole::Body)
                .borderless(),
        )
        .with_surface_appearance(
            "demo-side",
            SurfaceAppearance::new(Tone::Secondary, SurfaceLevel::Raised, TextRole::Body),
        )
        .with_surface_appearance(
            "demo-console",
            SurfaceAppearance::new(Tone::Tertiary, SurfaceLevel::Sunken, TextRole::Code),
        )
        .with_surface_appearance(
            "demo-header",
            SurfaceAppearance::new(Tone::Secondary, SurfaceLevel::Flat, TextRole::SectionLabel),
        )
        .with_button_appearance(
            "primary",
            ButtonAppearance::new(Tone::Accent, ButtonStyle::Solid, TextRole::BodyStrong),
        )
        .with_button_appearance(
            "ghost",
            ButtonAppearance::new(Tone::Secondary, ButtonStyle::Ghost, TextRole::Body),
        )
        .with_tab_strip_appearance(
            "demo-tabs",
            TabStripAppearance::new(Tone::Secondary, TabStripStyle::Editor, TextRole::TabLabel),
        )
}
