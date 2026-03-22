use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gtk::prelude::ButtonExt;
use maruzzella_api::MzViewPlacement;

use crate::base_plugin;
use crate::layout::PersistedShell;
use crate::plugins::PluginRuntime;
use crate::shell::{tabbed_panel, workbench_custom::CustomWorkbenchGroupHandle};
use crate::spec::{plugin_tab_with_instance, ShellSpec, TabGroupSpec, WorkbenchNodeSpec};

pub type ShellState = Rc<RefCell<PersistedShell>>;
pub type GroupHandles = Rc<RefCell<HashMap<String, CustomWorkbenchGroupHandle>>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenPluginViewOutcome {
    Opened,
    FocusedExisting,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenPluginViewRequest {
    pub plugin_view_id: String,
    pub placement: MzViewPlacement,
    pub instance_key: Option<String>,
    pub payload: Vec<u8>,
    pub requested_title: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActivePluginTab {
    pub plugin_view_id: String,
    pub instance_key: Option<String>,
}

thread_local! {
    static LAST_ACTIVE_PLUGIN_TAB: RefCell<Option<ActivePluginTab>> = const { RefCell::new(None) };
}

impl OpenPluginViewRequest {
    pub fn new(plugin_view_id: impl Into<String>, placement: MzViewPlacement) -> Self {
        Self {
            plugin_view_id: plugin_view_id.into(),
            placement,
            instance_key: None,
            payload: Vec::new(),
            requested_title: None,
        }
    }
}

pub fn focus_plugin_view(
    shell_state: &ShellState,
    group_handles: &GroupHandles,
    plugin_view_id: &str,
    instance_key: Option<&str>,
) -> bool {
    let Some((group_id, tab_id)) = find_plugin_view_tab(
        &shell_state.borrow().spec,
        plugin_view_id,
        instance_key,
        true,
    ) else {
        return false;
    };
    let Some(handle) = group_handles.borrow().get(&group_id).cloned() else {
        return false;
    };
    handle.set_active_tab(&tab_id);
    true
}

pub fn is_plugin_view_open(
    shell_state: &ShellState,
    plugin_view_id: &str,
    instance_key: Option<&str>,
) -> bool {
    find_plugin_view_tab(
        &shell_state.borrow().spec,
        plugin_view_id,
        instance_key,
        true,
    )
    .is_some()
}

pub fn update_plugin_view_title(
    shell_state: &ShellState,
    group_handles: &GroupHandles,
    persistence_id: &str,
    plugin_view_id: &str,
    instance_key: Option<&str>,
    title: &str,
) -> bool {
    let Some((group_id, tab_id)) = find_plugin_view_tab(
        &shell_state.borrow().spec,
        plugin_view_id,
        instance_key,
        true,
    ) else {
        return false;
    };

    {
        let mut shell = shell_state.borrow_mut();
        let Some(group) = find_group_mut(&mut shell.spec, &group_id) else {
            return false;
        };
        let Some(tab) = group.tabs.iter_mut().find(|tab| tab.id == tab_id) else {
            return false;
        };
        tab.title = title.to_string();
        crate::layout::save(persistence_id, &shell.clone());
    }

    if let Some(handle) = group_handles.borrow().get(&group_id).cloned() {
        handle.set_tab_title(&tab_id, title);
    }
    true
}

pub fn close_plugin_view_tab(
    shell_state: &ShellState,
    persistence_id: &str,
    handle: &CustomWorkbenchGroupHandle,
    group_id: &str,
    tab_id: &str,
) -> bool {
    {
        let shell = shell_state.borrow();
        let Some(group) = find_group(&shell.spec, group_id) else {
            return false;
        };
        let Some(tab) = group.tabs.iter().find(|tab| tab.id == tab_id) else {
            return false;
        };
        if !base_plugin::can_close_editor_tab(
            tab.plugin_view_id.as_deref(),
            tab.instance_key.as_deref(),
        ) {
            return false;
        }
    }
    handle.remove_tab(tab_id);
    let active_tab_id = handle.active_tab_id();
    let remaining_tab_ids = handle.tab_ids();

    let mut shell = shell_state.borrow_mut();
    let Some(group) = find_group_mut(&mut shell.spec, group_id) else {
        return false;
    };
    group.tabs.retain(|tab| tab.id != tab_id);
    group.active_tab_id = active_tab_id
        .filter(|active| remaining_tab_ids.iter().any(|tab| tab == active));
    crate::layout::save(persistence_id, &shell.clone());
    true
}

pub fn open_or_focus_plugin_view(
    runtime: &Rc<PluginRuntime>,
    persistence_id: &str,
    shell_state: &ShellState,
    group_handles: &GroupHandles,
    request: &OpenPluginViewRequest,
) -> Option<OpenPluginViewOutcome> {
    if let Some(instance_key) = request.instance_key.as_deref() {
        if focus_plugin_view(
            shell_state,
            group_handles,
            &request.plugin_view_id,
            Some(instance_key),
        ) {
            return Some(OpenPluginViewOutcome::FocusedExisting);
        }
    } else if focus_plugin_view(shell_state, group_handles, &request.plugin_view_id, None) {
        return Some(OpenPluginViewOutcome::FocusedExisting);
    }

    let Some(view) = runtime
        .view_factories()
        .iter()
        .find(|view| view.view_id == request.plugin_view_id)
    else {
        return None;
    };
    let Some(group_id) = target_group_id_for_placement(request.placement, group_handles) else {
        return None;
    };
    let Some(handle) = group_handles.borrow().get(&group_id).cloned() else {
        return None;
    };

    let tab = {
        let mut shell = shell_state.borrow_mut();
        let title = request
            .requested_title
            .clone()
            .unwrap_or_else(|| view.title.clone());
        let tab = plugin_tab_with_instance(
            &next_dynamic_tab_id(&shell.spec, &request.plugin_view_id),
            &group_id,
            &title,
            &request.plugin_view_id,
            request.instance_key.as_deref(),
            request.payload.clone(),
            "Plugin view opened from the shell view browser.",
            true,
        );
        if let Some(group) = find_group_mut(&mut shell.spec, &group_id) {
            group.tabs.push(tab.clone());
            group.active_tab_id = Some(tab.id.clone());
        } else {
            return None;
        }
        crate::layout::save(persistence_id, &shell.clone());
        tab
    };

    let page = tabbed_panel::build_tab_page(pane_css_class(&group_id), &tab, Some(runtime));
    if let Some(close_button) = page.close_button.clone() {
        base_plugin::bind_editor_close_button(
            tab.plugin_view_id.as_deref(),
            tab.instance_key.as_deref(),
            &close_button,
        );
        let shell_state = shell_state.clone();
        let persistence_id = persistence_id.to_string();
        let handle = handle.clone();
        let group_id = group_id.clone();
        let tab_id = tab.id.clone();
        close_button.connect_clicked(move |_| {
            close_plugin_view_tab(
                &shell_state,
                &persistence_id,
                &handle,
                &group_id,
                &tab_id,
            );
        });
    }
    handle.append_page(page, true);
    Some(OpenPluginViewOutcome::Opened)
}

pub fn remember_active_plugin_tab(shell_state: &ShellState, group_id: &str, tab_id: &str) {
    let active = {
        let shell = shell_state.borrow();
        find_group(&shell.spec, group_id)
            .and_then(|group| group.tabs.iter().find(|tab| tab.id == tab_id))
            .and_then(|tab| {
                tab.plugin_view_id.as_ref().map(|plugin_view_id| ActivePluginTab {
                    plugin_view_id: plugin_view_id.clone(),
                    instance_key: tab.instance_key.clone(),
                })
            })
    };
    LAST_ACTIVE_PLUGIN_TAB.with(|slot| {
        *slot.borrow_mut() = active;
    });
}

pub fn last_active_plugin_tab() -> Option<ActivePluginTab> {
    LAST_ACTIVE_PLUGIN_TAB.with(|slot| slot.borrow().clone())
}

fn find_plugin_view_tab(
    spec: &ShellSpec,
    plugin_view_id: &str,
    instance_key: Option<&str>,
    match_any_instance_when_none: bool,
) -> Option<(String, String)> {
    find_plugin_view_in_group(
        &spec.left_panel,
        plugin_view_id,
        instance_key,
        match_any_instance_when_none,
    )
    .or_else(|| {
        find_plugin_view_in_group(
            &spec.right_panel,
            plugin_view_id,
            instance_key,
            match_any_instance_when_none,
        )
    })
    .or_else(|| {
        find_plugin_view_in_group(
            &spec.bottom_panel,
            plugin_view_id,
            instance_key,
            match_any_instance_when_none,
        )
    })
    .or_else(|| {
        find_plugin_view_in_workbench(
            &spec.workbench,
            plugin_view_id,
            instance_key,
            match_any_instance_when_none,
        )
    })
}

fn find_plugin_view_in_workbench(
    node: &WorkbenchNodeSpec,
    plugin_view_id: &str,
    instance_key: Option<&str>,
    match_any_instance_when_none: bool,
) -> Option<(String, String)> {
    match node {
        WorkbenchNodeSpec::Group(group) => find_plugin_view_in_group(
            group,
            plugin_view_id,
            instance_key,
            match_any_instance_when_none,
        ),
        WorkbenchNodeSpec::Split { children, .. } => children.iter().find_map(|child| {
            find_plugin_view_in_workbench(
                child,
                plugin_view_id,
                instance_key,
                match_any_instance_when_none,
            )
        }),
    }
}

fn find_plugin_view_in_group(
    group: &TabGroupSpec,
    plugin_view_id: &str,
    instance_key: Option<&str>,
    match_any_instance_when_none: bool,
) -> Option<(String, String)> {
    group.tabs.iter().find_map(|tab| {
        if tab.plugin_view_id.as_deref() != Some(plugin_view_id) {
            return None;
        }

        let instance_matches = match instance_key {
            Some(instance_key) => tab.instance_key.as_deref() == Some(instance_key),
            None if match_any_instance_when_none => true,
            None => tab.instance_key.is_none(),
        };

        instance_matches.then(|| (group.id.clone(), tab.id.clone()))
    })
}

fn find_group<'a>(spec: &'a ShellSpec, group_id: &str) -> Option<&'a TabGroupSpec> {
    if spec.left_panel.id == group_id {
        return Some(&spec.left_panel);
    }
    if spec.right_panel.id == group_id {
        return Some(&spec.right_panel);
    }
    if spec.bottom_panel.id == group_id {
        return Some(&spec.bottom_panel);
    }
    find_group_in_workbench(&spec.workbench, group_id)
}

