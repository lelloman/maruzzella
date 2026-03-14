use crate::spec::{
    text_tab, CommandSpec, MenuItemSpec, MenuRootSpec, ShellSpec, SplitAxis, TabGroupSpec,
    ToolbarItemSpec, WorkbenchNodeSpec,
};

#[derive(Clone, Debug)]
pub struct BrandingSpec {
    pub title: String,
    pub search_placeholder: String,
    pub status_text: String,
}

#[derive(Clone, Debug)]
pub struct LayoutContribution {
    pub left_panel: TabGroupSpec,
    pub right_panel: TabGroupSpec,
    pub bottom_panel: TabGroupSpec,
    pub workbench: WorkbenchNodeSpec,
}

#[derive(Clone, Debug)]
pub struct ProductSpec {
    pub branding: BrandingSpec,
    pub menu_roots: Vec<MenuRootSpec>,
    pub menu_items: Vec<MenuItemSpec>,
    pub commands: Vec<CommandSpec>,
    pub toolbar_items: Vec<ToolbarItemSpec>,
    pub layout: LayoutContribution,
}

impl ProductSpec {
    pub fn shell_spec(&self) -> ShellSpec {
        ShellSpec {
            title: self.branding.title.clone(),
            search_placeholder: self.branding.search_placeholder.clone(),
            status_text: self.branding.status_text.clone(),
            menu_roots: self.menu_roots.clone(),
            menu_items: self.menu_items.clone(),
            commands: self.commands.clone(),
            toolbar_items: self.toolbar_items.clone(),
            left_panel: self.layout.left_panel.clone(),
            right_panel: self.layout.right_panel.clone(),
            bottom_panel: self.layout.bottom_panel.clone(),
            workbench: self.layout.workbench.clone(),
        }
    }
}

pub fn default_product_spec() -> ProductSpec {
    let branding = BrandingSpec {
        title: "Maruzzella".to_string(),
        search_placeholder: "Search Maruzzella".to_string(),
        status_text: "Neutral GTK desktop shell host".to_string(),
    };
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
    let layout = LayoutContribution {
        left_panel: TabGroupSpec::new(
            "panel-left",
            Some("navigation"),
            vec![
                text_tab("navigation", "panel-left", "Navigation", "Anonymous shell navigation goes here.", false),
                text_tab("library", "panel-left", "Library", "A product can mount its own content here.", false),
            ],
        ),
        right_panel: TabGroupSpec::new(
            "panel-right",
            Some("inspector"),
            vec![
                text_tab("inspector", "panel-right", "Inspector", "Selection-aware details live here.", false),
                text_tab("outline", "panel-right", "Outline", "Structure and metadata panels fit here.", false),
            ],
        ),
        bottom_panel: TabGroupSpec::new(
            "panel-bottom",
            Some("logs"),
            vec![
                text_tab("logs", "panel-bottom", "Logs", "Runtime output, tasks, and traces.", false),
                text_tab("problems", "panel-bottom", "Problems", "Validation and build output.", false),
            ],
        ),
        workbench: WorkbenchNodeSpec::Split {
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
        },
    };

    ProductSpec {
        branding,
        menu_roots,
        menu_items,
        commands,
        toolbar_items,
        layout,
    }
}
