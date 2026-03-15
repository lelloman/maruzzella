use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box as GtkBox, Orientation, Paned};

use crate::base_plugin;
use crate::MaruzzellaConfig;
use crate::commands;
use crate::layout::{self, PersistedShell};
use crate::plugins::{
    diagnostic_for_load_error, diagnostic_for_runtime_error, load_plugin, PluginHost,
    PluginRuntime,
};
use crate::product;
use crate::shell::topbar;
use crate::shell::workbench_custom::{self, BuiltCustomWorkbenchGroup, CustomWorkbenchGroupHandle};
use crate::spec::{ShellSpec, SplitAxis, TabGroupSpec, TabSpec, WorkbenchNodeSpec};
use crate::theme;

type ShellState = Rc<RefCell<PersistedShell>>;

const MIN_SIDE_PANEL_WIDTH: i32 = 220;
const MIN_BOTTOM_PANEL_HEIGHT: i32 = 140;

pub fn build(application: &Application, config: &MaruzzellaConfig) {
    theme::load();

    let state = Rc::new(RefCell::new(layout::load(
        &config.persistence_id,
        &config.product.shell_spec(),
    )));
    let mut spec = state.borrow().spec.clone();
    let plugin_host = Rc::new(build_plugin_host(config));
    if let Some(runtime) = plugin_host.runtime() {
        product::merge_plugin_runtime(&mut spec, runtime);
    }

    let window = ApplicationWindow::builder()
        .application(application)
        .title(&spec.title)
        .default_width(1600)
        .default_height(980)
        .build();
    window.add_css_class("app-window");
    let registry = commands::shell_registry(&window, &spec, Some(plugin_host.clone()));
    topbar::install_actions(&window, &spec, &registry);

    let root = GtkBox::new(Orientation::Vertical, 0);
    root.add_css_class("app-root");
    root.append(&topbar::build(&spec).root);
    root.append(&build_shell(state, config.persistence_id.clone(), plugin_host.runtime().cloned()));
    window.set_child(Some(&root));
    unsafe {
        window.set_data("maruzzella-plugin-host", plugin_host);
    }
    window.present();
}

fn build_plugin_host(config: &MaruzzellaConfig) -> PluginHost {
    let mut plugins = vec![base_plugin::load()];
    let mut diagnostics = Vec::new();
    for path in &config.plugin_paths {
        match load_plugin(path) {
            Ok(plugin) => plugins.push(plugin),
            Err(error) => diagnostics.push(diagnostic_for_load_error(path, &error)),
        }
    }

    match crate::plugins::PluginRuntime::activate_with_persistence_id(plugins, &config.persistence_id) {
        Ok(runtime) => PluginHost::new(Some(Rc::new(runtime)), diagnostics),
        Err(error) => {
            diagnostics.push(diagnostic_for_runtime_error(&error));
            PluginHost::new(None, diagnostics)
        }
    }
}