fn find_group_in_workbench<'a>(
    node: &'a WorkbenchNodeSpec,
    group_id: &str,
) -> Option<&'a TabGroupSpec> {
    match node {
        WorkbenchNodeSpec::Group(group) => (group.id == group_id).then_some(group),
        WorkbenchNodeSpec::Split { children, .. } => children
            .iter()
            .find_map(|child| find_group_in_workbench(child, group_id)),
    }
}

fn target_group_id_for_placement(
    placement: MzViewPlacement,
    group_handles: &GroupHandles,
) -> Option<String> {
    let preferred = match placement {
        MzViewPlacement::Workbench => ["workbench-a", "workbench-main"],
        MzViewPlacement::SidePanel => ["panel-left", "panel-right"],
        MzViewPlacement::BottomPanel => ["panel-bottom", "panel-bottom"],
        MzViewPlacement::Dialog => return None,
    };

    let borrowed = group_handles.borrow();
    preferred
        .into_iter()
        .find(|group_id| borrowed.contains_key(*group_id))
        .map(str::to_string)
        .or_else(|| {
            borrowed
                .keys()
                .find(|group_id| matches_group_placement(group_id, placement))
                .cloned()
        })
}

fn matches_group_placement(group_id: &str, placement: MzViewPlacement) -> bool {
    match placement {
        MzViewPlacement::Workbench => group_id.starts_with("workbench"),
        MzViewPlacement::SidePanel => {
            group_id.starts_with("panel-left") || group_id.starts_with("panel-right")
        }
        MzViewPlacement::BottomPanel => group_id.starts_with("panel-bottom"),
        MzViewPlacement::Dialog => false,
    }
}

