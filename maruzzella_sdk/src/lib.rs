pub use maruzzella_api as ffi;
use maruzzella_api::{
    MzBytes, MzCommandSpec, MzHostApi, MzMenuItemSpec, MzOpenViewRequest, MzPluginDependency,
    MzPluginDescriptorView, MzPluginVTable, MzStatus, MzStr, MzSurfaceContribution, MzVersion,
    MzViewFactorySpec, MzViewQuery, MzViewRequest, MZ_ABI_VERSION_V1,
};
pub use maruzzella_api::{
    MzAboutCatalog, MzCommandCatalog, MzCommandSummary, MzContributionSurface,
    MzDiagnosticCatalog, MzLogLevel, MzMenuSurface, MzPluginSnapshot, MzSettingsCatalog,
    MzSettingsCategory, MzStartupTab, MzStatusCode, MzToolbarItem, MzViewCatalog,
    MzViewOpenDisposition, MzViewPlacement, MzViewSummary,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Version(pub MzVersion);

impl Version {
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self(MzVersion::new(major, minor, patch))
    }

    pub const fn into_ffi(self) -> MzVersion {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginDependency {
    pub plugin_id: &'static str,
    pub min_version: Version,
    pub max_version_exclusive: Version,
    pub required: bool,
}

impl PluginDependency {
    pub const fn required(
        plugin_id: &'static str,
        min_version: Version,
        max_version_exclusive: Version,
    ) -> Self {
        Self {
            plugin_id,
            min_version,
            max_version_exclusive,
            required: true,
        }
    }

    pub const fn optional(
        plugin_id: &'static str,
        min_version: Version,
        max_version_exclusive: Version,
    ) -> Self {
        Self {
            plugin_id,
            min_version,
            max_version_exclusive,
            required: false,
        }
    }

    fn into_ffi(self) -> MzPluginDependency {
        MzPluginDependency {
            plugin_id: MzStr::from_static(self.plugin_id),
            min_version: self.min_version.into_ffi(),
            max_version_exclusive: self.max_version_exclusive.into_ffi(),
            required: self.required,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginDescriptor {
    pub id: &'static str,
    pub name: &'static str,
    pub version: Version,
    pub description: &'static str,
    pub dependencies: &'static [PluginDependency],
    pub required_abi_version: u32,
}

impl PluginDescriptor {
    pub const fn new(id: &'static str, name: &'static str, version: Version) -> Self {
        Self {
            id,
            name,
            version,
            description: "",
            dependencies: &[],
            required_abi_version: MZ_ABI_VERSION_V1,
        }
    }

    pub const fn with_description(mut self, description: &'static str) -> Self {
        self.description = description;
        self
    }

    pub const fn with_dependencies(mut self, dependencies: &'static [PluginDependency]) -> Self {
        self.dependencies = dependencies;
        self
    }

    pub const fn with_required_abi_version(mut self, required_abi_version: u32) -> Self {
        self.required_abi_version = required_abi_version;
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CommandSpec {
    pub plugin_id: &'static str,
    pub command_id: &'static str,
    pub title: &'static str,
    pub invoke: Option<maruzzella_api::MzCommandInvokeFn>,
}

impl CommandSpec {
    pub const fn new(
        plugin_id: &'static str,
        command_id: &'static str,
        title: &'static str,
    ) -> Self {
        Self {
            plugin_id,
            command_id,
            title,
            invoke: None,
        }
    }

    pub const fn with_handler(mut self, invoke: maruzzella_api::MzCommandInvokeFn) -> Self {
        self.invoke = Some(invoke);
        self
    }

    fn into_ffi(self) -> MzCommandSpec {
        MzCommandSpec {
            plugin_id: MzStr::from_static(self.plugin_id),
            command_id: MzStr::from_static(self.command_id),
            title: MzStr::from_static(self.title),
            invoke: self.invoke,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MenuItemSpec {
    pub plugin_id: &'static str,
    pub menu_id: &'static str,
    pub parent: MzMenuSurface,
    pub title: &'static str,
    pub command_id: &'static str,
}

impl MenuItemSpec {
    pub const fn new(
        plugin_id: &'static str,
        menu_id: &'static str,
        parent: MzMenuSurface,
        title: &'static str,
        command_id: &'static str,
    ) -> Self {
        Self {
            plugin_id,
            menu_id,
            parent,
            title,
            command_id,
        }
    }

    fn into_ffi(self) -> MzMenuItemSpec {
        MzMenuItemSpec {
            plugin_id: MzStr::from_static(self.plugin_id),
            menu_id: MzStr::from_static(self.menu_id),
            parent_id: MzStr::from_static(self.parent.as_str()),
            title: MzStr::from_static(self.title),
            command_id: MzStr::from_static(self.command_id),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SurfaceContributionSpec {
    pub plugin_id: &'static str,
    pub surface: MzContributionSurface,
    pub contribution_id: &'static str,
    pub payload: Vec<u8>,
}

#[derive(Clone, Copy, Debug)]
pub struct ViewFactorySpec {
    pub plugin_id: &'static str,
    pub view_id: &'static str,
    pub title: &'static str,
    pub placement: MzViewPlacement,
    pub create: maruzzella_api::MzCreateViewFn,
}

impl ViewFactorySpec {
    pub const fn new(
        plugin_id: &'static str,
        view_id: &'static str,
        title: &'static str,
        placement: MzViewPlacement,
        create: maruzzella_api::MzCreateViewFn,
    ) -> Self {
        Self {
            plugin_id,
            view_id,
            title,
            placement,
            create,
        }
    }

    fn into_ffi(self) -> MzViewFactorySpec {
        MzViewFactorySpec {
            plugin_id: MzStr::from_static(self.plugin_id),
            view_id: MzStr::from_static(self.view_id),
            title: MzStr::from_static(self.title),
            placement: self.placement,
            create: self.create,
        }
    }
}

impl SurfaceContributionSpec {
    pub fn new(
        plugin_id: &'static str,
        surface: MzContributionSurface,
        contribution_id: &'static str,
        payload: impl Into<Vec<u8>>,
    ) -> Self {
        Self {
            plugin_id,
            surface,
            contribution_id,
            payload: payload.into(),
        }
    }

    pub fn about_section(
        plugin_id: &'static str,
        contribution_id: &'static str,
        title: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        let payload = maruzzella_api::MzAboutSection::new(title, body)
            .to_bytes()
            .expect("about sections should serialize");
        Self::new(
            plugin_id,
            MzContributionSurface::AboutSections,
            contribution_id,
            payload,
        )
    }

    pub fn settings_page(
        plugin_id: &'static str,
        contribution_id: &'static str,
        page_id: impl Into<String>,
        title: impl Into<String>,
        summary: impl Into<String>,
        category: MzSettingsCategory,
    ) -> Self {
        let payload = maruzzella_api::MzSettingsPage::new(page_id, title, summary, category)
            .to_bytes()
            .expect("settings pages should serialize");
        Self::new(
            plugin_id,
            MzContributionSurface::PluginSettingsPages,
            contribution_id,
            payload,
        )
    }

    pub fn toolbar_item(
        plugin_id: &'static str,
        contribution_id: &'static str,
        item: MzToolbarItem,
    ) -> Self {
        let payload = item.to_bytes().expect("toolbar items should serialize");
        Self::new(
            plugin_id,
            MzContributionSurface::ToolbarItems,
            contribution_id,
            payload,
        )
    }

    pub fn startup_tab(
        plugin_id: &'static str,
        contribution_id: &'static str,
        tab: MzStartupTab,
    ) -> Self {
        let payload = tab.to_bytes().expect("startup tabs should serialize");
        Self::new(
            plugin_id,
            MzContributionSurface::StartupTabs,
            contribution_id,
            payload,
        )
    }

    fn as_ffi(&self) -> MzSurfaceContribution {
        MzSurfaceContribution {
            plugin_id: MzStr::from_static(self.plugin_id),
            surface_id: MzStr::from_static(self.surface.as_str()),
            contribution_id: MzStr::from_static(self.contribution_id),
            payload: MzBytes {
                ptr: self.payload.as_ptr(),
                len: self.payload.len(),
            },
        }
    }
}

pub struct HostApi<'a> {
    raw: &'a MzHostApi,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenViewRequest<'a> {
    pub plugin_id: &'a str,
    pub view_id: &'a str,
    pub placement: MzViewPlacement,
    pub instance_key: Option<&'a str>,
    pub requested_title: Option<&'a str>,
    pub payload: &'a [u8],
}

impl<'a> OpenViewRequest<'a> {
    pub fn new(plugin_id: &'a str, view_id: &'a str, placement: MzViewPlacement) -> Self {
        Self {
            plugin_id,
            view_id,
            placement,
            instance_key: None,
            requested_title: None,
            payload: &[],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ViewQuery<'a> {
    pub plugin_id: &'a str,
    pub view_id: &'a str,
    pub instance_key: Option<&'a str>,
}

impl<'a> ViewQuery<'a> {
    pub fn new(plugin_id: &'a str, view_id: &'a str) -> Self {
        Self {
            plugin_id,
            view_id,
            instance_key: None,
        }
    }
}

impl<'a> HostApi<'a> {
    pub fn from_raw(raw: &'a MzHostApi) -> Self {
        Self { raw }
    }

    pub fn abi_version(&self) -> u32 {
        self.raw.abi_version
    }

    pub fn log(&self, level: MzLogLevel, message: &'static str) {
        if let Some(log) = self.raw.log {
            log(level, MzStr::from_static(message));
        }
    }

    pub fn register_command(&self, command: CommandSpec) -> Result<(), MzStatusCode> {
        let Some(register) = self.raw.register_command else {
            return Err(MzStatusCode::NotFound);
        };
        let status = register(&command.into_ffi());
        if status.is_ok() {
            Ok(())
        } else {
            Err(status.code)
        }
    }

    pub fn register_menu_item(&self, item: MenuItemSpec) -> Result<(), MzStatusCode> {
        let Some(register) = self.raw.register_menu_item else {
            return Err(MzStatusCode::NotFound);
        };
        let status = register(&item.into_ffi());
        if status.is_ok() {
            Ok(())
        } else {
            Err(status.code)
        }
    }

    pub fn register_surface_contribution(
        &self,
        contribution: SurfaceContributionSpec,
    ) -> Result<(), MzStatusCode> {
        let Some(register) = self.raw.register_surface_contribution else {
            return Err(MzStatusCode::NotFound);
        };
        let ffi = contribution.as_ffi();
        let status = register(&ffi);
        if status.is_ok() {
            Ok(())
        } else {
            Err(status.code)
        }
    }

    pub fn register_view_factory(&self, factory: ViewFactorySpec) -> Result<(), MzStatusCode> {
        let Some(register) = self.raw.register_view_factory else {
            return Err(MzStatusCode::NotFound);
        };
        let status = register(&factory.into_ffi());
        if status.is_ok() {
            Ok(())
        } else {
            Err(status.code)
        }
    }

    pub fn dispatch_command(
        &self,
        command_id: &'static str,
        payload: &'static [u8],
    ) -> Result<(), MzStatusCode> {
        let Some(dispatch) = self.raw.dispatch_command else {
            return Err(MzStatusCode::NotFound);
        };
        let status = dispatch(
            MzStr::from_static(command_id),
            MzBytes {
                ptr: payload.as_ptr(),
                len: payload.len(),
            },
        );
        if status.is_ok() {
            Ok(())
        } else {
            Err(status.code)
        }
    }

    pub fn open_view(
        &self,
        request: &OpenViewRequest<'_>,
    ) -> Result<MzViewOpenDisposition, MzStatusCode> {
        let Some(open) = self.raw.open_view else {
            return Err(MzStatusCode::NotFound);
        };
        let instance_key = request.instance_key.unwrap_or("");
        let requested_title = request.requested_title.unwrap_or("");
        let ffi = MzOpenViewRequest {
            plugin_id: MzStr {
                ptr: request.plugin_id.as_ptr(),
                len: request.plugin_id.len(),
            },
            view_id: MzStr {
                ptr: request.view_id.as_ptr(),
                len: request.view_id.len(),
            },
            placement: request.placement,
            instance_key: MzStr {
                ptr: instance_key.as_ptr(),
                len: instance_key.len(),
            },
            requested_title: MzStr {
                ptr: requested_title.as_ptr(),
                len: requested_title.len(),
            },
            payload: MzBytes {
                ptr: request.payload.as_ptr(),
                len: request.payload.len(),
            },
        };
        let result = open(&ffi);
        if result.status.is_ok() {
            Ok(result.disposition)
        } else {
            Err(result.status.code)
        }
    }

    pub fn focus_view(&self, query: &ViewQuery<'_>) -> Result<(), MzStatusCode> {
        let Some(focus) = self.raw.focus_view else {
            return Err(MzStatusCode::NotFound);
        };
        let instance_key = query.instance_key.unwrap_or("");
        let status = focus(&MzViewQuery {
            plugin_id: MzStr {
                ptr: query.plugin_id.as_ptr(),
                len: query.plugin_id.len(),
            },
            view_id: MzStr {
                ptr: query.view_id.as_ptr(),
                len: query.view_id.len(),
            },
            instance_key: MzStr {
                ptr: instance_key.as_ptr(),
                len: instance_key.len(),
            },
        });
        if status.is_ok() {
            Ok(())
        } else {
            Err(status.code)
        }
    }

    pub fn is_view_open(&self, query: &ViewQuery<'_>) -> Result<bool, MzStatusCode> {
        let Some(is_open) = self.raw.is_view_open else {
            return Err(MzStatusCode::NotFound);
        };
        let instance_key = query.instance_key.unwrap_or("");
        let result = is_open(&MzViewQuery {
            plugin_id: MzStr {
                ptr: query.plugin_id.as_ptr(),
                len: query.plugin_id.len(),
            },
            view_id: MzStr {
                ptr: query.view_id.as_ptr(),
                len: query.view_id.len(),
            },
            instance_key: MzStr {
                ptr: instance_key.as_ptr(),
                len: instance_key.len(),
            },
        });
        if result.status.is_ok() {
            Ok(result.found)
        } else {
            Err(result.status.code)
        }
    }

    pub fn update_view_title(
        &self,
        query: &ViewQuery<'_>,
        title: &str,
    ) -> Result<(), MzStatusCode> {
        let Some(update) = self.raw.update_view_title else {
            return Err(MzStatusCode::NotFound);
        };
        let instance_key = query.instance_key.unwrap_or("");
        let status = update(
            &MzViewQuery {
                plugin_id: MzStr {
                    ptr: query.plugin_id.as_ptr(),
                    len: query.plugin_id.len(),
                },
                view_id: MzStr {
                    ptr: query.view_id.as_ptr(),
                    len: query.view_id.len(),
                },
                instance_key: MzStr {
                    ptr: instance_key.as_ptr(),
                    len: instance_key.len(),
                },
            },
            MzStr {
                ptr: title.as_ptr(),
                len: title.len(),
            },
        );
        if status.is_ok() {
            Ok(())
        } else {
            Err(status.code)
        }
    }

    pub fn read_command_catalog(&self) -> Result<MzCommandCatalog, MzStatusCode> {
        let Some(read) = self.raw.read_command_catalog else {
            return Err(MzStatusCode::NotFound);
        };
        let bytes = read();
        if bytes.ptr.is_null() || bytes.len == 0 {
            return Ok(MzCommandCatalog::default());
        }
        MzCommandCatalog::from_bytes(unsafe { std::slice::from_raw_parts(bytes.ptr, bytes.len) })
            .map_err(|_| MzStatusCode::InternalError)
    }

    pub fn read_view_catalog(&self) -> Result<MzViewCatalog, MzStatusCode> {
        let Some(read) = self.raw.read_view_catalog else {
            return Err(MzStatusCode::NotFound);
        };
        let bytes = read();
        if bytes.ptr.is_null() || bytes.len == 0 {
            return Ok(MzViewCatalog::default());
        }
        MzViewCatalog::from_bytes(unsafe { std::slice::from_raw_parts(bytes.ptr, bytes.len) })
            .map_err(|_| MzStatusCode::InternalError)
    }

    pub fn read_plugin_state(&self) -> Result<MzPluginSnapshot, MzStatusCode> {
        let Some(read) = self.raw.read_plugin_state else {
            return Err(MzStatusCode::NotFound);
        };
        let bytes = read();
        if bytes.ptr.is_null() || bytes.len == 0 {
            return Err(MzStatusCode::NotFound);
        }
        MzPluginSnapshot::from_bytes(unsafe { std::slice::from_raw_parts(bytes.ptr, bytes.len) })
            .map_err(|_| MzStatusCode::InternalError)
    }

    pub fn read_settings_catalog(&self) -> Result<MzSettingsCatalog, MzStatusCode> {
        let Some(read) = self.raw.read_settings_catalog else {
            return Err(MzStatusCode::NotFound);
        };
        let bytes = read();
        if bytes.ptr.is_null() || bytes.len == 0 {
            return Ok(MzSettingsCatalog::default());
        }
        MzSettingsCatalog::from_bytes(unsafe { std::slice::from_raw_parts(bytes.ptr, bytes.len) })
            .map_err(|_| MzStatusCode::InternalError)
    }

    pub fn read_diagnostic_catalog(&self) -> Result<MzDiagnosticCatalog, MzStatusCode> {
        let Some(read) = self.raw.read_diagnostic_catalog else {
            return Err(MzStatusCode::NotFound);
        };
        let bytes = read();
        if bytes.ptr.is_null() || bytes.len == 0 {
            return Ok(MzDiagnosticCatalog::default());
        }
        MzDiagnosticCatalog::from_bytes(unsafe { std::slice::from_raw_parts(bytes.ptr, bytes.len) })
            .map_err(|_| MzStatusCode::InternalError)
    }

    pub fn read_about_catalog(&self) -> Result<MzAboutCatalog, MzStatusCode> {
        let Some(read) = self.raw.read_about_catalog else {
            return Err(MzStatusCode::NotFound);
        };
        let bytes = read();
        if bytes.ptr.is_null() || bytes.len == 0 {
            return Ok(MzAboutCatalog::default());
        }
        MzAboutCatalog::from_bytes(unsafe { std::slice::from_raw_parts(bytes.ptr, bytes.len) })
            .map_err(|_| MzStatusCode::InternalError)
    }

    pub fn read_config(&self) -> Result<Vec<u8>, MzStatusCode> {
        let Some(read) = self.raw.read_config else {
            return Err(MzStatusCode::NotFound);
        };
        let bytes = read();
        if bytes.ptr.is_null() || bytes.len == 0 {
            return Ok(Vec::new());
        }
        Ok(unsafe { std::slice::from_raw_parts(bytes.ptr, bytes.len) }.to_vec())
    }

    pub fn write_config(&self, payload: &[u8]) -> Result<(), MzStatusCode> {
        let Some(write) = self.raw.write_config else {
            return Err(MzStatusCode::NotFound);
        };
        let status = write(MzBytes {
            ptr: payload.as_ptr(),
            len: payload.len(),
        });
        if status.is_ok() {
            Ok(())
        } else {
            Err(status.code)
        }
    }
}

pub trait Plugin: 'static {
    fn descriptor() -> PluginDescriptor;

    fn register(_host: &HostApi<'_>) -> Result<(), MzStatusCode> {
        Ok(())
    }

    fn startup(_host: &HostApi<'_>) -> Result<(), MzStatusCode> {
        Ok(())
    }

    fn shutdown(_host: &HostApi<'_>) {}
}

pub fn register_plugin<T: Plugin>(host: &MzHostApi) -> MzStatus {
    let host = HostApi::from_raw(host);
    into_status(T::register(&host))
}

pub fn startup_plugin<T: Plugin>(host: &MzHostApi) -> MzStatus {
    let host = HostApi::from_raw(host);
    into_status(T::startup(&host))
}

pub fn shutdown_plugin<T: Plugin>(host: &MzHostApi) {
    let host = HostApi::from_raw(host);
    T::shutdown(&host);
}

pub fn plugin_descriptor<T: Plugin>() -> MzPluginDescriptorView {
    plugin_descriptor_from(T::descriptor())
}

pub fn plugin_vtable<T: Plugin>() -> MzPluginVTable {
    MzPluginVTable {
        abi_version: MZ_ABI_VERSION_V1,
        descriptor: descriptor_bridge::<T>,
        register: register_bridge::<T>,
        startup: startup_bridge::<T>,
        shutdown: shutdown_bridge::<T>,
    }
}

fn into_status(result: Result<(), MzStatusCode>) -> MzStatus {
    match result {
        Ok(()) => MzStatus::OK,
        Err(code) => MzStatus::new(code),
    }
}

extern "C" fn descriptor_bridge<T: Plugin>() -> MzPluginDescriptorView {
    plugin_descriptor::<T>()
}

extern "C" fn register_bridge<T: Plugin>(host: *const MzHostApi) -> MzStatus {
    let Some(host) = (unsafe { host.as_ref() }) else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };
    register_plugin::<T>(host)
}

extern "C" fn startup_bridge<T: Plugin>(host: *const MzHostApi) -> MzStatus {
    let Some(host) = (unsafe { host.as_ref() }) else {
        return MzStatus::new(MzStatusCode::InvalidArgument);
    };
    startup_plugin::<T>(host)
}

extern "C" fn shutdown_bridge<T: Plugin>(host: *const MzHostApi) {
    let Some(host) = (unsafe { host.as_ref() }) else {
        return;
    };
    shutdown_plugin::<T>(host);
}

fn plugin_descriptor_from(descriptor: PluginDescriptor) -> MzPluginDescriptorView {
    let deps = descriptor
        .dependencies
        .iter()
        .copied()
        .map(PluginDependency::into_ffi)
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let deps = Box::leak(deps);

    MzPluginDescriptorView {
        id: MzStr::from_static(descriptor.id),
        name: MzStr::from_static(descriptor.name),
        version: descriptor.version.into_ffi(),
        required_abi_version: descriptor.required_abi_version,
        description: MzStr::from_static(descriptor.description),
        dependencies_ptr: deps.as_ptr(),
        dependencies_len: deps.len(),
    }
}

#[macro_export]
macro_rules! export_plugin {
    ($plugin_ty:ty) => {
        static MARUZZELLA_PLUGIN_VTABLE: ::std::sync::OnceLock<$crate::ffi::MzPluginVTable> =
            ::std::sync::OnceLock::new();

        #[no_mangle]
        pub extern "C" fn maruzzella_plugin_entry() -> *const $crate::ffi::MzPluginVTable {
            MARUZZELLA_PLUGIN_VTABLE.get_or_init(|| $crate::plugin_vtable::<$plugin_ty>())
                as *const $crate::ffi::MzPluginVTable
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use maruzzella_api::MzHostApi;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static REGISTER_CALLS: AtomicUsize = AtomicUsize::new(0);

    struct ExamplePlugin;

    impl Plugin for ExamplePlugin {
        fn descriptor() -> PluginDescriptor {
            static DEPS: &[PluginDependency] = &[PluginDependency::required(
                "maruzzella.base",
                Version::new(1, 0, 0),
                Version::new(2, 0, 0),
            )];

            PluginDescriptor::new(
                "com.example.plugin",
                "Example Plugin",
                Version::new(1, 2, 3),
            )
            .with_description("Example plugin used by tests")
            .with_dependencies(DEPS)
        }

        fn register(_host: &HostApi<'_>) -> Result<(), MzStatusCode> {
            REGISTER_CALLS.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[test]
    fn descriptor_bridge_returns_expected_metadata() {
        let descriptor = plugin_descriptor::<ExamplePlugin>();
        assert_eq!(descriptor.version, Version::new(1, 2, 3).into_ffi());
        assert_eq!(descriptor.dependencies_len, 1);
        assert_eq!(descriptor.required_abi_version, MZ_ABI_VERSION_V1);
    }

    #[test]
    fn register_wrapper_invokes_plugin_logic() {
        REGISTER_CALLS.store(0, Ordering::SeqCst);
        let status = register_plugin::<ExamplePlugin>(&MzHostApi::empty());
        assert!(status.is_ok());
        assert_eq!(REGISTER_CALLS.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn export_vtable_uses_v1_abi() {
        let vtable = plugin_vtable::<ExamplePlugin>();
        assert_eq!(vtable.abi_version, MZ_ABI_VERSION_V1);
    }
}
