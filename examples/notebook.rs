use maruzzella::{
    default_product_spec, run, text_tab, BottomPanelLayout, MaruzzellaConfig, SplitAxis,
    TabGroupSpec, ThemeSpec, WorkbenchNodeSpec,
};

fn main() {
    let mut product = default_product_spec();
    product.branding.title = "Notebook".to_string();
    product.branding.search_placeholder = "Search notes, boards, and logs".to_string();
    product.branding.status_text = "Example app built on Maruzzella".to_string();

    if let Some(view_root) = product.menu_roots.iter_mut().find(|root| root.id == "view") {
        view_root.label = "View".to_string();
    }

    product.layout.left_panel = TabGroupSpec::new(
        "panel-left",
        Some("spaces"),
        vec![
            text_tab(
                "spaces",
                "panel-left",
                "Spaces",
                "Personal, Team, Archive",
                false,
            ),
            text_tab(
                "bookmarks",
                "panel-left",
                "Bookmarks",
                "Pinned notes and saved searches.",
                false,
            ),
        ],
    );

    product.layout.right_panel = TabGroupSpec::new(
        "panel-right",
        Some("details"),
        vec![
            text_tab(
                "details",
                "panel-right",
                "Details",
                "Metadata, tags, and collaborators.",
                false,
            ),
            text_tab(
                "history",
                "panel-right",
                "History",
                "Recent edits and timeline events.",
                false,
            ),
        ],
    );

    product.layout.bottom_panel = TabGroupSpec::new(
        "panel-bottom",
        Some("activity"),
        vec![
            text_tab(
                "activity",
                "panel-bottom",
                "Activity",
                "Automations, sync jobs, and notifications.",
                false,
            ),
            text_tab(
                "diagnostics",
                "panel-bottom",
                "Diagnostics",
                "Parser output and indexing warnings.",
                false,
            ),
        ],
    );
    product.layout.bottom_panel_layout = BottomPanelLayout::FullWidth;

    product.layout.workbench = WorkbenchNodeSpec::Split {
        axis: SplitAxis::Vertical,
        children: vec![
            WorkbenchNodeSpec::Group(TabGroupSpec::new(
                "workbench-main",
                Some("today"),
                vec![
                    text_tab(
                        "today",
                        "workbench-main",
                        "Today",
                        "Daily dashboard for notes, tasks, and drafts.",
                        false,
                    ),
                    text_tab(
                        "draft",
                        "workbench-main",
                        "Draft",
                        "Write here. This stays intentionally neutral in the example.",
                        true,
                    ),
                ],
            )),
            WorkbenchNodeSpec::Group(TabGroupSpec::new(
                "workbench-secondary",
                Some("board"),
                vec![
                    text_tab(
                        "board",
                        "workbench-secondary",
                        "Board",
                        "Secondary workbench area for planning and structure.",
                        false,
                    ),
                    text_tab(
                        "review",
                        "workbench-secondary",
                        "Review",
                        "Compare revisions, notes, or external context here.",
                        true,
                    ),
                ],
            )),
        ],
    };

    let config = MaruzzellaConfig::new("com.example.notebook")
        .with_persistence_id("notebook-example")
        .with_theme(notebook_theme())
        .with_product(product);

    run(config);
}

fn notebook_theme() -> ThemeSpec {
    let mut theme = ThemeSpec::default();
    theme.typography.font_family = "\"IBM Plex Sans\", \"Cantarell\", sans-serif".to_string();
    theme.typography.mono_font_family = "\"IBM Plex Mono\", monospace".to_string();
    theme.typography.font_size_base = 15;
    theme.palette.bg_0 = "#f3efe6".to_string();
    theme.palette.bg_1 = "#e6dccd".to_string();
    theme.palette.workbench = "#f7f2e9".to_string();
    theme.palette.panel_left = "#ebe2d4".to_string();
    theme.palette.panel_right = "#efe6d8".to_string();
    theme.palette.panel_bottom = "#ddd0bd".to_string();
    theme.palette.border = "#cdbca6".to_string();
    theme.palette.border_strong = "#b39b7e".to_string();
    theme.palette.text_0 = "#34281f".to_string();
    theme.palette.text_1 = "#5f4d3f".to_string();
    theme.palette.text_2 = "#907a66".to_string();
    theme.palette.accent = "#b45f2e".to_string();
    theme.palette.accent_strong = "#d8783f".to_string();
    theme.density.radius_medium = 12;
    theme.density.radius_large = 16;
    theme.density.toolbar_height = 46;
    theme.density.tab_height = 32;
    theme.density.min_side_panel_width = 250;
    theme
        .overrides
        .insert("color_app_shell".to_string(), "#f1eadf".to_string());
    theme
        .overrides
        .insert("color_topbar".to_string(), "#e4d6c3".to_string());
    theme
        .overrides
        .insert("color_menu_bg".to_string(), "#ddcfbb".to_string());
    theme
        .overrides
        .insert("color_toolbar_bg".to_string(), "#e8dccb".to_string());
    theme
        .overrides
        .insert("color_toolbar_group".to_string(), "#f5ecdf".to_string());
    theme.overrides.insert(
        "color_toolbar_group_subtle".to_string(),
        "#ede2d2".to_string(),
    );
    theme
        .overrides
        .insert("color_notebook_tab_bg".to_string(), "#eadfce".to_string());
    theme.overrides.insert(
        "color_notebook_tab_hover".to_string(),
        "#e1d3be".to_string(),
    );
    theme.overrides.insert(
        "color_notebook_tab_active".to_string(),
        "#f7f1e7".to_string(),
    );
    theme
        .overrides
        .insert("color_workbench_tab_bg".to_string(), "#eee3d3".to_string());
    theme.overrides.insert(
        "color_workbench_tab_active".to_string(),
        "#f8f2e8".to_string(),
    );
    theme
        .overrides
        .insert("color_status_bar_bg".to_string(), "#e5d7c4".to_string());
    theme
}