fn pane_css_class(group_id: &str) -> &'static str {
    if group_id.starts_with("panel-bottom") {
        "console-pane"
    } else if group_id.starts_with("panel-left") || group_id.starts_with("panel-right") {
        "tool-window"
    } else {
        "workbench"
    }
}

fn next_dynamic_tab_id(spec: &ShellSpec, view_id: &str) -> String {
    let base = format!("plugin-{}", view_id.replace('.', "-"));
    if !tab_id_exists(spec, &base) {
        return base;
    }
    let mut index = 2usize;
    loop {
        let candidate = format!("{base}-{index}");
        if !tab_id_exists(spec, &candidate) {
            return candidate;
        }
        index += 1;
    }
}

fn tab_id_exists(spec: &ShellSpec, tab_id: &str) -> bool {
    all_tabs(spec).any(|tab| tab.id == tab_id)
}

fn all_tabs<'a>(spec: &'a ShellSpec) -> Box<dyn Iterator<Item = &'a crate::spec::TabSpec> + 'a> {
    Box::new(
        spec.left_panel
            .tabs
            .iter()
            .chain(spec.right_panel.tabs.iter())
            .chain(spec.bottom_panel.tabs.iter())
            .chain(workbench_tabs(&spec.workbench)),
    )
}

fn workbench_tabs<'a>(
    node: &'a WorkbenchNodeSpec,
) -> Box<dyn Iterator<Item = &'a crate::spec::TabSpec> + 'a> {
    match node {
        WorkbenchNodeSpec::Group(group) => Box::new(group.tabs.iter()),
        WorkbenchNodeSpec::Split { children, .. } => {
            Box::new(children.iter().flat_map(|child| workbench_tabs(child)))
        }
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
    use crate::spec::{plugin_tab, text_tab, TabGroupSpec};

    #[test]
    fn keyed_search_requires_exact_key_for_open_reuse() {
        let spec = ShellSpec {
            title: String::new(),
            search_placeholder: String::new(),
            status_text: String::new(),
            bottom_panel_layout: crate::spec::BottomPanelLayout::CenterOnly,
            menu_roots: Vec::new(),
            menu_items: Vec::new(),
            commands: Vec::new(),
            toolbar_items: Vec::new(),
            left_panel: TabGroupSpec::new("panel-left", None, Vec::new()),
            right_panel: TabGroupSpec::new("panel-right", None, Vec::new()),
            bottom_panel: TabGroupSpec::new("panel-bottom", None, Vec::new()),
            workbench: WorkbenchNodeSpec::Group(TabGroupSpec::new(
                "workbench-main",
                None,
                vec![plugin_tab_with_instance(
                    "repo-a",
                    "workbench-main",
                    "Repo A",
                    "com.example.repo",
                    Some("repo:a"),
                    b"{}".to_vec(),
                    "",
                    true,
                )],
            )),
        };

        assert!(find_plugin_view_tab(&spec, "com.example.repo", None, false).is_none());
        assert_eq!(
            find_plugin_view_tab(&spec, "com.example.repo", Some("repo:a"), false),
            Some(("workbench-main".to_string(), "repo-a".to_string()))
        );
    }

    #[test]
    fn unkeyed_focus_can_match_any_instance() {
        let spec = ShellSpec {
            title: String::new(),
            search_placeholder: String::new(),
            status_text: String::new(),
            bottom_panel_layout: crate::spec::BottomPanelLayout::CenterOnly,
            menu_roots: Vec::new(),
            menu_items: Vec::new(),
            commands: Vec::new(),
            toolbar_items: Vec::new(),
            left_panel: TabGroupSpec::new("panel-left", None, Vec::new()),
            right_panel: TabGroupSpec::new("panel-right", None, Vec::new()),
            bottom_panel: TabGroupSpec::new("panel-bottom", None, Vec::new()),
            workbench: WorkbenchNodeSpec::Group(TabGroupSpec::new(
                "workbench-main",
                None,
                vec![plugin_tab(
                    "plain",
                    "workbench-main",
                    "Plain",
                    "com.example.repo",
                    "",
                    true,
                )],
            )),
        };

        assert_eq!(
            find_plugin_view_tab(&spec, "com.example.repo", None, true),
            Some(("workbench-main".to_string(), "plain".to_string()))
        );
    }

    #[test]
    fn dynamic_tab_ids_skip_existing_tabs() {
        let spec = ShellSpec {
            title: String::new(),
            search_placeholder: String::new(),
            status_text: String::new(),
            bottom_panel_layout: crate::spec::BottomPanelLayout::CenterOnly,
            menu_roots: Vec::new(),
            menu_items: Vec::new(),
            commands: Vec::new(),
            toolbar_items: Vec::new(),
            left_panel: TabGroupSpec::new("panel-left", None, Vec::new()),
            right_panel: TabGroupSpec::new("panel-right", None, Vec::new()),
            bottom_panel: TabGroupSpec::new(
                "panel-bottom",
                None,
                vec![text_tab(
                    "plugin-com-example-repo",
                    "panel-bottom",
                    "",
                    "",
                    true,
                )],
            ),
            workbench: WorkbenchNodeSpec::Group(TabGroupSpec::new(
                "workbench-main",
                None,
                Vec::new(),
            )),
        };

        assert_eq!(
            next_dynamic_tab_id(&spec, "com.example.repo"),
            "plugin-com-example-repo-2"
        );
    }
}
