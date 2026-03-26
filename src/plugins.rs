use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::path::{Path, PathBuf};
use std::rc::{Rc, Weak};
use std::str;

use glib::translate::FromGlibPtrFull;
use gtk::Widget;
use libloading::{Library, Symbol};
use maruzzella_api::{
    MzAboutCatalog, MzAboutSection, MzBytes, MzCommandCatalog, MzCommandSpec, MzCommandSummary,
    MzConfigRecord, MzConfigState, MzConfigStateSummary, MzContributionSurface,
    MzDiagnosticCatalog, MzHostApi, MzHostEvent, MzLogLevel, MzMenuItemSpec, MzMenuSurface,
    MzOpenViewRequest, MzOpenViewResult, MzPluginDependency, MzPluginDependencySummary,
    MzPluginDescriptorView, MzPluginDiagnosticSummary, MzPluginLogSummary, MzPluginSnapshot,
    MzPluginSummary, MzPluginVTable, MzServiceCatalog, MzServiceQuery, MzServiceSpec,
    MzServiceSummary, MzSettingsCatalog, MzSettingsPage, MzSettingsPageSummary, MzStatus,
    MzStatusCode, MzStr, MzSurfaceContribution, MzViewCatalog, MzViewFactorySpec,
    MzViewOpenDisposition, MzViewPlacement, MzViewQuery, MzViewQueryResult, MzViewSummary,
    MZ_ABI_VERSION_V1,
};

use crate::layout;
use crate::plugin_tabs::{
    focus_plugin_view, is_plugin_view_open, open_or_focus_plugin_view, update_plugin_view_title,
    GroupHandles, OpenPluginViewOutcome, OpenPluginViewRequest as ShellOpenPluginViewRequest,
    ShellState,
};

