use std::collections::HashSet;

use maruzzella_api::{MzContributionSurface, MzStartupTab, MzToolbarItem};

use crate::plugins::PluginRuntime;
use crate::spec::{
    plugin_tab_with_instance, BottomPanelLayout, CommandSpec, MenuItemSpec, MenuRootSpec,
    PanelResizePolicy, ShellSpec, SplitAxis, TabGroupSpec, ToolbarItemSpec, WorkbenchNodeSpec,
};

#[derive(Clone, Debug)]
pub struct BrandingSpec {
    pub title: String,
    pub search_placeholder: String,
    pub search_command_id: Option<String>,
    pub status_text: String,
}

#[derive(Clone, Debug)]
pub struct LayoutContribution {
    pub bottom_panel_layout: BottomPanelLayout,
    pub left_panel: TabGroupSpec,
    pub right_panel: TabGroupSpec,
    pub bottom_panel: TabGroupSpec,
    pub workbench: WorkbenchNodeSpec,
    pub left_panel_resize: PanelResizePolicy,
    pub right_panel_resize: PanelResizePolicy,
    pub bottom_panel_resize: PanelResizePolicy,
}

#[derive(Clone, Debug)]
pub struct ProductSpec {
    pub branding: BrandingSpec,
    pub menu_roots: Vec<MenuRootSpec>,
    pub menu_items: Vec<MenuItemSpec>,
    pub commands: Vec<CommandSpec>,
    pub toolbar_items: Vec<ToolbarItemSpec>,
    pub include_base_toolbar_items: bool,
    pub layout: LayoutContribution,
}

impl ProductSpec {
    pub fn shell_spec(&self) -> ShellSpec {
        ShellSpec {
            title: self.branding.title.clone(),
            search_placeholder: self.branding.search_placeholder.clone(),
            search_command_id: self.branding.search_command_id.clone(),
            status_text: self.branding.status_text.clone(),
            app_appearance_id: "app-shell".to_string(),
            topbar_appearance_id: "topbar".to_string(),
            menu_appearance_id: "menu".to_string(),
            toolbar_appearance_id: "toolbar".to_string(),
            search_input_appearance_id: "search".to_string(),
            status_appearance_id: "status".to_string(),
            button_appearance_id: "secondary".to_string(),
            text_appearance_id: "body".to_string(),
            bottom_panel_layout: self.layout.bottom_panel_layout,
            menu_roots: self.menu_roots.clone(),
            menu_items: self.menu_items.clone(),
            commands: self.commands.clone(),
            toolbar_items: self.toolbar_items.clone(),
            left_panel: self.layout.left_panel.clone(),
            right_panel: self.layout.right_panel.clone(),
            bottom_panel: self.layout.bottom_panel.clone(),
            workbench: self.layout.workbench.clone(),
            left_panel_resize: self.layout.left_panel_resize,
            right_panel_resize: self.layout.right_panel_resize,
            bottom_panel_resize: self.layout.bottom_panel_resize,
        }
    }
}

pub fn merge_plugin_runtime(
    spec: &mut ShellSpec,
    runtime: &PluginRuntime,
    include_base_toolbar_items: bool,
) {
    merge_runtime_commands(spec, runtime);
    merge_runtime_menus(spec, runtime);
    merge_runtime_toolbar(spec, runtime, include_base_toolbar_items);
}

pub fn merge_runtime_startup_tabs(spec: &mut ShellSpec, runtime: &PluginRuntime) {
    for contribution in runtime
        .surface_contributions()
        .iter()
        .filter(|contribution| contribution.surface == Some(MzContributionSurface::StartupTabs))
    {
        let Ok(tab) = MzStartupTab::from_bytes(&contribution.payload) else {
            runtime.push_diagnostic(
                Some(contribution.plugin_id.clone()),
                format!(
                    "invalid startup tab contribution payload: {}",
                    contribution.contribution_id
                ),
            );
            continue;
        };
        let Some(group) = find_group_mut(spec, &tab.group_id) else {
            continue;
        };
        if !group.tabs.is_empty() {
            continue;
        }
        if group.tabs.iter().any(|existing| existing.id == tab.tab_id) {
            continue;
        }
        group.tabs.push(plugin_tab_with_instance(
            &tab.tab_id,
            &tab.group_id,
            &tab.title,
            &tab.plugin_view_id,
            tab.instance_key.as_deref(),
            tab.payload,
            &tab.placeholder,
            tab.closable,
        ));
        if tab.active {
            group.active_tab_id = Some(tab.tab_id);
        }
    }
}

fn merge_runtime_commands(spec: &mut ShellSpec, runtime: &PluginRuntime) {
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
}

fn merge_runtime_menus(spec: &mut ShellSpec, runtime: &PluginRuntime) {
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
                    payload: item.payload.clone(),
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
                payload: item.payload.clone(),
            });
        }
    }

}

