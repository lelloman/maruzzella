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

pub fn default_shell_spec() -> ShellSpec {
    let commands = vec![
        CommandSpec { id: "shell.open_command_palette".to_string(), title: "Command Palette".to_string() },
        CommandSpec { id: "shell.reload_theme".to_string(), title: "Reload Theme".to_string() },
        CommandSpec { id: "shell.about".to_string(), title: "About Maruzzella".to_string() },
    ];
    let toolbar_items = vec![
        ToolbarItemSpec {
            id: "palette".to_string(),
            icon_name: Some("system-search-symbolic".to_string()),
            label: Some("Palette".to_string()),
            command_id: "shell.open_command_palette".to_string(),
            secondary: false,
        },
        ToolbarItemSpec {
            id: "theme".to_string(),
            icon_name: Some("applications-graphics-symbolic".to_string()),
            label: None,
            command_id: "shell.reload_theme".to_string(),
            secondary: true,
        },
        ToolbarItemSpec {
            id: "about".to_string(),
            icon_name: Some("help-about-symbolic".to_string()),
            label: None,
            command_id: "shell.about".to_string(),
            secondary: true,
        },
    ];
    let menu_roots = vec![
        MenuRootSpec { id: "app".to_string(), label: "Maruzzella".to_string() },
        MenuRootSpec { id: "view".to_string(), label: "View".to_string() },
    ];
    let menu_items = vec![
        MenuItemSpec {
            id: "command-palette".to_string(),
            root_id: "app".to_string(),
            label: "Command Palette".to_string(),
            command_id: "shell.open_command_palette".to_string(),
        },
        MenuItemSpec {
            id: "about".to_string(),
            root_id: "app".to_string(),
            label: "About Maruzzella".to_string(),
            command_id: "shell.about".to_string(),
        },
        MenuItemSpec {
            id: "reload-theme".to_string(),
            root_id: "view".to_string(),
            label: "Reload Theme".to_string(),
            command_id: "shell.reload_theme".to_string(),
        },
    ];
    let left_panel = TabGroupSpec::new(
        "panel-left",
        Some("navigation"),
        vec![
            text_tab("navigation", "panel-left", "Navigation", "Anonymous shell navigation goes here.", false),
            text_tab("library", "panel-left", "Library", "A product can mount its own content here.", false),
        ],
    );
    let right_panel = TabGroupSpec::new(
        "panel-right",
        Some("inspector"),
        vec![
            text_tab("inspector", "panel-right", "Inspector", "Selection-aware details live here.", false),
            text_tab("outline", "panel-right", "Outline", "Structure and metadata panels fit here.", false),
        ],
    );
    let bottom_panel = TabGroupSpec::new(
        "panel-bottom",
        Some("logs"),
        vec![
            text_tab("logs", "panel-bottom", "Logs", "Runtime output, tasks, and traces.", false),
            text_tab("problems", "panel-bottom", "Problems", "Validation and build output.", false),
        ],
    );
    let workbench = WorkbenchNodeSpec::Split {
        axis: SplitAxis::Horizontal,
        children: vec![
            WorkbenchNodeSpec::Group(TabGroupSpec::new(
                "workbench-a",
                Some("overview"),
                vec![
                    text_tab("overview", "workbench-a", "Overview", "Maruzzella is a neutral desktop shell host.", false),
                    text_tab("notes", "workbench-a", "Notes", "Drop any product-specific editor or view into the center workbench.", true),
                    text_tab("scratch", "workbench-a", "Scratch", "This area is fully custom and no longer backed by GtkNotebook.", true),
                ],
            )),
            WorkbenchNodeSpec::Group(TabGroupSpec::new(
                "workbench-b",
                Some("automation"),
                vec![
                    text_tab("automation", "workbench-b", "Automation", "Tooling and workflows can sit in adjacent workbench groups.", false),
                    text_tab("chat", "workbench-b", "Chat", "This is placeholder content for the extraction pass.", true),
                ],
            )),
        ],
    };

    ShellSpec {
        title: "Maruzzella".to_string(),
        menu_roots,
        menu_items,
        commands,
        toolbar_items,
        left_panel,
        right_panel,
        bottom_panel,
        workbench,
    }
}
