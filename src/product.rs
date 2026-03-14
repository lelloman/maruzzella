use std::collections::HashSet;

use crate::plugins::PluginRuntime;
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

pub fn merge_plugin_runtime(spec: &mut ShellSpec, runtime: &PluginRuntime) {
    let mut known_command_ids = spec
        .commands
        .iter()
        .map(|command| command.id.clone())
        .collect::<HashSet<_>>();
    for command in runtime.commands() {
        if known_command_ids.insert(command.command_id.clone()) {
            spec.commands.push(CommandSpec {
                id: command.command_id.clone(),
                title: command.title.clone(),
            });
        }
    }

    let mut known_root_ids = spec
        .menu_roots
        .iter()
        .map(|root| root.id.clone())
        .collect::<HashSet<_>>();
    let mut known_menu_ids = spec
        .menu_items
        .iter()
        .map(|item| item.id.clone())
        .collect::<HashSet<_>>();

    for item in runtime.menu_items() {
        let Some((root_id, root_label)) = root_for_parent_surface(&item.parent_id) else {
            if !known_root_ids.contains(&item.parent_id) {
                continue;
            }
            if known_menu_ids.insert(item.menu_id.clone()) {
                spec.menu_items.push(MenuItemSpec {
                    id: item.menu_id.clone(),
                    root_id: item.parent_id.clone(),
                    label: item.title.clone(),
                    command_id: item.command_id.clone(),
                });
            }
            continue;
        };

        if known_root_ids.insert(root_id.to_string()) {
            spec.menu_roots.push(MenuRootSpec {
                id: root_id.to_string(),
                label: root_label.to_string(),
            });
        }

        if known_menu_ids.insert(item.menu_id.clone()) {
            spec.menu_items.push(MenuItemSpec {
                id: item.menu_id.clone(),
                root_id: root_id.to_string(),
                label: item.title.clone(),
                command_id: item.command_id.clone(),
            });
        }
    }
}

fn root_for_parent_surface(parent_id: &str) -> Option<(&'static str, &'static str)> {
    match parent_id {
        "maruzzella.menu.file.items" => Some(("file", "File")),
        "maruzzella.menu.help.items" => Some(("help", "Help")),
        "maruzzella.menu.view.items" => Some(("view", "View")),
        _ => None,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::{
        PluginRuntime, RegisteredCommand, RegisteredMenuItem, RegisteredSurfaceContribution,
    };

    #[test]
    fn merges_plugin_commands_and_surface_backed_menu_roots() {
        let runtime = PluginRuntime {
            plugins: Vec::new(),
            activation_order: vec!["maruzzella.base".to_string()],
            commands: vec![RegisteredCommand {
                plugin_id: "maruzzella.base".to_string(),
                command_id: "shell.plugins".to_string(),
                title: "Plugins".to_string(),
            }],
            menu_items: vec![RegisteredMenuItem {
                plugin_id: "maruzzella.base".to_string(),
                menu_id: "plugins".to_string(),
                parent_id: "maruzzella.menu.file.items".to_string(),
                title: "Plugins".to_string(),
                command_id: "shell.plugins".to_string(),
            }],
            surface_contributions: vec![RegisteredSurfaceContribution {
                plugin_id: "maruzzella.base".to_string(),
                surface_id: "maruzzella.about.sections".to_string(),
                contribution_id: "base.about".to_string(),
                payload: Vec::new(),
            }],
            view_factories: Vec::new(),
            logs: Vec::new(),
        };

        let mut spec = default_product_spec().shell_spec();
        merge_plugin_runtime(&mut spec, &runtime);

        assert!(spec.commands.iter().any(|command| command.id == "shell.plugins"));
        assert!(spec.menu_roots.iter().any(|root| root.id == "file"));
        assert!(spec.menu_items.iter().any(|item| item.id == "plugins" && item.root_id == "file"));
    }
}
