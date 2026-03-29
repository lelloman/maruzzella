use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use gtk::gdk::Display;
use gtk::{CssProvider, STYLE_PROVIDER_PRIORITY_USER};
pub use maruzzella_api::{
    button_css_class, input_css_class, surface_css_class, tab_strip_css_class, text_css_class,
    ButtonStyle, SurfaceLevel, TabStripStyle, TextRole, Tone,
};

const STYLE_TEMPLATE: &str = include_str!("../resources/style.css");

#[derive(Clone, Debug)]
pub struct ThemeSpec {
    pub stylesheet: ThemeStylesheet,
    pub palette: ThemePalette,
    pub typography: ThemeTypography,
    pub density: ThemeDensity,
    pub appearances: ThemeAppearances,
    pub overrides: BTreeMap<String, String>,
}

impl Default for ThemeSpec {
    fn default() -> Self {
        Self {
            stylesheet: ThemeStylesheet::Bundled,
            palette: ThemePalette::default(),
            typography: ThemeTypography::default(),
            density: ThemeDensity::default(),
            appearances: ThemeAppearances::default(),
            overrides: BTreeMap::new(),
        }
    }
}

impl ThemeSpec {
    pub fn with_stylesheet_path(mut self, path: impl AsRef<Path>) -> Self {
        self.stylesheet = ThemeStylesheet::File(path.as_ref().to_path_buf());
        self
    }

    pub fn with_override(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.overrides.insert(key.into(), value.into());
        self
    }

    pub fn with_surface_appearance(
        mut self,
        id: impl Into<String>,
        appearance: SurfaceAppearance,
    ) -> Self {
        self.appearances.surfaces.insert(id.into(), appearance);
        self
    }

    pub fn with_button_appearance(
        mut self,
        id: impl Into<String>,
        appearance: ButtonAppearance,
    ) -> Self {
        self.appearances.buttons.insert(id.into(), appearance);
        self
    }

    pub fn with_text_appearance(
        mut self,
        id: impl Into<String>,
        appearance: TextAppearance,
    ) -> Self {
        self.appearances.text.insert(id.into(), appearance);
        self
    }

    pub fn with_input_appearance(
        mut self,
        id: impl Into<String>,
        appearance: InputAppearance,
    ) -> Self {
        self.appearances.inputs.insert(id.into(), appearance);
        self
    }

    pub fn with_tab_strip_appearance(
        mut self,
        id: impl Into<String>,
        appearance: TabStripAppearance,
    ) -> Self {
        self.appearances.tab_strips.insert(id.into(), appearance);
        self
    }
}

#[derive(Clone, Debug)]
pub enum ThemeStylesheet {
    Bundled,
    File(PathBuf),
}

#[derive(Clone, Debug)]
pub struct ThemePalette {
    pub bg_0: String,
    pub bg_1: String,
    pub workbench: String,
    pub panel_left: String,
    pub panel_right: String,
    pub panel_bottom: String,
    pub border: String,
    pub border_strong: String,
    pub text_0: String,
    pub text_1: String,
    pub text_2: String,
    pub accent: String,
    pub accent_strong: String,
}

