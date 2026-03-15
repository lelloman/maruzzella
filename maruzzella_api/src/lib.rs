use core::ffi::c_void;
use serde::{Deserialize, Serialize};

pub const MZ_ABI_VERSION_V1: u32 = 1;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MzStr {
    pub ptr: *const u8,
    pub len: usize,
}

impl MzStr {
    pub const fn empty() -> Self {
        Self {
            ptr: core::ptr::null(),
            len: 0,
        }
    }

    pub const fn from_static(value: &'static str) -> Self {
        Self {
            ptr: value.as_ptr(),
            len: value.len(),
        }
    }

    pub const fn is_empty(self) -> bool {
        self.len == 0
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MzBytes {
    pub ptr: *const u8,
    pub len: usize,
}

impl MzBytes {
    pub const fn empty() -> Self {
        Self {
            ptr: core::ptr::null(),
            len: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct MzVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl MzVersion {
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MzHandle(pub usize);

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MzStatusCode {
    Ok = 0,
    InvalidArgument = 1,
    AbiMismatch = 2,
    AlreadyExists = 3,
    NotFound = 4,
    InternalError = 5,
}

impl Default for MzStatusCode {
    fn default() -> Self {
        Self::Ok
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MzStatus {
    pub code: MzStatusCode,
}

impl MzStatus {
    pub const OK: Self = Self {
        code: MzStatusCode::Ok,
    };

    pub const fn new(code: MzStatusCode) -> Self {
        Self { code }
    }

    pub const fn is_ok(self) -> bool {
        matches!(self.code, MzStatusCode::Ok)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MzPluginDependency {
    pub plugin_id: MzStr,
    pub min_version: MzVersion,
    pub max_version_exclusive: MzVersion,
    pub required: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MzPluginDescriptorView {
    pub id: MzStr,
    pub name: MzStr,
    pub version: MzVersion,
    pub required_abi_version: u32,
    pub description: MzStr,
    pub dependencies_ptr: *const MzPluginDependency,
    pub dependencies_len: usize,
}

impl MzPluginDescriptorView {
    pub const fn empty() -> Self {
        Self {
            id: MzStr::empty(),
            name: MzStr::empty(),
            version: MzVersion::new(0, 0, 0),
            required_abi_version: MZ_ABI_VERSION_V1,
            description: MzStr::empty(),
            dependencies_ptr: core::ptr::null(),
            dependencies_len: 0,
        }
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MzLogLevel {
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
    Trace = 5,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct MzCommandSpec {
    pub plugin_id: MzStr,
    pub command_id: MzStr,
    pub title: MzStr,
    pub invoke: Option<MzCommandInvokeFn>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MzMenuItemSpec {
    pub plugin_id: MzStr,
    pub menu_id: MzStr,
    pub parent_id: MzStr,
    pub title: MzStr,
    pub command_id: MzStr,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MzSurfaceContribution {
    pub plugin_id: MzStr,
    pub surface_id: MzStr,
    pub contribution_id: MzStr,
    pub payload: MzBytes,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MzViewRequest {
    pub plugin_id: MzStr,
    pub view_id: MzStr,
    pub payload: MzBytes,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MzAboutSection {
    pub title: String,
    pub body: String,
}

impl MzAboutSection {
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

pub type MzCreateViewFn =
    extern "C" fn(host: *const MzHostApi, request: *const MzViewRequest) -> *mut c_void;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MzViewFactorySpec {
    pub plugin_id: MzStr,
    pub view_id: MzStr,
    pub create: MzCreateViewFn,
}

pub type MzLogFn = extern "C" fn(level: MzLogLevel, message: MzStr);
pub type MzCommandInvokeFn = extern "C" fn(payload: MzBytes) -> MzStatus;
pub type MzRegisterCommandFn = extern "C" fn(command: *const MzCommandSpec) -> MzStatus;
pub type MzRegisterMenuItemFn = extern "C" fn(item: *const MzMenuItemSpec) -> MzStatus;
pub type MzRegisterSurfaceContributionFn =
    extern "C" fn(contribution: *const MzSurfaceContribution) -> MzStatus;
pub type MzRegisterViewFactoryFn =
    extern "C" fn(factory: *const MzViewFactorySpec) -> MzStatus;
pub type MzDispatchCommandFn = extern "C" fn(command_id: MzStr, payload: MzBytes) -> MzStatus;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MzHostApi {
    pub abi_version: u32,
    pub host_context: *mut c_void,
    pub log: Option<MzLogFn>,
    pub register_command: Option<MzRegisterCommandFn>,
    pub register_menu_item: Option<MzRegisterMenuItemFn>,
    pub register_surface_contribution: Option<MzRegisterSurfaceContributionFn>,
    pub register_view_factory: Option<MzRegisterViewFactoryFn>,
    pub dispatch_command: Option<MzDispatchCommandFn>,
}

impl MzHostApi {
    pub const fn empty() -> Self {
        Self {
            abi_version: MZ_ABI_VERSION_V1,
            host_context: core::ptr::null_mut(),
            log: None,
            register_command: None,
            register_menu_item: None,
            register_surface_contribution: None,
            register_view_factory: None,
            dispatch_command: None,
        }
    }
}

pub type MzPluginDescriptorFn = extern "C" fn() -> MzPluginDescriptorView;
pub type MzPluginRegisterFn = extern "C" fn(host: *const MzHostApi) -> MzStatus;
pub type MzPluginStartupFn = extern "C" fn(host: *const MzHostApi) -> MzStatus;
pub type MzPluginShutdownFn = extern "C" fn(host: *const MzHostApi);

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MzPluginVTable {
    pub abi_version: u32,
    pub descriptor: MzPluginDescriptorFn,
    pub register: MzPluginRegisterFn,
    pub startup: MzPluginStartupFn,
    pub shutdown: MzPluginShutdownFn,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_string_view_matches_literal() {
        let value = MzStr::from_static("maruzzella.base");
        assert_eq!(value.len, "maruzzella.base".len());
        assert!(!value.ptr.is_null());
    }

    #[test]
    fn empty_descriptor_defaults_to_v1() {
        let descriptor = MzPluginDescriptorView::empty();
        assert_eq!(descriptor.required_abi_version, MZ_ABI_VERSION_V1);
        assert!(descriptor.id.is_empty());
        assert!(descriptor.name.is_empty());
    }

    #[test]
    fn ok_status_reports_success() {
        assert!(MzStatus::OK.is_ok());
        assert!(!MzStatus::new(MzStatusCode::InternalError).is_ok());
    }

    #[test]
    fn about_section_roundtrips_through_json_bytes() {
        let section = MzAboutSection::new("Maruzzella", "Core shell services");
        let bytes = section.to_bytes().expect("about section should serialize");
        let decoded = MzAboutSection::from_bytes(&bytes).expect("about section should decode");
        assert_eq!(decoded, section);
    }
}