const ENTRY_SYMBOL: &[u8] = b"maruzzella_plugin_entry\0";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginDependencySpec {
    pub plugin_id: String,
    pub min_version: Version,
    pub max_version_exclusive: Version,
    pub required: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginDescriptor {
    pub id: String,
    pub name: String,
    pub version: Version,
    pub required_abi_version: u32,
    pub description: String,
    pub dependencies: Vec<PluginDependencySpec>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl Version {
    pub fn satisfies(self, min: Self, max_exclusive: Self) -> bool {
        self >= min && self < max_exclusive
    }
}

impl From<maruzzella_api::MzVersion> for Version {
    fn from(value: maruzzella_api::MzVersion) -> Self {
        Self {
            major: value.major,
            minor: value.minor,
            patch: value.patch,
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Debug)]
pub struct LoadedPlugin {
    path: PathBuf,
    descriptor: PluginDescriptor,
    vtable: &'static MzPluginVTable,
    _library: Library,
}

impl LoadedPlugin {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn descriptor(&self) -> &PluginDescriptor {
        &self.descriptor
    }

    pub fn vtable(&self) -> &'static MzPluginVTable {
        self.vtable
    }

    pub(crate) fn from_static_vtable(
        path: impl Into<PathBuf>,
        descriptor: PluginDescriptor,
        vtable: &'static MzPluginVTable,
    ) -> Self {
        Self {
            path: path.into(),
            descriptor,
            vtable,
            _library: current_process_library(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RegisteredCommand {
    pub plugin_id: String,
    pub command_id: String,
    pub title: String,
    pub invoke: Option<maruzzella_api::MzCommandInvokeFn>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisteredMenuItem {
    pub plugin_id: String,
    pub menu_id: String,
    pub parent_id: String,
    pub parent_surface: Option<MzMenuSurface>,
    pub title: String,
    pub command_id: String,
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisteredSurfaceContribution {
    pub plugin_id: String,
    pub surface_id: String,
    pub surface: Option<MzContributionSurface>,
    pub contribution_id: String,
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct RegisteredViewFactory {
    pub plugin_id: String,
    pub view_id: String,
    pub title: String,
    pub placement: MzViewPlacement,
    pub create: maruzzella_api::MzCreateViewFn,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisteredService {
    pub plugin_id: String,
    pub service_id: String,
    pub version: String,
    pub summary: String,
    pub payload: Vec<u8>,
}

#[derive(Clone)]
pub struct RegisteredHostEventSubscriber {
    pub plugin_id: String,
    pub event_id: String,
    pub handler: maruzzella_api::MzHostEventHandlerFn,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginLogEntry {
    pub plugin_id: String,
    pub level: MzLogLevel,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PluginDiagnosticLevel {
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginDiagnostic {
    pub level: PluginDiagnosticLevel,
    pub plugin_id: Option<String>,
    pub path: Option<PathBuf>,
    pub message: String,
}

#[derive(Default)]
pub struct PluginHost {
    runtime: Option<Rc<PluginRuntime>>,
    diagnostics: Vec<PluginDiagnostic>,
}

impl PluginHost {
    pub fn new(runtime: Option<Rc<PluginRuntime>>, diagnostics: Vec<PluginDiagnostic>) -> Self {
        Self {
            runtime,
            diagnostics,
        }
    }

    pub fn runtime(&self) -> Option<&Rc<PluginRuntime>> {
        self.runtime.as_ref()
    }

    pub fn diagnostics(&self) -> &[PluginDiagnostic] {
        &self.diagnostics
    }
}

pub struct PluginRuntime {
    pub(crate) plugins: Vec<LoadedPlugin>,
    pub(crate) activation_order: Vec<String>,
    pub(crate) commands: Vec<RegisteredCommand>,
    pub(crate) menu_items: Vec<RegisteredMenuItem>,
    pub(crate) surface_contributions: Vec<RegisteredSurfaceContribution>,
    pub(crate) view_factories: Vec<RegisteredViewFactory>,
    pub(crate) services: Vec<RegisteredService>,
    pub(crate) host_event_subscribers: Vec<RegisteredHostEventSubscriber>,
    pub(crate) logs: Vec<PluginLogEntry>,
    pub(crate) diagnostics: RefCell<Vec<PluginDiagnostic>>,
    view_host: RefCell<Option<Rc<PluginShellHost>>>,
}

struct PluginShellHost {
    layout_persistence_id: String,
    config_persistence_id: String,
    shell_state: ShellState,
    group_handles: GroupHandles,
    runtime: Weak<PluginRuntime>,
    view_api: Box<MzHostApi>,
    command_snapshot_buffer: RefCell<Vec<u8>>,
    view_snapshot_buffer: RefCell<Vec<u8>>,
    plugin_snapshot_buffer: RefCell<Vec<u8>>,
    service_snapshot_buffer: RefCell<Vec<u8>>,
    service_payload_buffer: RefCell<Vec<u8>>,
    settings_snapshot_buffer: RefCell<Vec<u8>>,
    diagnostic_snapshot_buffer: RefCell<Vec<u8>>,
    about_snapshot_buffer: RefCell<Vec<u8>>,
}

impl PluginRuntime {
    pub fn activate(plugins: Vec<LoadedPlugin>) -> Result<Self, PluginRuntimeError> {
        Self::activate_with_persistence_id(plugins, "maruzzella")
    }

    pub fn activate_with_persistence_id(
        plugins: Vec<LoadedPlugin>,
        persistence_id: &str,
    ) -> Result<Self, PluginRuntimeError> {
        let ordered = resolve_load_order(&plugins).map_err(PluginRuntimeError::Resolve)?;
        let activation_order = ordered
            .iter()
            .map(|plugin| plugin.descriptor.id.clone())
            .collect::<Vec<_>>();

        let mut host_state = HostState {
            persistence_id: persistence_id.to_string(),
            plugin_configs: layout::load_plugin_configs(persistence_id),
            ..HostState::default()
        };
        for plugin in ordered {
            host_state.current_plugin_id = Some(plugin.descriptor.id.clone());
            let _scope = ActiveHostScope::enter(&mut host_state);
            let host_api = host_state.host_api();

            let register_status = (plugin.vtable.register)(&host_api);
            if !register_status.is_ok() {
                return Err(PluginRuntimeError::RegisterFailed {
                    plugin_id: plugin.descriptor.id.clone(),
                    status: register_status.code,
                });
            }

            let startup_status = (plugin.vtable.startup)(&host_api);
            if !startup_status.is_ok() {
                return Err(PluginRuntimeError::StartupFailed {
                    plugin_id: plugin.descriptor.id.clone(),
                    status: startup_status.code,
                });
            }

            host_state.emit_host_event(
                "maruzzella.plugin.started",
                Some(plugin.descriptor.id.as_str()),
                Some(plugin.descriptor.id.as_str()),
                &[],
            );
        }
        host_state.current_plugin_id = None;
        host_state.emit_host_event("maruzzella.runtime.ready", None, None, &[]);

        Ok(Self {
            plugins,
            activation_order,
            commands: host_state.commands,
            menu_items: host_state.menu_items,
            surface_contributions: host_state.surface_contributions,
            view_factories: host_state.view_factories,
            services: host_state.services,
            host_event_subscribers: host_state.host_event_subscribers,
            logs: host_state.logs,
            diagnostics: RefCell::new(Vec::new()),
            view_host: RefCell::new(None),
        })
    }

    #[cfg(test)]
    pub(crate) fn empty_for_tests() -> Self {
        Self {
            plugins: Vec::new(),
            activation_order: Vec::new(),
            commands: Vec::new(),
            menu_items: Vec::new(),
            surface_contributions: Vec::new(),
            view_factories: Vec::new(),
            services: Vec::new(),
            host_event_subscribers: Vec::new(),
            logs: Vec::new(),
            diagnostics: RefCell::new(Vec::new()),
            view_host: RefCell::new(None),
        }
    }

    pub fn attach_shell_host(
        self: &Rc<Self>,
        layout_persistence_id: String,
        persistence_id: String,
        shell_state: ShellState,
        group_handles: GroupHandles,
    ) {
        let shell_host = Rc::new_cyclic(|weak| PluginShellHost {
            layout_persistence_id,
            config_persistence_id: persistence_id,
            shell_state,
            group_handles,
            runtime: Rc::downgrade(self),
            view_api: Box::new(MzHostApi {
                abi_version: MZ_ABI_VERSION_V1,
                host_context: weak.as_ptr() as *mut _,
                log: None,
                register_command: None,
                register_menu_item: None,
                register_surface_contribution: None,
                register_view_factory: None,
                register_service: None,
                register_host_event_subscriber: None,
                dispatch_command: Some(runtime_dispatch_command),
                open_view: Some(host_open_view),
                focus_view: Some(host_focus_view),
                is_view_open: Some(host_is_view_open),
                update_view_title: Some(host_update_view_title),
                read_command_catalog: Some(host_read_command_catalog),
                read_view_catalog: Some(host_read_view_catalog),
                read_plugin_state: Some(host_read_plugin_state),
                read_service_catalog: Some(host_read_service_catalog),
                read_service: Some(host_read_service),
                read_settings_catalog: Some(host_read_settings_catalog),
                read_diagnostic_catalog: Some(host_read_diagnostic_catalog),
                read_about_catalog: Some(host_read_about_catalog),
                read_config: None,
                write_config: None,
                read_config_record: None,
                write_config_record: None,
            }),
            command_snapshot_buffer: RefCell::new(Vec::new()),
            view_snapshot_buffer: RefCell::new(Vec::new()),
            plugin_snapshot_buffer: RefCell::new(Vec::new()),
            service_snapshot_buffer: RefCell::new(Vec::new()),
            service_payload_buffer: RefCell::new(Vec::new()),
            settings_snapshot_buffer: RefCell::new(Vec::new()),
            diagnostic_snapshot_buffer: RefCell::new(Vec::new()),
            about_snapshot_buffer: RefCell::new(Vec::new()),
        });
        ACTIVE_SHELL_HOST.with(|cell| cell.set(Rc::as_ptr(&shell_host)));
        self.view_host.replace(Some(shell_host));
    }

    pub fn plugins(&self) -> &[LoadedPlugin] {
        &self.plugins
    }

    pub fn activation_order(&self) -> &[String] {
        &self.activation_order
    }

    pub fn commands(&self) -> &[RegisteredCommand] {
        &self.commands
    }

    pub fn dispatch_command(&self, command_id: &str, payload: &[u8]) -> Result<(), MzStatusCode> {
        let Some(command) = self
            .commands
            .iter()
            .find(|command| command.command_id == command_id)
        else {
            self.record_command_diagnostic(None, command_id, payload.len(), MzStatusCode::NotFound);
            return Err(MzStatusCode::NotFound);
        };
        let Some(invoke) = command.invoke else {
            self.record_command_diagnostic(
                Some(command.plugin_id.as_str()),
                command_id,
                payload.len(),
                MzStatusCode::NotFound,
            );
            return Err(MzStatusCode::NotFound);
        };
        let status = invoke(MzBytes {
            ptr: payload.as_ptr(),
            len: payload.len(),
        });
        if status.is_ok() {
            self.emit_host_event(
                "maruzzella.command.dispatched",
                Some(command.plugin_id.as_str()),
                Some(command_id),
                payload,
            );
            Ok(())
        } else {
            self.record_command_diagnostic(
                Some(command.plugin_id.as_str()),
                command_id,
                payload.len(),
                status.code,
            );
            Err(status.code)
        }
    }

    fn record_command_diagnostic(
        &self,
        plugin_id: Option<&str>,
        command_id: &str,
        payload_len: usize,
        status: MzStatusCode,
    ) {
        self.diagnostics.borrow_mut().push(PluginDiagnostic {
            level: PluginDiagnosticLevel::Error,
            plugin_id: plugin_id.map(str::to_string),
            path: None,
            message: format!(
                "command dispatch failed: {command_id} (payload: {payload_len} bytes, status: {status:?})"
            ),
        });
    }

    pub fn menu_items(&self) -> &[RegisteredMenuItem] {
        &self.menu_items
    }

    pub fn surface_contributions(&self) -> &[RegisteredSurfaceContribution] {
        &self.surface_contributions
    }

    pub fn view_factories(&self) -> &[RegisteredViewFactory] {
        &self.view_factories
    }

    pub fn services(&self) -> &[RegisteredService] {
        &self.services
    }

    pub fn logs(&self) -> &[PluginLogEntry] {
        &self.logs
    }

    pub(crate) fn push_diagnostic(
        &self,
        plugin_id: Option<String>,
        message: impl Into<String>,
    ) {
        self.diagnostics.borrow_mut().push(PluginDiagnostic {
            level: PluginDiagnosticLevel::Error,
            plugin_id,
            path: None,
            message: message.into(),
        });
    }

    pub(crate) fn push_diagnostic_once(
        &self,
        plugin_id: Option<String>,
        message: impl Into<String>,
    ) {
        let message = message.into();
        let exists = self.diagnostics.borrow().iter().any(|diagnostic| {
            diagnostic.plugin_id == plugin_id && diagnostic.message == message
        });
        if !exists {
            self.push_diagnostic(plugin_id, message);
        }
    }

    fn emit_host_event(
        &self,
        event_id: &str,
        plugin_id: Option<&str>,
        subject_id: Option<&str>,
        payload: &[u8],
    ) {
        let event = MzHostEvent {
            event_id: event_id.to_string(),
            plugin_id: plugin_id.map(str::to_string),
            subject_id: subject_id.map(str::to_string),
            payload: payload.to_vec(),
        };
        for subscriber in self
            .host_event_subscribers
            .iter()
            .filter(|subscriber| subscriber.event_id == event_id)
        {
            let Ok(bytes) = serde_json::to_vec(&event) else {
                self.push_diagnostic(
                    Some(subscriber.plugin_id.clone()),
                    format!("failed to serialize host event {event_id}"),
                );
                continue;
            };
            let status = (subscriber.handler)(MzBytes {
                ptr: bytes.as_ptr(),
                len: bytes.len(),
            });
            if !status.is_ok() {
                self.push_diagnostic(
                    Some(subscriber.plugin_id.clone()),
                    format!(
                        "host event handler failed: {} for {} ({:?})",
                        event_id, subscriber.plugin_id, status.code
                    ),
                );
            }
        }
    }

    pub fn create_view(
        &self,
        view_id: &str,
        instance_key: Option<&str>,
        payload: &[u8],
    ) -> Result<Widget, PluginViewCreateError> {
        let Some(factory) = self
            .view_factories
            .iter()
            .find(|factory| factory.view_id == view_id)
        else {
            return Err(PluginViewCreateError::NotFound {
                view_id: view_id.to_string(),
            });
        };

        let _scope = ActiveRuntimeScope::enter(self);
        let view_host = self.view_host.borrow().clone();
        let host_api = view_host.as_ref().map(|host| host.view_api.as_ref());
        let plugin_id = MzStr {
            ptr: factory.plugin_id.as_ptr(),
            len: factory.plugin_id.len(),
        };
        let view_id_ffi = MzStr {
            ptr: factory.view_id.as_ptr(),
            len: factory.view_id.len(),
        };
        let request = maruzzella_api::MzViewRequest {
            plugin_id,
            view_id: view_id_ffi,
            instance_key: encode_optional_str(instance_key),
            payload: MzBytes {
                ptr: payload.as_ptr(),
                len: payload.len(),
            },
        };

        let widget_ptr = (factory.create)(
            host_api.map_or(std::ptr::null(), |host| host as *const _),
            &request,
        );
        if widget_ptr.is_null() {
            self.push_diagnostic(
                Some(factory.plugin_id.clone()),
                format!("view factory returned null for {}", factory.view_id),
            );
            return Err(PluginViewCreateError::FactoryReturnedNull {
                plugin_id: factory.plugin_id.clone(),
                view_id: factory.view_id.clone(),
            });
        }

        let widget = unsafe { Widget::from_glib_full(widget_ptr as *mut gtk::ffi::GtkWidget) };
        Ok(widget)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum PluginLoadError {
    LibraryOpen {
        path: PathBuf,
        message: String,
    },
    MissingEntryPoint {
        path: PathBuf,
        message: String,
    },
    NullVTable {
        path: PathBuf,
    },
    AbiMismatch {
        path: PathBuf,
        plugin_abi_version: u32,
    },
    DescriptorAbiMismatch {
        path: PathBuf,
        plugin_id: String,
        required_abi_version: u32,
    },
    InvalidUtf8 {
        path: PathBuf,
        field: &'static str,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum PluginResolveError {
    DuplicatePluginId {
        plugin_id: String,
    },
    MissingRequiredDependency {
        plugin_id: String,
        dependency_id: String,
    },
    IncompatibleDependencyVersion {
        plugin_id: String,
        dependency_id: String,
        found_version: Version,
        min_version: Version,
        max_version_exclusive: Version,
    },
    DependencyCycle {
        plugin_ids: Vec<String>,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum PluginRuntimeError {
    Resolve(PluginResolveError),
    RegisterFailed {
        plugin_id: String,
        status: MzStatusCode,
    },
    StartupFailed {
        plugin_id: String,
        status: MzStatusCode,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum PluginViewCreateError {
    NotFound {
        view_id: String,
    },
    FactoryReturnedNull {
        plugin_id: String,
        view_id: String,
    },
    CommandDispatchFailed {
        command_id: String,
        status: MzStatusCode,
    },
}

type PluginEntryPoint = unsafe extern "C" fn() -> *const MzPluginVTable;

pub fn load_plugin(path: impl AsRef<Path>) -> Result<LoadedPlugin, PluginLoadError> {
    let path = path.as_ref().to_path_buf();
    let library = unsafe { Library::new(&path) }.map_err(|error| PluginLoadError::LibraryOpen {
        path: path.clone(),
        message: error.to_string(),
    })?;

    let entry: Symbol<'_, PluginEntryPoint> =
        unsafe { library.get(ENTRY_SYMBOL) }.map_err(|error| {
            PluginLoadError::MissingEntryPoint {
                path: path.clone(),
                message: error.to_string(),
            }
        })?;

    let vtable = unsafe { entry() };
    let Some(vtable) = (unsafe { vtable.as_ref() }) else {
        return Err(PluginLoadError::NullVTable { path });
    };

    if vtable.abi_version != MZ_ABI_VERSION_V1 {
        return Err(PluginLoadError::AbiMismatch {
            path,
            plugin_abi_version: vtable.abi_version,
        });
    }

    let descriptor_view = (vtable.descriptor)();
    let descriptor = descriptor_from_view(&path, descriptor_view)?;
    if descriptor.required_abi_version != MZ_ABI_VERSION_V1 {
        return Err(PluginLoadError::DescriptorAbiMismatch {
            path,
            plugin_id: descriptor.id,
            required_abi_version: descriptor.required_abi_version,
        });
    }

    Ok(LoadedPlugin {
        path,
        descriptor,
        vtable,
        _library: library,
    })
}

pub fn load_static_plugin(
    path: impl Into<PathBuf>,
    entry: PluginEntryPoint,
) -> Result<LoadedPlugin, PluginLoadError> {
    let path = path.into();
    let vtable = unsafe { entry() };
    let Some(vtable) = (unsafe { vtable.as_ref() }) else {
        return Err(PluginLoadError::NullVTable { path });
    };

    if vtable.abi_version != MZ_ABI_VERSION_V1 {
        return Err(PluginLoadError::AbiMismatch {
            path,
            plugin_abi_version: vtable.abi_version,
        });
    }

    let descriptor_view = (vtable.descriptor)();
    let descriptor = descriptor_from_view(&path, descriptor_view)?;
    if descriptor.required_abi_version != MZ_ABI_VERSION_V1 {
        return Err(PluginLoadError::DescriptorAbiMismatch {
            path,
            plugin_id: descriptor.id,
            required_abi_version: descriptor.required_abi_version,
        });
    }

    Ok(LoadedPlugin::from_static_vtable(path, descriptor, vtable))
}

pub fn resolve_load_order<'a>(
    plugins: &'a [LoadedPlugin],
) -> Result<Vec<&'a LoadedPlugin>, PluginResolveError> {
    let mut by_id = HashMap::with_capacity(plugins.len());
    for plugin in plugins {
        if by_id.insert(plugin.descriptor.id.clone(), plugin).is_some() {
            return Err(PluginResolveError::DuplicatePluginId {
                plugin_id: plugin.descriptor.id.clone(),
            });
        }
    }

    for plugin in plugins {
        for dependency in plugin
            .descriptor
            .dependencies
            .iter()
            .filter(|dependency| dependency.required)
        {
            let Some(target) = by_id.get(&dependency.plugin_id) else {
                return Err(PluginResolveError::MissingRequiredDependency {
                    plugin_id: plugin.descriptor.id.clone(),
                    dependency_id: dependency.plugin_id.clone(),
                });
            };
            let found_version = target.descriptor.version;
            if !found_version.satisfies(dependency.min_version, dependency.max_version_exclusive) {
                return Err(PluginResolveError::IncompatibleDependencyVersion {
                    plugin_id: plugin.descriptor.id.clone(),
                    dependency_id: dependency.plugin_id.clone(),
                    found_version,
                    min_version: dependency.min_version,
                    max_version_exclusive: dependency.max_version_exclusive,
                });
            }
        }
    }

    let mut indegree = HashMap::<String, usize>::with_capacity(plugins.len());
    let mut adjacency = HashMap::<String, Vec<String>>::with_capacity(plugins.len());
    for plugin in plugins {
        indegree.entry(plugin.descriptor.id.clone()).or_insert(0);
        adjacency.entry(plugin.descriptor.id.clone()).or_default();
    }

    for plugin in plugins {
        for dependency in plugin
            .descriptor
            .dependencies
            .iter()
            .filter(|dependency| dependency.required)
        {
            adjacency
                .entry(dependency.plugin_id.clone())
                .or_default()
                .push(plugin.descriptor.id.clone());
            *indegree.entry(plugin.descriptor.id.clone()).or_insert(0) += 1;
        }
    }

    let mut ready = indegree
        .iter()
        .filter_map(|(plugin_id, count)| (*count == 0).then_some(plugin_id.clone()))
        .collect::<Vec<_>>();
    ready.sort();
    let mut queue = VecDeque::from(ready);
    let mut ordered = Vec::with_capacity(plugins.len());

    while let Some(plugin_id) = queue.pop_front() {
        ordered.push(*by_id.get(&plugin_id).expect("plugin id disappeared"));
        if let Some(children) = adjacency.get(&plugin_id) {
            let mut newly_ready = Vec::new();
            for child in children {
                let count = indegree
                    .get_mut(child)
                    .expect("child plugin must exist in indegree");
                *count -= 1;
                if *count == 0 {
                    newly_ready.push(child.clone());
                }
            }
            newly_ready.sort();
            queue.extend(newly_ready);
        }
    }

    if ordered.len() != plugins.len() {
        let mut remaining = indegree
            .into_iter()
            .filter_map(|(plugin_id, count)| (count > 0).then_some(plugin_id))
            .collect::<Vec<_>>();
        remaining.sort();
        return Err(PluginResolveError::DependencyCycle {
            plugin_ids: remaining,
        });
    }

    Ok(ordered)
}

fn descriptor_from_view(
    path: &Path,
    descriptor: MzPluginDescriptorView,
) -> Result<PluginDescriptor, PluginLoadError> {
    let dependencies = dependency_slice(descriptor.dependencies_ptr, descriptor.dependencies_len)
        .iter()
        .map(|dependency| dependency_from_view(path, *dependency))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(PluginDescriptor {
        id: decode_str(path, "id", descriptor.id)?,
        name: decode_str(path, "name", descriptor.name)?,
        version: descriptor.version.into(),
        required_abi_version: descriptor.required_abi_version,
        description: decode_str(path, "description", descriptor.description)?,
        dependencies,
    })
}

fn dependency_from_view(
    path: &Path,
    dependency: MzPluginDependency,
) -> Result<PluginDependencySpec, PluginLoadError> {
    Ok(PluginDependencySpec {
        plugin_id: decode_str(path, "dependency.plugin_id", dependency.plugin_id)?,
        min_version: dependency.min_version.into(),
        max_version_exclusive: dependency.max_version_exclusive.into(),
        required: dependency.required,
    })
}

fn decode_str(path: &Path, field: &'static str, value: MzStr) -> Result<String, PluginLoadError> {
    if value.len == 0 {
        return Ok(String::new());
    }
    if value.ptr.is_null() {
        return Err(PluginLoadError::InvalidUtf8 {
            path: path.to_path_buf(),
            field,
        });
    }
    let bytes = unsafe { std::slice::from_raw_parts(value.ptr, value.len) };
    let value = str::from_utf8(bytes).map_err(|_| PluginLoadError::InvalidUtf8 {
        path: path.to_path_buf(),
        field,
    })?;
    Ok(value.to_string())
}

fn dependency_slice<'a>(ptr: *const MzPluginDependency, len: usize) -> &'a [MzPluginDependency] {
    if ptr.is_null() || len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(ptr, len) }
    }
}

fn bytes_to_slice<'a>(bytes: MzBytes) -> &'a [u8] {
    if bytes.ptr.is_null() || bytes.len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(bytes.ptr, bytes.len) }
    }
}

#[derive(Default)]
struct HostState {
    persistence_id: String,
    current_plugin_id: Option<String>,
    commands: Vec<RegisteredCommand>,
    command_ids: HashSet<String>,
    command_handlers: HashMap<String, maruzzella_api::MzCommandInvokeFn>,
    menu_items: Vec<RegisteredMenuItem>,
    menu_item_ids: HashSet<String>,
    surface_contributions: Vec<RegisteredSurfaceContribution>,
    surface_contribution_ids: HashSet<(String, String)>,
    view_factories: Vec<RegisteredViewFactory>,
    view_factory_ids: HashSet<String>,
    services: Vec<RegisteredService>,
    service_ids: HashSet<String>,
    host_event_subscribers: Vec<RegisteredHostEventSubscriber>,
    logs: Vec<PluginLogEntry>,
    plugin_configs: layout::PluginConfigs,
    read_config_buffer: Vec<u8>,
    read_config_record_buffer: Vec<u8>,
    read_service_catalog_buffer: Vec<u8>,
    read_service_buffer: Vec<u8>,
}

impl HostState {
    fn host_api(&mut self) -> MzHostApi {
        MzHostApi {
            abi_version: MZ_ABI_VERSION_V1,
            host_context: self as *mut Self as *mut _,
            log: Some(host_log),
            register_command: Some(host_register_command),
            register_menu_item: Some(host_register_menu_item),
            register_surface_contribution: Some(host_register_surface_contribution),
            register_view_factory: Some(host_register_view_factory),
            register_service: Some(host_register_service),
            register_host_event_subscriber: Some(host_register_host_event_subscriber),
            dispatch_command: Some(host_dispatch_command),
            open_view: None,
            focus_view: None,
            is_view_open: None,
            update_view_title: None,
            read_command_catalog: None,
            read_view_catalog: None,
            read_plugin_state: None,
            read_service_catalog: Some(host_read_service_catalog),
            read_service: Some(host_read_service),
            read_settings_catalog: None,
            read_diagnostic_catalog: None,
            read_about_catalog: None,
            read_config: Some(host_read_config),
            write_config: Some(host_write_config),
            read_config_record: Some(host_read_config_record),
            write_config_record: Some(host_write_config_record),
        }
    }

    fn plugin_id(&self) -> &str {
        self.current_plugin_id
            .as_deref()
            .unwrap_or("<unknown-plugin>")
    }

    fn emit_host_event(
        &mut self,
        event_id: &str,
        plugin_id: Option<&str>,
        subject_id: Option<&str>,
        payload: &[u8],
    ) {
        let event = MzHostEvent {
            event_id: event_id.to_string(),
            plugin_id: plugin_id.map(str::to_string),
            subject_id: subject_id.map(str::to_string),
            payload: payload.to_vec(),
        };
        let Ok(bytes) = serde_json::to_vec(&event) else {
            self.logs.push(PluginLogEntry {
                plugin_id: self.plugin_id().to_string(),
                level: MzLogLevel::Error,
                message: format!("failed to serialize host event: {event_id}"),
            });
            return;
        };
        for subscriber in self
            .host_event_subscribers
            .iter()
            .filter(|subscriber| subscriber.event_id == event_id)
        {
            let status = (subscriber.handler)(MzBytes {
                ptr: bytes.as_ptr(),
                len: bytes.len(),
            });
            if !status.is_ok() {
                self.logs.push(PluginLogEntry {
                    plugin_id: subscriber.plugin_id.clone(),
                    level: MzLogLevel::Error,
                    message: format!(
                        "host event handler failed: {event_id} ({:?})",
                        status.code
                    ),
                });
            }
        }
    }
}

extern "C" fn host_log(level: MzLogLevel, message: MzStr) {
    let Some(state) = current_host_state() else {
        return;
    };
    let Ok(message) = decode_runtime_str("log.message", message) else {
        return;
    };
    state.logs.push(PluginLogEntry {
        plugin_id: state.plugin_id().to_string(),
        level,
        message,
    });
}

extern "C" fn host_register_command(command: *const MzCommandSpec) -> MzStatus {
    let Some(state) = current_host_state() else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };
    let Some(command) = (unsafe { command.as_ref() }) else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };

    let Ok(plugin_id) = decode_runtime_str("command.plugin_id", command.plugin_id) else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };
    let Ok(command_id) = decode_runtime_str("command.command_id", command.command_id) else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };
    let Ok(title) = decode_runtime_str("command.title", command.title) else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };

    if !state.command_ids.insert(command_id.clone()) {
        return MzStatus::new(MzStatusCode::AlreadyExists);
    }
    let handler_command_id = command_id.clone();
    state.commands.push(RegisteredCommand {
        plugin_id,
        command_id,
        title,
        invoke: command.invoke,
    });
    if let Some(invoke) = command.invoke {
        state.command_handlers.insert(handler_command_id, invoke);
    }
    MzStatus::OK
}

extern "C" fn host_register_menu_item(item: *const MzMenuItemSpec) -> MzStatus {
    let Some(state) = current_host_state() else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };
    let Some(item) = (unsafe { item.as_ref() }) else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };

    let Ok(plugin_id) = decode_runtime_str("menu.plugin_id", item.plugin_id) else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };
    let Ok(menu_id) = decode_runtime_str("menu.menu_id", item.menu_id) else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };
    let Ok(parent_id) = decode_runtime_str("menu.parent_id", item.parent_id) else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };
    let Ok(title) = decode_runtime_str("menu.title", item.title) else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };
    let Ok(command_id) = decode_runtime_str("menu.command_id", item.command_id) else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };

    if !state.menu_item_ids.insert(menu_id.clone()) {
        return MzStatus::new(MzStatusCode::AlreadyExists);
    }
    state.menu_items.push(RegisteredMenuItem {
        plugin_id,
        menu_id,
        parent_surface: MzMenuSurface::parse(&parent_id),
        parent_id,
        title,
        command_id,
        payload: bytes_to_vec(item.payload),
    });
    MzStatus::OK
}

extern "C" fn host_register_surface_contribution(
    contribution: *const MzSurfaceContribution,
) -> MzStatus {
    let Some(state) = current_host_state() else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };
    let Some(contribution) = (unsafe { contribution.as_ref() }) else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };

    let Ok(plugin_id) = decode_runtime_str("surface.plugin_id", contribution.plugin_id) else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };
    let Ok(surface_id) = decode_runtime_str("surface.surface_id", contribution.surface_id) else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };
    let Ok(contribution_id) =
        decode_runtime_str("surface.contribution_id", contribution.contribution_id)
    else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };
    let payload = bytes_to_vec(contribution.payload);

    let key = (surface_id.clone(), contribution_id.clone());
    if !state.surface_contribution_ids.insert(key) {
        return MzStatus::new(MzStatusCode::AlreadyExists);
    }
    state
        .surface_contributions
        .push(RegisteredSurfaceContribution {
            plugin_id,
            surface: MzContributionSurface::parse(&surface_id),
            surface_id,
            contribution_id,
            payload,
        });
    MzStatus::OK
}

