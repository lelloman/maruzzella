use maruzzella::{
    default_product_spec, run, text_tab, BottomPanelLayout, ButtonAppearance, ButtonStyle,
    InputAppearance, MaruzzellaConfig, SurfaceAppearance, SurfaceLevel, TabGroupSpec,
    TabStripAppearance, TabStripStyle, TextAppearance, TextRole, ThemeSpec, Tone,
    WorkbenchNodeSpec,
};

fn main() {
    let mut product = default_product_spec();
    product.branding.title = "Semantic Appearance Demo".to_string();
    product.branding.search_placeholder = "Search semantic appearance demo".to_string();
    product.branding.status_text = "Panels, buttons, typography, and tabs styled by named roles"
        .to_string();

    product.layout.left_panel = TabGroupSpec::new(
        "panel-left",
        Some("overview"),
        vec![
            text_tab(
                "overview",
                "panel-left",
                "Overview",
                "This panel uses the `primary-panel` appearance id.",
                false,
            ),
            text_tab(
                "teams",
                "panel-left",
                "Teams",
                "Downstream apps can override the meaning of `primary-panel` in ThemeSpec.",
                false,
            ),
        ],
    )
    .with_panel_appearance("primary-panel")
    .with_panel_header_appearance("panel-header")
    .with_tab_strip_appearance("side-tabs");

    product.layout.right_panel = TabGroupSpec::new(
        "panel-right",
        Some("inspector"),
        vec![
            text_tab(
                "inspector",
                "panel-right",
                "Inspector",
                "This panel uses `secondary-panel` without any raw CSS selectors.",
                false,
            ),
            text_tab(
                "history",
                "panel-right",
                "History",
                "Buttons, labels, and tab strips resolve through the same semantic registry.",
                false,
            ),
        ],
    )
    .with_panel_appearance("secondary-panel")
    .with_panel_header_appearance("panel-header")
    .with_tab_strip_appearance("side-tabs");

    product.layout.bottom_panel = TabGroupSpec::new(
        "panel-bottom",
        Some("console"),
        vec![
            text_tab(
                "console",
                "panel-bottom",
                "Console",
                "The bottom panel uses a console-specific panel and tab-strip role.",
                false,
            )
            .with_text_appearance("code"),
        ],
    )
    .with_panel_appearance("console-panel")
    .with_panel_header_appearance("panel-header")
    .with_tab_strip_appearance("console-tabs")
    .with_text_appearance("code");
    product.layout.bottom_panel_layout = BottomPanelLayout::FullWidth;

    product.layout.workbench = WorkbenchNodeSpec::Group(
        TabGroupSpec::new(
            "workbench-main",
            Some("brief"),
            vec![
                text_tab(
                    "brief",
                    "workbench-main",
                    "Brief",
                    "The central workbench uses `canvas-panel` plus an `editor-tabs` strip.",
                    false,
                ),
                text_tab(
                    "copy",
                    "workbench-main",
                    "Typography",
                    "This example also overrides named text roles such as `title`, `meta`, and `code`.",
                    true,
                )
                .with_text_appearance("meta"),
            ],
        )
        .with_panel_appearance("canvas-panel")
        .with_panel_header_appearance("panel-header")
        .with_tab_strip_appearance("editor-tabs"),
    );

    let config = MaruzzellaConfig::new("com.example.semantic-appearance")
        .with_persistence_id("semantic-appearance-demo")
        .with_theme(semantic_theme())
        .with_product(product);

    run(config);
}

fn semantic_theme() -> ThemeSpec {
    let mut theme = ThemeSpec::default();
    theme.typography.font_family = "\"Azeret Mono\", \"Noto Sans\", sans-serif".to_string();
    theme.typography.mono_font_family = "\"IBM Plex Mono\", monospace".to_string();
    theme.typography.font_size_base = 14;
    theme.typography.font_size_ui = 13;
    theme.typography.font_size_small = 12;
    theme.typography.font_size_tiny = 11;
    theme.typography.font_size_title = 28;
    theme.palette.bg_0 = "#f1efe6".to_string();
    theme.palette.bg_1 = "#e3dfd3".to_string();
    theme.palette.workbench = "#f8f6ef".to_string();
    theme.palette.panel_left = "#d7e1dd".to_string();
    theme.palette.panel_right = "#e7ddd0".to_string();
    theme.palette.panel_bottom = "#d9d5df".to_string();
    theme.palette.border = "#b5b0a3".to_string();
    theme.palette.border_strong = "#8f8777".to_string();
    theme.palette.text_0 = "#1f1e1a".to_string();
    theme.palette.text_1 = "#4b463d".to_string();
    theme.palette.text_2 = "#6d665a".to_string();
    theme.palette.accent = "#0f766e".to_string();
    theme.palette.accent_strong = "#0b5f58".to_string();
    theme.density.radius_medium = 12;
    theme.density.radius_large = 16;
    theme.density.toolbar_height = 46;
    theme.density.tab_height = 32;
    theme.density.panel_header_height = 30;

    theme
        .with_surface_appearance(
            "primary-panel",
            SurfaceAppearance::new(Tone::Primary, SurfaceLevel::Raised, TextRole::Body),
        )
        .with_surface_appearance(
            "secondary-panel",
            SurfaceAppearance::new(Tone::Secondary, SurfaceLevel::Raised, TextRole::Body),
        )
        .with_surface_appearance(
            "console-panel",
            SurfaceAppearance::new(Tone::Tertiary, SurfaceLevel::Sunken, TextRole::Code),
        )
        .with_surface_appearance(
            "canvas-panel",
            SurfaceAppearance::new(Tone::Neutral, SurfaceLevel::Sunken, TextRole::Body)
                .borderless(),
        )
        .with_surface_appearance(
            "panel-header",
            SurfaceAppearance::new(Tone::Neutral, SurfaceLevel::Flat, TextRole::SectionLabel),
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
            ButtonAppearance::new(Tone::Neutral, ButtonStyle::Ghost, TextRole::Body),
        )
        .with_input_appearance(
            "search",
            InputAppearance::new(Tone::Secondary, SurfaceLevel::Sunken, TextRole::Body),
        )
        .with_text_appearance(
            "title",
            TextAppearance {
                role: TextRole::Title,
                tone: Tone::Accent,
            },
        )
        .with_text_appearance(
            "meta",
            TextAppearance {
                role: TextRole::Meta,
                tone: Tone::Neutral,
            },
        )
        .with_tab_strip_appearance(
            "side-tabs",
            TabStripAppearance::new(Tone::Primary, TabStripStyle::Utility, TextRole::TabLabel),
        )
        .with_tab_strip_appearance(
            "editor-tabs",
            TabStripAppearance::new(Tone::Neutral, TabStripStyle::Editor, TextRole::TabLabel),
        )
        .with_tab_strip_appearance(
            "console-tabs",
            TabStripAppearance::new(Tone::Tertiary, TabStripStyle::Console, TextRole::TabLabel),
        )
}
