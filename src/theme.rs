use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use gtk::gdk::Display;
use gtk::{CssProvider, STYLE_PROVIDER_PRIORITY_USER};

const STYLE_TEMPLATE: &str = include_str!("../resources/style.css");

#[derive(Clone, Debug)]
pub struct ThemeSpec {
    pub stylesheet: ThemeStylesheet,
    pub palette: ThemePalette,
    pub typography: ThemeTypography,
    pub density: ThemeDensity,
    pub overrides: BTreeMap<String, String>,
}

impl Default for ThemeSpec {
    fn default() -> Self {
        Self {
            stylesheet: ThemeStylesheet::Bundled,
            palette: ThemePalette::default(),
            typography: ThemeTypography::default(),
            density: ThemeDensity::default(),
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

    render_template(&template, &spec.token_map())
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
        ("tab_strip_height".to_string(), "42px".to_string()),
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
        ("workbench_tab_padding".to_string(), "0 16px".to_string()),
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
}