extern "C" fn host_register_view_factory(factory: *const MzViewFactorySpec) -> MzStatus {
    let Some(state) = current_host_state() else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };
    let Some(factory) = (unsafe { factory.as_ref() }) else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };

    let Ok(plugin_id) = decode_runtime_str("view.plugin_id", factory.plugin_id) else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };
    let Ok(view_id) = decode_runtime_str("view.view_id", factory.view_id) else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };
    let Ok(title) = decode_runtime_str("view.title", factory.title) else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };

    if !state.view_factory_ids.insert(view_id.clone()) {
        return MzStatus::new(MzStatusCode::AlreadyExists);
    }
    state.view_factories.push(RegisteredViewFactory {
        plugin_id,
        view_id,
        title,
        placement: factory.placement,
        create: factory.create,
    });
    MzStatus::OK
}

extern "C" fn host_register_service(service: *const MzServiceSpec) -> MzStatus {
    let Some(state) = current_host_state() else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };
    let Some(service) = (unsafe { service.as_ref() }) else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };

    let Ok(plugin_id) = decode_runtime_str("service.plugin_id", service.plugin_id) else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };
    let Ok(service_id) = decode_runtime_str("service.service_id", service.service_id) else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };
    let Ok(version) = decode_runtime_str("service.version", service.version) else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };
    let Ok(summary) = decode_runtime_str("service.summary", service.summary) else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };

    if !state.service_ids.insert(service_id.clone()) {
        return MzStatus::new(MzStatusCode::AlreadyExists);
    }
    state.services.push(RegisteredService {
        plugin_id,
        service_id,
        version,
        summary,
        payload: bytes_to_vec(service.payload),
    });
    MzStatus::OK
}