impl Default for ThemePalette {
    fn default() -> Self {
        Self {
            bg_0: "#3c3f41".to_string(),
            bg_1: "#45494a".to_string(),
            workbench: "#2b2b2b".to_string(),
            panel_left: "#3c3f41".to_string(),
            panel_right: "#3c3f41".to_string(),
            panel_bottom: "#3c3f41".to_string(),
            border: "#323232".to_string(),
            border_strong: "#515151".to_string(),
            text_0: "#bbbbbb".to_string(),
            text_1: "#a9b7c6".to_string(),
            text_2: "#787878".to_string(),
            accent: "#4b6eaf".to_string(),
            accent_strong: "#589df6".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ThemeTypography {
    pub font_family: String,
    pub mono_font_family: String,
    pub font_size_base: u16,
    pub font_size_ui: u16,
    pub font_size_small: u16,
    pub font_size_tiny: u16,
    pub font_size_title: u16,
}

impl Default for ThemeTypography {
    fn default() -> Self {
        Self {
            font_family: "\"Inter\", \"Noto Sans\", sans-serif".to_string(),
            mono_font_family: "\"JetBrains Mono\", monospace".to_string(),
            font_size_base: 13,
            font_size_ui: 12,
            font_size_small: 11,
            font_size_tiny: 10,
            font_size_title: 24,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ThemeDensity {
    pub window_default_width: i32,
    pub window_default_height: i32,
    pub min_side_panel_width: i32,
    pub min_bottom_panel_height: i32,
    pub radius_none: u16,
    pub radius_small: u16,
    pub radius_medium: u16,
    pub radius_large: u16,
    pub radius_pill: u16,
    pub space_xs: u16,
    pub space_sm: u16,
    pub space_md: u16,
    pub space_lg: u16,
    pub space_xl: u16,
    pub control_height_small: u16,
    pub control_height_medium: u16,
    pub control_height_large: u16,
    pub toolbar_height: u16,
    pub tab_height: u16,
    pub icon_size: u16,
    pub search_width_min: u16,
    pub search_width_max: u16,
    pub command_width_min: u16,
    pub panel_header_height: u16,
}

impl Default for ThemeDensity {
    fn default() -> Self {
        Self {
            window_default_width: 1600,
            window_default_height: 980,
            min_side_panel_width: 200,
            min_bottom_panel_height: 200,
            radius_none: 0,
            radius_small: 2,
            radius_medium: 3,
            radius_large: 4,
            radius_pill: 3,
            space_xs: 2,
            space_sm: 4,
            space_md: 4,
            space_lg: 6,
            space_xl: 8,
            control_height_small: 22,
            control_height_medium: 30,
            control_height_large: 36,
            toolbar_height: 30,
            tab_height: 26,
            icon_size: 16,
            search_width_min: 300,
            search_width_max: 420,
            command_width_min: 220,
            panel_header_height: 26,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ThemeAppearances {
    pub surfaces: BTreeMap<String, SurfaceAppearance>,
    pub buttons: BTreeMap<String, ButtonAppearance>,
    pub text: BTreeMap<String, TextAppearance>,
    pub inputs: BTreeMap<String, InputAppearance>,
    pub tab_strips: BTreeMap<String, TabStripAppearance>,
}

#[derive(Clone, Debug)]
pub struct SurfaceAppearance {
    pub tone: Tone,
    pub level: SurfaceLevel,
    pub text_role: TextRole,
    pub border: bool,
}

#[derive(Clone, Debug)]
pub struct ButtonAppearance {
    pub tone: Tone,
    pub style: ButtonStyle,
    pub text_role: TextRole,
}

#[derive(Clone, Debug)]
pub struct TextAppearance {
    pub role: TextRole,
    pub tone: Tone,
}

#[derive(Clone, Debug)]
pub struct InputAppearance {
    pub tone: Tone,
    pub level: SurfaceLevel,
    pub text_role: TextRole,
}

#[derive(Clone, Debug)]
pub struct TabStripAppearance {
    pub tone: Tone,
    pub style: TabStripStyle,
    pub text_role: TextRole,
}

impl ThemeAppearances {
    fn default_registry() -> Self {
        let mut surfaces = BTreeMap::new();
        surfaces.insert(
            "app-shell".to_string(),
            SurfaceAppearance::new(Tone::Neutral, SurfaceLevel::Sunken, TextRole::Body).borderless(),
        );
        surfaces.insert(
            "topbar".to_string(),
            SurfaceAppearance::new(Tone::Primary, SurfaceLevel::Raised, TextRole::BodyStrong),
        );
        surfaces.insert(
            "menu".to_string(),
            SurfaceAppearance::new(Tone::Primary, SurfaceLevel::Flat, TextRole::BodyStrong).borderless(),
        );
        surfaces.insert(
            "toolbar".to_string(),
            SurfaceAppearance::new(Tone::Primary, SurfaceLevel::Flat, TextRole::Body),
        );
        surfaces.insert(
            "status".to_string(),
            SurfaceAppearance::new(Tone::Secondary, SurfaceLevel::Raised, TextRole::Meta),
        );
        surfaces.insert(
            "primary".to_string(),
            SurfaceAppearance::new(Tone::Primary, SurfaceLevel::Raised, TextRole::Body),
        );
        surfaces.insert(
            "secondary".to_string(),
            SurfaceAppearance::new(Tone::Secondary, SurfaceLevel::Raised, TextRole::Body),
        );
        surfaces.insert(
            "tertiary".to_string(),
            SurfaceAppearance::new(Tone::Tertiary, SurfaceLevel::Raised, TextRole::Body),
        );
        surfaces.insert(
            "workbench".to_string(),
            SurfaceAppearance::new(Tone::Neutral, SurfaceLevel::Sunken, TextRole::Body).borderless(),
        );
        surfaces.insert(
            "console".to_string(),
            SurfaceAppearance::new(Tone::Tertiary, SurfaceLevel::Sunken, TextRole::Code),
        );
        surfaces.insert(
            "inspector".to_string(),
            SurfaceAppearance::new(Tone::Secondary, SurfaceLevel::Raised, TextRole::Body),
        );

        let mut buttons = BTreeMap::new();
        buttons.insert(
            "primary".to_string(),
            ButtonAppearance::new(Tone::Accent, ButtonStyle::Solid, TextRole::BodyStrong),
        );
        buttons.insert(
            "secondary".to_string(),
            ButtonAppearance::new(Tone::Primary, ButtonStyle::Soft, TextRole::Body),
        );
        buttons.insert(
            "ghost".to_string(),
            ButtonAppearance::new(Tone::Neutral, ButtonStyle::Ghost, TextRole::Body),
        );
        buttons.insert(
            "toolbar".to_string(),
            ButtonAppearance::new(Tone::Primary, ButtonStyle::Ghost, TextRole::Body),
        );
        buttons.insert(
            "icon".to_string(),
            ButtonAppearance::new(Tone::Neutral, ButtonStyle::Ghost, TextRole::Body),
        );
        buttons.insert(
            "danger".to_string(),
            ButtonAppearance::new(Tone::Danger, ButtonStyle::Soft, TextRole::BodyStrong),
        );

        let mut text = BTreeMap::new();
        for (id, role, tone) in [
            ("title", TextRole::Title, Tone::Primary),
            ("subtitle", TextRole::Subtitle, Tone::Secondary),
            ("body", TextRole::Body, Tone::Primary),
            ("body-strong", TextRole::BodyStrong, Tone::Primary),
            ("meta", TextRole::Meta, Tone::Neutral),
            ("section-label", TextRole::SectionLabel, Tone::Neutral),
            ("tab-label", TextRole::TabLabel, Tone::Primary),
            ("code", TextRole::Code, Tone::Primary),
        ] {
            text.insert(id.to_string(), TextAppearance { role, tone });
        }

        let mut inputs = BTreeMap::new();
        inputs.insert(
            "search".to_string(),
            InputAppearance::new(Tone::Secondary, SurfaceLevel::Sunken, TextRole::Body),
        );
        inputs.insert(
            "command".to_string(),
            InputAppearance::new(Tone::Secondary, SurfaceLevel::Sunken, TextRole::Body),
        );
        inputs.insert(
            "field".to_string(),
            InputAppearance::new(Tone::Primary, SurfaceLevel::Sunken, TextRole::Body),
        );

        let mut tab_strips = BTreeMap::new();
        tab_strips.insert(
            "editor".to_string(),
            TabStripAppearance::new(Tone::Neutral, TabStripStyle::Editor, TextRole::TabLabel),
        );
        tab_strips.insert(
            "utility".to_string(),
            TabStripAppearance::new(Tone::Primary, TabStripStyle::Utility, TextRole::TabLabel),
        );
        tab_strips.insert(
            "console".to_string(),
            TabStripAppearance::new(Tone::Tertiary, TabStripStyle::Console, TextRole::TabLabel),
        );

        Self {
            surfaces,
            buttons,
            text,
            inputs,
            tab_strips,
        }
    }
}

impl Default for ThemeAppearances {
    fn default() -> Self {
        Self::default_registry()
    }
}

impl SurfaceAppearance {
    pub fn new(tone: Tone, level: SurfaceLevel, text_role: TextRole) -> Self {
        Self {
            tone,
            level,
            text_role,
            border: true,
        }
    }

    pub fn borderless(mut self) -> Self {
        self.border = false;
        self
    }
}

impl ButtonAppearance {
    pub fn new(tone: Tone, style: ButtonStyle, text_role: TextRole) -> Self {
        Self {
            tone,
            style,
            text_role,
        }
    }
}

impl InputAppearance {
    pub fn new(tone: Tone, level: SurfaceLevel, text_role: TextRole) -> Self {
        Self {
            tone,
            level,
            text_role,
        }
    }
}

impl TabStripAppearance {
    pub fn new(tone: Tone, style: TabStripStyle, text_role: TextRole) -> Self {
        Self {
            tone,
            style,
            text_role,
        }
    }
}

struct ThemeRuntime {
    provider: CssProvider,
    spec: ThemeSpec,
}

thread_local! {
    static THEME_RUNTIME: RefCell<Option<ThemeRuntime>> = const { RefCell::new(None) };
}

pub fn install(spec: ThemeSpec) {
    THEME_RUNTIME.with(|runtime| {
        let mut runtime = runtime.borrow_mut();
        let theme_runtime = runtime.get_or_insert_with(|| ThemeRuntime {
            provider: CssProvider::new(),
            spec: ThemeSpec::default(),
        });
        theme_runtime.spec = spec;
        load_into_provider(&theme_runtime.provider, &theme_runtime.spec);
    });
}

pub fn reload() {
    THEME_RUNTIME.with(|runtime| {
        if let Some(theme_runtime) = runtime.borrow().as_ref() {
            load_into_provider(&theme_runtime.provider, &theme_runtime.spec);
        } else {
            install(ThemeSpec::default());
        }
    });
}

fn load_into_provider(provider: &CssProvider, spec: &ThemeSpec) {
    let stylesheet = match build_stylesheet(spec) {
        Ok(stylesheet) => stylesheet,
        Err(error) => {
            eprintln!("theme load failed: {error}");
            render_template(STYLE_TEMPLATE, &ThemeSpec::default().token_map()).unwrap_or_else(
                |fallback_error| panic!("default theme render failed: {fallback_error}"),
            )
        }
    };

    provider.load_from_data(&stylesheet);

    if let Some(display) = Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            provider,
            STYLE_PROVIDER_PRIORITY_USER,
        );
    }
}

fn build_stylesheet(spec: &ThemeSpec) -> Result<String, String> {
    let template = match &spec.stylesheet {
        ThemeStylesheet::Bundled => STYLE_TEMPLATE.to_string(),
        ThemeStylesheet::File(path) => fs::read_to_string(path)
            .map_err(|error| format!("failed to read stylesheet {}: {error}", path.display()))?,
    };

    let mut stylesheet = render_template(&template, &spec.token_map())?;
    stylesheet.push_str(&render_appearance_stylesheet(spec));
    Ok(stylesheet)
}

fn render_template(template: &str, tokens: &BTreeMap<String, String>) -> Result<String, String> {
    let mut rendered = template.to_string();
    for (key, value) in tokens {
        let placeholder = format!("{{{{{key}}}}}");
        rendered = rendered.replace(&placeholder, value);
    }

    if let Some(start) = rendered.find("{{") {
        if let Some(end) = rendered[start + 2..].find("}}") {
            let unresolved = &rendered[start + 2..start + 2 + end];
            return Err(format!("unresolved theme token `{unresolved}`"));
        }
    }

    Ok(rendered)
}

impl ThemeSpec {
    fn token_map(&self) -> BTreeMap<String, String> {
        let mut tokens = BTreeMap::from([
            ("color_bg_0".to_string(), self.palette.bg_0.clone()),
            ("color_bg_1".to_string(), self.palette.bg_1.clone()),
            (
                "color_workbench".to_string(),
                self.palette.workbench.clone(),
            ),
            (
                "color_panel_left".to_string(),
                self.palette.panel_left.clone(),
            ),
            (
                "color_panel_right".to_string(),
                self.palette.panel_right.clone(),
            ),
            (
                "color_panel_bottom".to_string(),
                self.palette.panel_bottom.clone(),
            ),
            ("color_border".to_string(), self.palette.border.clone()),
            (
                "color_border_strong".to_string(),
                self.palette.border_strong.clone(),
            ),
            ("color_text_0".to_string(), self.palette.text_0.clone()),
            ("color_text_1".to_string(), self.palette.text_1.clone()),
            ("color_text_2".to_string(), self.palette.text_2.clone()),
            ("color_accent".to_string(), self.palette.accent.clone()),
            (
                "color_accent_strong".to_string(),
                self.palette.accent_strong.clone(),
            ),
            (
                "font_family_base".to_string(),
                self.typography.font_family.clone(),
            ),
            (
                "font_family_mono".to_string(),
                self.typography.mono_font_family.clone(),
            ),
            (
                "font_size_base".to_string(),
                format!("{}px", self.typography.font_size_base),
            ),
            (
                "font_size_ui".to_string(),
                format!("{}px", self.typography.font_size_ui),
            ),
            (
                "font_size_small".to_string(),
                format!("{}px", self.typography.font_size_small),
            ),
            (
                "font_size_tiny".to_string(),
                format!("{}px", self.typography.font_size_tiny),
            ),
            (
                "font_size_title".to_string(),
                format!("{}px", self.typography.font_size_title),
            ),
            (
                "radius_none".to_string(),
                format!("{}px", self.density.radius_none),
            ),
            (
                "radius_small".to_string(),
                format!("{}px", self.density.radius_small),
            ),
            (
                "radius_medium".to_string(),
                format!("{}px", self.density.radius_medium),
            ),
            (
                "radius_large".to_string(),
                format!("{}px", self.density.radius_large),
            ),
            (
                "radius_pill".to_string(),
                format!("{}px", self.density.radius_pill),
            ),
            (
                "space_xs".to_string(),
                format!("{}px", self.density.space_xs),
            ),
            (
                "space_sm".to_string(),
                format!("{}px", self.density.space_sm),
            ),
            (
                "space_md".to_string(),
                format!("{}px", self.density.space_md),
            ),
            (
                "space_lg".to_string(),
                format!("{}px", self.density.space_lg),
            ),
            (
                "space_xl".to_string(),
                format!("{}px", self.density.space_xl),
            ),
            (
                "control_height_small".to_string(),
                format!("{}px", self.density.control_height_small),
            ),
            (
                "control_height_medium".to_string(),
                format!("{}px", self.density.control_height_medium),
            ),
            (
                "control_height_large".to_string(),
                format!("{}px", self.density.control_height_large),
            ),
            (
                "toolbar_height".to_string(),
                format!("{}px", self.density.toolbar_height),
            ),
            (
                "tab_height".to_string(),
                format!("{}px", self.density.tab_height),
            ),
            (
                "icon_size".to_string(),
                format!("{}px", self.density.icon_size),
            ),
            (
                "search_width_min".to_string(),
                format!("{}px", self.density.search_width_min),
            ),
            (
                "search_width_max".to_string(),
                format!("{}px", self.density.search_width_max),
            ),
            (
                "command_width_min".to_string(),
                format!("{}px", self.density.command_width_min),
            ),
            (
                "panel_header_height".to_string(),
                format!("{}px", self.density.panel_header_height),
            ),
            (
                "window_default_width".to_string(),
                self.density.window_default_width.to_string(),
            ),
            (
                "window_default_height".to_string(),
                self.density.window_default_height.to_string(),
            ),
            (
                "min_side_panel_width".to_string(),
                self.density.min_side_panel_width.to_string(),
            ),
            (
                "min_bottom_panel_height".to_string(),
                self.density.min_bottom_panel_height.to_string(),
            ),
        ]);

        tokens.extend(default_component_tokens());
        tokens.extend(self.overrides.clone());
        tokens
    }
}

fn default_component_tokens() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("color_app_shell".to_string(), "#2b2b2b".to_string()),
        ("color_topbar".to_string(), "#3c3f41".to_string()),
        ("color_topbar_border".to_string(), "#323232".to_string()),
        ("topbar_border".to_string(), "1px solid #323232".to_string()),
        (
            "color_window_strip_border".to_string(),
            "#323232".to_string(),
        ),
        (
            "window_strip_border".to_string(),
            "1px solid #323232".to_string(),
        ),
        ("color_window_meta".to_string(), "#787878".to_string()),
        ("color_window_title".to_string(), "#bbbbbb".to_string()),
        ("color_window_branch_bg".to_string(), "#313335".to_string()),
        ("space_window_strip_inline".to_string(), "16px".to_string()),
        ("window_strip_height".to_string(), "26px".to_string()),
        ("window_branch_padding".to_string(), "0 10px".to_string()),
        ("window_controls_spacing".to_string(), "8px".to_string()),
        ("menu_bar_height".to_string(), "24px".to_string()),
        ("space_menu_inline".to_string(), "6px".to_string()),
        ("color_menu_bg".to_string(), "#3c3f41".to_string()),
        ("color_menu_text".to_string(), "#bbbbbb".to_string()),
        (
            "color_menu_hover".to_string(),
            "alpha(#bbbbbb, 0.10)".to_string(),
        ),
        ("space_menu_button_inline".to_string(), "8px".to_string()),
        ("color_toolbar_bg".to_string(), "#3c3f41".to_string()),
        (
            "toolbar_bottom_border".to_string(),
            "1px solid #323232".to_string(),
        ),
        ("color_toolbar_group".to_string(), "transparent".to_string()),
        (
            "color_toolbar_group_border".to_string(),
            "#515151".to_string(),
        ),
        (
            "toolbar_group_border".to_string(),
            "1px solid #515151".to_string(),
        ),
        ("space_toolbar_group".to_string(), "4px".to_string()),
        (
            "space_toolbar_group_primary".to_string(),
            "12px".to_string(),
        ),
        (
            "color_toolbar_group_subtle".to_string(),
            "transparent".to_string(),
        ),
        (
            "color_toolbar_title_chip".to_string(),
            "#313335".to_string(),
        ),
        ("color_toolbar_title".to_string(), "#bbbbbb".to_string()),
        ("color_toolbar_meta".to_string(), "#787878".to_string()),
        ("toolbar_title_tracking".to_string(), "0.01em".to_string()),
        (
            "padding_toolbar_status_cluster".to_string(),
            "0".to_string(),
        ),
        ("control_height_button".to_string(), "26px".to_string()),
        ("space_button_inline".to_string(), "6px".to_string()),
        ("button_radius".to_string(), "2px".to_string()),
        ("color_button_text".to_string(), "#a9b7c6".to_string()),
        (
            "color_button_hover".to_string(),
            "alpha(#bbbbbb, 0.08)".to_string(),
        ),
        (
            "color_button_active".to_string(),
            "alpha(#bbbbbb, 0.14)".to_string(),
        ),
        ("context_chip_icon_opacity".to_string(), "0.78".to_string()),
        ("color_muted_action_text".to_string(), "#a9b7c6".to_string()),
        (
            "color_muted_action_hover".to_string(),
            "#45494a".to_string(),
        ),
        (
            "color_muted_action_active".to_string(),
            "#515151".to_string(),
        ),
        ("space_project_chip_inline".to_string(), "12px".to_string()),
        ("flat_chip_height".to_string(), "28px".to_string()),
        ("flat_chip_padding".to_string(), "0 12px".to_string()),
        ("flat_control_height".to_string(), "30px".to_string()),
        ("flat_control_padding".to_string(), "0 14px".to_string()),
        (
            "color_accent_action_bg".to_string(),
            "transparent".to_string(),
        ),
        (
            "color_accent_action_text".to_string(),
            "#a9b7c6".to_string(),
        ),
        (
            "color_accent_action_hover".to_string(),
            "alpha(#bbbbbb, 0.10)".to_string(),
        ),
        ("nav_rail_width".to_string(), "44px".to_string()),
        ("nav_rail_padding".to_string(), "12px 0".to_string()),
        ("color_nav_rail_bg".to_string(), "#313335".to_string()),
        ("space_nav_rail_group".to_string(), "8px".to_string()),
        ("color_nav_separator".to_string(), "#515151".to_string()),
        ("hairline_size".to_string(), "1px".to_string()),
        ("nav_separator_margin".to_string(), "8px 0".to_string()),
        ("nav_button_width".to_string(), "32px".to_string()),
        ("nav_button_height".to_string(), "32px".to_string()),
        ("nav_button_padding".to_string(), "6px".to_string()),
        ("color_nav_button_text".to_string(), "#787878".to_string()),
        ("space_entry_inline".to_string(), "14px".to_string()),
        ("color_entry_bg".to_string(), "#45494a".to_string()),
        ("search_radius".to_string(), "2px".to_string()),
        ("color_search_bg".to_string(), "#45494a".to_string()),
        ("color_search_border".to_string(), "#515151".to_string()),
        ("search_border".to_string(), "1px solid #515151".to_string()),
        ("space_search_inline".to_string(), "16px".to_string()),
        (
            "color_search_focus_border".to_string(),
            "#589df6".to_string(),
        ),
        (
            "search_focus_border".to_string(),
            "1px solid #589df6".to_string(),
        ),
        ("color_search_focus_bg".to_string(), "#45494a".to_string()),
        ("icon_button_width".to_string(), "24px".to_string()),
        ("icon_button_height".to_string(), "24px".to_string()),
        ("icon_button_padding".to_string(), "2px".to_string()),
        ("icon_button_border".to_string(), "0".to_string()),
        (
            "color_icon_button_hover".to_string(),
            "alpha(#bbbbbb, 0.10)".to_string(),
        ),
        (
            "color_icon_button_hover_border".to_string(),
            "transparent".to_string(),
        ),
        ("space_utility_group".to_string(), "6px".to_string()),
        ("color_selection_bg".to_string(), "#2d5c88".to_string()),
        ("color_selection_text".to_string(), "#ffffff".to_string()),
        ("color_separator_fill".to_string(), "#323232".to_string()),
        ("separator_alpha".to_string(), "1.0".to_string()),
        ("separator_size".to_string(), "1px".to_string()),
        ("paned_separator_size".to_string(), "3px".to_string()),
        ("drop_zone_width".to_string(), "40px".to_string()),
        ("drop_zone_height".to_string(), "40px".to_string()),
        ("drop_zone_side_width".to_string(), "72px".to_string()),
        ("drop_zone_bottom_height".to_string(), "64px".to_string()),
        ("color_drop_zone_fill".to_string(), "#589df6".to_string()),
        ("drop_zone_fill_alpha".to_string(), "0.18".to_string()),
        ("color_drop_zone_border".to_string(), "#589df6".to_string()),
        ("drop_zone_border_alpha".to_string(), "0.48".to_string()),
        (
            "drop_zone_border".to_string(),
            "1px solid alpha(#589df6, 0.48)".to_string(),
        ),
        ("tab_radius".to_string(), "0".to_string()),
        ("notebook_tab_padding".to_string(), "0 14px".to_string()),
        ("notebook_tab_border_width".to_string(), "1px".to_string()),
        (
            "color_notebook_tab_bg".to_string(),
            "transparent".to_string(),
        ),
        ("color_notebook_tab_text".to_string(), "#787878".to_string()),
        (
            "color_notebook_tab_hover".to_string(),
            "alpha(#bbbbbb, 0.06)".to_string(),
        ),
        (
            "color_notebook_tab_hover_border".to_string(),
            "alpha(#bbbbbb, 0.10)".to_string(),
        ),
        (
            "color_notebook_tab_hover_text".to_string(),
            "#a9b7c6".to_string(),
        ),
        (
            "color_notebook_tab_active".to_string(),
            "transparent".to_string(),
        ),
        (
            "notebook_tab_active_border_width".to_string(),
            "2px".to_string(),
        ),
        (
            "color_notebook_tab_active_border".to_string(),
            "#4b6eaf".to_string(),
        ),
        (
            "color_notebook_tab_drop_border".to_string(),
            "#589df6".to_string(),
        ),
        (
            "notebook_tab_drop_shadow".to_string(),
            "inset 0 -2px #589df6".to_string(),
        ),
        ("color_editor_tab_bg".to_string(), "#313335".to_string()),
        ("color_editor_tab_active".to_string(), "#3c3f41".to_string()),
        ("tool_tab_height".to_string(), "30px".to_string()),
        ("space_tab_header".to_string(), "8px".to_string()),
        ("tab_close_width".to_string(), "22px".to_string()),
        ("tab_close_height".to_string(), "22px".to_string()),
        ("tab_close_padding".to_string(), "4px".to_string()),
        ("color_tab_close_text".to_string(), "#787878".to_string()),
        ("color_tab_close_hover".to_string(), "#515151".to_string()),
        ("color_preview_bg".to_string(), "#2b2b2b".to_string()),
        ("color_preview_border".to_string(), "#515151".to_string()),
        ("preview_border_alpha".to_string(), "0.18".to_string()),
        (
            "preview_top_border".to_string(),
            "1px solid alpha(#515151, 0.14)".to_string(),
        ),
        ("color_preview_header_bg".to_string(), "#313335".to_string()),
        (
            "color_tab_strip_scroller_bg".to_string(),
            "#3c3f41".to_string(),
        ),
        ("tab_strip_border_alpha".to_string(), "0.12".to_string()),
        (
            "tab_strip_scroller_border".to_string(),
            "1px solid #323232".to_string(),
        ),
        ("tab_strip_height".to_string(), "26px".to_string()),
        (
            "color_workbench_tab_bg".to_string(),
            "transparent".to_string(),
        ),
        (
            "color_workbench_tab_text".to_string(),
            "#787878".to_string(),
        ),
        (
            "color_workbench_tab_hover".to_string(),
            "alpha(#bbbbbb, 0.06)".to_string(),
        ),
        (
            "color_workbench_tab_hover_text".to_string(),
            "#a9b7c6".to_string(),
        ),
        (
            "color_workbench_tab_active".to_string(),
            "transparent".to_string(),
        ),
        ("workbench_tab_padding".to_string(), "0 0 0 8px".to_string()),
        ("workbench_tab_border_width".to_string(), "2px".to_string()),
        ("color_drag_preview_bg".to_string(), "#45494a".to_string()),
        (
            "color_drag_preview_border".to_string(),
            "#515151".to_string(),
        ),
        (
            "drag_preview_shadow".to_string(),
            "0 16px 32px alpha(black, 0.32)".to_string(),
        ),
        ("drag_preview_opacity".to_string(), "0.96".to_string()),
        ("tab_drag_gap_width".to_string(), "56px".to_string()),
        ("tab_drag_gap_height".to_string(), "34px".to_string()),
        (
            "tab_drag_gap_border".to_string(),
            "1px dashed alpha(#a9b7c6, 0.58)".to_string(),
        ),
        (
            "tab_drag_gap_bg".to_string(),
            "alpha(#a9b7c6, 0.1)".to_string(),
        ),
        ("workbench_split_margin".to_string(), "10px".to_string()),
        (
            "workbench_split_fill".to_string(),
            "alpha(#4b6eaf, 0.18)".to_string(),
        ),
        (
            "workbench_split_border".to_string(),
            "1px solid alpha(#a9b7c6, 0.42)".to_string(),
        ),
        ("split_preview_side_width".to_string(), "120px".to_string()),
        (
            "split_preview_bottom_height".to_string(),
            "96px".to_string(),
        ),
        ("panel_title_tracking".to_string(), "0.04em".to_string()),
        ("panel_title_color".to_string(), "#787878".to_string()),
        ("section_title_tracking".to_string(), "0.08em".to_string()),
        ("panel_content_padding".to_string(), "16px".to_string()),
        ("dense_row_height".to_string(), "36px".to_string()),
        ("dense_row_selected_bg".to_string(), "#2d5c88".to_string()),
        ("dense_row_selected_text".to_string(), "#ffffff".to_string()),
        (
            "dense_row_hover_bg".to_string(),
            "alpha(#2d5c88, 0.3)".to_string(),
        ),
        ("list_row_padding".to_string(), "10px 14px".to_string()),
        ("inspector_value_color".to_string(), "#bbbbbb".to_string()),
        ("status_badge_padding".to_string(), "4px 8px".to_string()),
        ("color_status_badge_bg".to_string(), "#515151".to_string()),
        ("color_status_badge_text".to_string(), "#bbbbbb".to_string()),
        ("color_status_idle_bg".to_string(), "#45494a".to_string()),
        ("color_status_idle_text".to_string(), "#a9b7c6".to_string()),
        ("color_status_loaded_bg".to_string(), "#45493a".to_string()),
        (
            "color_status_loaded_text".to_string(),
            "#f0cf7c".to_string(),
        ),
        ("color_status_running_bg".to_string(), "#3a4a40".to_string()),
        (
            "color_status_running_text".to_string(),
            "#83d0ae".to_string(),
        ),
        ("color_textview_text".to_string(), "#a9b7c6".to_string()),
        (
            "color_tool_window_surface".to_string(),
            "#3c3f41".to_string(),
        ),
        (
            "color_scrollbar_trough".to_string(),
            "transparent".to_string(),
        ),
        ("scrollbar_slider_size".to_string(), "8px".to_string()),
        (
            "color_scrollbar_slider".to_string(),
            "alpha(#888888, 0.3)".to_string(),
        ),
        (
            "color_scrollbar_slider_hover".to_string(),
            "alpha(#888888, 0.5)".to_string(),
        ),
        ("status_bar_height".to_string(), "32px".to_string()),
        ("color_status_bar_bg".to_string(), "#3c3f41".to_string()),
        ("color_status_item".to_string(), "#787878".to_string()),
        (
            "color_status_item_strong".to_string(),
            "#a9b7c6".to_string(),
        ),
        ("plugin_search_margin_bottom".to_string(), "2px".to_string()),
        (
            "plugin_list_row_border".to_string(),
            "1px solid alpha(#515151, 0.35)".to_string(),
        ),
        (
            "plugin_list_row_padding".to_string(),
            "10px 12px".to_string(),
        ),
        (
            "color_plugin_source_badge_bg".to_string(),
            "#3a4a5f".to_string(),
        ),
        (
            "color_plugin_source_badge_text".to_string(),
            "#a9b7c6".to_string(),
        ),
        ("color_plugin_hero_bg".to_string(), "#45494a".to_string()),
        ("plugin_hero_padding".to_string(), "14px".to_string()),
        ("color_plugin_name".to_string(), "#bbbbbb".to_string()),
        (
            "color_plugin_description".to_string(),
            "#a9b7c6".to_string(),
        ),
        ("color_plugin_overview".to_string(), "#787878".to_string()),
        (
            "plugin_footer_padding".to_string(),
            "8px 4px 4px 0".to_string(),
        ),
        ("color_popover_bg".to_string(), "#45494a".to_string()),
        (
            "popover_border".to_string(),
            "1px solid #515151".to_string(),
        ),
        ("popover_content_padding".to_string(), "4px".to_string()),
        ("popover_button_height".to_string(), "25px".to_string()),
        ("popover_button_padding".to_string(), "3px 12px".to_string()),
        (
            "color_popover_button_text".to_string(),
            "#bbbbbb".to_string(),
        ),
        (
            "color_popover_button_hover".to_string(),
            "#4b6eaf".to_string(),
        ),
        (
            "color_popover_button_hover_text".to_string(),
            "#ffffff".to_string(),
        ),
        (
            "color_popover_button_disabled".to_string(),
            "#515151".to_string(),
        ),
        ("color_popover_separator".to_string(), "#515151".to_string()),
        ("popover_separator_margin".to_string(), "4px 0".to_string()),
    ])
}

fn render_appearance_stylesheet(spec: &ThemeSpec) -> String {
    let mut css = String::new();

    for (id, appearance) in &spec.appearances.surfaces {
        css.push_str(&render_surface_css(spec, id, appearance));
    }
    for (id, appearance) in &spec.appearances.buttons {
        css.push_str(&render_button_css(spec, id, appearance));
    }
    for (id, appearance) in &spec.appearances.text {
        css.push_str(&render_text_css(spec, id, appearance));
    }
    for (id, appearance) in &spec.appearances.inputs {
        css.push_str(&render_input_css(spec, id, appearance));
    }
    for (id, appearance) in &spec.appearances.tab_strips {
        css.push_str(&render_tab_strip_css(spec, id, appearance));
    }

    css
}

fn render_surface_css(spec: &ThemeSpec, id: &str, appearance: &SurfaceAppearance) -> String {
    let class = surface_css_class(id);
    let colors = surface_colors(spec, appearance.tone, appearance.level);
    let text = text_style(spec, appearance.text_role, appearance.tone);
    let border = if appearance.border {
        format!("1px solid {}", colors.border)
    } else {
        "0".to_string()
    };
    format!(
        "
.{class} {{
  background: {bg};
  color: {fg};
  border-color: {border_color};
  border: {border};
}}
.{class} label,
.{class} text,
.{class} image {{
  color: {fg};
}}
.{class}.panel-header {{
  min-height: {header_height}px;
}}
",
        bg = colors.background,
        fg = colors.foreground,
        border_color = colors.border,
        border = border,
        header_height = spec.density.panel_header_height,
    ) + &format_text_css_block(&format!(".{class}"), &text)
}

fn render_button_css(spec: &ThemeSpec, id: &str, appearance: &ButtonAppearance) -> String {
    let class = button_css_class(id);
    let text = text_style(spec, appearance.text_role, appearance.tone);
    let base = surface_colors(spec, appearance.tone, SurfaceLevel::Raised);
    let soft = surface_colors(spec, appearance.tone, SurfaceLevel::Flat);
    let (background, border, foreground, hover, active) = match appearance.style {
        ButtonStyle::Solid => (
            base.background.clone(),
            base.border.clone(),
            base.foreground.clone(),
            lighten(&base.background, 0.08),
            darken(&base.background, 0.06),
        ),
        ButtonStyle::Soft => (
            blend(&soft.background, &base.background, 0.35),
            base.border.clone(),
            soft.foreground.clone(),
            blend(&soft.background, &base.background, 0.55),
            blend(&soft.background, &base.background, 0.70),
        ),
        ButtonStyle::Ghost => (
            "transparent".to_string(),
            "transparent".to_string(),
            soft.foreground.clone(),
            blend(&soft.background, &base.background, 0.35),
            blend(&soft.background, &base.background, 0.55),
        ),
        ButtonStyle::Outline => (
            "transparent".to_string(),
            base.border.clone(),
            base.foreground.clone(),
            blend(&soft.background, &base.background, 0.20),
            blend(&soft.background, &base.background, 0.35),
        ),
    };

    format!(
        "
button.{class} {{
  min-height: {height}px;
  padding: 0 {padding}px;
  border-radius: {radius}px;
  background: {background};
  border: 1px solid {border};
  color: {foreground};
}}
button.{class}:hover {{
  background: {hover};
}}
button.{class}:active,
button.{class}:checked {{
  background: {active};
}}
button.{class} label,
button.{class} image {{
  color: {foreground};
}}
",
        height = spec.density.control_height_medium,
        padding = spec.density.space_xl,
        radius = spec.density.radius_medium,
        background = background,
        border = border,
        foreground = foreground,
        hover = hover,
        active = active,
    ) + &format_text_css_block(&format!("button.{class} label"), &text)
}

fn render_text_css(spec: &ThemeSpec, id: &str, appearance: &TextAppearance) -> String {
    let class = text_css_class(id);
    let text = text_style(spec, appearance.role, appearance.tone);
    format_text_css_block(&format!(".{class}"), &text)
}

fn render_input_css(spec: &ThemeSpec, id: &str, appearance: &InputAppearance) -> String {
    let class = input_css_class(id);
    let colors = surface_colors(spec, appearance.tone, appearance.level);
    let text = text_style(spec, appearance.text_role, appearance.tone);
    let focus = surface_colors(spec, Tone::Accent, SurfaceLevel::Raised);
    format!(
        "
entry.{class} {{
  min-height: {height}px;
  padding: 0 {padding}px;
  border-radius: {radius}px;
  background: {background};
  border: 1px solid {border};
  color: {foreground};
}}
entry.{class}:focus {{
  border-color: {focus};
}}
",
        height = spec.density.control_height_medium,
        padding = spec.density.space_xl,
        radius = spec.density.radius_medium,
        background = colors.background,
        border = colors.border,
        foreground = colors.foreground,
        focus = focus.border,
    ) + &format_text_css_block(&format!("entry.{class}"), &text)
}

fn render_tab_strip_css(spec: &ThemeSpec, id: &str, appearance: &TabStripAppearance) -> String {
    let class = tab_strip_css_class(id);
    let surface = surface_colors(spec, appearance.tone, SurfaceLevel::Flat);
    let active = surface_colors(spec, Tone::Accent, SurfaceLevel::Raised);
    let accent = tone_color(&spec.palette, Tone::Accent);
    let focused_border = accent;
    let unfocused_border = blend(&focused_border, &surface.background, 0.6);
    let hover = blend(&surface.background, &active.background, 0.22);
    let text = text_style(spec, appearance.text_role, appearance.tone);
    let active_background = match appearance.style {
        TabStripStyle::Editor => "transparent".to_string(),
        TabStripStyle::Utility | TabStripStyle::Console => {
            lighten(&surface.background, 0.08)
        }
    };
    let tab_height = match appearance.style {
        TabStripStyle::Editor => spec.density.tab_height,
        TabStripStyle::Utility => spec.density.tab_height.saturating_sub(2),
        TabStripStyle::Console => spec.density.tab_height.saturating_sub(2),
    };
    format!(
        "
notebook.{class} > header,
.workbench-tab-strip-scroller.{class} {{
  background: {strip_bg};
  border-bottom: 1px solid {strip_border};
}}
notebook.{class} > header tabs tab {{
  min-height: {tab_height}px;
  background: transparent;
  color: {text_color};
}}
notebook.{class} > header tabs tab:hover {{
  background: {hover};
  color: {text_color};
}}
notebook.{class} > header tabs tab:checked {{
  background: {active_bg};
  border-bottom-color: {unfocused_border};
  color: {active_fg};
}}
.pane-focused notebook.{class} > header tabs tab:checked {{
  border-bottom-color: {active_border};
}}
.workbench-tab-strip.{class} > .tab-header {{
  min-height: {tab_height}px;
  background: transparent;
  color: {text_color};
}}
.workbench-tab-strip.{class} > .tab-header:hover {{
  background: {hover};
  color: {text_color};
}}
.workbench-tab-strip.{class} > .tab-header.active {{
  background: {active_bg};
  border-bottom-color: {unfocused_border};
  color: {active_fg};
}}
.pane-focused .workbench-tab-strip.{class} > .tab-header.active {{
  border-bottom-color: {active_border};
}}
",
        strip_bg = surface.background.clone(),
        strip_border = surface.border.clone(),
        tab_height = tab_height,
        text_color = text.color.clone(),
        hover = hover,
        active_bg = active_background,
        active_border = focused_border,
        unfocused_border = unfocused_border,
        active_fg = active.foreground.clone(),
    ) + &format_text_css_block(&format!(".workbench-tab-strip.{class} > .tab-header .tab-label"), &text)
}

struct SurfaceColors {
    background: String,
    foreground: String,
    border: String,
}

struct ResolvedTextStyle {
    color: String,
    font_family: String,
    font_size_px: u16,
    font_weight: u16,
    letter_spacing: &'static str,
    text_transform: &'static str,
}

fn surface_colors(spec: &ThemeSpec, tone: Tone, level: SurfaceLevel) -> SurfaceColors {
    let base = tone_color(&spec.palette, tone);
    let background = match level {
        SurfaceLevel::Flat => blend(&base, &spec.palette.bg_0, 0.22),
        SurfaceLevel::Raised => blend(&base, &spec.palette.bg_1, 0.10),
        SurfaceLevel::Sunken => darken(&blend(&base, &spec.palette.workbench, 0.22), 0.05),
    };
    let border = darken(&blend(&background, &spec.palette.border_strong, 0.35), 0.08);
    let foreground = readable_text_for(&background, &spec.palette.text_0, &spec.palette.bg_0);
    SurfaceColors {
        background,
        foreground,
        border,
    }
}

fn text_style(spec: &ThemeSpec, role: TextRole, tone: Tone) -> ResolvedTextStyle {
    let tone_color = tone_color(&spec.palette, tone);
    let color = match role {
        TextRole::Title | TextRole::BodyStrong | TextRole::TabLabel | TextRole::Code => {
            readable_text_for(&tone_color, &spec.palette.text_0, &spec.palette.bg_0)
        }
        TextRole::Subtitle | TextRole::Body => spec.palette.text_1.clone(),
        TextRole::Meta | TextRole::SectionLabel => spec.palette.text_2.clone(),
    };
    let (font_size_px, font_weight, letter_spacing, text_transform, font_family) = match role {
        TextRole::Title => (
            spec.typography.font_size_title,
            700,
            "0",
            "none",
            spec.typography.font_family.clone(),
        ),
        TextRole::Subtitle => (
            spec.typography.font_size_base,
            600,
            "0",
            "none",
            spec.typography.font_family.clone(),
        ),
        TextRole::Body => (
            spec.typography.font_size_ui,
            400,
            "0",
            "none",
            spec.typography.font_family.clone(),
        ),
        TextRole::BodyStrong => (
            spec.typography.font_size_ui,
            600,
            "0",
            "none",
            spec.typography.font_family.clone(),
        ),
        TextRole::Meta => (
            spec.typography.font_size_small,
            500,
            "0",
            "none",
            spec.typography.font_family.clone(),
        ),
        TextRole::SectionLabel => (
            spec.typography.font_size_tiny,
            700,
            "0.08em",
            "uppercase",
            spec.typography.font_family.clone(),
        ),
        TextRole::TabLabel => (
            spec.typography.font_size_ui,
            500,
            "0",
            "none",
            spec.typography.font_family.clone(),
        ),
        TextRole::Code => (
            spec.typography.font_size_ui,
            400,
            "0",
            "none",
            spec.typography.mono_font_family.clone(),
        ),
    };
    ResolvedTextStyle {
        color,
        font_family,
        font_size_px,
        font_weight,
        letter_spacing,
        text_transform,
    }
}

fn format_text_css_block(selector: &str, style: &ResolvedTextStyle) -> String {
    format!(
        "
{selector} {{
  color: {color};
  font-family: {font_family};
  font-size: {font_size}px;
  font-weight: {font_weight};
  letter-spacing: {letter_spacing};
  text-transform: {text_transform};
}}
",
        selector = selector,
        color = style.color,
        font_family = style.font_family,
        font_size = style.font_size_px,
        font_weight = style.font_weight,
        letter_spacing = style.letter_spacing,
        text_transform = style.text_transform,
    )
}

fn tone_color(palette: &ThemePalette, tone: Tone) -> String {
    match tone {
        Tone::Neutral => palette.bg_1.clone(),
        Tone::Primary => palette.panel_left.clone(),
        Tone::Secondary => palette.panel_right.clone(),
        Tone::Tertiary => palette.panel_bottom.clone(),
        Tone::Accent => palette.accent.clone(),
        Tone::Success => "#3f8f5a".to_string(),
        Tone::Warning => "#c38b2e".to_string(),
        Tone::Danger => "#b85151".to_string(),
    }
}

fn readable_text_for(background: &str, light: &str, dark: &str) -> String {
    if is_light_color(background) {
        dark.to_string()
    } else {
        light.to_string()
    }
}

fn is_light_color(color: &str) -> bool {
    parse_hex_color(color)
        .map(|(r, g, b)| (0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64) / 255.0 > 0.62)
        .unwrap_or(false)
}

fn blend(a: &str, b: &str, ratio: f32) -> String {
    match (parse_hex_color(a), parse_hex_color(b)) {
        (Some((ar, ag, ab)), Some((br, bg, bb))) => format!(
            "#{:02x}{:02x}{:02x}",
            mix_channel(ar, br, ratio),
            mix_channel(ag, bg, ratio),
            mix_channel(ab, bb, ratio)
        ),
        _ => a.to_string(),
    }
}

fn lighten(color: &str, amount: f32) -> String {
    blend(color, "#ffffff", amount)
}

fn darken(color: &str, amount: f32) -> String {
    blend(color, "#000000", amount)
}

fn mix_channel(a: u8, b: u8, ratio: f32) -> u8 {
    ((a as f32 * (1.0 - ratio)) + (b as f32 * ratio)).round() as u8
}

fn parse_hex_color(color: &str) -> Option<(u8, u8, u8)> {
    let value = color.trim();
    let hex = value.strip_prefix('#')?;
    match hex.len() {
        6 => Some((
            u8::from_str_radix(&hex[0..2], 16).ok()?,
            u8::from_str_radix(&hex[2..4], 16).ok()?,
            u8::from_str_radix(&hex[4..6], 16).ok()?,
        )),
        3 => Some((
            u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?,
            u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?,
            u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?,
        )),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_builtin_template_with_default_tokens() {
        let stylesheet = build_stylesheet(&ThemeSpec::default()).expect("default theme renders");
        assert!(stylesheet.contains("@define-color bg_0 #3c3f41;"));
        assert!(stylesheet.contains("font-size: 12px;"));
        assert!(!stylesheet.contains("{{"));
    }

    #[test]
    fn allows_custom_template_overrides() {
        let theme = ThemeSpec::default()
            .with_override("hero_padding", "42px")
            .with_override("hero_color", "#abcdef");
        let rendered = render_template(
            ".hero { padding: {{hero_padding}}; color: {{hero_color}}; }",
            &theme.token_map(),
        )
        .expect("override tokens render");
        assert_eq!(rendered, ".hero { padding: 42px; color: #abcdef; }");
    }

    #[test]
    fn renders_custom_semantic_appearance_classes() {
        let stylesheet = build_stylesheet(
            &ThemeSpec::default()
                .with_surface_appearance(
                    "secondary-panel",
                    SurfaceAppearance::new(Tone::Secondary, SurfaceLevel::Raised, TextRole::Body),
                )
                .with_button_appearance(
                    "danger-action",
                    ButtonAppearance::new(Tone::Danger, ButtonStyle::Soft, TextRole::BodyStrong),
                ),
        )
        .expect("appearance stylesheet renders");

        assert!(stylesheet.contains(".mz-surface-secondary-panel"));
        assert!(stylesheet.contains("button.mz-button-danger-action"));
    }
}
