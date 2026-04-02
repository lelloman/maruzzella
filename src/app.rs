use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Box as GtkBox, GestureClick, Orientation, Overlay, Paned,
};

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
use crate::shell::workbench_custom::{
    self, BuiltCustomWorkbenchGroup, CustomWorkbenchGroupHandle, SplitPreviewSide,
};
use crate::spec::PanelResizePolicy;
use crate::spec::{
    BottomPanelLayout, ShellSpec, SplitAxis, TabGroupSpec, TabSpec, WorkbenchNodeSpec,
};
use crate::theme;
use crate::{MaruzzellaConfig, ProductSpec};

type ShellState = Rc<RefCell<PersistedShell>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellMode {
    Launcher,
    Workspace,
}

impl ShellMode {
    fn persistence_slot(self) -> &'static str {
        match self {
            Self::Launcher => "launcher",
            Self::Workspace => "workspace",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShellChrome {
    pub show_menu_bar: bool,
    pub show_toolbar: bool,
    pub show_search: bool,
}

impl ShellChrome {
    pub fn launcher_default() -> Self {
        Self {
            show_menu_bar: false,
            show_toolbar: false,
            show_search: false,
        }
    }

    pub fn workspace_default() -> Self {
        Self {
            show_menu_bar: true,
            show_toolbar: true,
            show_search: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WindowPolicy {
    pub default_width: i32,
    pub default_height: i32,
    pub start_maximized: bool,
}

impl WindowPolicy {
    pub fn new(default_width: i32, default_height: i32) -> Self {
        Self {
            default_width,
            default_height,
            start_maximized: false,
        }
    }

    pub fn with_start_maximized(mut self, start_maximized: bool) -> Self {
        self.start_maximized = start_maximized;
        self
    }

    fn launcher_default() -> Self {
        Self::new(980, 720)
    }

    fn workspace_default(density: &theme::ThemeDensity) -> Self {
        Self::new(density.window_default_width, density.window_default_height).with_start_maximized(true)
    }
}

#[derive(Clone, Debug)]
pub struct LauncherSpec {
    pub title: String,
    pub search_placeholder: String,
    pub search_command_id: Option<String>,
    pub status_text: String,
    pub menu_roots: Vec<crate::spec::MenuRootSpec>,
    pub menu_items: Vec<crate::spec::MenuItemSpec>,
    pub commands: Vec<crate::spec::CommandSpec>,
    pub toolbar_items: Vec<crate::spec::ToolbarItemSpec>,
    pub include_base_toolbar_items: bool,
    pub content: TabGroupSpec,
    pub chrome: ShellChrome,
}

impl LauncherSpec {
    pub fn new(title: impl Into<String>, content: TabGroupSpec) -> Self {
        Self {
            title: title.into(),
            search_placeholder: String::new(),
            search_command_id: None,
            status_text: String::new(),
            menu_roots: Vec::new(),
            menu_items: Vec::new(),
            commands: Vec::new(),
            toolbar_items: Vec::new(),
            include_base_toolbar_items: false,
            content,
            chrome: ShellChrome::launcher_default(),
        }
    }

    pub fn shell_spec(&self) -> ShellSpec {
        ShellSpec {
            title: self.title.clone(),
            search_placeholder: self.search_placeholder.clone(),
            search_command_id: self.search_command_id.clone(),
            status_text: self.status_text.clone(),
            app_appearance_id: "app-shell".to_string(),
            topbar_appearance_id: "topbar".to_string(),
            menu_appearance_id: "menu".to_string(),
            toolbar_appearance_id: "toolbar".to_string(),
            search_input_appearance_id: "search".to_string(),
            status_appearance_id: "status".to_string(),
            button_appearance_id: "secondary".to_string(),
            text_appearance_id: "body".to_string(),
            bottom_panel_layout: BottomPanelLayout::CenterOnly,
            menu_roots: self.menu_roots.clone(),
            menu_items: self.menu_items.clone(),
            commands: self.commands.clone(),
            toolbar_items: self.toolbar_items.clone(),
            left_panel: TabGroupSpec::new("launcher-left", None, Vec::new()),
            right_panel: TabGroupSpec::new("launcher-right", None, Vec::new()),
            bottom_panel: TabGroupSpec::new("launcher-bottom", None, Vec::new()),
            workbench: WorkbenchNodeSpec::Group(self.content.clone()),
            left_panel_resize: PanelResizePolicy::Fixed,
            right_panel_resize: PanelResizePolicy::Fixed,
            bottom_panel_resize: PanelResizePolicy::Fixed,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct WorkspaceSession {
    pub project_handle: Option<Vec<u8>>,
    pub shell_spec: Option<ShellSpec>,
    pub window_policy: Option<WindowPolicy>,
}

impl WorkspaceSession {
    pub fn new(shell_spec: ShellSpec) -> Self {
        Self {
            project_handle: None,
            shell_spec: Some(shell_spec),
            window_policy: None,
        }
    }

    pub fn from_product(product: &ProductSpec) -> Self {
        Self::new(product.shell_spec())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModeSwitchError {
    NotActivated,
    MissingLauncherSpec,
}

impl fmt::Display for ModeSwitchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotActivated => write!(f, "application has not been activated yet"),
            Self::MissingLauncherSpec => write!(f, "no launcher spec is configured"),
        }
    }
}

impl std::error::Error for ModeSwitchError {}

#[derive(Clone, Default)]
pub struct MaruzzellaHandle {
    controller: Rc<RefCell<Option<Rc<AppController>>>>,
}

impl MaruzzellaHandle {
    pub fn switch_to_workspace(&self, session: WorkspaceSession) -> Result<(), ModeSwitchError> {
        let Some(controller) = self.controller.borrow().clone() else {
            return Err(ModeSwitchError::NotActivated);
        };
        controller.switch_to_workspace(session);
        Ok(())
    }

    pub fn switch_to_launcher(&self) -> Result<(), ModeSwitchError> {
        let Some(controller) = self.controller.borrow().clone() else {
            return Err(ModeSwitchError::NotActivated);
        };
        controller.switch_to_launcher()
    }

    pub fn current_mode(&self) -> Result<ShellMode, ModeSwitchError> {
        let Some(controller) = self.controller.borrow().clone() else {
            return Err(ModeSwitchError::NotActivated);
        };
        Ok(controller.current_mode())
    }

    pub fn current_project_handle(&self) -> Result<Option<Vec<u8>>, ModeSwitchError> {
        let Some(controller) = self.controller.borrow().clone() else {
            return Err(ModeSwitchError::NotActivated);
        };
        Ok(controller.current_project_handle())
    }
}

#[derive(Default)]
struct WorkbenchDragContext {
    source_group_id: Option<String>,
    target_group_id: Option<String>,
    target_index: Option<usize>,
}

#[derive(Default)]
struct PanePositionController {
    suppress_persist_depth: std::cell::Cell<u32>,
    last_bucket: std::cell::Cell<i32>,
}

impl PanePositionController {
    fn should_persist(&self) -> bool {
        self.suppress_persist_depth.get() == 0
    }

    fn is_programmatic_update(&self) -> bool {
        self.suppress_persist_depth.get() > 0
    }

    fn last_bucket(&self) -> i32 {
        self.last_bucket.get()
    }

    fn set_last_bucket(&self, bucket: i32) {
        self.last_bucket.set(bucket);
    }

    fn run_programmatic_update(&self, update: impl FnOnce()) {
        self.suppress_persist_depth
            .set(self.suppress_persist_depth.get().saturating_add(1));
        update();
        self.suppress_persist_depth
            .set(self.suppress_persist_depth.get().saturating_sub(1));
    }
}

struct AppController {
    window: ApplicationWindow,
    config: MaruzzellaConfig,
    plugin_host: Rc<PluginHost>,
    mode: RefCell<ShellMode>,
    project_handle: RefCell<Option<Vec<u8>>>,
    installed_actions: RefCell<Vec<String>>,
}

impl AppController {
    fn new(window: ApplicationWindow, config: MaruzzellaConfig, plugin_host: Rc<PluginHost>) -> Rc<Self> {
        Rc::new(Self {
            window,
            config,
            plugin_host,
            mode: RefCell::new(ShellMode::Workspace),
            project_handle: RefCell::new(None),
            installed_actions: RefCell::new(Vec::new()),
        })
    }

    fn show_startup_mode(&self) -> Result<(), ModeSwitchError> {
        match self.config.startup_mode {
            ShellMode::Launcher => self.show_launcher(self.config.launcher.clone()),
            ShellMode::Workspace => {
                self.show_workspace(WorkspaceSession::from_product(&self.config.product));
                Ok(())
            }
        }
    }

    fn switch_to_workspace(&self, session: WorkspaceSession) {
        eprintln!(
            "maruzzella: AppController::switch_to_workspace project_handle_present={}",
            session.project_handle.is_some()
        );
        self.show_workspace(session);
    }

    fn switch_to_launcher(&self) -> Result<(), ModeSwitchError> {
        self.show_launcher(self.config.launcher.clone())
    }

    fn current_mode(&self) -> ShellMode {
        *self.mode.borrow()
    }

    fn current_project_handle(&self) -> Option<Vec<u8>> {
        self.project_handle.borrow().clone()
    }

    fn show_workspace(&self, session: WorkspaceSession) {
        eprintln!(
            "maruzzella: AppController::show_workspace project_handle_len={}",
            session.project_handle.as_ref().map(|bytes| bytes.len()).unwrap_or(0)
        );
        let shell_spec = session
            .shell_spec
            .unwrap_or_else(|| self.config.product.shell_spec());
        let policy = session
            .window_policy
            .or_else(|| self.config.workspace_window_policy.clone())
            .unwrap_or_else(|| WindowPolicy::workspace_default(&self.config.theme.density));
        self.project_handle.replace(session.project_handle);
        self.render_mode(
            ShellMode::Workspace,
            shell_spec,
            policy,
            ShellChrome::workspace_default(),
            self.config.product.include_base_toolbar_items,
        );
    }

    fn show_launcher(&self, launcher: Option<LauncherSpec>) -> Result<(), ModeSwitchError> {
        let Some(launcher) = launcher else {
            return Err(ModeSwitchError::MissingLauncherSpec);
        };
        let policy = self
            .config
            .launcher_window_policy
            .clone()
            .unwrap_or_else(WindowPolicy::launcher_default);
        self.project_handle.replace(None);
        self.render_mode(
            ShellMode::Launcher,
            launcher.shell_spec(),
            policy,
            launcher.chrome,
            launcher.include_base_toolbar_items,
        );
        Ok(())
    }

    fn render_mode(
        &self,
        mode: ShellMode,
        default_spec: ShellSpec,
        window_policy: WindowPolicy,
        chrome: ShellChrome,
        include_base_toolbar_items: bool,
    ) {
        eprintln!(
            "maruzzella: render_mode mode={:?} persistence_slot={}",
            mode,
            mode.persistence_slot()
        );
        let persistence_slot = mode.persistence_slot();
        let layout_persistence_id =
            layout::scoped_persistence_id(&self.config.persistence_id, persistence_slot);
        let has_persisted_layout =
            layout::path_for_slot(&self.config.persistence_id, persistence_slot).exists();
        let state = Rc::new(RefCell::new(layout::load_for_slot(
            &self.config.persistence_id,
            persistence_slot,
            &default_spec,
        )));
        let mut spec = state.borrow().spec.clone();
        if let Some(runtime) = self.plugin_host.runtime() {
            product::merge_plugin_runtime(&mut spec, runtime, include_base_toolbar_items);
            if mode == ShellMode::Workspace && !has_persisted_layout {
                product::merge_runtime_startup_tabs(&mut spec, runtime);
            }
            state.borrow_mut().spec = spec.clone();
        }

        let group_handles = Rc::new(RefCell::new(HashMap::new()));
        if let Some(runtime) = self.plugin_host.runtime() {
            runtime.attach_shell_host(
                self.window.clone(),
                layout_persistence_id.clone(),
                self.config.persistence_id.clone(),
                state.clone(),
                group_handles.clone(),
            );
        }

        let (shell, mut pane_roots) = match mode {
            ShellMode::Launcher => build_launcher_shell(
                state.clone(),
                layout_persistence_id.clone(),
                group_handles.clone(),
                self.plugin_host.runtime().cloned(),
            ),
            ShellMode::Workspace => build_shell(
                state.clone(),
                layout_persistence_id.clone(),
                group_handles.clone(),
                self.plugin_host.runtime().cloned(),
                &self.config.theme.density,
            ),
        };

        let registry = commands::shell_registry(
            &self.window,
            &spec,
            Some(self.plugin_host.clone()),
            &layout_persistence_id,
            Some(state),
            Some(group_handles),
        );

        for action_name in self.installed_actions.borrow_mut().drain(..) {
            self.window.remove_action(&action_name);
        }
        let installed_actions = topbar::install_actions(&self.window, &spec, &registry);
        self.installed_actions.replace(installed_actions);

        let root = GtkBox::new(Orientation::Vertical, 0);
        root.add_css_class("app-root");
        root.add_css_class(&theme::surface_css_class(&spec.app_appearance_id));
        let app_overlay = Overlay::new();
        app_overlay.set_child(Some(&root));

        if let Some(topbar) = topbar::build(&spec, chrome) {
            if let Some(search_command_id) = &spec.search_command_id {
                if let Some(handler) = registry.handler_for(search_command_id) {
                    topbar.search.connect_changed(move |entry| {
                        let text = entry.text();
                        handler(text.as_bytes());
                    });
                }
            }
            pane_roots.push(topbar.root.clone().upcast());
            root.append(&topbar.root);
            topbar.install_tooltip_overlay(&app_overlay);
        }

        install_pane_focus_tracking(&pane_roots);
        root.append(&shell);

        self.window.set_title(Some(&spec.title));
        self.window.set_default_size(window_policy.default_width, window_policy.default_height);
        if window_policy.start_maximized {
            self.window.maximize();
        } else {
            self.window.unmaximize();
        }
        self.window.set_child(Some(&app_overlay));
        self.window.present();
        self.mode.replace(mode);
    }
}

pub fn build(application: &Application, config: &MaruzzellaConfig, handle: &MaruzzellaHandle) {
    theme::install(config.theme.clone());
    let plugin_host = Rc::new(build_plugin_host(config));
    let window = ApplicationWindow::builder()
        .application(application)
        .title(&config.product.branding.title)
        .build();
    window.add_css_class("app-window");
    unsafe {
        window.set_data("maruzzella-handle", handle.clone());
        window.set_data("maruzzella-plugin-host", plugin_host.clone());
    }
    let controller = AppController::new(window, config.clone(), plugin_host);
    handle.controller.replace(Some(controller.clone()));
    let _ = controller.show_startup_mode();
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
) -> (gtk::Widget, Vec<gtk::Widget>) {
    let spec = state.borrow().spec.clone();
    let has_right_panel = !spec.right_panel.tabs.is_empty();
    let left = build_group(
        &spec.left_panel,
        state.clone(),
        persistence_id.clone(),
        plugin_runtime.clone(),
        group_handles.clone(),
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
            group_handles.clone(),
        );
        group_handles
            .borrow_mut()
            .insert(spec.right_panel.id.clone(), right.handle.clone());
        right
            .root
            .set_size_request(density.min_side_panel_width, -1);
        right
    });
    let bottom = build_group(
        &spec.bottom_panel,
        state.clone(),
        persistence_id.clone(),
        plugin_runtime.clone(),
        group_handles.clone(),
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
        plugin_runtime.clone(),
        &group_handles,
    );
    let workbench_drag_context = Rc::new(RefCell::new(WorkbenchDragContext::default()));
    install_workbench_interactions_for_handles(
        &group_handles,
        &workbench,
        state.clone(),
        persistence_id.clone(),
        plugin_runtime.clone(),
        workbench_drag_context,
    );

    let mut pane_roots: Vec<gtk::Widget> = vec![
        left.root.clone().upcast(),
        bottom.root.clone().upcast(),
        workbench.clone(),
    ];
    if let Some(ref right) = right {
        pane_roots.push(right.root.clone().upcast());
    }

    let left_center = Paned::new(Orientation::Horizontal);
    let left_center_controller = Rc::new(PanePositionController::default());
    left_center.set_wide_handle(true);
    left_center.set_shrink_start_child(false);
    left_center.set_start_child(Some(&left.root));
    left_center.set_end_child(Some(&workbench));
    apply_start_panel_resize_policy(
        &left_center,
        spec.left_panel_resize,
        left_center_controller.clone(),
        state.clone(),
        "shell.horizontal".to_string(),
        density.min_side_panel_width,
    );
    restore_pane_position(
        &left_center,
        &state,
        &persistence_id,
        left_center_controller.clone(),
        "shell.horizontal",
        280,
    );

    let bottom_resize = spec.bottom_panel_resize;
    let right_resize = spec.right_panel_resize;
    let min_side = density.min_side_panel_width;
    let min_bottom = density.min_bottom_panel_height;

    let shell = match spec.bottom_panel_layout {
        BottomPanelLayout::CenterOnly => {
            let vertical = Paned::new(Orientation::Vertical);
            let vertical_controller = Rc::new(PanePositionController::default());
            vertical.set_wide_handle(true);
            vertical.set_shrink_end_child(false);
            vertical.set_start_child(Some(&left_center));
            vertical.set_end_child(Some(&bottom.root));
            apply_end_panel_resize_policy(
                &vertical,
                bottom_resize,
                vertical_controller.clone(),
                state.clone(),
                "shell.vertical".to_string(),
                min_bottom,
            );
            restore_pane_position(
                &vertical,
                &state,
                &persistence_id,
                vertical_controller.clone(),
                "shell.vertical",
                720,
            );

            if let Some(right) = right {
                let outer = Paned::new(Orientation::Horizontal);
                let outer_controller = Rc::new(PanePositionController::default());
                outer.set_wide_handle(true);
                outer.set_shrink_end_child(false);
                outer.set_start_child(Some(&vertical));
                outer.set_end_child(Some(&right.root));
                apply_end_panel_resize_policy(
                    &outer,
                    right_resize,
                    outer_controller.clone(),
                    state.clone(),
                    "shell.outer".to_string(),
                    min_side,
                );
                restore_pane_position(
                    &outer,
                    &state,
                    &persistence_id,
                    outer_controller.clone(),
                    "shell.outer",
                    1260,
                );
                outer.upcast::<gtk::Widget>()
            } else {
                vertical.upcast::<gtk::Widget>()
            }
        }
        BottomPanelLayout::FullWidth => {
            let upper = if let Some(right) = right {
                let upper = Paned::new(Orientation::Horizontal);
                let upper_controller = Rc::new(PanePositionController::default());
                upper.set_wide_handle(true);
                upper.set_shrink_end_child(false);
                upper.set_start_child(Some(&left_center));
                upper.set_end_child(Some(&right.root));
                apply_end_panel_resize_policy(
                    &upper,
                    right_resize,
                    upper_controller.clone(),
                    state.clone(),
                    "shell.outer".to_string(),
                    min_side,
                );
                restore_pane_position(
                    &upper,
                    &state,
                    &persistence_id,
                    upper_controller.clone(),
                    "shell.outer",
                    1260,
                );
                upper.upcast::<gtk::Widget>()
            } else {
                left_center.upcast::<gtk::Widget>()
            };

            let vertical = Paned::new(Orientation::Vertical);
            let vertical_controller = Rc::new(PanePositionController::default());
            vertical.set_wide_handle(true);
            vertical.set_shrink_end_child(false);
            vertical.set_start_child(Some(&upper));
            vertical.set_end_child(Some(&bottom.root));
            apply_end_panel_resize_policy(
                &vertical,
                bottom_resize,
                vertical_controller.clone(),
                state.clone(),
                "shell.vertical".to_string(),
                min_bottom,
            );
            restore_pane_position(
                &vertical,
                &state,
                &persistence_id,
                vertical_controller.clone(),
                "shell.vertical",
                720,
            );
            vertical.upcast::<gtk::Widget>()
        }
    };

    (shell, pane_roots)
}

fn build_launcher_shell(
    state: ShellState,
    persistence_id: String,
    group_handles: GroupHandles,
    plugin_runtime: Option<Rc<PluginRuntime>>,
) -> (gtk::Widget, Vec<gtk::Widget>) {
    let spec = state.borrow().spec.clone();
    let Some(group) = first_workbench_group(&spec.workbench) else {
        let root = GtkBox::new(Orientation::Vertical, 0);
        root.set_hexpand(true);
        root.set_vexpand(true);
        return (root.upcast::<gtk::Widget>(), Vec::new());
    };

    let built = build_group(
        group,
        state,
        persistence_id,
        plugin_runtime,
        group_handles.clone(),
    );
    group_handles
        .borrow_mut()
        .insert(group.id.clone(), built.handle.clone());

    let root = GtkBox::new(Orientation::Vertical, 0);
    root.add_css_class("launcher-shell");
    root.set_hexpand(true);
    root.set_vexpand(true);
    root.append(&built.root);

    (root.upcast::<gtk::Widget>(), vec![built.root.upcast()])
}

fn first_workbench_group(node: &WorkbenchNodeSpec) -> Option<&TabGroupSpec> {
    match node {
        WorkbenchNodeSpec::Group(group) => Some(group),
        WorkbenchNodeSpec::Split { children, .. } => children.iter().find_map(first_workbench_group),
    }
}

fn build_group(
    group: &TabGroupSpec,
    state: ShellState,
    persistence_id: String,
    plugin_runtime: Option<Rc<PluginRuntime>>,
    group_handles: GroupHandles,
) -> BuiltCustomWorkbenchGroup {
    let extra_classes: Vec<&str> =
        if group.id.starts_with("workbench") || group.id.starts_with("panel-bottom") {
            vec!["dark-pane"]
        } else {
            vec![]
        };
    let built = workbench_custom::build_group(
        &group.id,
        &extra_classes,
        &group.tabs,
        group.active_tab_id.as_deref(),
        group.show_tab_strip,
        &group.panel_appearance_id,
        &group.panel_header_appearance_id,
        &group.tab_strip_appearance_id,
        &group.text_appearance_id,
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
        let group_handles = group_handles.clone();
        let handle = built.handle.clone();
        let group_id = group.id.clone();
        let persistence_id = persistence_id.clone();
        let tab_id = tab_id.clone();
        button.connect_clicked(move |_| {
            let closed = plugin_tabs::close_plugin_view_tab(
                &shell_state,
                &persistence_id,
                Some(&group_handles),
                &handle,
                &group_id,
                &tab_id,
            );
            if closed && handle.tab_ids().is_empty() && group_id.starts_with("workbench") {
                collapse_empty_group_widget(&handle.widget());
            }
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
            let built = build_group(
                group,
                state.clone(),
                persistence_id.clone(),
                plugin_runtime.clone(),
                group_handles.clone(),
            );
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
                let paned_controller = Rc::new(PanePositionController::default());
                paned.set_wide_handle(true);
                paned.set_resize_start_child(true);
                paned.set_resize_end_child(true);
                paned.set_start_child(Some(&current));
                paned.set_end_child(Some(&child));
                let pane_id = format!("{path}:split:{index}");
                restore_pane_position(
                    &paned,
                    &state,
                    &persistence_id,
                    paned_controller.clone(),
                    &pane_id,
                    520,
                );
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

fn install_workbench_group_interactions(
    handle: &CustomWorkbenchGroupHandle,
    workbench_root: &gtk::Widget,
    state: ShellState,
    persistence_id: String,
    plugin_runtime: Option<Rc<PluginRuntime>>,
    group_handles: GroupHandles,
    drag_context: Rc<RefCell<WorkbenchDragContext>>,
) {
    let source_handle = handle.clone();
    let source_handle_for_hover = handle.clone();
    let source_handle_for_drop = handle.clone();
    let workbench_root_for_split = workbench_root.clone();
    let workbench_root_for_hover = workbench_root.clone();
    let group_handles_for_hover = group_handles.clone();
    let group_handles_for_split = group_handles.clone();
    let group_handles_for_drop = group_handles.clone();
    let drag_context_for_hover = drag_context.clone();
    let drag_context_for_split = drag_context.clone();
    let drag_context_for_drop = drag_context.clone();
    let state_for_split = state.clone();
    let state_for_drop = state.clone();
    let persistence_id_for_split = persistence_id.clone();
    let persistence_id_for_drop = persistence_id.clone();
    let plugin_runtime_for_split = plugin_runtime.clone();
    let plugin_runtime_for_drop = plugin_runtime.clone();

    handle.set_drag_hover_handler(move |tab_id, pointer_x, pointer_y, _drag_height| {
        update_cross_group_drop_target(
            &source_handle_for_hover,
            &tab_id,
            pointer_x,
            pointer_y,
            &workbench_root_for_hover,
            &group_handles_for_hover,
            &drag_context_for_hover,
        );
    });

    handle.set_split_drop_handler(move |tab_id, side| {
        let source_group_id = source_handle.group_id().to_string();
        let Some((new_group, split_position)) = split_workbench_group_in_state(
            &state_for_split,
            &persistence_id_for_split,
            &source_group_id,
            &tab_id,
            side,
        ) else {
            return;
        };

        clear_drop_placeholders(&group_handles_for_split);
        reset_drag_context(&drag_context_for_split);

        source_handle.remove_tab(&tab_id);

        let built = build_group(
            &new_group,
            state_for_split.clone(),
            persistence_id_for_split.clone(),
            plugin_runtime_for_split.clone(),
            group_handles_for_split.clone(),
        );
        install_workbench_group_interactions(
            &built.handle,
            &workbench_root_for_split,
            state_for_split.clone(),
            persistence_id_for_split.clone(),
            plugin_runtime_for_split.clone(),
            group_handles_for_split.clone(),
            drag_context_for_split.clone(),
        );
        group_handles_for_split
            .borrow_mut()
            .insert(new_group.id.clone(), built.handle.clone());

        replace_group_widget_with_split(&source_handle.widget(), &built.root, split_position);
    });

    handle.set_tab_drop_handler(move |tab_id| {
        let (target_group_id, target_index) = {
            let context = drag_context_for_drop.borrow();
            match (
                context.source_group_id.as_deref(),
                context.target_group_id.as_ref(),
                context.target_index,
            ) {
                (Some(source_group_id), Some(target_group_id), Some(target_index))
                    if source_group_id == source_handle_for_drop.group_id()
                        && target_group_id != source_group_id =>
                {
                    (target_group_id.clone(), target_index)
                }
                _ => {
                    clear_drop_placeholders(&group_handles_for_drop);
                    reset_drag_context(&drag_context_for_drop);
                    return;
                }
            }
        };

        clear_drop_placeholders(&group_handles_for_drop);
        reset_drag_context(&drag_context_for_drop);

        let Some(target_handle) = group_handles_for_drop
            .borrow()
            .get(&target_group_id)
            .cloned()
        else {
            return;
        };
        let Some((moved_tab, source_became_empty)) = move_workbench_tab_between_groups_in_state(
            &state_for_drop,
            &persistence_id_for_drop,
            source_handle_for_drop.group_id(),
            &target_group_id,
            &tab_id,
            target_index,
        ) else {
            return;
        };

        source_handle_for_drop.remove_tab(&tab_id);
        let page = crate::shell::tabbed_panel::build_tab_page(
            "workbench",
            &moved_tab,
            plugin_runtime_for_drop.as_ref(),
        );
        if let Some(close_button) = page.close_button.clone() {
            crate::base_plugin::bind_editor_close_button(
                moved_tab.plugin_view_id.as_deref(),
                moved_tab.instance_key.as_deref(),
                &close_button,
            );
            let shell_state = state_for_drop.clone();
            let persistence_id = persistence_id_for_drop.clone();
            let group_handles = group_handles_for_drop.clone();
            let handle = target_handle.clone();
            let group_id = target_group_id.clone();
            let tab_id = moved_tab.id.clone();
            close_button.connect_clicked(move |_| {
                plugin_tabs::close_plugin_view_tab(
                    &shell_state,
                    &persistence_id,
                    Some(&group_handles),
                    &handle,
                    &group_id,
                    &tab_id,
                );
            });
        }
        target_handle.append_page(page, true);
        target_handle.move_tab_to_index(&moved_tab.id, target_index);
        target_handle.set_active_tab(&moved_tab.id);

        if source_became_empty {
            group_handles_for_drop
                .borrow_mut()
                .remove(source_handle_for_drop.group_id());
            collapse_empty_group_widget(&source_handle_for_drop.widget());
        }
    });
}

fn install_workbench_interactions_for_handles(
    group_handles: &GroupHandles,
    workbench_root: &gtk::Widget,
    state: ShellState,
    persistence_id: String,
    plugin_runtime: Option<Rc<PluginRuntime>>,
    drag_context: Rc<RefCell<WorkbenchDragContext>>,
) {
    for handle in group_handles.borrow().values() {
        if handle.group_id().starts_with("workbench") {
            install_workbench_group_interactions(
                handle,
                workbench_root,
                state.clone(),
                persistence_id.clone(),
                plugin_runtime.clone(),
                group_handles.clone(),
                drag_context.clone(),
            );
        }
    }
}

fn update_cross_group_drop_target(
    source_handle: &CustomWorkbenchGroupHandle,
    tab_id: &str,
    pointer_x: f64,
    pointer_y: f64,
    workbench_root: &gtk::Widget,
    group_handles: &GroupHandles,
    drag_context: &Rc<RefCell<WorkbenchDragContext>>,
) {
    let Some((source_x, source_y, _, _)) = source_handle.bounds_in(workbench_root) else {
        clear_drop_placeholders(group_handles);
        reset_drag_context(drag_context);
        return;
    };
    let host_x = source_x + pointer_x;
    let host_y = source_y + pointer_y;

    let mut hovered_target = None;
    for handle in group_handles.borrow().values() {
        if !handle.group_id().starts_with("workbench") {
            continue;
        }
        let Some((group_x, group_y, group_width, _)) = handle.bounds_in(workbench_root) else {
            continue;
        };
        let local_x = host_x - group_x;
        let local_y = host_y - group_y;
        if local_x < 0.0 || local_x > group_width {
            continue;
        }
        if local_y >= 0.0 && local_y <= handle.strip_band_height() {
            hovered_target = Some((handle.clone(), local_x));
            break;
        }
    }

    clear_drop_placeholders(group_handles);
    let mut context = drag_context.borrow_mut();
    context.source_group_id = Some(source_handle.group_id().to_string());
    context.target_group_id = None;
    context.target_index = None;

    let Some((target_handle, local_x)) = hovered_target else {
        return;
    };
    if target_handle.group_id() == source_handle.group_id() {
        return;
    }

    let target_index = target_handle.insertion_index_for_local_x(tab_id, local_x);
    target_handle.show_drop_placeholder(target_index, 120);
    context.target_group_id = Some(target_handle.group_id().to_string());
    context.target_index = Some(target_index);
}

fn clear_drop_placeholders(group_handles: &GroupHandles) {
    for handle in group_handles.borrow().values() {
        if handle.group_id().starts_with("workbench") {
            handle.hide_drop_placeholder();
        }
    }
}

fn reset_drag_context(drag_context: &Rc<RefCell<WorkbenchDragContext>>) {
    if let Ok(mut context) = drag_context.try_borrow_mut() {
        *context = WorkbenchDragContext::default();
    }
}

fn split_workbench_group_in_state(
    state: &ShellState,
    persistence_id: &str,
    group_id: &str,
    tab_id: &str,
    side: SplitPreviewSide,
) -> Option<(TabGroupSpec, SplitPreviewSide)> {
    let mut shell = state.borrow_mut();
    let new_group_id = next_split_group_id(&shell.spec.workbench, group_id);
    let new_group = split_workbench_node(
        &mut shell.spec.workbench,
        group_id,
        tab_id,
        side,
        &new_group_id,
    )?;
    let snapshot = shell.clone();
    drop(shell);
    layout::save(persistence_id, &snapshot);
    Some((new_group, side))
}

fn move_workbench_tab_between_groups_in_state(
    state: &ShellState,
    persistence_id: &str,
    source_group_id: &str,
    target_group_id: &str,
    tab_id: &str,
    target_index: usize,
) -> Option<(TabSpec, bool)> {
    let mut shell = state.borrow_mut();
    let mut moved_tab = {
        let source_group = find_workbench_group_mut(&mut shell.spec.workbench, source_group_id)?;
        let source_index = source_group.tabs.iter().position(|tab| tab.id == tab_id)?;
        let moved_tab = source_group.tabs.remove(source_index);
        if source_group.active_tab_id.as_deref() == Some(tab_id) {
            source_group.active_tab_id = source_group.tabs.first().map(|tab| tab.id.clone());
        }
        moved_tab
    };

    moved_tab.panel_id = target_group_id.to_string();
    {
        let target_group = find_workbench_group_mut(&mut shell.spec.workbench, target_group_id)?;
        let insert_at = target_index.min(target_group.tabs.len());
        target_group.tabs.insert(insert_at, moved_tab.clone());
        target_group.active_tab_id = Some(moved_tab.id.clone());
    }
    normalize_workbench_node(&mut shell.spec.workbench);
    let source_became_empty = find_workbench_group(&shell.spec.workbench, source_group_id)
        .is_none_or(|group| group.tabs.is_empty());
    let snapshot = shell.clone();
    drop(shell);
    layout::save(persistence_id, &snapshot);
    Some((moved_tab, source_became_empty))
}

fn next_split_group_id(node: &WorkbenchNodeSpec, base_group_id: &str) -> String {
    let mut suffix = 2usize;
    loop {
        let candidate = format!("{base_group_id}-split-{suffix}");
        if !workbench_group_id_exists(node, &candidate) {
            return candidate;
        }
        suffix += 1;
    }
}

fn workbench_group_id_exists(node: &WorkbenchNodeSpec, group_id: &str) -> bool {
    match node {
        WorkbenchNodeSpec::Group(group) => group.id == group_id,
        WorkbenchNodeSpec::Split { children, .. } => children
            .iter()
            .any(|child| workbench_group_id_exists(child, group_id)),
    }
}

fn find_workbench_group<'a>(
    node: &'a WorkbenchNodeSpec,
    group_id: &str,
) -> Option<&'a TabGroupSpec> {
    match node {
        WorkbenchNodeSpec::Group(group) => (group.id == group_id).then_some(group),
        WorkbenchNodeSpec::Split { children, .. } => children
            .iter()
            .find_map(|child| find_workbench_group(child, group_id)),
    }
}

fn find_workbench_group_mut<'a>(
    node: &'a mut WorkbenchNodeSpec,
    group_id: &str,
) -> Option<&'a mut TabGroupSpec> {
    match node {
        WorkbenchNodeSpec::Group(group) => (group.id == group_id).then_some(group),
        WorkbenchNodeSpec::Split { children, .. } => children
            .iter_mut()
            .find_map(|child| find_workbench_group_mut(child, group_id)),
    }
}

fn normalize_workbench_node(node: &mut WorkbenchNodeSpec) -> bool {
    match node {
        WorkbenchNodeSpec::Group(group) => group.tabs.is_empty(),
        WorkbenchNodeSpec::Split { children, .. } => {
            let mut index = 0usize;
            while index < children.len() {
                if normalize_workbench_node(&mut children[index]) {
                    children.remove(index);
                } else {
                    index += 1;
                }
            }
            if children.is_empty() {
                true
            } else if children.len() == 1 {
                *node = children.remove(0);
                false
            } else {
                false
            }
        }
    }
}

fn split_workbench_node(
    node: &mut WorkbenchNodeSpec,
    group_id: &str,
    tab_id: &str,
    side: SplitPreviewSide,
    new_group_id: &str,
) -> Option<TabGroupSpec> {
    match node {
        WorkbenchNodeSpec::Group(group) => {
            if group.id != group_id || group.tabs.len() <= 1 {
                return None;
            }

            let dragged_index = group.tabs.iter().position(|tab| tab.id == tab_id)?;
            let mut dragged_tab = group.tabs.remove(dragged_index);
            dragged_tab.panel_id = new_group_id.to_string();

            if group.active_tab_id.as_deref() == Some(tab_id) {
                group.active_tab_id = group.tabs.first().map(|tab| tab.id.clone());
            }

            let new_group = TabGroupSpec {
                id: new_group_id.to_string(),
                active_tab_id: Some(dragged_tab.id.clone()),
                show_tab_strip: group.show_tab_strip,
                panel_appearance_id: group.panel_appearance_id.clone(),
                panel_header_appearance_id: group.panel_header_appearance_id.clone(),
                tab_strip_appearance_id: group.tab_strip_appearance_id.clone(),
                text_appearance_id: group.text_appearance_id.clone(),
                tabs: vec![dragged_tab],
            };
            let existing_group = group.clone();
            let split = match side {
                SplitPreviewSide::Left => WorkbenchNodeSpec::Split {
                    axis: SplitAxis::Horizontal,
                    children: vec![
                        WorkbenchNodeSpec::Group(new_group.clone()),
                        WorkbenchNodeSpec::Group(existing_group),
                    ],
                },
                SplitPreviewSide::Right => WorkbenchNodeSpec::Split {
                    axis: SplitAxis::Horizontal,
                    children: vec![
                        WorkbenchNodeSpec::Group(existing_group),
                        WorkbenchNodeSpec::Group(new_group.clone()),
                    ],
                },
                SplitPreviewSide::Bottom => WorkbenchNodeSpec::Split {
                    axis: SplitAxis::Vertical,
                    children: vec![
                        WorkbenchNodeSpec::Group(existing_group),
                        WorkbenchNodeSpec::Group(new_group.clone()),
                    ],
                },
            };
            *node = split;
            Some(new_group)
        }
        WorkbenchNodeSpec::Split { children, .. } => children
            .iter_mut()
            .find_map(|child| split_workbench_node(child, group_id, tab_id, side, new_group_id)),
    }
}

fn replace_group_widget_with_split<W: IsA<gtk::Widget>, N: IsA<gtk::Widget>>(
    current_group: &W,
    new_group: &N,
    side: SplitPreviewSide,
) {
    let current_widget = current_group.clone().upcast::<gtk::Widget>();
    let new_widget = new_group.clone().upcast::<gtk::Widget>();
    let Some(parent) = current_widget.parent() else {
        return;
    };
    let Ok(parent_paned) = parent.downcast::<Paned>() else {
        return;
    };
    let is_start_child = parent_paned
        .start_child()
        .map(|child| child.as_ptr() == current_widget.as_ptr())
        .unwrap_or(false);

    let axis = match side {
        SplitPreviewSide::Left | SplitPreviewSide::Right => Orientation::Horizontal,
        SplitPreviewSide::Bottom => Orientation::Vertical,
    };
    let split = Paned::new(axis);
    split.set_wide_handle(true);
    split.set_resize_start_child(true);
    split.set_resize_end_child(true);
    split.set_shrink_start_child(false);
    split.set_shrink_end_child(false);

    let default_position = match axis {
        Orientation::Horizontal => (current_widget.width() / 2).max(220),
        Orientation::Vertical => (current_widget.height() / 2).max(180),
        _ => 220,
    };
    if is_start_child {
        parent_paned.set_start_child(None::<&gtk::Widget>);
    } else {
        parent_paned.set_end_child(None::<&gtk::Widget>);
    }

    match side {
        SplitPreviewSide::Left => {
            split.set_start_child(Some(&new_widget));
            split.set_end_child(Some(&current_widget));
        }
        SplitPreviewSide::Right | SplitPreviewSide::Bottom => {
            split.set_start_child(Some(&current_widget));
            split.set_end_child(Some(&new_widget));
        }
    }

    split.set_position(default_position);

    if is_start_child {
        parent_paned.set_start_child(Some(&split));
    } else {
        parent_paned.set_end_child(Some(&split));
    }
}

fn collapse_empty_group_widget<W: IsA<gtk::Widget>>(empty_group: &W) {
    let empty_widget = empty_group.clone().upcast::<gtk::Widget>();
    let Some(parent) = empty_widget.parent() else {
        return;
    };
    let Ok(parent_paned) = parent.downcast::<Paned>() else {
        return;
    };
    let sibling = if parent_paned
        .start_child()
        .map(|child| child.as_ptr() == empty_widget.as_ptr())
        .unwrap_or(false)
    {
        parent_paned.end_child()
    } else {
        parent_paned.start_child()
    };
    let Some(sibling) = sibling else {
        return;
    };
    let Some(grandparent) = parent_paned.parent() else {
        return;
    };

    let Ok(grandparent_paned) = grandparent.downcast::<Paned>() else {
        return;
    };
    let parent_is_start_child = grandparent_paned
        .start_child()
        .map(|child| child.as_ptr() == parent_paned.clone().upcast::<gtk::Widget>().as_ptr())
        .unwrap_or(false);

    parent_paned.set_start_child(None::<&gtk::Widget>);
    parent_paned.set_end_child(None::<&gtk::Widget>);

    if parent_is_start_child {
        grandparent_paned.set_start_child(None::<&gtk::Widget>);
        grandparent_paned.set_start_child(Some(&sibling));
    } else {
        grandparent_paned.set_end_child(None::<&gtk::Widget>);
        grandparent_paned.set_end_child(Some(&sibling));
    }
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

fn restore_pane_position(
    paned: &Paned,
    state: &ShellState,
    persistence_id: &str,
    controller: Rc<PanePositionController>,
    pane_id: &str,
    default: i32,
) {
    controller.set_last_bucket(layout::pane_extent_bucket(paned_total(paned)).unwrap_or(0));
    let position = state
        .borrow_mut()
        .panes
        .preferred_position(pane_id, paned_total(paned))
        .unwrap_or(default);
    controller.run_programmatic_update(|| {
        paned.set_position(position);
    });
    install_preferred_pane_restore(
        paned,
        state.clone(),
        pane_id.to_string(),
        controller.clone(),
    );
    persist_pane_position(
        paned,
        state.clone(),
        persistence_id.to_string(),
        controller,
        pane_id.to_string(),
    );
}

fn persist_pane_position(
    paned: &Paned,
    state: ShellState,
    persistence_id: String,
    controller: Rc<PanePositionController>,
    pane_id: String,
) {
    let prev_total = Rc::new(std::cell::Cell::new(0i32));
    paned.connect_position_notify(move |paned| {
        let total = paned_total(paned);
        let old_total = prev_total.get();
        prev_total.set(total);
        if !controller.should_persist() {
            return;
        }
        // Only persist when the user drags the divider (total unchanged).
        // During window resizes the total changes and the preferred-restore
        // handler is responsible for applying the cached layout — persisting
        // here would overwrite it with the intermediate proportional position
        // and poison last_bucket.
        if total != old_total {
            return;
        }
        state
            .borrow_mut()
            .panes
            .remember_position(&pane_id, total, paned.position());
        controller.set_last_bucket(layout::pane_extent_bucket(total).unwrap_or(0));
        persist_state(&state, &persistence_id);
    });
}

fn install_preferred_pane_restore(
    paned: &Paned,
    state: ShellState,
    pane_id: String,
    controller: Rc<PanePositionController>,
) {
    // Listen on position_notify rather than notify::width/height because
    // GTK adjusts the paned position proportionally during size_allocate
    // (when both resize children are true).  notify::width/height fires
    // before that adjustment, so any position we set there gets
    // overridden.  position_notify fires after, so our override sticks.
    paned.connect_position_notify(move |paned| {
        if controller.is_programmatic_update() {
            return;
        }
        let extent = paned_total(paned);
        let bucket = layout::pane_extent_bucket(extent).unwrap_or(0);
        if bucket == controller.last_bucket() {
            return;
        }
        controller.set_last_bucket(bucket);

        let has_preferred = state
            .borrow()
            .panes
            .has_preferred_position(&pane_id, extent);
        if !has_preferred {
            return;
        }
        let Some(position) = state
            .borrow_mut()
            .panes
            .preferred_position(&pane_id, extent)
        else {
            return;
        };
        if paned.position() != position {
            controller.run_programmatic_update(|| {
                paned.set_position(position);
            });
        }
    });
}

fn persist_state(state: &ShellState, persistence_id: &str) {
    let snapshot = state.borrow().clone();
    layout::save(persistence_id, &snapshot);
}

fn paned_total(paned: &Paned) -> i32 {
    if paned.orientation() == Orientation::Horizontal {
        paned.width()
    } else {
        paned.height()
    }
}

/// Configure a Paned where the **start** child is the panel (e.g. left panel).
/// Position = panel size. On window grow, keep panel at previous size.
/// Skips the resize override when a resolution-aware preferred layout exists.
fn apply_start_panel_resize_policy(
    paned: &Paned,
    policy: PanelResizePolicy,
    controller: Rc<PanePositionController>,
    state: ShellState,
    pane_id: String,
    _min_size: i32,
) {
    paned.set_resize_start_child(true);
    paned.set_resize_end_child(true);
    match policy {
        PanelResizePolicy::Proportional => {}
        PanelResizePolicy::Fixed | PanelResizePolicy::CappedProportional { .. } => {
            let prev_total = Rc::new(std::cell::Cell::new(0i32));
            let prev_pos = Rc::new(std::cell::Cell::new(0i32));
            let controller = controller.clone();
            paned.connect_position_notify(move |paned| {
                if controller.is_programmatic_update() {
                    return;
                }
                let total = paned_total(paned);
                let pos = paned.position();
                let old_total = prev_total.get();
                let old_pos = prev_pos.get();

                if old_total > 0 && total > old_total {
                    let has_preferred = state
                        .borrow()
                        .panes
                        .has_preferred_position(&pane_id, total);
                    if has_preferred {
                        prev_total.set(total);
                        prev_pos.set(pos);
                        return;
                    }
                    // Window grew — restore panel to its previous size.
                    if pos != old_pos {
                        controller.run_programmatic_update(|| {
                            paned.set_position(old_pos);
                        });
                        prev_total.set(total);
                        return;
                    }
                }

                prev_total.set(total);
                prev_pos.set(pos);
            });
        }
    }
}

/// Configure a Paned where the **end** child is the panel (e.g. bottom or right panel).
/// Panel size = total - position. On window grow, keep panel at previous size.
/// Skips the resize override when a resolution-aware preferred layout exists.
fn apply_end_panel_resize_policy(
    paned: &Paned,
    policy: PanelResizePolicy,
    controller: Rc<PanePositionController>,
    state: ShellState,
    pane_id: String,
    _min_size: i32,
) {
    paned.set_resize_start_child(true);
    paned.set_resize_end_child(true);
    match policy {
        PanelResizePolicy::Proportional => {}
        PanelResizePolicy::Fixed | PanelResizePolicy::CappedProportional { .. } => {
            let prev_total = Rc::new(std::cell::Cell::new(0i32));
            let prev_panel_size = Rc::new(std::cell::Cell::new(0i32));
            let controller = controller.clone();
            paned.connect_position_notify(move |paned| {
                if controller.is_programmatic_update() {
                    return;
                }
                let total = paned_total(paned);
                let panel_size = total - paned.position();
                let old_total = prev_total.get();
                let old_panel_size = prev_panel_size.get();

                if old_total > 0 && total > old_total {
                    let has_preferred = state
                        .borrow()
                        .panes
                        .has_preferred_position(&pane_id, total);
                    if has_preferred {
                        prev_total.set(total);
                        prev_panel_size.set(panel_size);
                        return;
                    }
                    // Window grew — restore panel to its previous size.
                    if panel_size != old_panel_size {
                        controller.run_programmatic_update(|| {
                            paned.set_position(total - old_panel_size);
                        });
                        prev_total.set(total);
                        return;
                    }
                }

                prev_total.set(total);
                prev_panel_size.set(panel_size);
            });
        }
    }
}

fn install_pane_focus_tracking(panes: &[gtk::Widget]) {
    let all_panes = Rc::new(panes.to_vec());
    for pane in panes {
        let all = all_panes.clone();
        let this = pane.clone();
        let click = GestureClick::new();
        click.set_propagation_phase(gtk::PropagationPhase::Capture);
        click.connect_pressed(move |_, _, _, _| {
            for p in all.iter() {
                if p == &this {
                    p.add_css_class("pane-focused");
                } else {
                    p.remove_css_class("pane-focused");
                }
            }
        });
        pane.add_controller(click);
    }
}