extern "C" fn host_register_host_event_subscriber(
    event_id: MzStr,
    handler: maruzzella_api::MzHostEventHandlerFn,
) -> MzStatus {
    let Some(state) = current_host_state() else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };
    let Ok(event_id) = decode_runtime_str("event.event_id", event_id) else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };
    state.host_event_subscribers.push(RegisteredHostEventSubscriber {
        plugin_id: state.plugin_id().to_string(),
        event_id,
        handler,
    });
    MzStatus::OK
}

extern "C" fn host_dispatch_command(command_id: MzStr, payload: MzBytes) -> MzStatus {
    let Some(state) = current_host_state() else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };
    let Ok(command_id) = decode_runtime_str("dispatch.command_id", command_id) else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };
    let Some(invoke) = state.command_handlers.get(&command_id).copied() else {
        return MzStatus::new(MzStatusCode::NotFound);
    };
    invoke(payload)
}

extern "C" fn host_read_config() -> MzBytes {
    let Some(state) = current_host_state() else {
        return MzBytes::empty();
    };
    let plugin_id = state.plugin_id().to_string();
    state.read_config_buffer = state
        .plugin_configs
        .entries
        .get(&plugin_id)
        .map(|entry| entry.payload.clone())
        .unwrap_or_default();
    MzBytes {
        ptr: state.read_config_buffer.as_ptr(),
        len: state.read_config_buffer.len(),
    }
}

