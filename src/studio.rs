#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabHost {
    NavigationList,
    IdentityList,
    InspectorDetails,
    CommandList,
    TextBuffer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug)]
pub struct DockTabSpec {
    pub id: String,
    pub dock_id: String,
    pub title: String,
    pub tab_type: String,
    pub instance_key: Option<String>,
    pub host: TabHost,
    pub placeholder: String,
    pub closable: bool,
    pub close_prompt: Option<String>,
}

#[derive(Clone, Debug)]
pub struct TabGroupSpec {
    pub id: String,
    pub active_tab_id: Option<String>,
    pub tabs: Vec<DockTabSpec>,
}

#[derive(Clone, Debug)]
pub struct ShellSpec {
    pub title: String,
    pub menu_roots: Vec<MenuRootSpec>,
    pub menu_items: Vec<MenuItemSpec>,
    pub actions: Vec<ActionSpec>,
    pub action_bar_items: Vec<ActionBarItemSpec>,
    pub left: TabGroupSpec,
    pub right: TabGroupSpec,
    pub bottom: TabGroupSpec,
    pub workbench: WorkbenchNodeSpec,
}

#[derive(Clone, Debug)]
pub enum WorkbenchNodeSpec {
    Group(TabGroupSpec),
    Split {
        direction: SplitDirection,
        children: Vec<WorkbenchNodeSpec>,
    },
}

#[derive(Clone, Debug)]
pub struct ActionSpec {
    pub id: String,
    pub title: String,
}

#[derive(Clone, Debug)]
pub struct ActionBarItemSpec {
    pub id: String,
    pub icon_name: Option<String>,
    pub label: Option<String>,
    pub action_id: String,
    pub secondary: bool,
}

#[derive(Clone, Debug)]
pub struct MenuRootSpec {
    pub id: String,
    pub label: String,
}

#[derive(Clone, Debug)]
pub struct MenuItemSpec {
    pub id: String,
    pub root_id: String,
    pub label: String,
    pub action_id: String,
}

impl TabGroupSpec {
    pub fn new(id: &str, active_tab_id: Option<&str>, tabs: Vec<DockTabSpec>) -> Self {
        Self {
            id: id.to_string(),
            active_tab_id: active_tab_id.map(str::to_string),
            tabs,
        }
    }
}

pub fn action_name(action_id: &str) -> String {
    action_id.replace('.', "-")
}

pub fn menu_action_ref(action_id: &str) -> String {
    format!("win.{}", action_name(action_id))
}

pub fn text_tab(id: &str, dock_id: &str, title: &str, body: &str, closable: bool) -> DockTabSpec {
    DockTabSpec {
        id: id.to_string(),
        dock_id: dock_id.to_string(),
        title: title.to_string(),
        tab_type: "text".to_string(),
        instance_key: None,
        host: TabHost::TextBuffer,
        placeholder: body.to_string(),
        closable,
        close_prompt: None,
    }
}

pub fn default_shell_spec() -> ShellSpec {
    let actions = vec![
        ActionSpec { id: "shell.open_command_palette".to_string(), title: "Command Palette".to_string() },
        ActionSpec { id: "shell.reload_theme".to_string(), title: "Reload Theme".to_string() },
        ActionSpec { id: "shell.about".to_string(), title: "About Maruzzella".to_string() },
    ];
    let action_bar_items = vec![
        ActionBarItemSpec {
            id: "palette".to_string(),
            icon_name: Some("system-search-symbolic".to_string()),
            label: Some("Palette".to_string()),
            action_id: "shell.open_command_palette".to_string(),
            secondary: false,
        },
        ActionBarItemSpec {
            id: "theme".to_string(),
            icon_name: Some("applications-graphics-symbolic".to_string()),
            label: None,
            action_id: "shell.reload_theme".to_string(),
            secondary: true,
        },
        ActionBarItemSpec {
            id: "about".to_string(),
            icon_name: Some("help-about-symbolic".to_string()),
            label: None,
            action_id: "shell.about".to_string(),
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
            action_id: "shell.open_command_palette".to_string(),
        },
        MenuItemSpec {
            id: "about".to_string(),
            root_id: "app".to_string(),
            label: "About Maruzzella".to_string(),
            action_id: "shell.about".to_string(),
        },
        MenuItemSpec {
            id: "reload-theme".to_string(),
            root_id: "view".to_string(),
            label: "Reload Theme".to_string(),
            action_id: "shell.reload_theme".to_string(),
        },
    ];
    let left = TabGroupSpec::new(
        "tool-left",
        Some("navigation"),
        vec![
            text_tab("navigation", "tool-left", "Navigation", "Anonymous shell navigation goes here.", false),
            text_tab("library", "tool-left", "Library", "A product can mount its own content here.", false),
        ],
    );
    let right = TabGroupSpec::new(
        "tool-right",
        Some("inspector"),
        vec![
            text_tab("inspector", "tool-right", "Inspector", "Selection-aware details live here.", false),
            text_tab("outline", "tool-right", "Outline", "Structure and metadata panels fit here.", false),
        ],
    );
    let bottom = TabGroupSpec::new(
        "tool-bottom",
        Some("logs"),
        vec![
            text_tab("logs", "tool-bottom", "Logs", "Runtime output, tasks, and traces.", false),
            text_tab("problems", "tool-bottom", "Problems", "Validation and build output.", false),
        ],
    );
    let workbench = WorkbenchNodeSpec::Split {
        direction: SplitDirection::Horizontal,
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
        actions,
        action_bar_items,
        left,
        right,
        bottom,
        workbench,
    }
}
