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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MzLogLevel {
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
    Trace = 5,
}

impl Default for MzLogLevel {
    fn default() -> Self {
        Self::Info
    }
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
    pub instance_key: MzStr,
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MzSettingsPage {
    pub page_id: String,
    pub title: String,
    pub summary: String,
    pub category: MzSettingsCategory,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MzToolbarItem {
    pub item_id: String,
    pub icon_name: Option<String>,
    pub label: Option<String>,
    pub command_id: String,
    pub secondary: bool,
}

impl MzToolbarItem {
    pub fn new(
        item_id: impl Into<String>,
        icon_name: Option<String>,
        label: Option<String>,
        command_id: impl Into<String>,
        secondary: bool,
    ) -> Self {
        Self {
            item_id: item_id.into(),
            icon_name,
            label,
            command_id: command_id.into(),
            secondary,
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MzStartupTab {
    pub group_id: String,
    pub tab_id: String,
    pub title: String,
    pub plugin_view_id: String,
    pub instance_key: Option<String>,
    pub payload: Vec<u8>,
    pub placeholder: String,
    pub closable: bool,
    pub active: bool,
}

impl MzStartupTab {
    pub fn new(
        group_id: impl Into<String>,
        tab_id: impl Into<String>,
        title: impl Into<String>,
        plugin_view_id: impl Into<String>,
    ) -> Self {
        Self {
            group_id: group_id.into(),
            tab_id: tab_id.into(),
            title: title.into(),
            plugin_view_id: plugin_view_id.into(),
            instance_key: None,
            payload: Vec::new(),
            placeholder: String::new(),
            closable: true,
            active: false,
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MzCommandSummary {
    pub command_id: String,
    pub title: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MzCommandCatalog {
    pub commands: Vec<MzCommandSummary>,
}

impl MzCommandCatalog {
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MzViewSummary {
    pub plugin_id: String,
    pub view_id: String,
    pub title: String,
    pub placement: MzViewPlacement,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MzViewCatalog {
    pub views: Vec<MzViewSummary>,
}

impl MzViewCatalog {
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MzPluginDiagnosticSummary {
    pub level: String,
    pub plugin_id: Option<String>,
    pub path: Option<String>,
    pub message: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MzPluginLogSummary {
    pub level: MzLogLevel,
    pub message: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MzPluginSummary {
    pub plugin_id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub views: Vec<MzViewSummary>,
    pub settings_pages: Vec<MzSettingsPage>,
    pub logs: Vec<MzPluginLogSummary>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MzPluginSnapshot {
    pub activation_order: Vec<String>,
    pub diagnostics: Vec<MzPluginDiagnosticSummary>,
    pub plugins: Vec<MzPluginSummary>,
}

impl MzPluginSnapshot {
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MzAboutCatalog {
    pub sections: Vec<MzAboutSection>,
}

impl MzAboutCatalog {
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

impl MzSettingsPage {
    pub fn new(
        page_id: impl Into<String>,
        title: impl Into<String>,
        summary: impl Into<String>,
        category: MzSettingsCategory,
    ) -> Self {
        Self {
            page_id: page_id.into(),
            title: title.into(),
            summary: summary.into(),
            category,
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MzSettingsCategory {
    General,
    Workspace,
    Integrations,
    Diagnostics,
}

impl MzSettingsCategory {
    pub const fn label(self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Workspace => "Workspace",
            Self::Integrations => "Integrations",
            Self::Diagnostics => "Diagnostics",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MzMenuSurface {
    FileItems,
    ViewItems,
    HelpItems,
}

impl MzMenuSurface {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FileItems => "maruzzella.menu.file.items",
            Self::ViewItems => "maruzzella.menu.view.items",
            Self::HelpItems => "maruzzella.menu.help.items",
        }
    }

    pub const fn root_id(self) -> &'static str {
        match self {
            Self::FileItems => "file",
            Self::ViewItems => "view",
            Self::HelpItems => "help",
        }
    }

    pub const fn root_label(self) -> &'static str {
        match self {
            Self::FileItems => "File",
            Self::ViewItems => "View",
            Self::HelpItems => "Help",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "maruzzella.menu.file.items" => Some(Self::FileItems),
            "maruzzella.menu.view.items" => Some(Self::ViewItems),
            "maruzzella.menu.help.items" => Some(Self::HelpItems),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MzContributionSurface {
    AboutSections,
    PluginSettingsPages,
    ToolbarItems,
    StartupTabs,
}

impl MzContributionSurface {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AboutSections => "maruzzella.about.sections",
            Self::PluginSettingsPages => "maruzzella.plugins.settings_pages",
            Self::ToolbarItems => "maruzzella.toolbar.items",
            Self::StartupTabs => "maruzzella.startup.tabs",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "maruzzella.about.sections" => Some(Self::AboutSections),
            "maruzzella.plugins.settings_pages" => Some(Self::PluginSettingsPages),
            "maruzzella.toolbar.items" => Some(Self::ToolbarItems),
            "maruzzella.startup.tabs" => Some(Self::StartupTabs),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MzViewPlacement {
    Workbench,
    SidePanel,
    BottomPanel,
    Dialog,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MzViewOpenDisposition {
    #[default]
    Opened = 0,
    FocusedExisting = 1,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MzOpenViewRequest {
    pub plugin_id: MzStr,
    pub view_id: MzStr,
    pub placement: MzViewPlacement,
    pub instance_key: MzStr,
    pub requested_title: MzStr,
    pub payload: MzBytes,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MzOpenViewResult {
    pub status: MzStatus,
    pub disposition: MzViewOpenDisposition,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MzViewQuery {
    pub plugin_id: MzStr,
    pub view_id: MzStr,
    pub instance_key: MzStr,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MzViewQueryResult {
    pub status: MzStatus,
    pub found: bool,
}

impl MzViewPlacement {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Workbench => "Workbench",
            Self::SidePanel => "Side Panel",
            Self::BottomPanel => "Bottom Panel",
            Self::Dialog => "Dialog",
        }
    }
}

impl Default for MzViewPlacement {
    fn default() -> Self {
        Self::Workbench
    }
}

pub type MzCreateViewFn =
    extern "C" fn(host: *const MzHostApi, request: *const MzViewRequest) -> *mut c_void;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MzViewFactorySpec {
    pub plugin_id: MzStr,
    pub view_id: MzStr,
    pub title: MzStr,
    pub placement: MzViewPlacement,
    pub create: MzCreateViewFn,
}

pub type MzLogFn = extern "C" fn(level: MzLogLevel, message: MzStr);
pub type MzCommandInvokeFn = extern "C" fn(payload: MzBytes) -> MzStatus;
pub type MzRegisterCommandFn = extern "C" fn(command: *const MzCommandSpec) -> MzStatus;
pub type MzRegisterMenuItemFn = extern "C" fn(item: *const MzMenuItemSpec) -> MzStatus;
pub type MzRegisterSurfaceContributionFn =
    extern "C" fn(contribution: *const MzSurfaceContribution) -> MzStatus;
pub type MzRegisterViewFactoryFn = extern "C" fn(factory: *const MzViewFactorySpec) -> MzStatus;
pub type MzDispatchCommandFn = extern "C" fn(command_id: MzStr, payload: MzBytes) -> MzStatus;
pub type MzOpenViewFn = extern "C" fn(request: *const MzOpenViewRequest) -> MzOpenViewResult;
pub type MzFocusViewFn = extern "C" fn(query: *const MzViewQuery) -> MzStatus;
pub type MzIsViewOpenFn = extern "C" fn(query: *const MzViewQuery) -> MzViewQueryResult;
pub type MzUpdateViewTitleFn = extern "C" fn(query: *const MzViewQuery, title: MzStr) -> MzStatus;
pub type MzReadCommandCatalogFn = extern "C" fn() -> MzBytes;
pub type MzReadViewCatalogFn = extern "C" fn() -> MzBytes;
pub type MzReadPluginStateFn = extern "C" fn() -> MzBytes;
pub type MzReadAboutCatalogFn = extern "C" fn() -> MzBytes;
pub type MzReadConfigFn = extern "C" fn() -> MzBytes;
pub type MzWriteConfigFn = extern "C" fn(payload: MzBytes) -> MzStatus;

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
    pub open_view: Option<MzOpenViewFn>,
    pub focus_view: Option<MzFocusViewFn>,
    pub is_view_open: Option<MzIsViewOpenFn>,
    pub update_view_title: Option<MzUpdateViewTitleFn>,
    pub read_command_catalog: Option<MzReadCommandCatalogFn>,
    pub read_view_catalog: Option<MzReadViewCatalogFn>,
    pub read_plugin_state: Option<MzReadPluginStateFn>,
    pub read_about_catalog: Option<MzReadAboutCatalogFn>,
    pub read_config: Option<MzReadConfigFn>,
    pub write_config: Option<MzWriteConfigFn>,
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
            open_view: None,
            focus_view: None,
            is_view_open: None,
            update_view_title: None,
            read_command_catalog: None,
            read_view_catalog: None,
            read_plugin_state: None,
            read_about_catalog: None,
            read_config: None,
            write_config: None,
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

    #[test]
    fn settings_page_roundtrips_through_json_bytes() {
        let page = MzSettingsPage::new(
            "general",
            "General",
            "Example plugin settings",
            MzSettingsCategory::General,
        );
        let bytes = page.to_bytes().expect("settings page should serialize");
        let decoded = MzSettingsPage::from_bytes(&bytes).expect("settings page should decode");
        assert_eq!(decoded, page);
    }

    #[test]
    fn menu_surface_helpers_roundtrip() {
        let surface =
            MzMenuSurface::parse("maruzzella.menu.file.items").expect("menu surface should parse");
        assert_eq!(surface, MzMenuSurface::FileItems);
        assert_eq!(surface.root_id(), "file");
        assert_eq!(surface.root_label(), "File");
        assert_eq!(surface.as_str(), "maruzzella.menu.file.items");
    }

    #[test]
    fn contribution_surface_helpers_roundtrip() {
        let surface = MzContributionSurface::parse("maruzzella.plugins.settings_pages")
            .expect("surface should parse");
        assert_eq!(surface, MzContributionSurface::PluginSettingsPages);
        assert_eq!(surface.as_str(), "maruzzella.plugins.settings_pages");
    }

    #[test]
    fn settings_category_labels_are_stable() {
        assert_eq!(MzSettingsCategory::Workspace.label(), "Workspace");
    }

    #[test]
    fn view_placement_labels_are_stable() {
        assert_eq!(MzViewPlacement::BottomPanel.label(), "Bottom Panel");
    }
}