extern "C" fn host_write_config(payload: MzBytes) -> MzStatus {
    let Some(state) = current_host_state() else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };
    let plugin_id = state.plugin_id().to_string();
    let bytes = bytes_to_vec(payload);
    state.plugin_configs.entries.insert(
        plugin_id.clone(),
        layout::PluginConfigEntry {
            schema_version: None,
            payload: bytes,
        },
    );
    state.plugin_configs.invalid_entries.remove(&plugin_id);
    layout::save_plugin_configs(&state.persistence_id, &state.plugin_configs);
    MzStatus::OK
}

extern "C" fn host_read_config_record() -> MzBytes {
    let Some(state) = current_host_state() else {
        return MzBytes::empty();
    };
    let plugin_id = state.plugin_id().to_string();
    let record = state
        .plugin_configs
        .entries
        .get(&plugin_id)
        .map(|entry| MzConfigRecord {
            schema_version: entry.schema_version,
            payload: entry.payload.clone(),
        })
        .unwrap_or_default();
    state.read_config_record_buffer = record.to_bytes().unwrap_or_default();
    MzBytes {
        ptr: state.read_config_record_buffer.as_ptr(),
        len: state.read_config_record_buffer.len(),
    }
}

extern "C" fn host_write_config_record(payload: MzBytes) -> MzStatus {
    let Some(state) = current_host_state() else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };
    let plugin_id = state.plugin_id().to_string();
    let Ok(record) = MzConfigRecord::from_bytes(bytes_to_slice(payload)) else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };
    state.plugin_configs.entries.insert(
        plugin_id.clone(),
        layout::PluginConfigEntry {
            schema_version: record.schema_version,
            payload: record.payload,
        },
    );
    state.plugin_configs.invalid_entries.remove(&plugin_id);
    layout::save_plugin_configs(&state.persistence_id, &state.plugin_configs);
    MzStatus::OK
}

extern "C" fn host_open_view(request: *const MzOpenViewRequest) -> MzOpenViewResult {
    let Some(shell_host) = current_shell_host() else {
        return MzOpenViewResult {
            status: MzStatus::new(MzStatusCode::NotFound),
            disposition: MzViewOpenDisposition::Opened,
        };
    };
    let Some(request) = (unsafe { request.as_ref() }) else {
        return MzOpenViewResult {
            status: MzStatus::new(MzStatusCode::InvalidArgument),
            disposition: MzViewOpenDisposition::Opened,
        };
    };

    let Ok(plugin_id) = decode_runtime_str("open_view.plugin_id", request.plugin_id) else {
        return invalid_open_view_result();
    };
    let Ok(view_id) = decode_runtime_str("open_view.view_id", request.view_id) else {
        return invalid_open_view_result();
    };
    let Ok(instance_key) = decode_runtime_str("open_view.instance_key", request.instance_key)
    else {
        return invalid_open_view_result();
    };
    let Ok(requested_title) =
        decode_runtime_str("open_view.requested_title", request.requested_title)
    else {
        return invalid_open_view_result();
    };
    let Some(runtime) = shell_host.runtime.upgrade() else {
        return MzOpenViewResult {
            status: MzStatus::new(MzStatusCode::NotFound),
            disposition: MzViewOpenDisposition::Opened,
        };
    };
    let result = open_or_focus_plugin_view(
        &runtime,
        &shell_host.layout_persistence_id,
        &shell_host.shell_state,
        &shell_host.group_handles,
        &ShellOpenPluginViewRequest {
            plugin_view_id: resolve_plugin_view_id(&plugin_id, &view_id),
            placement: request.placement,
            instance_key: empty_to_none(instance_key),
            payload: bytes_to_vec(request.payload),
            requested_title: empty_to_none(requested_title),
        },
    );

    match result {
        Some(OpenPluginViewOutcome::Opened) => {
            runtime.emit_host_event(
                "maruzzella.view.opened",
                Some(plugin_id.as_str()),
                Some(view_id.as_str()),
                bytes_to_slice(request.payload),
            );
            MzOpenViewResult {
                status: MzStatus::OK,
                disposition: MzViewOpenDisposition::Opened,
            }
        }
        Some(OpenPluginViewOutcome::FocusedExisting) => {
            runtime.emit_host_event(
                "maruzzella.view.focused",
                Some(plugin_id.as_str()),
                Some(view_id.as_str()),
                bytes_to_slice(request.payload),
            );
            MzOpenViewResult {
                status: MzStatus::OK,
                disposition: MzViewOpenDisposition::FocusedExisting,
            }
        }
        None => {
            runtime.push_diagnostic(
                Some(plugin_id.clone()),
                format!("open view failed: {}", view_id),
            );
            MzOpenViewResult {
                status: MzStatus::new(MzStatusCode::NotFound),
                disposition: MzViewOpenDisposition::Opened,
            }
        }
    }
}

extern "C" fn host_focus_view(query: *const MzViewQuery) -> MzStatus {
    let Some(shell_host) = current_shell_host() else {
        return MzStatus::new(MzStatusCode::NotFound);
    };
    let Some(query) = (unsafe { query.as_ref() }) else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };
    let Ok(plugin_id) = decode_runtime_str("focus_view.plugin_id", query.plugin_id) else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };
    let Ok(view_id) = decode_runtime_str("focus_view.view_id", query.view_id) else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };
    let Ok(instance_key) = decode_runtime_str("focus_view.instance_key", query.instance_key) else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };

    if focus_plugin_view(
        &shell_host.shell_state,
        &shell_host.group_handles,
        &resolve_plugin_view_id(&plugin_id, &view_id),
        empty_to_none(instance_key).as_deref(),
    ) {
        if let Some(runtime) = shell_host.runtime.upgrade() {
            runtime.emit_host_event(
                "maruzzella.view.focused",
                Some(plugin_id.as_str()),
                Some(view_id.as_str()),
                &[],
            );
        }
        MzStatus::OK
    } else {
        if let Some(runtime) = shell_host.runtime.upgrade() {
            runtime.push_diagnostic(
                Some(plugin_id.clone()),
                format!("focus view failed: {}", view_id),
            );
        }
        MzStatus::new(MzStatusCode::NotFound)
    }
}

extern "C" fn host_is_view_open(query: *const MzViewQuery) -> MzViewQueryResult {
    let Some(shell_host) = current_shell_host() else {
        return MzViewQueryResult {
            status: MzStatus::new(MzStatusCode::NotFound),
            found: false,
        };
    };
    let Some(query) = (unsafe { query.as_ref() }) else {
        return MzViewQueryResult {
            status: MzStatus::new(MzStatusCode::InvalidArgument),
            found: false,
        };
    };
    let Ok(plugin_id) = decode_runtime_str("is_view_open.plugin_id", query.plugin_id) else {
        return invalid_query_result();
    };
    let Ok(view_id) = decode_runtime_str("is_view_open.view_id", query.view_id) else {
        return invalid_query_result();
    };
    let Ok(instance_key) = decode_runtime_str("is_view_open.instance_key", query.instance_key)
    else {
        return invalid_query_result();
    };

    MzViewQueryResult {
        status: MzStatus::OK,
        found: is_plugin_view_open(
            &shell_host.shell_state,
            &resolve_plugin_view_id(&plugin_id, &view_id),
            empty_to_none(instance_key).as_deref(),
        ),
    }
}

extern "C" fn host_update_view_title(query: *const MzViewQuery, title: MzStr) -> MzStatus {
    let Some(shell_host) = current_shell_host() else {
        return MzStatus::new(MzStatusCode::NotFound);
    };
    let Some(query) = (unsafe { query.as_ref() }) else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };
    let Ok(plugin_id) = decode_runtime_str("update_view_title.plugin_id", query.plugin_id) else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };
    let Ok(view_id) = decode_runtime_str("update_view_title.view_id", query.view_id) else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };
    let Ok(instance_key) = decode_runtime_str("update_view_title.instance_key", query.instance_key)
    else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };
    let Ok(title) = decode_runtime_str("update_view_title.title", title) else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };

    if update_plugin_view_title(
        &shell_host.shell_state,
        &shell_host.group_handles,
        &shell_host.layout_persistence_id,
        &resolve_plugin_view_id(&plugin_id, &view_id),
        empty_to_none(instance_key).as_deref(),
        &title,
    ) {
        if let Some(runtime) = shell_host.runtime.upgrade() {
            runtime.emit_host_event(
                "maruzzella.view.title_updated",
                Some(plugin_id.as_str()),
                Some(view_id.as_str()),
                title.as_bytes(),
            );
        }
        MzStatus::OK
    } else {
        if let Some(runtime) = shell_host.runtime.upgrade() {
            runtime.push_diagnostic(
                Some(plugin_id.clone()),
                format!("update view title failed: {}", view_id),
            );
        }
        MzStatus::new(MzStatusCode::NotFound)
    }
}

extern "C" fn host_read_command_catalog() -> MzBytes {
    let Some(shell_host) = current_shell_host() else {
        return MzBytes::empty();
    };
    let commands = shell_host
        .shell_state
        .borrow()
        .spec
        .commands
        .iter()
        .map(|command| MzCommandSummary {
            command_id: command.id.clone(),
            title: command.title.clone(),
        })
        .collect::<Vec<_>>();
    snapshot_bytes(
        &shell_host.command_snapshot_buffer,
        &MzCommandCatalog { commands },
    )
}

extern "C" fn host_read_view_catalog() -> MzBytes {
    let Some(shell_host) = current_shell_host() else {
        return MzBytes::empty();
    };
    let Some(runtime) = shell_host.runtime.upgrade() else {
        return MzBytes::empty();
    };
    let views = runtime
        .view_factories()
        .iter()
        .map(|view| MzViewSummary {
            plugin_id: view.plugin_id.clone(),
            view_id: view.view_id.clone(),
            title: view.title.clone(),
            placement: view.placement,
        })
        .collect::<Vec<_>>();
    snapshot_bytes(&shell_host.view_snapshot_buffer, &MzViewCatalog { views })
}

