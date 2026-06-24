use maruzzella::{
    default_product_spec, run, text_tab, BottomPanelLayout, CommandSpec, MaruzzellaConfig,
    MenuItemSpec, MenuRootSpec, TabGroupSpec, ToolbarDisplayMode, ToolbarItemSpec,
    WorkbenchNodeSpec,
};

fn main() {
    let mut product = default_product_spec();
    product.branding.title = "Default Theme Demo".to_string();
    product.branding.search_placeholder = "Search default theme demo".to_string();
    product.branding.status_text = "Bundled Maruzzella theme without custom overrides".to_string();

    product.menu_roots = vec![MenuRootSpec {
        id: "demo".to_string(),
        label: "Demo".to_string(),
    }];
    product.menu_items = vec![MenuItemSpec {
        id: "reload-theme".to_string(),
        root_id: "demo".to_string(),
        label: "Reload Theme".to_string(),
        command_id: "shell.reload_theme".to_string(),
        payload: Vec::new(),
    }];
    product.commands = vec![CommandSpec {
        id: "shell.reload_theme".to_string(),
        title: "Reload Theme".to_string(),
    }];
    product.toolbar_items = vec![ToolbarItemSpec {
        id: "reload-theme".to_string(),
        icon_name: Some("view-refresh-symbolic".to_string()),
        label: Some("Reload".to_string()),
        command_id: "shell.reload_theme".to_string(),
        payload: Vec::new(),
        secondary: false,
        display_mode: ToolbarDisplayMode::IconAndText,
        appearance_id: "primary".to_string(),
        options: Vec::new(),
        selected_index: 0,
    }];

    product.layout.left_panel = TabGroupSpec::new(
        "panel-left",
        Some("project"),
        vec![
            text_tab(
                "project",
                "panel-left",
                "Project",
                "src\nexamples\nplugins\nresources",
                false,
            ),
            text_tab(
                "outline",
                "panel-left",
                "Outline",
                "Workbench\nPanels\nToolbar\nTheme tokens",
                false,
            ),
        ],
    );

    product.layout.right_panel = TabGroupSpec::new(
        "panel-right",
        Some("inspector"),
        vec![
            text_tab(
                "inspector",
                "panel-right",
                "Inspector",
                "Appearance: default\nSurface: workbench\nState: active",
                false,
            ),
            text_tab(
                "metadata",
                "panel-right",
                "Metadata",
                "No custom ThemeSpec is installed in this example.",
                false,
            ),
        ],
    );

    product.layout.bottom_panel = TabGroupSpec::new(
        "panel-bottom",
        Some("log"),
        vec![
            text_tab(
                "log",
                "panel-bottom",
                "Log",
                "Default theme demo started.\nUse this example as the visual baseline.",
                false,
            )
            .with_text_appearance("code"),
            text_tab("tasks", "panel-bottom", "Tasks", "No running tasks.", false),
        ],
    )
    .with_text_appearance("code");
    product.layout.bottom_panel_layout = BottomPanelLayout::FullWidth;

    product.layout.workbench = WorkbenchNodeSpec::Group(TabGroupSpec::new(
        "workbench-main",
        Some("welcome"),
        vec![
            text_tab(
                "welcome",
                "workbench-main",
                "Welcome",
                "This example intentionally uses ThemeSpec::default() through MaruzzellaConfig::new. It shows the bundled shell styling without custom palette, typography, density, or appearance overrides.",
                false,
            ),
            text_tab(
                "document",
                "workbench-main",
                "Document",
                "A second workbench tab for checking default tab states, close buttons, text rendering, and panel contrast.",
                true,
            ),
        ],
    ));

    let config = MaruzzellaConfig::new("com.example.maruzzella.default-theme")
        .with_persistence_id("default-theme-demo")
        .with_product(product);

    run(config);
}
