use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box as GtkBox, Orientation, Paned};

use crate::base_plugin;
use crate::commands;
use crate::layout::{self, PersistedShell};
use crate::plugin_tabs::{self, GroupHandles};
use crate::plugins::{
    diagnostic_for_load_error, diagnostic_for_runtime_error, load_plugin, PluginDiagnostic,
    PluginDiagnosticLevel, PluginHost, PluginRuntime,
};
use crate::product;
use crate::shell::topbar;
use crate::shell::workbench_custom::{self, BuiltCustomWorkbenchGroup, CustomWorkbenchGroupHandle};
use crate::spec::{
    BottomPanelLayout, ShellSpec, SplitAxis, TabGroupSpec, TabSpec, WorkbenchNodeSpec,
};
use crate::theme;
use crate::MaruzzellaConfig;

type ShellState = Rc<RefCell<PersistedShell>>;

pub fn build(application: &Application, config: &MaruzzellaConfig) {
    theme::install(config.theme.clone());
    let density = &config.theme.density;
    let has_persisted_layout = layout::path(&config.persistence_id).exists();

    let state = Rc::new(RefCell::new(layout::load(
        &config.persistence_id,
        &config.product.shell_spec(),
    )));
    let mut spec = state.borrow().spec.clone();
    let product_spec = config.product.shell_spec();
    spec.title = product_spec.title;
    spec.search_placeholder = product_spec.search_placeholder;
    spec.search_command_id = product_spec.search_command_id;
    spec.status_text = product_spec.status_text;
    let plugin_host = Rc::new(build_plugin_host(config));
    if let Some(runtime) = plugin_host.runtime() {
        product::merge_plugin_runtime(&mut spec, runtime);
        if !has_persisted_layout {
            product::merge_runtime_startup_tabs(&mut spec, runtime);
        }
        state.borrow_mut().spec = spec.clone();
    }
    let group_handles = Rc::new(RefCell::new(HashMap::new()));
    if let Some(runtime) = plugin_host.runtime() {
        runtime.attach_shell_host(
            config.persistence_id.clone(),
            state.clone(),
            group_handles.clone(),
        );
    }

    let window = ApplicationWindow::builder()
        .application(application)
        .title(&spec.title)
        .default_width(density.window_default_width)
        .default_height(density.window_default_height)
        .build();
    window.add_css_class("app-window");
    let shell = build_shell(
        state.clone(),
        config.persistence_id.clone(),
        group_handles.clone(),
        plugin_host.runtime().cloned(),
        &config.theme.density,
    );
    let registry = commands::shell_registry(
        &window,
        &spec,
        Some(plugin_host.clone()),
        &config.persistence_id,
        Some(state),
        Some(group_handles),
    );
    topbar::install_actions(&window, &spec, &registry);

    let topbar = topbar::build(&spec);

    if let Some(search_command_id) = &spec.search_command_id {
        if let Some(handler) = registry.handler_for(search_command_id) {
            topbar.search.connect_changed(move |entry| {
                let text = entry.text();
                handler(text.as_bytes());
            });
        }
    }

    let root = GtkBox::new(Orientation::Vertical, 0);
    root.add_css_class("app-root");
    root.append(&topbar.root);
    root.append(&shell);
    window.set_child(Some(&root));
    unsafe {
        window.set_data("maruzzella-plugin-host", plugin_host);
    }
    window.present();
}

fn build_plugin_host(config: &MaruzzellaConfig) -> PluginHost {
    let mut plugins = vec![base_plugin::load()];
    let mut diagnostics = Vec::new();
    for loader in &config.builtin_plugins {
        match loader() {
            Ok(plugin) => plugins.push(plugin),
            Err(error) => diagnostics.push(PluginDiagnostic {
                level: PluginDiagnosticLevel::Error,
                plugin_id: None,
                path: None,
                message: format!("builtin plugin load failed: {error:?}"),
            }),
        }
    }
    for path in discovered_plugin_paths(config) {
        match load_plugin(&path) {
            Ok(plugin) => plugins.push(plugin),
            Err(error) => diagnostics.push(diagnostic_for_load_error(&path, &error)),
        }
    }

    match crate::plugins::PluginRuntime::activate_with_persistence_id(
        plugins,
        &config.persistence_id,
    ) {
        Ok(runtime) => {
            runtime.diagnostics.replace(diagnostics.clone());
            PluginHost::new(Some(Rc::new(runtime)), diagnostics)
        }
        Err(error) => {
            diagnostics.push(diagnostic_for_runtime_error(&error));
            PluginHost::new(None, diagnostics)
        }
    }
}