extern "C" fn host_read_plugin_state() -> MzBytes {
    let Some(shell_host) = current_shell_host() else {
        return MzBytes::empty();
    };
    let Some(runtime) = shell_host.runtime.upgrade() else {
        return MzBytes::empty();
    };

    let plugins = runtime
        .plugins()
        .iter()
        .map(|plugin| {
            let descriptor = plugin.descriptor();
            let views = runtime
                .view_factories()
                .iter()
                .filter(|view| view.plugin_id == descriptor.id)
                .map(|view| MzViewSummary {
                    plugin_id: view.plugin_id.clone(),
                    view_id: view.view_id.clone(),
                    title: view.title.clone(),
                    placement: view.placement,
                })
                .collect::<Vec<_>>();
            let logs = runtime
                .logs()
                .iter()
                .filter(|entry| entry.plugin_id == descriptor.id)
                .map(|entry| MzPluginLogSummary {
                    level: entry.level,
                    message: entry.message.clone(),
                })
                .collect::<Vec<_>>();

            MzPluginSummary {
                plugin_id: descriptor.id.clone(),
                name: descriptor.name.clone(),
                version: descriptor.version.to_string(),
                description: descriptor.description.clone(),
                dependencies: descriptor
                    .dependencies
                    .iter()
                    .map(|dependency| MzPluginDependencySummary {
                        plugin_id: dependency.plugin_id.clone(),
                        min_version: dependency.min_version.to_string(),
                        max_version_exclusive: dependency.max_version_exclusive.to_string(),
                        required: dependency.required,
                    })
                    .collect(),
                views,
                logs,
            }
        })
        .collect::<Vec<_>>();

    let snapshot = MzPluginSnapshot {
        activation_order: runtime.activation_order().to_vec(),
        plugins,
    };
    snapshot_bytes(&shell_host.plugin_snapshot_buffer, &snapshot)
}

extern "C" fn host_read_service_catalog() -> MzBytes {
    if let Some(shell_host) = current_shell_host() {
        let Some(runtime) = shell_host.runtime.upgrade() else {
            return MzBytes::empty();
        };
        let services = runtime
            .services()
            .iter()
            .map(|service| MzServiceSummary {
                plugin_id: service.plugin_id.clone(),
                service_id: service.service_id.clone(),
                version: service.version.clone(),
                summary: service.summary.clone(),
                payload: service.payload.clone(),
            })
            .collect::<Vec<_>>();
        return snapshot_bytes(
            &shell_host.service_snapshot_buffer,
            &MzServiceCatalog { services },
        );
    }
    let Some(state) = current_host_state() else {
        return MzBytes::empty();
    };
    let catalog = MzServiceCatalog {
        services: state
            .services
            .iter()
            .map(|service| MzServiceSummary {
                plugin_id: service.plugin_id.clone(),
                service_id: service.service_id.clone(),
                version: service.version.clone(),
                summary: service.summary.clone(),
                payload: service.payload.clone(),
            })
            .collect(),
    };
    state.read_service_catalog_buffer = catalog.to_bytes().unwrap_or_default();
    MzBytes {
        ptr: state.read_service_catalog_buffer.as_ptr(),
        len: state.read_service_catalog_buffer.len(),
    }
}

extern "C" fn host_read_service(query: MzServiceQuery) -> MzBytes {
    let Ok(service_id) = decode_runtime_str("service_query.service_id", query.service_id) else {
        return MzBytes::empty();
    };
    if let Some(shell_host) = current_shell_host() {
        let Some(runtime) = shell_host.runtime.upgrade() else {
            return MzBytes::empty();
        };
        let Some(service) = runtime
            .services()
            .iter()
            .find(|service| service.service_id == service_id)
        else {
            return MzBytes::empty();
        };
        let mut buffer = shell_host.service_payload_buffer.borrow_mut();
        *buffer = service.payload.clone();
        return MzBytes {
            ptr: buffer.as_ptr(),
            len: buffer.len(),
        };
    }
    let Some(state) = current_host_state() else {
        return MzBytes::empty();
    };
    let Some(service) = state
        .services
        .iter()
        .find(|service| service.service_id == service_id)
    else {
        return MzBytes::empty();
    };
    state.read_service_buffer = service.payload.clone();
    MzBytes {
        ptr: state.read_service_buffer.as_ptr(),
        len: state.read_service_buffer.len(),
    }
}

extern "C" fn host_read_settings_catalog() -> MzBytes {
    let Some(shell_host) = current_shell_host() else {
        return MzBytes::empty();
    };
    let Some(runtime) = shell_host.runtime.upgrade() else {
        return MzBytes::empty();
    };
    let pages = runtime
        .surface_contributions()
        .iter()
        .filter(|contribution| {
            contribution.surface == Some(MzContributionSurface::PluginSettingsPages)
        })
        .filter_map(|contribution| {
            MzSettingsPage::from_bytes(&contribution.payload)
                .map_err(|_| {
                    runtime.push_diagnostic_once(
                        Some(contribution.plugin_id.clone()),
                        format!(
                            "invalid settings contribution payload: {}",
                            contribution.contribution_id
                        ),
                    )
                })
                .ok()
                .map(|page| MzSettingsPageSummary {
                    plugin_id: contribution.plugin_id.clone(),
                    contribution_id: contribution.contribution_id.clone(),
                    config_state: config_state_summary(
                        &shell_host.config_persistence_id,
                        &contribution.plugin_id,
                        page.config.as_ref(),
                    ),
                    page,
                })
        })
        .collect::<Vec<_>>();
    snapshot_bytes(
        &shell_host.settings_snapshot_buffer,
        &MzSettingsCatalog { pages },
    )
}

fn config_state_summary(
    persistence_id: &str,
    plugin_id: &str,
    contract: Option<&maruzzella_api::MzConfigContract>,
) -> Option<MzConfigStateSummary> {
    let contract = contract?;
    let configs = layout::load_plugin_configs(persistence_id);
    if let Some(message) = configs.invalid_entries.get(plugin_id) {
        return Some(MzConfigStateSummary {
            state: MzConfigState::Invalid,
            stored_schema_version: None,
            expected_schema_version: Some(contract.schema_version),
            migration_hook: contract.migration_hook.clone(),
            message: message.clone(),
        });
    }
    let Some(entry) = configs.entries.get(plugin_id) else {
        return Some(MzConfigStateSummary {
            state: MzConfigState::Missing,
            stored_schema_version: None,
            expected_schema_version: Some(contract.schema_version),
            migration_hook: contract.migration_hook.clone(),
            message: "No persisted plugin config found yet.".to_string(),
        });
    };
    match entry.schema_version {
        Some(version) if version == contract.schema_version => Some(MzConfigStateSummary {
            state: MzConfigState::Ready,
            stored_schema_version: Some(version),
            expected_schema_version: Some(contract.schema_version),
            migration_hook: contract.migration_hook.clone(),
            message: format!("Config schema v{} loaded.", version),
        }),
        Some(version) if version < contract.schema_version => Some(MzConfigStateSummary {
            state: MzConfigState::MigrationRequired,
            stored_schema_version: Some(version),
            expected_schema_version: Some(contract.schema_version),
            migration_hook: contract.migration_hook.clone(),
            message: format!(
                "Stored config schema v{} needs migration to v{}.",
                version, contract.schema_version
            ),
        }),
        Some(version) => Some(MzConfigStateSummary {
            state: MzConfigState::Invalid,
            stored_schema_version: Some(version),
            expected_schema_version: Some(contract.schema_version),
            migration_hook: contract.migration_hook.clone(),
            message: format!(
                "Stored config schema v{} is newer than the expected v{}.",
                version, contract.schema_version
            ),
        }),
        None => Some(MzConfigStateSummary {
            state: MzConfigState::MigrationRequired,
            stored_schema_version: None,
            expected_schema_version: Some(contract.schema_version),
            migration_hook: contract.migration_hook.clone(),
            message: format!(
                "Stored config predates schema-aware persistence; expected schema v{}.",
                contract.schema_version
            ),
        }),
    }
}

extern "C" fn host_read_diagnostic_catalog() -> MzBytes {
    let Some(shell_host) = current_shell_host() else {
        return MzBytes::empty();
    };
    let Some(runtime) = shell_host.runtime.upgrade() else {
        return MzBytes::empty();
    };
    let diagnostics = runtime
        .diagnostics
        .borrow()
        .iter()
        .map(|diagnostic| MzPluginDiagnosticSummary {
            level: format!("{:?}", diagnostic.level),
            plugin_id: diagnostic.plugin_id.clone(),
            path: diagnostic
                .path
                .as_ref()
                .map(|path| path.display().to_string()),
            message: diagnostic.message.clone(),
        })
        .collect::<Vec<_>>();
    snapshot_bytes(
        &shell_host.diagnostic_snapshot_buffer,
        &MzDiagnosticCatalog { diagnostics },
    )
}

