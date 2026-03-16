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
    theme.typography.font_size_ui = 14;
    theme.typography.font_size_small = 13;
    theme.typography.font_size_tiny = 12;
    theme.typography.font_size_title = 26;
    theme.palette.bg_0 = "#f5f0e6".to_string();
    theme.palette.bg_1 = "#ebe1d2".to_string();
    theme.palette.workbench = "#fbf7f0".to_string();
    theme.palette.panel_left = "#f1e7d8".to_string();
    theme.palette.panel_right = "#f4ebde".to_string();
    theme.palette.panel_bottom = "#e7d9c6".to_string();
    theme.palette.border = "#ceb89a".to_string();
    theme.palette.border_strong = "#b99669".to_string();
    theme.palette.text_0 = "#2f2418".to_string();
    theme.palette.text_1 = "#5d4a37".to_string();
    theme.palette.text_2 = "#8a7258".to_string();
    theme.palette.accent = "#9b5f2b".to_string();
    theme.palette.accent_strong = "#bc753b".to_string();
    theme.density.radius_small = 8;
    theme.density.radius_medium = 14;
    theme.density.radius_large = 18;
    theme.density.space_sm = 6;
    theme.density.space_md = 10;
    theme.density.space_lg = 14;
    theme.density.space_xl = 20;
    theme.density.control_height_small = 26;
    theme.density.control_height_medium = 36;
    theme.density.control_height_large = 40;
    theme.density.toolbar_height = 58;
    theme.density.tab_height = 36;
    theme.density.search_width_min = 360;
    theme.density.search_width_max = 560;
    theme.density.panel_header_height = 32;
    theme.density.min_side_panel_width = 266;
    theme.density.min_bottom_panel_height = 176;
    theme
        .overrides
        .insert("color_app_shell".to_string(), "#f2eadf".to_string());
    theme
        .overrides
        .insert("color_topbar".to_string(), "#f6efe5".to_string());
    theme.overrides.insert(
        "topbar_border".to_string(),
        "1px solid rgba(126, 96, 64, 0.18)".to_string(),
    );
    theme.overrides.insert(
        "window_strip_border".to_string(),
        "1px solid rgba(126, 96, 64, 0.12)".to_string(),
    );
    theme
        .overrides
        .insert("color_window_title".to_string(), "#342515".to_string());
    theme
        .overrides
        .insert("color_window_meta".to_string(), "#846b52".to_string());
    theme
        .overrides
        .insert("window_strip_height".to_string(), "54px".to_string());
    theme
        .overrides
        .insert("space_window_strip_inline".to_string(), "18px".to_string());
    theme
        .overrides
        .insert("menu_bar_height".to_string(), "36px".to_string());
    theme
        .overrides
        .insert("color_menu_bg".to_string(), "transparent".to_string());
    theme
        .overrides
        .insert("color_menu_text".to_string(), "#604c39".to_string());
    theme.overrides.insert(
        "color_menu_hover".to_string(),
        "rgba(124, 93, 58, 0.12)".to_string(),
    );
    theme
        .overrides
        .insert("color_toolbar_bg".to_string(), "#efe5d7".to_string());
    theme.overrides.insert(
        "toolbar_bottom_border".to_string(),
        "1px solid rgba(126, 96, 64, 0.16)".to_string(),
    );
    theme
        .overrides
        .insert("color_toolbar_group".to_string(), "transparent".to_string());
    theme.overrides.insert(
        "color_toolbar_group_subtle".to_string(),
        "transparent".to_string(),
    );
    theme.overrides.insert(
        "color_toolbar_title_chip".to_string(),
        "#f8f3eb".to_string(),
    );
    theme.overrides.insert(
        "toolbar_group_border".to_string(),
        "1px solid rgba(126, 96, 64, 0.14)".to_string(),
    );
    theme
        .overrides
        .insert("color_toolbar_meta".to_string(), "#846b52".to_string());
    theme
        .overrides
        .insert("space_button_inline".to_string(), "15px".to_string());
    theme
        .overrides
        .insert("color_button_text".to_string(), "#5a4733".to_string());
    theme.overrides.insert(
        "color_button_hover".to_string(),
        "rgba(124, 93, 58, 0.12)".to_string(),
    );
    theme.overrides.insert(
        "color_button_active".to_string(),
        "rgba(124, 93, 58, 0.18)".to_string(),
    );
    theme
        .overrides
        .insert("color_accent_action_bg".to_string(), "#e2c39c".to_string());
    theme.overrides.insert(
        "color_accent_action_text".to_string(),
        "#4f341c".to_string(),
    );
    theme.overrides.insert(
        "color_accent_action_hover".to_string(),
        "#d9b487".to_string(),
    );
    theme
        .overrides
        .insert("color_nav_rail_bg".to_string(), "#eadfce".to_string());
    theme.overrides.insert(
        "color_nav_separator".to_string(),
        "rgba(126, 96, 64, 0.18)".to_string(),
    );
    theme
        .overrides
        .insert("color_nav_button_text".to_string(), "#7b6348".to_string());
    theme
        .overrides
        .insert("color_entry_bg".to_string(), "#fbf7f0".to_string());
    theme
        .overrides
        .insert("color_search_bg".to_string(), "#fbf7f0".to_string());
    theme.overrides.insert(
        "search_border".to_string(),
        "1px solid rgba(126, 96, 64, 0.22)".to_string(),
    );
    theme.overrides.insert(
        "color_search_focus_border".to_string(),
        "#b87942".to_string(),
    );
    theme.overrides.insert(
        "search_focus_border".to_string(),
        "1px solid #b87942".to_string(),
    );
    theme
        .overrides
        .insert("color_search_focus_bg".to_string(), "#fffaf3".to_string());
    theme.overrides.insert(
        "color_icon_button_hover".to_string(),
        "rgba(124, 93, 58, 0.12)".to_string(),
    );
    theme.overrides.insert(
        "color_icon_button_hover_border".to_string(),
        "rgba(126, 96, 64, 0.20)".to_string(),
    );
    theme.overrides.insert(
        "icon_button_border".to_string(),
        "1px solid rgba(126, 96, 64, 0.12)".to_string(),
    );
    theme
        .overrides
        .insert("color_selection_bg".to_string(), "#d8b48a".to_string());
    theme
        .overrides
        .insert("color_selection_text".to_string(), "#332213".to_string());
    theme
        .overrides
        .insert("color_separator_fill".to_string(), "#ceb89a".to_string());
    theme
        .overrides
        .insert("separator_alpha".to_string(), "0.42".to_string());
    theme
        .overrides
        .insert("color_notebook_tab_bg".to_string(), "#eadfce".to_string());
    theme
        .overrides
        .insert("color_notebook_tab_text".to_string(), "#6d5741".to_string());
    theme.overrides.insert(
        "color_notebook_tab_hover".to_string(),
        "#e8d8c4".to_string(),
    );
    theme.overrides.insert(
        "color_notebook_tab_hover_border".to_string(),
        "#ca9e70".to_string(),
    );
    theme.overrides.insert(
        "color_notebook_tab_hover_text".to_string(),
        "#4e3926".to_string(),
    );
    theme.overrides.insert(
        "color_notebook_tab_active".to_string(),
        "#f7f1e7".to_string(),
    );
    theme.overrides.insert(
        "color_notebook_tab_active_border".to_string(),
        "#af7642".to_string(),
    );
    theme
        .overrides
        .insert("color_editor_tab_bg".to_string(), "#ede0cf".to_string());
    theme
        .overrides
        .insert("color_editor_tab_active".to_string(), "#f8f1e6".to_string());
    theme.overrides.insert(
        "color_tab_strip_scroller_bg".to_string(),
        "#efe4d4".to_string(),
    );
    theme.overrides.insert(
        "tab_strip_scroller_border".to_string(),
        "1px solid rgba(126, 96, 64, 0.16)".to_string(),
    );
    theme
        .overrides
        .insert("tab_strip_height".to_string(), "40px".to_string());
    theme
        .overrides
        .insert("color_workbench_tab_bg".to_string(), "#ecdfcd".to_string());
    theme.overrides.insert(
        "color_workbench_tab_text".to_string(),
        "#6a5540".to_string(),
    );
    theme.overrides.insert(
        "color_workbench_tab_hover".to_string(),
        "#e5d5c0".to_string(),
    );
    theme.overrides.insert(
        "color_workbench_tab_hover_text".to_string(),
        "#4f3926".to_string(),
    );
    theme.overrides.insert(
        "color_workbench_tab_active".to_string(),
        "#f8f2e8".to_string(),
    );
    theme
        .overrides
        .insert("panel_title_color".to_string(), "#886e52".to_string());
    theme
        .overrides
        .insert("panel_title_tracking".to_string(), "0.08em".to_string());
    theme
        .overrides
        .insert("panel_content_padding".to_string(), "18px 20px".to_string());
    theme
        .overrides
        .insert("dense_row_hover_bg".to_string(), "#eadbc8".to_string());
    theme
        .overrides
        .insert("dense_row_selected_bg".to_string(), "#dcc0a0".to_string());
    theme
        .overrides
        .insert("dense_row_selected_text".to_string(), "#332315".to_string());
    theme
        .overrides
        .insert("color_textview_text".to_string(), "#594531".to_string());
    theme.overrides.insert(
        "color_tool_window_surface".to_string(),
        "rgba(255, 251, 245, 0.35)".to_string(),
    );
    theme.overrides.insert(
        "color_scrollbar_trough".to_string(),
        "rgba(126, 96, 64, 0.08)".to_string(),
    );
    theme
        .overrides
        .insert("color_scrollbar_slider".to_string(), "#bda385".to_string());
    theme.overrides.insert(
        "color_scrollbar_slider_hover".to_string(),
        "#a98964".to_string(),
    );
    theme
        .overrides
        .insert("color_status_bar_bg".to_string(), "#e9dccb".to_string());
    theme
        .overrides
        .insert("color_status_item".to_string(), "#735c45".to_string());
    theme.overrides.insert(
        "color_status_item_strong".to_string(),
        "#4c3926".to_string(),
    );
    theme
        .overrides
        .insert("color_popover_bg".to_string(), "#fbf7f0".to_string());
    theme.overrides.insert(
        "popover_border".to_string(),
        "1px solid rgba(126, 96, 64, 0.18)".to_string(),
    );
    theme.overrides.insert(
        "color_popover_button_text".to_string(),
        "#5e4a37".to_string(),
    );
    theme.overrides.insert(
        "color_popover_button_hover".to_string(),
        "#efe4d4".to_string(),
    );
    theme.overrides.insert(
        "color_popover_button_hover_text".to_string(),
        "#332315".to_string(),
    );
    theme.overrides.insert(
        "color_popover_button_disabled".to_string(),
        "#ab957e".to_string(),
    );
    theme.overrides.insert(
        "color_popover_separator".to_string(),
        "rgba(126, 96, 64, 0.14)".to_string(),
    );
    theme
}
