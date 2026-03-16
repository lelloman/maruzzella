use std::collections::HashSet;

use crate::plugins::PluginRuntime;
use crate::spec::{
    plugin_tab, BottomPanelLayout, CommandSpec, MenuItemSpec, MenuRootSpec, ShellSpec, SplitAxis,
    TabGroupSpec, ToolbarItemSpec, WorkbenchNodeSpec,
};

#[derive(Clone, Debug)]
pub struct BrandingSpec {
    pub title: String,
    pub search_placeholder: String,
    pub status_text: String,
}

#[derive(Clone, Debug)]
pub struct LayoutContribution {
    pub bottom_panel_layout: BottomPanelLayout,
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
            bottom_panel_layout: self.layout.bottom_panel_layout,
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
        let Some(parent_surface) = item.parent_surface else {
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

        let root_id = parent_surface.root_id();
        let root_label = parent_surface.root_label();

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

    spec.menu_roots.sort_by_key(|root| menu_root_rank(&root.id));
}

fn menu_root_rank(root_id: &str) -> (usize, String) {
    let rank = match root_id {
        "file" => 0,
        "app" => 1,
        "view" => 2,
        "help" => 3,
        _ => 10,
    };
    (rank, root_id.to_string())
}

pub fn default_product_spec() -> ProductSpec {
    let branding = BrandingSpec {
        title: "Maruzzella".to_string(),
        search_placeholder: "Search Maruzzella".to_string(),
        status_text: "Plugin-ready GTK workspace shell".to_string(),
    };
    let commands = vec![
        CommandSpec {
            id: "shell.open_command_palette".to_string(),
            title: "Command Palette".to_string(),
        },
        CommandSpec {
            id: "shell.browse_views".to_string(),
            title: "Browse Views".to_string(),
        },
        CommandSpec {
            id: "shell.reload_theme".to_string(),
            title: "Reload Theme".to_string(),
        },
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
            id: "views".to_string(),
            icon_name: Some("view-grid-symbolic".to_string()),
            label: None,
            command_id: "shell.browse_views".to_string(),
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
    let menu_roots = vec![MenuRootSpec {
        id: "view".to_string(),
        label: "View".to_string(),
    }];
    let menu_items = vec![
        MenuItemSpec {
            id: "command-palette".to_string(),
            root_id: "view".to_string(),
            label: "Command Palette".to_string(),
            command_id: "shell.open_command_palette".to_string(),
        },
        MenuItemSpec {
            id: "reload-theme".to_string(),
            root_id: "view".to_string(),
            label: "Reload Theme".to_string(),
            command_id: "shell.reload_theme".to_string(),
        },
        MenuItemSpec {
            id: "browse-views".to_string(),
            root_id: "view".to_string(),
            label: "Browse Views".to_string(),
            command_id: "shell.browse_views".to_string(),
        },
    ];
    let layout = LayoutContribution {
        bottom_panel_layout: BottomPanelLayout::CenterOnly,
        left_panel: TabGroupSpec::new(
            "panel-left",
            Some("workspace-nav"),
            vec![
                plugin_tab(
                    "workspace-nav",
                    "panel-left",
                    "Workspace",
                    "maruzzella.base.panel.navigator",
                    "Primary workspace navigation is provided by the built-in base plugin.",
                    false,
                ),
                plugin_tab(
                    "resource-index",
                    "panel-left",
                    "Resources",
                    "maruzzella.base.panel.resources",
                    "Reference material and starter assets can live here.",
                    false,
                ),
            ],
        ),
        right_panel: TabGroupSpec::new(
            "panel-right",
            Some("selection-inspector"),
            vec![
                plugin_tab(
                    "selection-inspector",
                    "panel-right",
                    "Inspector",
                    "maruzzella.base.panel.inspector",
                    "Selection-aware details and shell health live here.",
                    false,
                ),
                plugin_tab(
                    "delivery-checklist",
                    "panel-right",
                    "Release",
                    "maruzzella.base.panel.delivery",
                    "Delivery notes and polish checkpoints live here.",
                    false,
                ),
            ],
        ),
        bottom_panel: TabGroupSpec::new(
            "panel-bottom",
            Some("runtime-activity"),
            vec![
                plugin_tab(
                    "runtime-activity",
                    "panel-bottom",
                    "Activity",
                    "maruzzella.base.panel.activity",
                    "Runtime diagnostics and theme workflows live here.",
                    false,
                ),
                plugin_tab(
                    "extension-health",
                    "panel-bottom",
                    "Extensions",
                    "maruzzella.base.panel.extensions",
                    "Plugin runtime state and settings surface summaries live here.",
                    false,
                ),
            ],
        ),
        workbench: WorkbenchNodeSpec::Split {
            axis: SplitAxis::Horizontal,
            children: vec![
                WorkbenchNodeSpec::Group(TabGroupSpec::new(
                    "workbench-a",
                    Some("studio-home"),
                    vec![
                        plugin_tab(
                            "studio-home",
                            "workbench-a",
                            "Studio Home",
                            "maruzzella.base.workspace.home",
                            "The default shell slice overview lives here.",
                            false,
                        ),
                        plugin_tab(
                            "work-queue",
                            "workbench-a",
                            "Work Queue",
                            "maruzzella.base.workspace.queue",
                            "The current roadmap queue lives here.",
                            true,
                        ),
                    ],
                )),
                WorkbenchNodeSpec::Group(TabGroupSpec::new(
                    "workbench-b",
                    Some("integration-surfaces"),
                    vec![
                        plugin_tab(
                            "integration-surfaces",
                            "workbench-b",
                            "Contribution Surfaces",
                            "maruzzella.base.workspace.surfaces",
                            "Shared plugin contribution surfaces live here.",
                            false,
                        ),
                        plugin_tab(
                            "system-ops",
                            "workbench-b",
                            "System Ops",
                            "maruzzella.base.workspace.ops",
                            "System and runtime operations live here.",
                            true,
                        ),
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
    use maruzzella_api::MzMenuSurface;

    #[test]
    fn merges_plugin_commands_and_surface_backed_menu_roots() {
        let runtime = PluginRuntime {
            plugins: Vec::new(),
            activation_order: vec!["maruzzella.base".to_string()],
            commands: vec![RegisteredCommand {
                plugin_id: "maruzzella.base".to_string(),
                command_id: "shell.plugins".to_string(),
                title: "Plugins".to_string(),
                invoke: None,
            }],
            menu_items: vec![RegisteredMenuItem {
                plugin_id: "maruzzella.base".to_string(),
                menu_id: "plugins".to_string(),
                parent_id: "maruzzella.menu.file.items".to_string(),
                parent_surface: Some(MzMenuSurface::FileItems),
                title: "Plugins".to_string(),
                command_id: "shell.plugins".to_string(),
            }],
            surface_contributions: vec![RegisteredSurfaceContribution {
                plugin_id: "maruzzella.base".to_string(),
                surface_id: "maruzzella.about.sections".to_string(),
                surface: Some(maruzzella_api::MzContributionSurface::AboutSections),
                contribution_id: "base.about".to_string(),
                payload: Vec::new(),
            }],
            view_factories: Vec::new(),
            logs: Vec::new(),
        };

        let mut spec = default_product_spec().shell_spec();
        merge_plugin_runtime(&mut spec, &runtime);

        assert!(spec
            .commands
            .iter()
            .any(|command| command.id == "shell.plugins"));
        assert!(spec.menu_roots.iter().any(|root| root.id == "file"));
        assert!(spec
            .menu_items
            .iter()
            .any(|item| item.id == "plugins" && item.root_id == "file"));
    }
}