extern "C" fn host_read_about_catalog() -> MzBytes {
    let Some(shell_host) = current_shell_host() else {
        return MzBytes::empty();
    };
    let Some(runtime) = shell_host.runtime.upgrade() else {
        return MzBytes::empty();
    };
    let sections = runtime
        .surface_contributions()
        .iter()
        .filter(|contribution| contribution.surface == Some(MzContributionSurface::AboutSections))
        .filter_map(|contribution| {
            MzAboutSection::from_bytes(&contribution.payload)
                .map_err(|_| {
                    runtime.push_diagnostic_once(
                        Some(contribution.plugin_id.clone()),
                        format!(
                            "invalid about contribution payload: {}",
                            contribution.contribution_id
                        ),
                    )
                })
                .ok()
        })
        .collect::<Vec<_>>();
    snapshot_bytes(
        &shell_host.about_snapshot_buffer,
        &MzAboutCatalog { sections },
    )
}

thread_local! {
    static ACTIVE_HOST_STATE: std::cell::Cell<*mut HostState> = const { std::cell::Cell::new(std::ptr::null_mut()) };
    static ACTIVE_RUNTIME: std::cell::Cell<*const PluginRuntime> = const { std::cell::Cell::new(std::ptr::null()) };
    static ACTIVE_SHELL_HOST: std::cell::Cell<*const PluginShellHost> = const { std::cell::Cell::new(std::ptr::null()) };
}

fn current_host_state() -> Option<&'static mut HostState> {
    ACTIVE_HOST_STATE.with(|cell| {
        let ptr = cell.get();
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { &mut *ptr })
        }
    })
}

fn current_runtime() -> Option<&'static PluginRuntime> {
    ACTIVE_RUNTIME.with(|cell| {
        let ptr = cell.get();
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { &*ptr })
        }
    })
}

fn current_shell_host() -> Option<&'static PluginShellHost> {
    ACTIVE_SHELL_HOST.with(|cell| {
        let ptr = cell.get();
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { &*ptr })
        }
    })
}

fn encode_optional_str(value: Option<&str>) -> MzStr {
    match value {
        Some(value) => MzStr {
            ptr: value.as_ptr(),
            len: value.len(),
        },
        None => MzStr::empty(),
    }
}

fn empty_to_none(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}

fn resolve_plugin_view_id(_plugin_id: &str, view_id: &str) -> String {
    view_id.to_string()
}

fn invalid_open_view_result() -> MzOpenViewResult {
    MzOpenViewResult {
        status: MzStatus::new(MzStatusCode::InvalidArgument),
        disposition: MzViewOpenDisposition::Opened,
    }
}

fn invalid_query_result() -> MzViewQueryResult {
    MzViewQueryResult {
        status: MzStatus::new(MzStatusCode::InvalidArgument),
        found: false,
    }
}

fn snapshot_bytes<T: serde::Serialize>(buffer: &RefCell<Vec<u8>>, value: &T) -> MzBytes {
    let mut buffer = buffer.borrow_mut();
    *buffer = serde_json::to_vec(value).unwrap_or_default();
    MzBytes {
        ptr: buffer.as_ptr(),
        len: buffer.len(),
    }
}

struct ActiveHostScope;
struct ActiveRuntimeScope;

impl ActiveHostScope {
    fn enter(state: &mut HostState) -> Self {
        ACTIVE_HOST_STATE.with(|cell| cell.set(state as *mut _));
        Self
    }
}

impl Drop for ActiveHostScope {
    fn drop(&mut self) {
        ACTIVE_HOST_STATE.with(|cell| cell.set(std::ptr::null_mut()));
    }
}

impl ActiveRuntimeScope {
    fn enter(runtime: &PluginRuntime) -> Self {
        ACTIVE_RUNTIME.with(|cell| cell.set(runtime as *const _));
        Self
    }
}

impl Drop for ActiveRuntimeScope {
    fn drop(&mut self) {
        ACTIVE_RUNTIME.with(|cell| cell.set(std::ptr::null()));
    }
}

extern "C" fn runtime_dispatch_command(command_id: MzStr, payload: MzBytes) -> MzStatus {
    let Some(runtime) = current_runtime() else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };
    let Ok(command_id) = decode_runtime_str("dispatch.command_id", command_id) else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };

    match runtime.dispatch_command(&command_id, unsafe {
        std::slice::from_raw_parts(payload.ptr, payload.len)
    }) {
        Ok(()) => MzStatus::OK,
        Err(status) => MzStatus::new(status),
    }
}

