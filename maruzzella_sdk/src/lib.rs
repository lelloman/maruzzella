pub use maruzzella_api as ffi;
use maruzzella_api::{
    MzBytes, MzCommandSpec, MzHostApi, MzMenuItemSpec, MzPluginDependency, MzPluginDescriptorView,
    MzPluginVTable, MzStatus, MzStr, MzSurfaceContribution, MzVersion, MzViewFactorySpec,
    MzViewRequest, MZ_ABI_VERSION_V1,
};
pub use maruzzella_api::{MzLogLevel, MzStatusCode};

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
    pub parent_id: &'static str,
    pub title: &'static str,
    pub command_id: &'static str,
}

impl MenuItemSpec {
    pub const fn new(
        plugin_id: &'static str,
        menu_id: &'static str,
        parent_id: &'static str,
        title: &'static str,
        command_id: &'static str,
    ) -> Self {
        Self {
            plugin_id,
            menu_id,
            parent_id,
            title,
            command_id,
        }
    }

    fn into_ffi(self) -> MzMenuItemSpec {
        MzMenuItemSpec {
            plugin_id: MzStr::from_static(self.plugin_id),
            menu_id: MzStr::from_static(self.menu_id),
            parent_id: MzStr::from_static(self.parent_id),
            title: MzStr::from_static(self.title),
            command_id: MzStr::from_static(self.command_id),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SurfaceContributionSpec {
    pub plugin_id: &'static str,
    pub surface_id: &'static str,
    pub contribution_id: &'static str,
    pub payload: Vec<u8>,
}

#[derive(Clone, Copy, Debug)]
pub struct ViewFactorySpec {
    pub plugin_id: &'static str,
    pub view_id: &'static str,
    pub create: maruzzella_api::MzCreateViewFn,
}

impl ViewFactorySpec {
    pub const fn new(
        plugin_id: &'static str,
        view_id: &'static str,
        create: maruzzella_api::MzCreateViewFn,
    ) -> Self {
        Self {
            plugin_id,
            view_id,
            create,
        }
    }

    fn into_ffi(self) -> MzViewFactorySpec {
        MzViewFactorySpec {
            plugin_id: MzStr::from_static(self.plugin_id),
            view_id: MzStr::from_static(self.view_id),
            create: self.create,
        }
    }
}

impl SurfaceContributionSpec {
    pub fn new(
        plugin_id: &'static str,
        surface_id: &'static str,
        contribution_id: &'static str,
        payload: impl Into<Vec<u8>>,
    ) -> Self {
        Self {
            plugin_id,
            surface_id,
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
            "maruzzella.about.sections",
            contribution_id,
            payload,
        )
    }

    pub fn settings_page(
        plugin_id: &'static str,
        contribution_id: &'static str,
        title: impl Into<String>,
        summary: impl Into<String>,
    ) -> Self {
        let payload = maruzzella_api::MzSettingsPage::new(title, summary)
            .to_bytes()
            .expect("settings pages should serialize");
        Self::new(
            plugin_id,
            "maruzzella.plugins.settings_pages",
            contribution_id,
            payload,
        )
    }

    fn as_ffi(&self) -> MzSurfaceContribution {
        MzSurfaceContribution {
            plugin_id: MzStr::from_static(self.plugin_id),
            surface_id: MzStr::from_static(self.surface_id),
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
