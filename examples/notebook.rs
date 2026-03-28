use maruzzella::{
    default_product_spec, run, text_tab, BottomPanelLayout, ButtonAppearance, ButtonStyle,
    InputAppearance, MaruzzellaConfig, SplitAxis, SurfaceAppearance, SurfaceLevel, TabGroupSpec,
    TabStripAppearance, TabStripStyle, TextAppearance, TextRole, ThemeSpec, Tone,
    WorkbenchNodeSpec,
};

fn main() {
    let mut product = default_product_spec();
    product.branding.title = "Notebook".to_string();
    product.branding.search_placeholder = "Search notes, boards, and logs".to_string();
    product.branding.status_text = "Example app styled with semantic appearances".to_string();

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
    )
    .with_panel_appearance("library-panel")
    .with_panel_header_appearance("library-header")
    .with_tab_strip_appearance("library-strip");

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
    )
    .with_panel_appearance("detail-panel")
    .with_panel_header_appearance("detail-header")
    .with_tab_strip_appearance("utility");

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
            )
            .with_text_appearance("meta"),
            text_tab(
                "diagnostics",
                "panel-bottom",
                "Diagnostics",
                "Parser output and indexing warnings.",
                false,
            )
            .with_text_appearance("code"),
        ],
    )
    .with_panel_appearance("console")
    .with_panel_header_appearance("detail-header")
    .with_tab_strip_appearance("console")
    .with_text_appearance("code");
    product.layout.bottom_panel_layout = BottomPanelLayout::FullWidth;

    product.layout.workbench = WorkbenchNodeSpec::Split {
        axis: SplitAxis::Vertical,
        children: vec![
            WorkbenchNodeSpec::Group(
                TabGroupSpec::new(
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
                )
                .with_panel_appearance("writing-surface")
                .with_panel_header_appearance("detail-header")
                .with_tab_strip_appearance("editor"),
            ),
            WorkbenchNodeSpec::Group(
                TabGroupSpec::new(
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
                        )
                        .with_text_appearance("meta"),
                    ],
                )
                .with_panel_appearance("workbench")
                .with_panel_header_appearance("detail-header")
                .with_tab_strip_appearance("editor"),
            ),
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
        .with_surface_appearance(
            "app-shell",
            SurfaceAppearance::new(Tone::Neutral, SurfaceLevel::Sunken, TextRole::Body)
                .borderless(),
        )
        .with_surface_appearance(
            "topbar",
            SurfaceAppearance::new(Tone::Primary, SurfaceLevel::Flat, TextRole::BodyStrong),
        )
        .with_surface_appearance(
            "toolbar",
            SurfaceAppearance::new(Tone::Primary, SurfaceLevel::Raised, TextRole::Body),
        )
        .with_surface_appearance(
            "library-panel",
            SurfaceAppearance::new(Tone::Primary, SurfaceLevel::Raised, TextRole::Body),
        )
        .with_surface_appearance(
            "library-header",
            SurfaceAppearance::new(Tone::Primary, SurfaceLevel::Flat, TextRole::SectionLabel),
        )
        .with_surface_appearance(
            "detail-panel",
            SurfaceAppearance::new(Tone::Secondary, SurfaceLevel::Raised, TextRole::Body),
        )
        .with_surface_appearance(
            "detail-header",
            SurfaceAppearance::new(Tone::Secondary, SurfaceLevel::Flat, TextRole::SectionLabel),
        )
        .with_surface_appearance(
            "writing-surface",
            SurfaceAppearance::new(Tone::Neutral, SurfaceLevel::Raised, TextRole::Body)
                .borderless(),
        )
        .with_button_appearance(
            "primary",
            ButtonAppearance::new(Tone::Accent, ButtonStyle::Solid, TextRole::BodyStrong),
        )
        .with_button_appearance(
            "secondary",
            ButtonAppearance::new(Tone::Primary, ButtonStyle::Soft, TextRole::Body),
        )
        .with_button_appearance(
            "ghost",
            ButtonAppearance::new(Tone::Secondary, ButtonStyle::Ghost, TextRole::Body),
        )
        .with_input_appearance(
            "search",
            InputAppearance::new(Tone::Secondary, SurfaceLevel::Sunken, TextRole::Body),
        )
        .with_text_appearance(
            "title",
            TextAppearance {
                role: TextRole::Title,
                tone: Tone::Primary,
            },
        )
        .with_text_appearance(
            "meta",
            TextAppearance {
                role: TextRole::Meta,
                tone: Tone::Secondary,
            },
        )
        .with_tab_strip_appearance(
            "library-strip",
            TabStripAppearance::new(Tone::Primary, TabStripStyle::Utility, TextRole::TabLabel),
        )
        .with_tab_strip_appearance(
            "editor",
            TabStripAppearance::new(Tone::Neutral, TabStripStyle::Editor, TextRole::TabLabel),
        )
        .with_tab_strip_appearance(
            "console",
            TabStripAppearance::new(Tone::Tertiary, TabStripStyle::Console, TextRole::TabLabel),
        )
}