fn decode_runtime_str(field: &'static str, value: MzStr) -> Result<String, &'static str> {
    if value.len == 0 {
        return Ok(String::new());
    }
    if value.ptr.is_null() {
        return Err(field);
    }
    let bytes = unsafe { std::slice::from_raw_parts(value.ptr, value.len) };
    let value = std::str::from_utf8(bytes).map_err(|_| field)?;
    Ok(value.to_string())
}

fn bytes_to_vec(bytes: MzBytes) -> Vec<u8> {
    if bytes.ptr.is_null() || bytes.len == 0 {
        Vec::new()
    } else {
        unsafe { std::slice::from_raw_parts(bytes.ptr, bytes.len) }.to_vec()
    }
}

fn current_process_library() -> Library {
    #[cfg(unix)]
    {
        libloading::os::unix::Library::this().into()
    }
    #[cfg(windows)]
    {
        libloading::os::windows::Library::this()
            .expect("current process library should be loadable")
            .into()
    }
}

pub fn diagnostic_for_load_error(path: &Path, error: &PluginLoadError) -> PluginDiagnostic {
    PluginDiagnostic {
        level: PluginDiagnosticLevel::Error,
        plugin_id: load_error_plugin_id(error),
        path: Some(path.to_path_buf()),
        message: format!("load failed: {error:?}"),
    }
}

pub fn diagnostic_for_runtime_error(error: &PluginRuntimeError) -> PluginDiagnostic {
    PluginDiagnostic {
        level: PluginDiagnosticLevel::Error,
        plugin_id: runtime_error_plugin_id(error),
        path: None,
        message: format!("activation failed: {error:?}"),
    }
}

fn load_error_plugin_id(error: &PluginLoadError) -> Option<String> {
    match error {
        PluginLoadError::DescriptorAbiMismatch { plugin_id, .. } => Some(plugin_id.clone()),
        _ => None,
    }
}

fn runtime_error_plugin_id(error: &PluginRuntimeError) -> Option<String> {
    match error {
        PluginRuntimeError::Resolve(_) => None,
        PluginRuntimeError::RegisterFailed { plugin_id, .. } => Some(plugin_id.clone()),
        PluginRuntimeError::StartupFailed { plugin_id, .. } => Some(plugin_id.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static REGISTERED_PLUGIN_A: AtomicUsize = AtomicUsize::new(0);
    static STARTED_PLUGIN_A: AtomicUsize = AtomicUsize::new(0);
    static REGISTERED_PLUGIN_B: AtomicUsize = AtomicUsize::new(0);
    static INVOKED_PLUGIN_A: AtomicUsize = AtomicUsize::new(0);
    static OBSERVED_HOST_EVENTS: AtomicUsize = AtomicUsize::new(0);

    extern "C" fn test_descriptor() -> MzPluginDescriptorView {
        MzPluginDescriptorView::empty()
    }

    extern "C" fn test_register(_: *const maruzzella_api::MzHostApi) -> maruzzella_api::MzStatus {
        maruzzella_api::MzStatus::OK
    }

    extern "C" fn test_startup(_: *const maruzzella_api::MzHostApi) -> maruzzella_api::MzStatus {
        maruzzella_api::MzStatus::OK
    }

    extern "C" fn test_shutdown(_: *const maruzzella_api::MzHostApi) {}

    extern "C" fn plugin_a_register(
        host: *const maruzzella_api::MzHostApi,
    ) -> maruzzella_api::MzStatus {
        REGISTERED_PLUGIN_A.fetch_add(1, Ordering::SeqCst);
        let Some(host) = (unsafe { host.as_ref() }) else {
            return maruzzella_api::MzStatus::new(MzStatusCode::InvalidArgument);
        };
        let command = maruzzella_api::MzCommandSpec {
            plugin_id: MzStr::from_static("maruzzella.base"),
            command_id: MzStr::from_static("shell.plugins"),
            title: MzStr::from_static("Plugins"),
            invoke: Some(plugin_a_invoke),
        };
        let menu = maruzzella_api::MzMenuItemSpec {
            plugin_id: MzStr::from_static("maruzzella.base"),
            menu_id: MzStr::from_static("plugins"),
            parent_id: MzStr::from_static(maruzzella_api::MzMenuSurface::FileItems.as_str()),
            title: MzStr::from_static("Plugins"),
            command_id: MzStr::from_static("shell.plugins"),
            payload: MzBytes::empty(),
        };
        let surface = maruzzella_api::MzSurfaceContribution {
            plugin_id: MzStr::from_static("maruzzella.base"),
            surface_id: MzStr::from_static(
                maruzzella_api::MzContributionSurface::AboutSections.as_str(),
            ),
            contribution_id: MzStr::from_static("base.about"),
            payload: maruzzella_api::MzBytes {
                ptr: br#"{"title":"Base"}"#.as_ptr(),
                len: br#"{"title":"Base"}"#.len(),
            },
        };
        let service = maruzzella_api::MzServiceSpec {
            plugin_id: MzStr::from_static("maruzzella.base"),
            service_id: MzStr::from_static("maruzzella.base.runtime"),
            version: MzStr::from_static("1.0.0"),
            summary: MzStr::from_static("Base runtime service"),
            payload: MzBytes {
                ptr: br#"{"kind":"base"}"#.as_ptr(),
                len: br#"{"kind":"base"}"#.len(),
            },
        };

        host.register_command.expect("command registrar")(&command);
        host.register_menu_item.expect("menu registrar")(&menu);
        host.register_surface_contribution
            .expect("surface registrar")(&surface);
        host.register_service.expect("service registrar")(&service);
        host.register_host_event_subscriber
            .expect("event registrar")(MzStr::from_static("maruzzella.command.dispatched"), observe_host_event);
        maruzzella_api::MzStatus::OK
    }

    extern "C" fn plugin_a_startup(
        host: *const maruzzella_api::MzHostApi,
    ) -> maruzzella_api::MzStatus {
        STARTED_PLUGIN_A.fetch_add(1, Ordering::SeqCst);
        let Some(host) = (unsafe { host.as_ref() }) else {
            return maruzzella_api::MzStatus::new(MzStatusCode::InvalidArgument);
        };
        host.log.expect("logger")(MzLogLevel::Info, MzStr::from_static("base plugin started"));
        maruzzella_api::MzStatus::OK
    }

    extern "C" fn plugin_b_register(
        host: *const maruzzella_api::MzHostApi,
    ) -> maruzzella_api::MzStatus {
        REGISTERED_PLUGIN_B.fetch_add(1, Ordering::SeqCst);
        let Some(host) = (unsafe { host.as_ref() }) else {
            return maruzzella_api::MzStatus::new(MzStatusCode::InvalidArgument);
        };
        let menu = maruzzella_api::MzMenuItemSpec {
            plugin_id: MzStr::from_static("com.example.notes"),
            menu_id: MzStr::from_static("notes"),
            parent_id: MzStr::from_static(maruzzella_api::MzMenuSurface::FileItems.as_str()),
            title: MzStr::from_static("Notes"),
            command_id: MzStr::from_static("notes.open"),
            payload: MzBytes::empty(),
        };
        host.register_menu_item.expect("menu registrar")(&menu);
        maruzzella_api::MzStatus::OK
    }

    extern "C" fn plugin_a_invoke(_: MzBytes) -> maruzzella_api::MzStatus {
        INVOKED_PLUGIN_A.fetch_add(1, Ordering::SeqCst);
        maruzzella_api::MzStatus::OK
    }

    extern "C" fn observe_host_event(payload: MzBytes) -> maruzzella_api::MzStatus {
        let Ok(event) = serde_json::from_slice::<MzHostEvent>(bytes_to_slice(payload)) else {
            return maruzzella_api::MzStatus::new(MzStatusCode::InvalidArgument);
        };
        if event.event_id == "maruzzella.command.dispatched" {
            OBSERVED_HOST_EVENTS.fetch_add(1, Ordering::SeqCst);
        }
        maruzzella_api::MzStatus::OK
    }

    fn plugin(id: &str, version: Version, dependencies: Vec<PluginDependencySpec>) -> LoadedPlugin {
        LoadedPlugin {
            path: PathBuf::from(format!("{id}.so")),
            descriptor: PluginDescriptor {
                id: id.to_string(),
                name: id.to_string(),
                version,
                required_abi_version: MZ_ABI_VERSION_V1,
                description: String::new(),
                dependencies,
            },
            vtable: Box::leak(Box::new(MzPluginVTable {
                abi_version: MZ_ABI_VERSION_V1,
                descriptor: test_descriptor,
                register: test_register,
                startup: test_startup,
                shutdown: test_shutdown,
            })),
            _library: {
                #[cfg(unix)]
                {
                    libloading::os::unix::Library::this().into()
                }
                #[cfg(windows)]
                {
                    libloading::os::windows::Library::this()
                        .expect("current process library should be loadable")
                        .into()
                }
            },
        }
    }

    #[test]
    fn resolver_orders_dependencies_before_dependents() {
        let base = plugin(
            "maruzzella.base",
            Version {
                major: 1,
                minor: 0,
                patch: 0,
            },
            vec![],
        );
        let notes = plugin(
            "com.example.notes",
            Version {
                major: 1,
                minor: 0,
                patch: 0,
            },
            vec![PluginDependencySpec {
                plugin_id: "maruzzella.base".to_string(),
                min_version: Version {
                    major: 1,
                    minor: 0,
                    patch: 0,
                },
                max_version_exclusive: Version {
                    major: 2,
                    minor: 0,
                    patch: 0,
                },
                required: true,
            }],
        );

        let plugins = [notes, base];
        let ordered = resolve_load_order(&plugins).expect("dependencies should resolve");
        let ids = ordered
            .into_iter()
            .map(|plugin| plugin.descriptor.id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(ids, vec!["maruzzella.base", "com.example.notes"]);
    }

    #[test]
    fn resolver_rejects_missing_required_dependency() {
        let notes = plugin(
            "com.example.notes",
            Version {
                major: 1,
                minor: 0,
                patch: 0,
            },
            vec![PluginDependencySpec {
                plugin_id: "maruzzella.base".to_string(),
                min_version: Version {
                    major: 1,
                    minor: 0,
                    patch: 0,
                },
                max_version_exclusive: Version {
                    major: 2,
                    minor: 0,
                    patch: 0,
                },
                required: true,
            }],
        );

        let error = resolve_load_order(&[notes]).expect_err("dependency should be required");
        assert_eq!(
            error,
            PluginResolveError::MissingRequiredDependency {
                plugin_id: "com.example.notes".to_string(),
                dependency_id: "maruzzella.base".to_string(),
            }
        );
    }

    #[test]
    fn runtime_activates_plugins_and_collects_contributions() {
        REGISTERED_PLUGIN_A.store(0, Ordering::SeqCst);
        STARTED_PLUGIN_A.store(0, Ordering::SeqCst);
        REGISTERED_PLUGIN_B.store(0, Ordering::SeqCst);
        INVOKED_PLUGIN_A.store(0, Ordering::SeqCst);
        OBSERVED_HOST_EVENTS.store(0, Ordering::SeqCst);

        let base = LoadedPlugin {
            path: PathBuf::from("base.so"),
            descriptor: PluginDescriptor {
                id: "maruzzella.base".to_string(),
                name: "Base".to_string(),
                version: Version {
                    major: 1,
                    minor: 0,
                    patch: 0,
                },
                required_abi_version: MZ_ABI_VERSION_V1,
                description: String::new(),
                dependencies: vec![],
            },
            vtable: Box::leak(Box::new(MzPluginVTable {
                abi_version: MZ_ABI_VERSION_V1,
                descriptor: test_descriptor,
                register: plugin_a_register,
                startup: plugin_a_startup,
                shutdown: test_shutdown,
            })),
            _library: {
                #[cfg(unix)]
                {
                    libloading::os::unix::Library::this().into()
                }
                #[cfg(windows)]
                {
                    libloading::os::windows::Library::this()
                        .expect("current process library should be loadable")
                        .into()
                }
            },
        };
        let notes = LoadedPlugin {
            path: PathBuf::from("notes.so"),
            descriptor: PluginDescriptor {
                id: "com.example.notes".to_string(),
                name: "Notes".to_string(),
                version: Version {
                    major: 1,
                    minor: 0,
                    patch: 0,
                },
                required_abi_version: MZ_ABI_VERSION_V1,
                description: String::new(),
                dependencies: vec![PluginDependencySpec {
                    plugin_id: "maruzzella.base".to_string(),
                    min_version: Version {
                        major: 1,
                        minor: 0,
                        patch: 0,
                    },
                    max_version_exclusive: Version {
                        major: 2,
                        minor: 0,
                        patch: 0,
                    },
                    required: true,
                }],
            },
            vtable: Box::leak(Box::new(MzPluginVTable {
                abi_version: MZ_ABI_VERSION_V1,
                descriptor: test_descriptor,
                register: plugin_b_register,
                startup: test_startup,
                shutdown: test_shutdown,
            })),
            _library: {
                #[cfg(unix)]
                {
                    libloading::os::unix::Library::this().into()
                }
                #[cfg(windows)]
                {
                    libloading::os::windows::Library::this()
                        .expect("current process library should be loadable")
                        .into()
                }
            },
        };

        let runtime = PluginRuntime::activate(vec![notes, base]).expect("runtime should activate");
        assert_eq!(
            runtime.activation_order(),
            &[
                "maruzzella.base".to_string(),
                "com.example.notes".to_string()
            ]
        );
        assert_eq!(REGISTERED_PLUGIN_A.load(Ordering::SeqCst), 1);
        assert_eq!(STARTED_PLUGIN_A.load(Ordering::SeqCst), 1);
        assert_eq!(REGISTERED_PLUGIN_B.load(Ordering::SeqCst), 1);
        assert_eq!(runtime.commands().len(), 1);
        assert_eq!(runtime.menu_items().len(), 2);
        assert_eq!(runtime.surface_contributions().len(), 1);
        assert_eq!(runtime.services().len(), 1);
        assert_eq!(runtime.services()[0].service_id, "maruzzella.base.runtime");
        assert_eq!(runtime.logs().len(), 1);
        assert_eq!(runtime.logs()[0].message, "base plugin started");
        runtime
            .dispatch_command("shell.plugins", &[])
            .expect("plugin command should dispatch");
        assert_eq!(INVOKED_PLUGIN_A.load(Ordering::SeqCst), 1);
        assert_eq!(OBSERVED_HOST_EVENTS.load(Ordering::SeqCst), 1);
    }
}