fn merge_runtime_toolbar(
    spec: &mut ShellSpec,
    runtime: &PluginRuntime,
    include_base_toolbar_items: bool,
) {
    let mut known_toolbar_ids = spec
        .toolbar_items
        .iter()
        .map(|item| item.id.clone())
        .collect::<HashSet<_>>();

    for contribution in runtime
        .surface_contributions()
        .iter()
        .filter(|contribution| contribution.surface == Some(MzContributionSurface::ToolbarItems))
    {
        if !include_base_toolbar_items && contribution.plugin_id == "maruzzella.base" {
            continue;
        }
        let Ok(item) = MzToolbarItem::from_bytes(&contribution.payload) else {
            runtime.push_diagnostic(
                Some(contribution.plugin_id.clone()),
                format!(
                    "invalid toolbar contribution payload: {}",
                    contribution.contribution_id
                ),
            );
            continue;
        };
        if known_toolbar_ids.insert(item.item_id.clone()) {
            spec.toolbar_items.push(ToolbarItemSpec {
                id: item.item_id,
                icon_name: item.icon_name,
                label: item.label,
                command_id: item.command_id,
                payload: item.payload,
                secondary: item.secondary,
                display_mode: match item.display_mode {
                    maruzzella_api::MzToolbarDisplayMode::IconOnly => {
                        crate::spec::ToolbarDisplayMode::IconOnly
                    }
                    maruzzella_api::MzToolbarDisplayMode::IconAndText => {
                        crate::spec::ToolbarDisplayMode::IconAndText
                    }
                    maruzzella_api::MzToolbarDisplayMode::TextOnly => {
                        crate::spec::ToolbarDisplayMode::TextOnly
                    }
                },
                appearance_id: if item.secondary {
                    "ghost".to_string()
                } else {
                    "primary".to_string()
                },
            });
        }
    }
}

pub fn default_product_spec() -> ProductSpec {
    let branding = BrandingSpec {
        title: "Maruzzella".to_string(),
        search_placeholder: "Search Maruzzella".to_string(),
        search_command_id: None,
        status_text: "Neutral GTK workspace shell".to_string(),
    };
    let commands = vec![CommandSpec {
        id: "shell.reload_theme".to_string(),
        title: "Reload Theme".to_string(),
    }];
    let toolbar_items = Vec::new();
    let menu_roots = Vec::new();
    let menu_items = Vec::new();
    let layout = LayoutContribution {
        bottom_panel_layout: BottomPanelLayout::CenterOnly,
        left_panel: TabGroupSpec::new("panel-left", None, Vec::new())
            .with_panel_appearance("primary")
            .with_panel_header_appearance("secondary")
            .with_tab_strip_appearance("utility"),
        right_panel: TabGroupSpec::new("panel-right", None, Vec::new())
            .with_panel_appearance("secondary")
            .with_panel_header_appearance("secondary")
            .with_tab_strip_appearance("utility"),
        bottom_panel: TabGroupSpec::new("panel-bottom", None, Vec::new())
            .with_panel_appearance("console")
            .with_panel_header_appearance("secondary")
            .with_tab_strip_appearance("console")
            .with_text_appearance("code"),
        workbench: WorkbenchNodeSpec::Split {
            axis: SplitAxis::Horizontal,
            children: vec![
                WorkbenchNodeSpec::Group(
                    TabGroupSpec::new("workbench-main", None, Vec::new())
                        .with_panel_appearance("workbench")
                        .with_panel_header_appearance("secondary")
                        .with_tab_strip_appearance("editor"),
                ),
                WorkbenchNodeSpec::Group(TabGroupSpec::new(
                    "workbench-secondary",
                    None,
                    Vec::new(),
                )
                .with_panel_appearance("workbench")
                .with_panel_header_appearance("secondary")
                .with_tab_strip_appearance("editor")),
            ],
        },
        left_panel_resize: PanelResizePolicy::default(),
        right_panel_resize: PanelResizePolicy::default(),
        bottom_panel_resize: PanelResizePolicy::default(),
    };

    ProductSpec {
        branding,
        menu_roots,
        menu_items,
        commands,
        toolbar_items,
        include_base_toolbar_items: true,
        layout,
    }
}

fn find_group_mut<'a>(spec: &'a mut ShellSpec, group_id: &str) -> Option<&'a mut TabGroupSpec> {
    if spec.left_panel.id == group_id {
        return Some(&mut spec.left_panel);
    }
    if spec.right_panel.id == group_id {
        return Some(&mut spec.right_panel);
    }
    if spec.bottom_panel.id == group_id {
        return Some(&mut spec.bottom_panel);
    }
    find_group_mut_in_workbench(&mut spec.workbench, group_id)
}

fn find_group_mut_in_workbench<'a>(
    node: &'a mut WorkbenchNodeSpec,
    group_id: &str,
) -> Option<&'a mut TabGroupSpec> {
    match node {
        WorkbenchNodeSpec::Group(group) => (group.id == group_id).then_some(group),
        WorkbenchNodeSpec::Split { children, .. } => children
            .iter_mut()
            .find_map(|child| find_group_mut_in_workbench(child, group_id)),
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
        let mut runtime = PluginRuntime::empty_for_tests();
        runtime.activation_order = vec!["maruzzella.base".to_string()];
        runtime.commands = vec![RegisteredCommand {
            plugin_id: "maruzzella.base".to_string(),
            command_id: "shell.plugins".to_string(),
            title: "Plugins".to_string(),
            invoke: None,
        }];
        runtime.menu_items = vec![RegisteredMenuItem {
            plugin_id: "maruzzella.base".to_string(),
            menu_id: "plugins".to_string(),
            parent_id: "maruzzella.menu.file.items".to_string(),
            parent_surface: Some(MzMenuSurface::FileItems),
            title: "Plugins".to_string(),
            command_id: "shell.plugins".to_string(),
            payload: Vec::new(),
        }];
        runtime.surface_contributions = vec![RegisteredSurfaceContribution {
            plugin_id: "maruzzella.base".to_string(),
            surface_id: "maruzzella.about.sections".to_string(),
            surface: Some(maruzzella_api::MzContributionSurface::AboutSections),
            contribution_id: "base.about".to_string(),
            payload: Vec::new(),
        }];

        let mut spec = default_product_spec().shell_spec();
        merge_plugin_runtime(&mut spec, &runtime, true);

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