fn build_shell(
    state: ShellState,
    persistence_id: String,
    plugin_runtime: Option<Rc<PluginRuntime>>,
) -> gtk::Widget {
    let spec = state.borrow().spec.clone();
    let left = build_group(
        &spec.left_panel,
        state.clone(),
        persistence_id.clone(),
        plugin_runtime.clone(),
    );
    left.root.set_size_request(MIN_SIDE_PANEL_WIDTH, -1);
    let right = build_group(
        &spec.right_panel,
        state.clone(),
        persistence_id.clone(),
        plugin_runtime.clone(),
    );
    right.root.set_size_request(MIN_SIDE_PANEL_WIDTH, -1);
    let bottom = build_group(
        &spec.bottom_panel,
        state.clone(),
        persistence_id.clone(),
        plugin_runtime.clone(),
    );
    bottom.root.set_size_request(-1, MIN_BOTTOM_PANEL_HEIGHT);
    let workbench = build_workbench_node(
        &spec.workbench,
        state.clone(),
        persistence_id.clone(),
        "workbench-root",
        plugin_runtime,
    );

    let horizontal = Paned::new(Orientation::Horizontal);
    horizontal.set_wide_handle(true);
    horizontal.set_resize_start_child(true);
    horizontal.set_resize_end_child(true);
    horizontal.set_shrink_start_child(false);
    horizontal.set_start_child(Some(&left.root));
    horizontal.set_end_child(Some(&workbench));
    restore_pane_position(&horizontal, &state, "shell.horizontal", 280);
    persist_pane_position(&horizontal, state.clone(), persistence_id.clone(), "shell.horizontal");

    let vertical = Paned::new(Orientation::Vertical);
    vertical.set_wide_handle(true);
    vertical.set_resize_start_child(true);
    vertical.set_resize_end_child(true);
    vertical.set_shrink_end_child(false);
    vertical.set_start_child(Some(&horizontal));
    vertical.set_end_child(Some(&bottom.root));
    restore_pane_position(&vertical, &state, "shell.vertical", 720);
    persist_pane_position(&vertical, state.clone(), persistence_id.clone(), "shell.vertical");

    let outer = Paned::new(Orientation::Horizontal);
    outer.set_wide_handle(true);
    outer.set_resize_start_child(true);
    outer.set_resize_end_child(true);
    outer.set_shrink_end_child(false);
    outer.set_start_child(Some(&vertical));
    outer.set_end_child(Some(&right.root));
    restore_pane_position(&outer, &state, "shell.outer", 1260);
    persist_pane_position(&outer, state, persistence_id, "shell.outer");
    outer.upcast::<gtk::Widget>()
}

fn build_group(
    group: &TabGroupSpec,
    state: ShellState,
    persistence_id: String,
    plugin_runtime: Option<Rc<PluginRuntime>>,
) -> BuiltCustomWorkbenchGroup {
    let built = workbench_custom::build_group(
        &group.id,
        &group.tabs,
        group.active_tab_id.as_deref(),
        plugin_runtime,
    );
    install_group_persistence(&built.handle, state, persistence_id);
    built
}

fn build_workbench_node(
    node: &WorkbenchNodeSpec,
    state: ShellState,
    persistence_id: String,
    path: &str,
    plugin_runtime: Option<Rc<PluginRuntime>>,
) -> gtk::Widget {
    match node {
        WorkbenchNodeSpec::Group(group) => {
            build_group(group, state, persistence_id, plugin_runtime).root.upcast::<gtk::Widget>()
        }
        WorkbenchNodeSpec::Split { axis, children } => {
            let mut child_widgets = children
                .iter()
                .enumerate()
                .map(|(index, child)| {
                    build_workbench_node(
                        child,
                        state.clone(),
                        persistence_id.clone(),
                        &format!("{path}:{index}"),
                        plugin_runtime.clone(),
                    )
                })
                .collect::<Vec<_>>();
            let first = child_widgets.remove(0);
            let mut current = first;
            for (index, child) in child_widgets.into_iter().enumerate() {
                let paned = Paned::new(match axis {
                    SplitAxis::Horizontal => Orientation::Horizontal,
                    SplitAxis::Vertical => Orientation::Vertical,
                });
                paned.set_wide_handle(true);
                paned.set_resize_start_child(true);
                paned.set_resize_end_child(true);
                paned.set_start_child(Some(&current));
                paned.set_end_child(Some(&child));
                let pane_id = format!("{path}:split:{index}");
                restore_pane_position(&paned, &state, &pane_id, 520);
                persist_pane_position(&paned, state.clone(), persistence_id.clone(), &pane_id);
                current = paned.upcast::<gtk::Widget>();
            }
            current
        }
    }
}