fn discovered_plugin_paths(config: &MaruzzellaConfig) -> Vec<std::path::PathBuf> {
    let mut ordered_paths = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for path in &config.plugin_paths {
        if seen.insert(path.clone()) {
            ordered_paths.push(path.clone());
        }
    }

    let discovery_dirs = if config.enable_default_plugin_discovery {
        crate::default_plugin_discovery_dirs(&config.persistence_id)
            .into_iter()
            .chain(config.plugin_dirs.iter().cloned())
            .collect::<Vec<_>>()
    } else {
        config.plugin_dirs.clone()
    };

    for dir in discovery_dirs {
        for path in crate::discover_plugin_paths_in_dir(&dir) {
            if seen.insert(path.clone()) {
                ordered_paths.push(path);
            }
        }
    }

    ordered_paths
}

fn build_shell(
    state: ShellState,
    persistence_id: String,
    group_handles: GroupHandles,
    plugin_runtime: Option<Rc<PluginRuntime>>,
    density: &theme::ThemeDensity,
) -> gtk::Widget {
    let spec = state.borrow().spec.clone();
    let has_right_panel = !spec.right_panel.tabs.is_empty();
    let left = build_group(
        &spec.left_panel,
        state.clone(),
        persistence_id.clone(),
        plugin_runtime.clone(),
    );
    group_handles
        .borrow_mut()
        .insert(spec.left_panel.id.clone(), left.handle.clone());
    left.root.set_size_request(density.min_side_panel_width, -1);
    let right = has_right_panel.then(|| {
        let right = build_group(
            &spec.right_panel,
            state.clone(),
            persistence_id.clone(),
            plugin_runtime.clone(),
        );
        group_handles
            .borrow_mut()
            .insert(spec.right_panel.id.clone(), right.handle.clone());
        right.root.set_size_request(density.min_side_panel_width, -1);
        right
    });
    let bottom = build_group(
        &spec.bottom_panel,
        state.clone(),
        persistence_id.clone(),
        plugin_runtime.clone(),
    );
    group_handles
        .borrow_mut()
        .insert(spec.bottom_panel.id.clone(), bottom.handle.clone());
    bottom
        .root
        .set_size_request(-1, density.min_bottom_panel_height);
    let workbench = build_workbench_node(
        &spec.workbench,
        state.clone(),
        persistence_id.clone(),
        "workbench-root",
        plugin_runtime,
        &group_handles,
    );

    let left_center = Paned::new(Orientation::Horizontal);
    left_center.set_wide_handle(true);
    left_center.set_resize_start_child(true);
    left_center.set_resize_end_child(true);
    left_center.set_shrink_start_child(false);
    left_center.set_start_child(Some(&left.root));
    left_center.set_end_child(Some(&workbench));
    restore_pane_position(&left_center, &state, "shell.horizontal", 280);
    persist_pane_position(
        &left_center,
        state.clone(),
        persistence_id.clone(),
        "shell.horizontal",
    );

    match spec.bottom_panel_layout {
        BottomPanelLayout::CenterOnly => {
            let vertical = Paned::new(Orientation::Vertical);
            vertical.set_wide_handle(true);
            vertical.set_resize_start_child(true);
            vertical.set_resize_end_child(true);
            vertical.set_shrink_end_child(false);
            vertical.set_start_child(Some(&left_center));
            vertical.set_end_child(Some(&bottom.root));
            restore_pane_position(&vertical, &state, "shell.vertical", 720);
            persist_pane_position(
                &vertical,
                state.clone(),
                persistence_id.clone(),
                "shell.vertical",
            );

            if let Some(right) = right {
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
            } else {
                vertical.upcast::<gtk::Widget>()
            }
        }
        BottomPanelLayout::FullWidth => {
            let upper = if let Some(right) = right {
                let upper = Paned::new(Orientation::Horizontal);
                upper.set_wide_handle(true);
                upper.set_resize_start_child(true);
                upper.set_resize_end_child(true);
                upper.set_shrink_end_child(false);
                upper.set_start_child(Some(&left_center));
                upper.set_end_child(Some(&right.root));
                restore_pane_position(&upper, &state, "shell.outer", 1260);
                persist_pane_position(&upper, state.clone(), persistence_id.clone(), "shell.outer");
                upper.upcast::<gtk::Widget>()
            } else {
                left_center.upcast::<gtk::Widget>()
            };

            let vertical = Paned::new(Orientation::Vertical);
            vertical.set_wide_handle(true);
            vertical.set_resize_start_child(true);
            vertical.set_resize_end_child(true);
            vertical.set_shrink_end_child(false);
            vertical.set_start_child(Some(&upper));
            vertical.set_end_child(Some(&bottom.root));
            restore_pane_position(&vertical, &state, "shell.vertical", 720);
            persist_pane_position(&vertical, state, persistence_id, "shell.vertical");
            vertical.upcast::<gtk::Widget>()
        }
    }
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
        group.show_tab_strip,
        plugin_runtime,
    );
    for tab in &group.tabs {
        if let Some(button) = built.close_buttons.get(&tab.id) {
            crate::base_plugin::bind_editor_close_button(
                tab.plugin_view_id.as_deref(),
                tab.instance_key.as_deref(),
                button,
            );
        }
    }
    for (tab_id, button) in &built.close_buttons {
        let shell_state = state.clone();
        let handle = built.handle.clone();
        let group_id = group.id.clone();
        let persistence_id = persistence_id.clone();
        let tab_id = tab_id.clone();
        button.connect_clicked(move |_| {
            plugin_tabs::close_plugin_view_tab(
                &shell_state,
                &persistence_id,
                &handle,
                &group_id,
                &tab_id,
            );
        });
    }
    install_group_persistence(&built.handle, state, persistence_id);
    built
}

