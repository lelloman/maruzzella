use serde::{Deserialize, Serialize};

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TabSpec {
    pub id: String,
    pub panel_id: String,
    pub title: String,
    pub view_kind: String,
    pub instance_key: Option<String>,
    pub content_kind: PanelContentKind,
    pub placeholder: String,
    pub closable: bool,
    pub close_prompt: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TabGroupSpec {
    pub id: String,
    pub active_tab_id: Option<String>,
    pub tabs: Vec<TabSpec>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShellSpec {
    pub title: String,
    pub search_placeholder: String,
    pub status_text: String,
    pub menu_roots: Vec<MenuRootSpec>,
    pub menu_items: Vec<MenuItemSpec>,
    pub commands: Vec<CommandSpec>,
    pub toolbar_items: Vec<ToolbarItemSpec>,
    pub left_panel: TabGroupSpec,
    pub right_panel: TabGroupSpec,
    pub bottom_panel: TabGroupSpec,
    pub workbench: WorkbenchNodeSpec,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WorkbenchNodeSpec {
    Group(TabGroupSpec),
    Split {
        axis: SplitAxis,
        children: Vec<WorkbenchNodeSpec>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandSpec {
    pub id: String,
    pub title: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolbarItemSpec {
    pub id: String,
    pub icon_name: Option<String>,
    pub label: Option<String>,
    pub command_id: String,
    pub secondary: bool,
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
}

impl TabGroupSpec {
    pub fn new(id: &str, active_tab_id: Option<&str>, tabs: Vec<TabSpec>) -> Self {
        Self {
            id: id.to_string(),
            active_tab_id: active_tab_id.map(str::to_string),
            tabs,
        }
    }
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
        instance_key: None,
        content_kind: PanelContentKind::TextBuffer,
        placeholder: body.to_string(),
        closable,
        close_prompt: None,
    }
}