fn install_group_persistence(
    handle: &CustomWorkbenchGroupHandle,
    state: ShellState,
    persistence_id: String,
) {
    let handle_for_active = handle.clone();
    let state_for_active = state.clone();
    let persistence_id_for_active = persistence_id.clone();
    handle.set_active_changed_handler(move |_| {
        sync_group_into_state(&state_for_active, &handle_for_active, &persistence_id_for_active);
    });

    let handle_for_drag = handle.clone();
    let state_for_drag = state;
    let persistence_id_for_drag = persistence_id;
    handle.set_drag_end_handler(move || {
        sync_group_into_state(&state_for_drag, &handle_for_drag, &persistence_id_for_drag);
    });
}

fn sync_group_into_state(
    state: &ShellState,
    handle: &CustomWorkbenchGroupHandle,
    persistence_id: &str,
) {
    let group_id = handle.group_id().to_string();
    let tab_ids = handle.tab_ids();
    let active_tab_id = handle.active_tab_id();

    {
        let mut shell = state.borrow_mut();
        if !sync_group_spec(
            &mut shell.spec,
            &group_id,
            &tab_ids,
            active_tab_id.as_ref(),
        ) {
            return;
        }
    }

    persist_state(state, persistence_id);
}

fn sync_group_spec(
    spec: &mut ShellSpec,
    group_id: &str,
    ordered_tab_ids: &[String],
    active_tab_id: Option<&String>,
) -> bool {
    sync_single_group(&mut spec.left_panel, group_id, ordered_tab_ids, active_tab_id)
        || sync_single_group(&mut spec.right_panel, group_id, ordered_tab_ids, active_tab_id)
        || sync_single_group(&mut spec.bottom_panel, group_id, ordered_tab_ids, active_tab_id)
        || sync_workbench_node(&mut spec.workbench, group_id, ordered_tab_ids, active_tab_id)
}

fn sync_workbench_node(
    node: &mut WorkbenchNodeSpec,
    group_id: &str,
    ordered_tab_ids: &[String],
    active_tab_id: Option<&String>,
) -> bool {
    match node {
        WorkbenchNodeSpec::Group(group) => {
            sync_single_group(group, group_id, ordered_tab_ids, active_tab_id)
        }
        WorkbenchNodeSpec::Split { children, .. } => children
            .iter_mut()
            .any(|child| sync_workbench_node(child, group_id, ordered_tab_ids, active_tab_id)),
    }
}

fn sync_single_group(
    group: &mut TabGroupSpec,
    group_id: &str,
    ordered_tab_ids: &[String],
    active_tab_id: Option<&String>,
) -> bool {
    if group.id != group_id {
        return false;
    }

    let mut tabs_by_id = group
        .tabs
        .drain(..)
        .map(|tab| (tab.id.clone(), tab))
        .collect::<HashMap<String, TabSpec>>();

    let mut tabs = Vec::with_capacity(ordered_tab_ids.len());
    for tab_id in ordered_tab_ids {
        if let Some(tab) = tabs_by_id.remove(tab_id) {
            tabs.push(tab);
        }
    }
    tabs.extend(tabs_by_id.into_values());
    group.tabs = tabs;
    group.active_tab_id = active_tab_id.cloned();
    true
}

fn restore_pane_position(paned: &Paned, state: &ShellState, pane_id: &str, default: i32) {
    let position = state
        .borrow()
        .panes
        .positions
        .get(pane_id)
        .copied()
        .unwrap_or(default);
    paned.set_position(position);
}

fn persist_pane_position(
    paned: &Paned,
    state: ShellState,
    persistence_id: String,
    pane_id: &str,
) {
    let pane_id = pane_id.to_string();
    paned.connect_position_notify(move |paned| {
        state
            .borrow_mut()
            .panes
            .positions
            .insert(pane_id.clone(), paned.position());
        persist_state(&state, &persistence_id);
    });
}

fn persist_state(state: &ShellState, persistence_id: &str) {
    let snapshot = state.borrow().clone();
    layout::save(persistence_id, &snapshot);
}