fn build_workbench_node(
    node: &WorkbenchNodeSpec,
    state: ShellState,
    persistence_id: String,
    path: &str,
    plugin_runtime: Option<Rc<PluginRuntime>>,
    group_handles: &GroupHandles,
) -> gtk::Widget {
    match node {
        WorkbenchNodeSpec::Group(group) => {
            let built = build_group(group, state, persistence_id, plugin_runtime);
            group_handles
                .borrow_mut()
                .insert(group.id.clone(), built.handle.clone());
            built.root.upcast::<gtk::Widget>()
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
                        group_handles,
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
    let group_id_for_active = handle.group_id().to_string();
    handle.set_active_changed_handler(move |tab_id| {
        sync_group_into_state(
            &state_for_active,
            &handle_for_active,
            &persistence_id_for_active,
        );
        plugin_tabs::remember_active_plugin_tab(&state_for_active, &group_id_for_active, &tab_id);
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
        if !sync_group_spec(&mut shell.spec, &group_id, &tab_ids, active_tab_id.as_ref()) {
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
    sync_single_group(
        &mut spec.left_panel,
        group_id,
        ordered_tab_ids,
        active_tab_id,
    ) || sync_single_group(
        &mut spec.right_panel,
        group_id,
        ordered_tab_ids,
        active_tab_id,
    ) || sync_single_group(
        &mut spec.bottom_panel,
        group_id,
        ordered_tab_ids,
        active_tab_id,
    ) || sync_workbench_node(
        &mut spec.workbench,
        group_id,
        ordered_tab_ids,
        active_tab_id,
    )
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

fn persist_pane_position(paned: &Paned, state: ShellState, persistence_id: String, pane_id: &str) {
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
