use serde::{Deserialize, Serialize};

fn default_shell_surface_appearance() -> String {
    "app-shell".to_string()
}

fn default_topbar_appearance() -> String {
    "topbar".to_string()
}

fn default_menu_appearance() -> String {
    "menu".to_string()
}

fn default_toolbar_appearance() -> String {
    "toolbar".to_string()
}

fn default_search_input_appearance() -> String {
    "search".to_string()
}

fn default_status_appearance() -> String {
    "status".to_string()
}

fn default_panel_appearance() -> String {
    "primary".to_string()
}

fn default_panel_header_appearance() -> String {
    "secondary".to_string()
}

fn default_tab_strip_appearance() -> String {
    "utility".to_string()
}

fn default_text_appearance() -> String {
    "body".to_string()
}

fn default_button_appearance() -> String {
    "secondary".to_string()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PanelContentKind {
    NavigationList,
    IdentityList,
    InspectorDetails,
    CommandList,
    TextBuffer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplitAxis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BottomPanelLayout {
    CenterOnly,
    FullWidth,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum PanelResizePolicy {
    /// Panel resizes proportionally with the window, no cap.
    Proportional,
    /// Panel resizes proportionally up to min_size * max_factor, then stops.
    CappedProportional { max_factor: f64 },
    /// Panel stays at its minimum size; the workbench takes all extra space.
    Fixed,
}

impl Default for PanelResizePolicy {
    fn default() -> Self {
        PanelResizePolicy::Proportional
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TabSpec {
    pub id: String,
    pub panel_id: String,
    pub title: String,
    pub view_kind: String,
    pub plugin_view_id: Option<String>,
    pub instance_key: Option<String>,
    #[serde(default)]
    pub payload: Vec<u8>,
    pub content_kind: PanelContentKind,
    pub placeholder: String,
    pub closable: bool,
    pub close_prompt: Option<String>,
    #[serde(default = "default_text_appearance")]
    pub text_appearance_id: String,
}

impl TabSpec {
    pub fn with_text_appearance(mut self, appearance_id: impl Into<String>) -> Self {
        self.text_appearance_id = appearance_id.into();
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TabGroupSpec {
    pub id: String,
    pub active_tab_id: Option<String>,
    #[serde(default = "default_show_tab_strip")]
    pub show_tab_strip: bool,
    #[serde(default = "default_panel_appearance")]
    pub panel_appearance_id: String,
    #[serde(default = "default_panel_header_appearance")]
    pub panel_header_appearance_id: String,
    #[serde(default = "default_tab_strip_appearance")]
    pub tab_strip_appearance_id: String,
    #[serde(default = "default_text_appearance")]
    pub text_appearance_id: String,
    pub tabs: Vec<TabSpec>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShellSpec {
    pub title: String,
    pub search_placeholder: String,
    pub search_command_id: Option<String>,
    pub status_text: String,
    #[serde(default = "default_shell_surface_appearance")]
    pub app_appearance_id: String,
    #[serde(default = "default_topbar_appearance")]
    pub topbar_appearance_id: String,
    #[serde(default = "default_menu_appearance")]
    pub menu_appearance_id: String,
    #[serde(default = "default_toolbar_appearance")]
    pub toolbar_appearance_id: String,
    #[serde(default = "default_search_input_appearance")]
    pub search_input_appearance_id: String,
    #[serde(default = "default_status_appearance")]
    pub status_appearance_id: String,
    #[serde(default = "default_button_appearance")]
    pub button_appearance_id: String,
    #[serde(default = "default_text_appearance")]
    pub text_appearance_id: String,
    pub bottom_panel_layout: BottomPanelLayout,
    pub menu_roots: Vec<MenuRootSpec>,
    pub menu_items: Vec<MenuItemSpec>,
    pub commands: Vec<CommandSpec>,
    pub toolbar_items: Vec<ToolbarItemSpec>,
    pub left_panel: TabGroupSpec,
    pub right_panel: TabGroupSpec,
    pub bottom_panel: TabGroupSpec,
    pub workbench: WorkbenchNodeSpec,
    #[serde(default)]
    pub left_panel_resize: PanelResizePolicy,
    #[serde(default)]
    pub right_panel_resize: PanelResizePolicy,
    #[serde(default)]
    pub bottom_panel_resize: PanelResizePolicy,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WorkbenchNodeSpec {
    Group(TabGroupSpec),
    Split {
        axis: SplitAxis,
        children: Vec<WorkbenchNodeSpec>,
    },
}

pub fn make_workbench_tabs_closeable(node: &mut WorkbenchNodeSpec) {
    match node {
        WorkbenchNodeSpec::Group(group) => {
            for tab in &mut group.tabs {
                tab.closable = true;
            }
        }
        WorkbenchNodeSpec::Split { children, .. } => {
            for child in children {
                make_workbench_tabs_closeable(child);
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandSpec {
    pub id: String,
    pub title: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolbarDisplayMode {
    IconOnly,
    #[default]
    IconAndText,
    TextOnly,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolbarItemSpec {
    pub id: String,
    pub icon_name: Option<String>,
    pub label: Option<String>,
    pub command_id: String,
    #[serde(default)]
    pub payload: Vec<u8>,
    pub secondary: bool,
    #[serde(default)]
    pub display_mode: ToolbarDisplayMode,
    #[serde(default = "default_button_appearance")]
    pub appearance_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MenuRootSpec {
    pub id: String,
    pub label: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MenuItemSpec {
    pub id: String,
    pub root_id: String,
    pub label: String,
    pub command_id: String,
    #[serde(default)]
    pub payload: Vec<u8>,
}

impl TabGroupSpec {
    pub fn new(id: &str, active_tab_id: Option<&str>, tabs: Vec<TabSpec>) -> Self {
        Self {
            id: id.to_string(),
            active_tab_id: active_tab_id.map(str::to_string),
            show_tab_strip: true,
            panel_appearance_id: default_panel_appearance(),
            panel_header_appearance_id: default_panel_header_appearance(),
            tab_strip_appearance_id: default_tab_strip_appearance(),
            text_appearance_id: default_text_appearance(),
            tabs,
        }
    }

    pub fn with_tab_strip_hidden(mut self) -> Self {
        self.show_tab_strip = false;
        self
    }

    pub fn with_panel_appearance(mut self, appearance_id: impl Into<String>) -> Self {
        self.panel_appearance_id = appearance_id.into();
        self
    }

    pub fn with_panel_header_appearance(mut self, appearance_id: impl Into<String>) -> Self {
        self.panel_header_appearance_id = appearance_id.into();
        self
    }

    pub fn with_tab_strip_appearance(mut self, appearance_id: impl Into<String>) -> Self {
        self.tab_strip_appearance_id = appearance_id.into();
        self
    }

    pub fn with_text_appearance(mut self, appearance_id: impl Into<String>) -> Self {
        self.text_appearance_id = appearance_id.into();
        self
    }
}

fn default_show_tab_strip() -> bool {
    true
}

pub fn command_name(command_id: &str) -> String {
    command_id.replace('.', "-")
}

pub fn menu_action_ref(command_id: &str) -> String {
    format!("win.{}", command_name(command_id))
}

pub fn text_tab(id: &str, panel_id: &str, title: &str, body: &str, closable: bool) -> TabSpec {
    TabSpec {
        id: id.to_string(),
        panel_id: panel_id.to_string(),
        title: title.to_string(),
        view_kind: "text".to_string(),
        plugin_view_id: None,
        instance_key: None,
        payload: Vec::new(),
        content_kind: PanelContentKind::TextBuffer,
        placeholder: body.to_string(),
        closable,
        close_prompt: None,
        text_appearance_id: default_text_appearance(),
    }
}

pub fn plugin_tab(
    id: &str,
    panel_id: &str,
    title: &str,
    plugin_view_id: &str,
    placeholder: &str,
    closable: bool,
) -> TabSpec {
    TabSpec {
        id: id.to_string(),
        panel_id: panel_id.to_string(),
        title: title.to_string(),
        view_kind: "plugin".to_string(),
        plugin_view_id: Some(plugin_view_id.to_string()),
        instance_key: None,
        payload: Vec::new(),
        content_kind: PanelContentKind::TextBuffer,
        placeholder: placeholder.to_string(),
        closable,
        close_prompt: None,
        text_appearance_id: default_text_appearance(),
    }
}

pub fn plugin_tab_with_instance(
    id: &str,
    panel_id: &str,
    title: &str,
    plugin_view_id: &str,
    instance_key: Option<&str>,
    payload: impl Into<Vec<u8>>,
    placeholder: &str,
    closable: bool,
) -> TabSpec {
    TabSpec {
        id: id.to_string(),
        panel_id: panel_id.to_string(),
        title: title.to_string(),
        view_kind: "plugin".to_string(),
        plugin_view_id: Some(plugin_view_id.to_string()),
        instance_key: instance_key.map(str::to_string),
        payload: payload.into(),
        content_kind: PanelContentKind::TextBuffer,
        placeholder: placeholder.to_string(),
        closable,
        close_prompt: None,
        text_appearance_id: default_text_appearance(),
    }
}
