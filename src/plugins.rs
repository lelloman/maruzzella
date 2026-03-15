use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::path::{Path, PathBuf};
use std::str;

use glib::translate::FromGlibPtrFull;
use gtk::Widget;
use libloading::{Library, Symbol};
use maruzzella_api::{
    MzBytes, MzCommandSpec, MzHostApi, MzLogLevel, MzMenuItemSpec, MzPluginDependency,
    MzPluginDescriptorView, MzPluginVTable, MzStatus, MzStatusCode, MzStr,
    MzSurfaceContribution, MzViewFactorySpec, MZ_ABI_VERSION_V1,
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
    pub title: String,
    pub command_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisteredSurfaceContribution {
    pub plugin_id: String,
    pub surface_id: String,
    pub contribution_id: String,
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct RegisteredViewFactory {
    pub plugin_id: String,
    pub view_id: String,
    pub create: maruzzella_api::MzCreateViewFn,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginLogEntry {
    pub plugin_id: String,
    pub level: MzLogLevel,
    pub message: String,
}

#[derive(Debug)]
pub struct PluginRuntime {
    pub(crate) plugins: Vec<LoadedPlugin>,
    pub(crate) activation_order: Vec<String>,
    pub(crate) commands: Vec<RegisteredCommand>,
    pub(crate) menu_items: Vec<RegisteredMenuItem>,
    pub(crate) surface_contributions: Vec<RegisteredSurfaceContribution>,
    pub(crate) view_factories: Vec<RegisteredViewFactory>,
    pub(crate) logs: Vec<PluginLogEntry>,
}

impl PluginRuntime {
    pub fn activate(plugins: Vec<LoadedPlugin>) -> Result<Self, PluginRuntimeError> {
        let ordered = resolve_load_order(&plugins).map_err(PluginRuntimeError::Resolve)?;
        let activation_order = ordered
            .iter()
            .map(|plugin| plugin.descriptor.id.clone())
            .collect::<Vec<_>>();

        let mut host_state = HostState::default();
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
        }
        host_state.current_plugin_id = None;

        Ok(Self {
            plugins,
            activation_order,
            commands: host_state.commands,
            menu_items: host_state.menu_items,
            surface_contributions: host_state.surface_contributions,
            view_factories: host_state.view_factories,
            logs: host_state.logs,
        })
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

    pub fn dispatch_command(
        &self,
        command_id: &str,
        payload: &[u8],
    ) -> Result<(), MzStatusCode> {
        let Some(command) = self
            .commands
            .iter()
            .find(|command| command.command_id == command_id)
        else {
            return Err(MzStatusCode::NotFound);
        };
        let Some(invoke) = command.invoke else {
            return Err(MzStatusCode::NotFound);
        };
        let status = invoke(MzBytes {
            ptr: payload.as_ptr(),
            len: payload.len(),
        });
        if status.is_ok() {
            Ok(())
        } else {
            Err(status.code)
        }
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

    pub fn logs(&self) -> &[PluginLogEntry] {
        &self.logs
    }

    pub fn create_view(
        &self,
        view_id: &str,
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
        let host_api = MzHostApi {
            abi_version: MZ_ABI_VERSION_V1,
            host_context: std::ptr::null_mut(),
            log: None,
            register_command: None,
            register_menu_item: None,
            register_surface_contribution: None,
            register_view_factory: None,
            dispatch_command: Some(runtime_dispatch_command),
        };
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
            payload: MzBytes {
                ptr: payload.as_ptr(),
                len: payload.len(),
            },
        };

        let widget_ptr = (factory.create)(&host_api, &request);
        if widget_ptr.is_null() {
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
        unsafe { library.get(ENTRY_SYMBOL) }.map_err(|error| PluginLoadError::MissingEntryPoint {
            path: path.clone(),
            message: error.to_string(),
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

pub fn resolve_load_order<'a>(
    plugins: &'a [LoadedPlugin],
) -> Result<Vec<&'a LoadedPlugin>, PluginResolveError> {
    let mut by_id = HashMap::with_capacity(plugins.len());
    for plugin in plugins {
        if by_id
            .insert(plugin.descriptor.id.clone(), plugin)
            .is_some()
        {
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
            if !found_version.satisfies(
                dependency.min_version,
                dependency.max_version_exclusive,
            ) {
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

fn dependency_slice<'a>(
    ptr: *const MzPluginDependency,
    len: usize,
) -> &'a [MzPluginDependency] {
    if ptr.is_null() || len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(ptr, len) }
    }
}

#[derive(Default)]
struct HostState {
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
    logs: Vec<PluginLogEntry>,
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
            dispatch_command: Some(host_dispatch_command),
        }
    }

    fn plugin_id(&self) -> &str {
        self.current_plugin_id
            .as_deref()
            .unwrap_or("<unknown-plugin>")
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
        parent_id,
        title,
        command_id,
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
    state.surface_contributions.push(RegisteredSurfaceContribution {
        plugin_id,
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

    if !state.view_factory_ids.insert(view_id.clone()) {
        return MzStatus::new(MzStatusCode::AlreadyExists);
    }
    state.view_factories.push(RegisteredViewFactory {
        plugin_id,
        view_id,
        create: factory.create,
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

thread_local! {
    static ACTIVE_HOST_STATE: std::cell::Cell<*mut HostState> = const { std::cell::Cell::new(std::ptr::null_mut()) };
    static ACTIVE_RUNTIME: std::cell::Cell<*const PluginRuntime> = const { std::cell::Cell::new(std::ptr::null()) };
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static REGISTERED_PLUGIN_A: AtomicUsize = AtomicUsize::new(0);
    static STARTED_PLUGIN_A: AtomicUsize = AtomicUsize::new(0);
    static REGISTERED_PLUGIN_B: AtomicUsize = AtomicUsize::new(0);
    static INVOKED_PLUGIN_A: AtomicUsize = AtomicUsize::new(0);

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

    extern "C" fn plugin_a_register(host: *const maruzzella_api::MzHostApi) -> maruzzella_api::MzStatus {
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
            parent_id: MzStr::from_static("maruzzella.menu.file.items"),
            title: MzStr::from_static("Plugins"),
            command_id: MzStr::from_static("shell.plugins"),
        };
        let surface = maruzzella_api::MzSurfaceContribution {
            plugin_id: MzStr::from_static("maruzzella.base"),
            surface_id: MzStr::from_static("maruzzella.about.sections"),
            contribution_id: MzStr::from_static("base.about"),
            payload: maruzzella_api::MzBytes {
                ptr: br#"{"title":"Base"}"#.as_ptr(),
                len: br#"{"title":"Base"}"#.len(),
            },
        };

        host.register_command.expect("command registrar")(&command);
        host.register_menu_item.expect("menu registrar")(&menu);
        host.register_surface_contribution
            .expect("surface registrar")(&surface);
        maruzzella_api::MzStatus::OK
    }

    extern "C" fn plugin_a_startup(host: *const maruzzella_api::MzHostApi) -> maruzzella_api::MzStatus {
        STARTED_PLUGIN_A.fetch_add(1, Ordering::SeqCst);
        let Some(host) = (unsafe { host.as_ref() }) else {
            return maruzzella_api::MzStatus::new(MzStatusCode::InvalidArgument);
        };
        host.log.expect("logger")(
            MzLogLevel::Info,
            MzStr::from_static("base plugin started"),
        );
        maruzzella_api::MzStatus::OK
    }

    extern "C" fn plugin_b_register(host: *const maruzzella_api::MzHostApi) -> maruzzella_api::MzStatus {
        REGISTERED_PLUGIN_B.fetch_add(1, Ordering::SeqCst);
        let Some(host) = (unsafe { host.as_ref() }) else {
            return maruzzella_api::MzStatus::new(MzStatusCode::InvalidArgument);
        };
        let menu = maruzzella_api::MzMenuItemSpec {
            plugin_id: MzStr::from_static("com.example.notes"),
            menu_id: MzStr::from_static("notes"),
            parent_id: MzStr::from_static("maruzzella.menu.file.items"),
            title: MzStr::from_static("Notes"),
            command_id: MzStr::from_static("notes.open"),
        };
        host.register_menu_item.expect("menu registrar")(&menu);
        maruzzella_api::MzStatus::OK
    }

    extern "C" fn plugin_a_invoke(_: MzBytes) -> maruzzella_api::MzStatus {
        INVOKED_PLUGIN_A.fetch_add(1, Ordering::SeqCst);
        maruzzella_api::MzStatus::OK
    }

    fn plugin(
        id: &str,
        version: Version,
        dependencies: Vec<PluginDependencySpec>,
    ) -> LoadedPlugin {
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
        let base = plugin("maruzzella.base", Version { major: 1, minor: 0, patch: 0 }, vec![]);
        let notes = plugin(
            "com.example.notes",
            Version { major: 1, minor: 0, patch: 0 },
            vec![PluginDependencySpec {
                plugin_id: "maruzzella.base".to_string(),
                min_version: Version { major: 1, minor: 0, patch: 0 },
                max_version_exclusive: Version { major: 2, minor: 0, patch: 0 },
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
            Version { major: 1, minor: 0, patch: 0 },
            vec![PluginDependencySpec {
                plugin_id: "maruzzella.base".to_string(),
                min_version: Version { major: 1, minor: 0, patch: 0 },
                max_version_exclusive: Version { major: 2, minor: 0, patch: 0 },
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

        let base = LoadedPlugin {
            path: PathBuf::from("base.so"),
            descriptor: PluginDescriptor {
                id: "maruzzella.base".to_string(),
                name: "Base".to_string(),
                version: Version { major: 1, minor: 0, patch: 0 },
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
                version: Version { major: 1, minor: 0, patch: 0 },
                required_abi_version: MZ_ABI_VERSION_V1,
                description: String::new(),
                dependencies: vec![PluginDependencySpec {
                    plugin_id: "maruzzella.base".to_string(),
                    min_version: Version { major: 1, minor: 0, patch: 0 },
                    max_version_exclusive: Version { major: 2, minor: 0, patch: 0 },
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
            &["maruzzella.base".to_string(), "com.example.notes".to_string()]
        );
        assert_eq!(REGISTERED_PLUGIN_A.load(Ordering::SeqCst), 1);
        assert_eq!(STARTED_PLUGIN_A.load(Ordering::SeqCst), 1);
        assert_eq!(REGISTERED_PLUGIN_B.load(Ordering::SeqCst), 1);
        assert_eq!(runtime.commands().len(), 1);
        assert_eq!(runtime.menu_items().len(), 2);
        assert_eq!(runtime.surface_contributions().len(), 1);
        assert_eq!(runtime.logs().len(), 1);
        assert_eq!(runtime.logs()[0].message, "base plugin started");
        runtime
            .dispatch_command("shell.plugins", &[])
            .expect("plugin command should dispatch");
        assert_eq!(INVOKED_PLUGIN_A.load(Ordering::SeqCst), 1);
    }
}
