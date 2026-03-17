use std::ffi::c_void;
use std::path::PathBuf;

use gtk::glib::translate::IntoGlibPtr;
use gtk::prelude::*;
use gtk::{Align, Box as GtkBox, Button, Label, Orientation, Separator};
use maruzzella_api::{
    MzAboutCatalog, MzAboutSection, MzBytes, MzCommandCatalog, MzCommandSpec,
    MzContributionSurface, MzDiagnosticCatalog, MzHostApi, MzLogLevel, MzMenuItemSpec,
    MzMenuSurface, MzOpenViewRequest, MzPluginDescriptorView, MzPluginSnapshot, MzPluginVTable,
    MzSettingsCatalog, MzSettingsCategory, MzSettingsPage, MzStartupTab, MzStatus, MzStr,
    MzSurfaceContribution, MzToolbarItem, MzVersion, MzViewCatalog, MzViewFactorySpec,
    MzViewPlacement, MzViewRequest, MZ_ABI_VERSION_V1,
};

use crate::plugins::{LoadedPlugin, PluginDescriptor, Version};

const BASE_PLUGIN_ID: &str = "maruzzella.base";

const VIEW_WORKSPACE_HOME: &str = "maruzzella.base.workspace.home";
const VIEW_WORKSPACE_QUEUE: &str = "maruzzella.base.workspace.queue";
const VIEW_WORKSPACE_SURFACES: &str = "maruzzella.base.workspace.surfaces";
const VIEW_WORKSPACE_OPS: &str = "maruzzella.base.workspace.ops";
const VIEW_WORKSPACE_COMMANDS: &str = "maruzzella.base.workspace.commands";
const VIEW_WORKSPACE_REGISTERED_VIEWS: &str = "maruzzella.base.workspace.registered_views";
const VIEW_WORKSPACE_PLUGINS: &str = "maruzzella.base.workspace.plugins";
const VIEW_WORKSPACE_SETTINGS: &str = "maruzzella.base.workspace.settings";
const VIEW_WORKSPACE_ABOUT: &str = "maruzzella.base.workspace.about";
const VIEW_PANEL_NAVIGATOR: &str = "maruzzella.base.panel.navigator";
const VIEW_PANEL_RESOURCES: &str = "maruzzella.base.panel.resources";
const VIEW_PANEL_INSPECTOR: &str = "maruzzella.base.panel.inspector";
const VIEW_PANEL_DELIVERY: &str = "maruzzella.base.panel.delivery";
const VIEW_PANEL_ACTIVITY: &str = "maruzzella.base.panel.activity";
const VIEW_PANEL_EXTENSIONS: &str = "maruzzella.base.panel.extensions";

pub fn load() -> LoadedPlugin {
    LoadedPlugin::from_static_vtable(
        PathBuf::from("<builtin:maruzzella.base>"),
        PluginDescriptor {
            id: BASE_PLUGIN_ID.to_string(),
            name: "Maruzzella Base".to_string(),
            version: Version {
                major: 1,
                minor: 0,
                patch: 0,
            },
            required_abi_version: MZ_ABI_VERSION_V1,
            description: "Built-in plugin providing core shell commands and menu surfaces"
                .to_string(),
            dependencies: Vec::new(),
        },
        &BASE_PLUGIN_VTABLE,
    )
}

static BASE_PLUGIN_VTABLE: MzPluginVTable = MzPluginVTable {
    abi_version: MZ_ABI_VERSION_V1,
    descriptor: base_descriptor,
    register: base_register,
    startup: base_startup,
    shutdown: base_shutdown,
};

extern "C" fn base_descriptor() -> MzPluginDescriptorView {
    MzPluginDescriptorView {
        id: MzStr::from_static(BASE_PLUGIN_ID),
        name: MzStr::from_static("Maruzzella Base"),
        version: MzVersion::new(1, 0, 0),
        required_abi_version: MZ_ABI_VERSION_V1,
        description: MzStr::from_static(
            "Built-in plugin providing core shell commands and menu surfaces",
        ),
        dependencies_ptr: std::ptr::null(),
        dependencies_len: 0,
    }
}

extern "C" fn base_register(host: *const MzHostApi) -> MzStatus {
    let Some(host) = (unsafe { host.as_ref() }) else {
        return MzStatus::new(maruzzella_api::MzStatusCode::InvalidArgument);
    };

    let commands = [
        MzCommandSpec {
            plugin_id: MzStr::from_static(BASE_PLUGIN_ID),
            command_id: MzStr::from_static("shell.open_command_palette"),
            title: MzStr::from_static("Command Palette"),
            invoke: None,
        },
        MzCommandSpec {
            plugin_id: MzStr::from_static(BASE_PLUGIN_ID),
            command_id: MzStr::from_static("shell.browse_views"),
            title: MzStr::from_static("Browse Views"),
            invoke: None,
        },
        MzCommandSpec {
            plugin_id: MzStr::from_static(BASE_PLUGIN_ID),
            command_id: MzStr::from_static("shell.reload_theme"),
            title: MzStr::from_static("Reload Theme"),
            invoke: None,
        },
        MzCommandSpec {
            plugin_id: MzStr::from_static(BASE_PLUGIN_ID),
            command_id: MzStr::from_static("shell.about"),
            title: MzStr::from_static("About Maruzzella"),
            invoke: None,
        },
        MzCommandSpec {
            plugin_id: MzStr::from_static(BASE_PLUGIN_ID),
            command_id: MzStr::from_static("shell.plugins"),
            title: MzStr::from_static("Plugins"),
            invoke: None,
        },
        MzCommandSpec {
            plugin_id: MzStr::from_static(BASE_PLUGIN_ID),
            command_id: MzStr::from_static("shell.settings"),
            title: MzStr::from_static("Settings"),
            invoke: None,
        },
    ];

    let menu_items = [
        MzMenuItemSpec {
            plugin_id: MzStr::from_static(BASE_PLUGIN_ID),
            menu_id: MzStr::from_static("command-palette"),
            parent_id: MzStr::from_static(MzMenuSurface::ViewItems.as_str()),
            title: MzStr::from_static("Command Palette"),
            command_id: MzStr::from_static("shell.open_command_palette"),
            payload: MzBytes::empty(),
        },
        MzMenuItemSpec {
            plugin_id: MzStr::from_static(BASE_PLUGIN_ID),
            menu_id: MzStr::from_static("reload-theme"),
            parent_id: MzStr::from_static(MzMenuSurface::ViewItems.as_str()),
            title: MzStr::from_static("Reload Theme"),
            command_id: MzStr::from_static("shell.reload_theme"),
            payload: MzBytes::empty(),
        },
        MzMenuItemSpec {
            plugin_id: MzStr::from_static(BASE_PLUGIN_ID),
            menu_id: MzStr::from_static("browse-views"),
            parent_id: MzStr::from_static(MzMenuSurface::ViewItems.as_str()),
            title: MzStr::from_static("Browse Views"),
            command_id: MzStr::from_static("shell.browse_views"),
            payload: MzBytes::empty(),
        },
        MzMenuItemSpec {
            plugin_id: MzStr::from_static(BASE_PLUGIN_ID),
            menu_id: MzStr::from_static("plugins"),
            parent_id: MzStr::from_static(MzMenuSurface::FileItems.as_str()),
            title: MzStr::from_static("Plugins"),
            command_id: MzStr::from_static("shell.plugins"),
            payload: MzBytes::empty(),
        },
        MzMenuItemSpec {
            plugin_id: MzStr::from_static(BASE_PLUGIN_ID),
            menu_id: MzStr::from_static("settings"),
            parent_id: MzStr::from_static(MzMenuSurface::FileItems.as_str()),
            title: MzStr::from_static("Settings"),
            command_id: MzStr::from_static("shell.settings"),
            payload: MzBytes::empty(),
        },
        MzMenuItemSpec {
            plugin_id: MzStr::from_static(BASE_PLUGIN_ID),
            menu_id: MzStr::from_static("about"),
            parent_id: MzStr::from_static(MzMenuSurface::HelpItems.as_str()),
            title: MzStr::from_static("About Maruzzella"),
            command_id: MzStr::from_static("shell.about"),
            payload: MzBytes::empty(),
        },
    ];

    let about_payload = MzAboutSection::new(
        "Maruzzella",
        "Core shell services and the default workspace slice are provided by the built-in base plugin.",
    )
    .to_bytes()
    .expect("built-in about section should serialize");
    let settings_payload = MzSettingsPage::new(
        "workspace-defaults",
        "Workspace Defaults",
        "Default shell areas are now base-plugin-backed views rather than placeholder ProductSpec tabs.",
        MzSettingsCategory::Workspace,
    )
    .with_view(VIEW_WORKSPACE_SETTINGS, MzViewPlacement::Workbench)
    .with_instance_key(format!("plugin:{BASE_PLUGIN_ID}"))
    .with_requested_title("Maruzzella Base Settings")
    .to_bytes()
    .expect("built-in settings page should serialize");
    let about = MzSurfaceContribution {
        plugin_id: MzStr::from_static(BASE_PLUGIN_ID),
        surface_id: MzStr::from_static(MzContributionSurface::AboutSections.as_str()),
        contribution_id: MzStr::from_static("maruzzella.base.about"),
        payload: MzBytes {
            ptr: about_payload.as_ptr(),
            len: about_payload.len(),
        },
    };
    let settings = MzSurfaceContribution {
        plugin_id: MzStr::from_static(BASE_PLUGIN_ID),
        surface_id: MzStr::from_static(MzContributionSurface::PluginSettingsPages.as_str()),
        contribution_id: MzStr::from_static("maruzzella.base.settings.workspace"),
        payload: MzBytes {
            ptr: settings_payload.as_ptr(),
            len: settings_payload.len(),
        },
    };
    let toolbar_payloads = [
        (
            "palette",
            toolbar_item_payload(
                "palette",
                Some("system-search-symbolic"),
                Some("Palette"),
                "shell.open_command_palette",
                false,
            ),
        ),
        (
            "theme",
            toolbar_item_payload(
                "theme",
                Some("applications-graphics-symbolic"),
                None,
                "shell.reload_theme",
                true,
            ),
        ),
        (
            "views",
            toolbar_item_payload(
                "views",
                Some("view-grid-symbolic"),
                None,
                "shell.browse_views",
                true,
            ),
        ),
        (
            "about",
            toolbar_item_payload(
                "about",
                Some("help-about-symbolic"),
                None,
                "shell.about",
                true,
            ),
        ),
        (
            "settings",
            toolbar_item_payload(
                "settings",
                Some("preferences-system-symbolic"),
                None,
                "shell.settings",
                true,
            ),
        ),
    ];
    let startup_tab_payloads = [
        (
            "workspace-nav",
            startup_tab_payload(
                "panel-left",
                "workspace-nav",
                "Workspace",
                VIEW_PANEL_NAVIGATOR,
                false,
                true,
            ),
        ),
        (
            "resource-index",
            startup_tab_payload(
                "panel-left",
                "resource-index",
                "Resources",
                VIEW_PANEL_RESOURCES,
                false,
                false,
            ),
        ),
        (
            "selection-inspector",
            startup_tab_payload(
                "panel-right",
                "selection-inspector",
                "Inspector",
                VIEW_PANEL_INSPECTOR,
                false,
                true,
            ),
        ),
        (
            "delivery-checklist",
            startup_tab_payload(
                "panel-right",
                "delivery-checklist",
                "Release",
                VIEW_PANEL_DELIVERY,
                false,
                false,
            ),
        ),
        (
            "runtime-activity",
            startup_tab_payload(
                "panel-bottom",
                "runtime-activity",
                "Activity",
                VIEW_PANEL_ACTIVITY,
                false,
                true,
            ),
        ),
        (
            "extension-health",
            startup_tab_payload(
                "panel-bottom",
                "extension-health",
                "Extensions",
                VIEW_PANEL_EXTENSIONS,
                false,
                false,
            ),
        ),
        (
            "studio-home",
            startup_tab_payload(
                "workbench-main",
                "studio-home",
                "Studio Home",
                VIEW_WORKSPACE_HOME,
                false,
                true,
            ),
        ),
        (
            "work-queue",
            startup_tab_payload(
                "workbench-main",
                "work-queue",
                "Work Queue",
                VIEW_WORKSPACE_QUEUE,
                true,
                false,
            ),
        ),
        (
            "integration-surfaces",
            startup_tab_payload(
                "workbench-secondary",
                "integration-surfaces",
                "Contribution Surfaces",
                VIEW_WORKSPACE_SURFACES,
                false,
                true,
            ),
        ),
        (
            "system-ops",
            startup_tab_payload(
                "workbench-secondary",
                "system-ops",
                "System Ops",
                VIEW_WORKSPACE_OPS,
                true,
                false,
            ),
        ),
    ];
    let view_factories = [
        view_factory(VIEW_WORKSPACE_HOME),
        view_factory(VIEW_WORKSPACE_QUEUE),
        view_factory(VIEW_WORKSPACE_SURFACES),
        view_factory(VIEW_WORKSPACE_OPS),
        view_factory(VIEW_WORKSPACE_COMMANDS),
        view_factory(VIEW_WORKSPACE_REGISTERED_VIEWS),
        view_factory(VIEW_WORKSPACE_PLUGINS),
        view_factory(VIEW_WORKSPACE_SETTINGS),
        view_factory(VIEW_WORKSPACE_ABOUT),
        view_factory(VIEW_PANEL_NAVIGATOR),
        view_factory(VIEW_PANEL_RESOURCES),
        view_factory(VIEW_PANEL_INSPECTOR),
        view_factory(VIEW_PANEL_DELIVERY),
        view_factory(VIEW_PANEL_ACTIVITY),
        view_factory(VIEW_PANEL_EXTENSIONS),
    ];

    for command in commands {
        let status = host.register_command.expect("command registrar")(&command);
        if !status.is_ok() {
            return status;
        }
    }

    for item in menu_items {
        let status = host.register_menu_item.expect("menu registrar")(&item);
        if !status.is_ok() {
            return status;
        }
    }

    let status = host
        .register_surface_contribution
        .expect("surface registrar")(&about);
    if !status.is_ok() {
        return status;
    }

    let status = host
        .register_surface_contribution
        .expect("surface registrar")(&settings);
    if !status.is_ok() {
        return status;
    }

    for (contribution_id, payload) in &toolbar_payloads {
        let contribution = MzSurfaceContribution {
            plugin_id: MzStr::from_static(BASE_PLUGIN_ID),
            surface_id: MzStr::from_static(MzContributionSurface::ToolbarItems.as_str()),
            contribution_id: MzStr {
                ptr: contribution_id.as_ptr(),
                len: contribution_id.len(),
            },
            payload: MzBytes {
                ptr: payload.as_ptr(),
                len: payload.len(),
            },
        };
        let status = host
            .register_surface_contribution
            .expect("surface registrar")(&contribution);
        if !status.is_ok() {
            return status;
        }
    }

    for (contribution_id, payload) in &startup_tab_payloads {
        let contribution = MzSurfaceContribution {
            plugin_id: MzStr::from_static(BASE_PLUGIN_ID),
            surface_id: MzStr::from_static(MzContributionSurface::StartupTabs.as_str()),
            contribution_id: MzStr {
                ptr: contribution_id.as_ptr(),
                len: contribution_id.len(),
            },
            payload: MzBytes {
                ptr: payload.as_ptr(),
                len: payload.len(),
            },
        };
        let status = host
            .register_surface_contribution
            .expect("surface registrar")(&contribution);
        if !status.is_ok() {
            return status;
        }
    }

    for factory in view_factories {
        let status = host.register_view_factory.expect("view registrar")(&factory);
        if !status.is_ok() {
            return status;
        }
    }

    MzStatus::OK
}

extern "C" fn base_startup(host: *const MzHostApi) -> MzStatus {
    let Some(host) = (unsafe { host.as_ref() }) else {
        return MzStatus::new(maruzzella_api::MzStatusCode::InvalidArgument);
    };
    if let Some(log) = host.log {
        log(
            MzLogLevel::Info,
            MzStr::from_static("maruzzella.base started"),
        );
    }
    MzStatus::OK
}

extern "C" fn base_shutdown(_host: *const MzHostApi) {}

fn view_factory(view_id: &'static str) -> MzViewFactorySpec {
    let (title, placement) = match view_id {
        VIEW_WORKSPACE_HOME => ("Studio Home", MzViewPlacement::Workbench),
        VIEW_WORKSPACE_QUEUE => ("Work Queue", MzViewPlacement::Workbench),
        VIEW_WORKSPACE_SURFACES => ("Contribution Surfaces", MzViewPlacement::Workbench),
        VIEW_WORKSPACE_OPS => ("System Ops", MzViewPlacement::Workbench),
        VIEW_WORKSPACE_COMMANDS => ("Command Palette", MzViewPlacement::Workbench),
        VIEW_WORKSPACE_REGISTERED_VIEWS => ("Registered Views", MzViewPlacement::Workbench),
        VIEW_WORKSPACE_PLUGINS => ("Plugins", MzViewPlacement::Workbench),
        VIEW_WORKSPACE_SETTINGS => ("Settings", MzViewPlacement::Workbench),
        VIEW_WORKSPACE_ABOUT => ("About", MzViewPlacement::Workbench),
        VIEW_PANEL_NAVIGATOR => ("Workspace", MzViewPlacement::SidePanel),
        VIEW_PANEL_RESOURCES => ("Resources", MzViewPlacement::SidePanel),
        VIEW_PANEL_INSPECTOR => ("Inspector", MzViewPlacement::SidePanel),
        VIEW_PANEL_DELIVERY => ("Release", MzViewPlacement::SidePanel),
        VIEW_PANEL_ACTIVITY => ("Activity", MzViewPlacement::BottomPanel),
        VIEW_PANEL_EXTENSIONS => ("Extensions", MzViewPlacement::BottomPanel),
        _ => ("Base View", MzViewPlacement::Dialog),
    };

    MzViewFactorySpec {
        plugin_id: MzStr::from_static(BASE_PLUGIN_ID),
        view_id: MzStr::from_static(view_id),
        title: MzStr::from_static(title),
        placement,
        create: create_base_view,
    }
}

extern "C" fn create_base_view(
    host: *const MzHostApi,
    request: *const MzViewRequest,
) -> *mut c_void {
    let Some(host) = (unsafe { host.as_ref() }) else {
        return std::ptr::null_mut();
    };
    let Some(request) = (unsafe { request.as_ref() }) else {
        return std::ptr::null_mut();
    };
    let Ok(view_id) = decode_str(request.view_id) else {
        return std::ptr::null_mut();
    };
    let instance_key = decode_optional_str(request.instance_key);

    let widget = match view_id.as_str() {
        VIEW_WORKSPACE_HOME => workspace_home_view(),
        VIEW_WORKSPACE_QUEUE => workspace_queue_view(),
        VIEW_WORKSPACE_SURFACES => workspace_surfaces_view(),
        VIEW_WORKSPACE_OPS => workspace_ops_view(),
        VIEW_WORKSPACE_COMMANDS => commands_view(host),
        VIEW_WORKSPACE_REGISTERED_VIEWS => registered_views_view(host),
        VIEW_WORKSPACE_PLUGINS => plugins_view(host),
        VIEW_WORKSPACE_SETTINGS => settings_view(host, instance_key.as_deref()),
        VIEW_WORKSPACE_ABOUT => about_view(host),
        VIEW_PANEL_NAVIGATOR => navigator_view(),
        VIEW_PANEL_RESOURCES => resources_view(),
        VIEW_PANEL_INSPECTOR => inspector_view(),
        VIEW_PANEL_DELIVERY => delivery_view(),
        VIEW_PANEL_ACTIVITY => activity_view(),
        VIEW_PANEL_EXTENSIONS => extensions_view(),
        _ => fallback_view(&format!("Unknown base view: {view_id}")),
    };

    unsafe {
        <gtk::Widget as IntoGlibPtr<*mut gtk::ffi::GtkWidget>>::into_glib_ptr(widget) as *mut c_void
    }
}

fn decode_str(value: MzStr) -> Result<String, ()> {
    if value.ptr.is_null() {
        return Err(());
    }
    let bytes = unsafe { std::slice::from_raw_parts(value.ptr, value.len) };
    std::str::from_utf8(bytes)
        .map(str::to_string)
        .map_err(|_| ())
}

fn decode_optional_str(value: MzStr) -> Option<String> {
    if value.ptr.is_null() || value.len == 0 {
        return None;
    }
    decode_str(value).ok()
}

fn workspace_home_view() -> gtk::Widget {
    let root = view_root();
    root.append(&hero(
        "Studio Home",
        "Maruzzella now boots into a coherent shell slice: navigation, delivery, runtime health, and extension surfaces are rendered as built-in plugin views.",
        Some(("Reference slice", "status-running")),
    ));
    root.append(&section(
        "What this shell proves",
        &[
            "A downstream app can launch into real UI instead of neutral placeholder tabs.",
            "The built-in base plugin now exercises views, menus, about sections, and settings surfaces together.",
            "The default shell is structured around product work, extension points, and runtime visibility.",
        ],
    ));
    root.append(&section(
        "Active focus areas",
        &[
            "Ship one polished built-in workflow before expanding platform scope.",
            "Keep contribution surfaces explicit and host-owned.",
            "Expose plugin state and settings through first-class shell UI.",
        ],
    ));
    root.upcast()
}

fn workspace_queue_view() -> gtk::Widget {
    let root = view_root();
    root.append(&hero(
        "Work Queue",
        "This queue encodes the current product-first roadmap sequence and keeps the platform work tied to visible value.",
        Some(("Roadmap refreshed", "status-loaded")),
    ));
    root.append(&status_list(&[
        ("1. Real shell slice", "Now active", "status-running"),
        (
            "2. Contribution surfaces",
            "Next structural milestone",
            "status-loaded",
        ),
        (
            "3. Plugin manager and settings",
            "Ready for deeper UI pass",
            "status-loaded",
        ),
        (
            "4. Plugin configuration and persistence",
            "Host plumbing exists, UI contract needs expansion",
            "status-idle",
        ),
        (
            "5. Packaging and authoring",
            "After the runtime contracts settle",
            "status-idle",
        ),
    ]));
    root.upcast()
}

fn workspace_surfaces_view() -> gtk::Widget {
    let root = view_root();
    root.append(&hero(
        "Contribution Surfaces",
        "These surfaces are the first shared contracts already visible in the shell and plugin manager flows.",
        Some(("Host-owned", "status-loaded")),
    ));
    root.append(&surface_list(&[
        (
            "maruzzella.menu.file.items",
            "Menu contributions under File",
        ),
        (
            "maruzzella.menu.help.items",
            "Menu contributions under Help",
        ),
        (
            "maruzzella.menu.view.items",
            "Menu contributions under View",
        ),
        (
            "maruzzella.about.sections",
            "Structured sections shown in the About dialog",
        ),
        (
            "maruzzella.plugins.settings_pages",
            "Plugin-owned settings entries exposed through the shared settings catalog",
        ),
    ]));
    root.upcast()
}

fn workspace_ops_view() -> gtk::Widget {
    let root = view_root();
    root.append(&hero(
        "System Ops",
        "Shell operations remain command-driven. Theme reload, About, Plugins, and the command palette are available through toolbar and menu actions.",
        Some(("Shell commands live", "status-running")),
    ));
    root.append(&section(
        "Available command surfaces",
        &[
            "shell.open_command_palette",
            "shell.reload_theme",
            "shell.browse_views",
            "shell.about",
            "shell.plugins",
        ],
    ));
    root.append(&section(
        "Notes",
        &[
            "Plugin views are mounted through the same runtime path as external dynamic plugins.",
            "The base plugin is the reference implementation for built-in shell capabilities.",
        ],
    ));
    root.upcast()
}

fn navigator_view() -> gtk::Widget {
    let root = view_root();
    root.append(&hero(
        "Workspace",
        "Primary shell areas are grouped by product work instead of anonymous host placeholders.",
        Some(("Focused", "status-running")),
    ));
    root.append(&section(
        "Areas",
        &[
            "Studio Home",
            "Work Queue",
            "Contribution Surfaces",
            "System Ops",
            "Plugins and About",
        ],
    ));
    root.upcast()
}

fn resources_view() -> gtk::Widget {
    let root = view_root();
    root.append(&hero(
        "Resources",
        "A downstream app can replace these with product-specific data sources, docs, or asset indexes.",
        Some(("Replaceable", "status-loaded")),
    ));
    root.append(&section(
        "Reference material",
        &[
            "Implementation roadmap",
            "Plugin ABI RFC",
            "README integration guide",
            "Example apps and example plugin",
        ],
    ));
    root.upcast()
}

fn inspector_view() -> gtk::Widget {
    let root = view_root();
    root.append(&hero(
        "Inspector",
        "The active built-in slice is healthy and now reflects actual shell capabilities rather than placeholder copy.",
        Some(("Healthy", "status-running")),
    ));
    root.append(&status_list(&[
        (
            "Default shell slice",
            "Base-plugin-backed",
            "status-running",
        ),
        ("Theming", "Configurable and tokenized", "status-loaded"),
        ("Plugin runtime", "Active at startup", "status-running"),
        (
            "Layout persistence",
            "Stored per persistence namespace",
            "status-loaded",
        ),
    ]));
    root.upcast()
}

fn delivery_view() -> gtk::Widget {
    let root = view_root();
    root.append(&hero(
        "Release",
        "This panel tracks the polish bar that makes the shell feel intentional to downstream app authors.",
        Some(("Current milestone", "status-loaded")),
    ));
    root.append(&section(
        "Checklist",
        &[
            "Remove placeholder-first UX from the default app",
            "Keep spacing, typography, and visual hierarchy coherent",
            "Expose plugin state in-app",
            "Document the roadmap in product-first terms",
        ],
    ));
    root.upcast()
}

fn activity_view() -> gtk::Widget {
    let root = view_root();
    root.append(&hero(
        "Activity",
        "Runtime health, theme workflows, and plugin diagnostics all have visible homes in the shell now.",
        Some(("Observed", "status-running")),
    ));
    root.append(&status_list(&[
        (
            "Theme reload",
            "Toolbar and View menu command",
            "status-loaded",
        ),
        (
            "About sections",
            "Aggregated from surface contributions",
            "status-loaded",
        ),
        (
            "Plugin diagnostics",
            "Shown in Plugins dialog",
            "status-loaded",
        ),
        (
            "Plugin logs",
            "Captured during startup and surfaced in-app",
            "status-loaded",
        ),
    ]));
    root.upcast()
}

fn extensions_view() -> gtk::Widget {
    let root = view_root();
    root.append(&hero(
        "Extensions",
        "The base plugin and example plugin already contribute commands, menus, settings surfaces, and views through the same runtime.",
        Some(("Platform proven", "status-running")),
    ));
    root.append(&section(
        "Enabled capabilities",
        &[
            "Dynamic plugin loading and dependency ordering",
            "Plugin commands dispatched from GTK actions",
            "Plugin settings surfaces aggregated through the host catalog",
            "Plugin-owned view factories mounted into shell tabs",
        ],
    ));
    root.upcast()
}

fn commands_view(host: &MzHostApi) -> gtk::Widget {
    let root = view_root();
    let commands = read_command_catalog(host).commands;
    root.append(&hero(
        "Command Palette",
        "The base plugin now owns the visible command browser while the host keeps the underlying shell capabilities.",
        Some(("Base-owned", "status-loaded")),
    ));
    if commands.is_empty() {
        root.append(&section(
            "Commands",
            &["No commands are currently registered."],
        ));
    } else {
        root.append(&summary_list(
            &commands
                .iter()
                .map(|command| format!("{}  ({})", command.title, command.command_id))
                .collect::<Vec<_>>(),
            true,
        ));
    }
    root.upcast()
}

fn registered_views_view(host: &MzHostApi) -> gtk::Widget {
    let root = view_root();
    let views = read_view_catalog(host).views;
    root.append(&hero(
        "Registered Views",
        "This page is contributed by the base plugin and rendered from host-provided view metadata.",
        Some(("Host query", "status-loaded")),
    ));
    if views.is_empty() {
        root.append(&section(
            "Views",
            &["No plugin views are currently registered."],
        ));
    } else {
        root.append(&summary_list(
            &views
                .iter()
                .map(|view| {
                    format!(
                        "{}  ({}, {})",
                        view.title,
                        view.plugin_id,
                        view.placement.label()
                    )
                })
                .collect::<Vec<_>>(),
            false,
        ));
    }
    root.upcast()
}

fn plugins_view(host: &MzHostApi) -> gtk::Widget {
    let root = view_root();
    let snapshot = read_plugin_state(host);
    let settings_catalog = read_settings_catalog(host);
    let diagnostic_catalog = read_diagnostic_catalog(host);
    root.append(&hero(
        "Plugins",
        "The default plugin manager page is now provided by the base plugin using explicit host catalogs for runtime inventory, settings, and diagnostics.",
        Some(("Base-owned", "status-running")),
    ));

    let settings_button = action_button("Open Settings", Some("preferences-system-symbolic"));
    let host_for_settings = *host;
    settings_button.connect_clicked(move |_| {
        open_host_view(
            &host_for_settings,
            BASE_PLUGIN_ID,
            VIEW_WORKSPACE_SETTINGS,
            MzViewPlacement::Workbench,
            None,
            Some("Settings"),
            &[],
        );
    });
    root.append(&settings_button);

    if !snapshot.activation_order.is_empty() {
        root.append(&section(
            "Activation Order",
            &snapshot
                .activation_order
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
        ));
    }

    if !diagnostic_catalog.diagnostics.is_empty() {
        root.append(&summary_list(
            &diagnostic_catalog
                .diagnostics
                .iter()
                .map(|diagnostic| {
                    format!(
                        "[{}] {}{}",
                        diagnostic.level,
                        diagnostic
                            .plugin_id
                            .as_deref()
                            .map(|plugin_id| format!("{plugin_id}: "))
                            .unwrap_or_default(),
                        diagnostic.message
                    )
                })
                .collect::<Vec<_>>(),
            true,
        ));
    }

    for plugin in snapshot.plugins {
        let card = GtkBox::new(Orientation::Vertical, 8);
        card.append(&hero(
            &plugin.name,
            &format!("{} ({})", plugin.version, plugin.plugin_id),
            Some(("Loaded", "status-loaded")),
        ));

        if !plugin.description.is_empty() {
            card.append(&section("Description", &[plugin.description.as_str()]));
        }

        if !plugin.dependencies.is_empty() {
            card.append(&section("Dependencies", &[]));
            card.append(&summary_list(
                &plugin
                    .dependencies
                    .iter()
                    .map(|dependency| {
                        format!(
                            "{}  [{} {}..{})",
                            dependency.plugin_id,
                            if dependency.required {
                                "required"
                            } else {
                                "optional"
                            },
                            dependency.min_version,
                            dependency.max_version_exclusive
                        )
                    })
                    .collect::<Vec<_>>(),
                true,
            ));
        }

        if !plugin.views.is_empty() {
            card.append(&section("Views", &[]));
            card.append(&summary_list(
                &plugin
                    .views
                    .iter()
                    .map(|view| format!("{}  ({})", view.title, view.view_id))
                    .collect::<Vec<_>>(),
                true,
            ));
        }

        let plugin_settings = settings_catalog
            .pages
            .iter()
            .filter(|page| page.plugin_id == plugin.plugin_id)
            .cloned()
            .collect::<Vec<_>>();
        if !plugin_settings.is_empty() {
            card.append(&section("Settings", &[]));
            card.append(&settings_page_list(&plugin_settings, Some(host)));
        }

        let plugin_diagnostics = diagnostic_catalog
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.plugin_id.as_deref() == Some(plugin.plugin_id.as_str()))
            .map(|diagnostic| format!("[{}] {}", diagnostic.level, diagnostic.message))
            .collect::<Vec<_>>();
        if !plugin_diagnostics.is_empty() {
            card.append(&section("Diagnostics", &[]));
            card.append(&summary_list(&plugin_diagnostics, false));
        }

        if !plugin.logs.is_empty() {
            card.append(&section("Logs", &[]));
            card.append(&summary_list(
                &plugin
                    .logs
                    .iter()
                    .map(|entry| format!("[{:?}] {}", entry.level, entry.message))
                    .collect::<Vec<_>>(),
                true,
            ));
        }

        root.append(&card);
        root.append(&Separator::new(Orientation::Horizontal));
    }

    root.upcast()
}

fn settings_view(host: &MzHostApi, instance_key: Option<&str>) -> gtk::Widget {
    let root = view_root();
    let snapshot = read_plugin_state(host);
    let settings_catalog = read_settings_catalog(host);
    let selected_plugin_id = instance_key.and_then(parse_plugin_instance_key);
    root.append(&hero(
        "Settings",
        "Plugin-owned settings entries are aggregated here by the base plugin. Entries can either summarize config state or open concrete plugin settings views.",
        Some(("Surface-driven", "status-running")),
    ));

    let plugin_ids = if let Some(plugin_id) = selected_plugin_id {
        vec![plugin_id.to_string()]
    } else {
        snapshot
            .plugins
            .iter()
            .map(|plugin| plugin.plugin_id.clone())
            .collect::<Vec<_>>()
    };

    let mut rendered_any = false;
    for plugin_id in plugin_ids {
        let plugin_settings = settings_catalog
            .pages
            .iter()
            .filter(|page| page.plugin_id == plugin_id)
            .cloned()
            .collect::<Vec<_>>();
        if plugin_settings.is_empty() {
            continue;
        }
        rendered_any = true;
        let plugin_title = snapshot
            .plugins
            .iter()
            .find(|plugin| plugin.plugin_id == plugin_id)
            .map(|plugin| plugin.name.clone())
            .unwrap_or_else(|| plugin_id.clone());
        root.append(&section(&plugin_title, &[plugin_id.as_str()]));
        root.append(&settings_page_list(&plugin_settings, Some(host)));
        root.append(&Separator::new(Orientation::Horizontal));
    }

    if !rendered_any {
        root.append(&section(
            "Settings",
            &["No plugin settings pages are currently registered."],
        ));
    }

    root.upcast()
}

fn about_view(host: &MzHostApi) -> gtk::Widget {
    let root = view_root();
    let sections = read_about_catalog(host).sections;
    root.append(&hero(
        "About Maruzzella",
        "The base plugin now owns the default About page and renders aggregated about sections from host contributions.",
        Some(("Base-owned", "status-loaded")),
    ));
    if sections.is_empty() {
        root.append(&section("About", &["Neutral GTK desktop shell host"]));
    } else {
        for section_data in sections {
            root.append(&section(&section_data.title, &[section_data.body.as_str()]));
        }
    }
    root.upcast()
}

fn fallback_view(message: &str) -> gtk::Widget {
    let root = view_root();
    root.append(&hero(
        "Base View Error",
        message,
        Some(("Error", "status-idle")),
    ));
    root.upcast()
}

fn view_root() -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 12);
    root.add_css_class("plugin-detail-root");
    root.set_margin_top(18);
    root.set_margin_bottom(18);
    root.set_margin_start(18);
    root.set_margin_end(18);
    root
}

fn hero(title_text: &str, body_text: &str, badge: Option<(&str, &str)>) -> GtkBox {
    let box_ = GtkBox::new(Orientation::Vertical, 8);
    box_.add_css_class("plugin-hero");

    if let Some((badge_text, badge_class)) = badge {
        let badge = Label::new(Some(badge_text));
        badge.set_halign(Align::Start);
        badge.add_css_class("status-badge");
        badge.add_css_class(badge_class);
        box_.append(&badge);
    }

    let title = Label::new(Some(title_text));
    title.set_xalign(0.0);
    title.set_wrap(true);
    title.add_css_class("plugin-detail-name");
    box_.append(&title);

    let body = Label::new(Some(body_text));
    body.set_xalign(0.0);
    body.set_wrap(true);
    body.add_css_class("plugin-detail-description");
    box_.append(&body);

    box_
}

fn section(title_text: &str, rows: &[&str]) -> GtkBox {
    let box_ = GtkBox::new(Orientation::Vertical, 8);
    let title = Label::new(Some(title_text));
    title.set_xalign(0.0);
    title.add_css_class("section-title");
    box_.append(&title);

    for row in rows {
        let label = Label::new(Some(row));
        label.set_xalign(0.0);
        label.set_wrap(true);
        box_.append(&label);
    }

    box_
}

fn status_list(rows: &[(&str, &str, &str)]) -> GtkBox {
    let box_ = GtkBox::new(Orientation::Vertical, 8);
    for (title_text, summary, badge_class) in rows {
        let card = GtkBox::new(Orientation::Vertical, 6);

        let top = GtkBox::new(Orientation::Horizontal, 8);
        let title = Label::new(Some(title_text));
        title.set_xalign(0.0);
        title.set_hexpand(true);
        title.add_css_class("section-title");
        top.append(&title);

        let badge = Label::new(Some(summary));
        badge.set_halign(Align::End);
        badge.add_css_class("status-badge");
        badge.add_css_class(badge_class);
        top.append(&badge);

        card.append(&top);

        let summary_label = Label::new(Some(summary));
        summary_label.set_xalign(0.0);
        summary_label.set_wrap(true);
        summary_label.add_css_class("muted");
        card.append(&summary_label);
        card.append(&Separator::new(Orientation::Horizontal));

        box_.append(&card);
    }
    box_
}

fn surface_list(rows: &[(&str, &str)]) -> GtkBox {
    let box_ = GtkBox::new(Orientation::Vertical, 10);
    for (surface_id, summary) in rows {
        let surface = GtkBox::new(Orientation::Vertical, 4);
        let id = Label::new(Some(surface_id));
        id.set_xalign(0.0);
        id.add_css_class("mono");
        surface.append(&id);

        let summary_label = Label::new(Some(summary));
        summary_label.set_xalign(0.0);
        summary_label.set_wrap(true);
        summary_label.add_css_class("muted");
        surface.append(&summary_label);
        surface.append(&Separator::new(Orientation::Horizontal));
        box_.append(&surface);
    }
    box_
}

fn summary_list(rows: &[String], mono: bool) -> GtkBox {
    let box_ = GtkBox::new(Orientation::Vertical, 8);
    for row in rows {
        let label = Label::new(Some(row));
        label.set_xalign(0.0);
        label.set_wrap(true);
        if mono {
            label.add_css_class("mono");
        }
        box_.append(&label);
        box_.append(&Separator::new(Orientation::Horizontal));
    }
    box_
}

fn settings_page_list(
    pages: &[maruzzella_api::MzSettingsPageSummary],
    host: Option<&MzHostApi>,
) -> GtkBox {
    let box_ = GtkBox::new(Orientation::Vertical, 10);
    for page_summary in pages {
        let row = GtkBox::new(Orientation::Vertical, 6);
        let title = Label::new(Some(&format!(
            "{}  [{}]",
            page_summary.page.title,
            page_summary.page.category.label()
        )));
        title.set_xalign(0.0);
        title.add_css_class("section-title");
        row.append(&title);

        if !page_summary.page.summary.is_empty() {
            let summary = Label::new(Some(&page_summary.page.summary));
            summary.set_xalign(0.0);
            summary.set_wrap(true);
            summary.add_css_class("muted");
            row.append(&summary);
        }

        if let Some(config_state) = &page_summary.config_state {
            let detail = Label::new(Some(&format!(
                "Config Status: {}. {}",
                config_state.state.label(),
                config_state.message
            )));
            detail.set_xalign(0.0);
            detail.set_wrap(true);
            detail.add_css_class("muted");
            row.append(&detail);

            if let Some(migration_hook) = &config_state.migration_hook {
                let hook = Label::new(Some(&format!(
                    "Reserved Migration Hook: {}",
                    migration_hook
                )));
                hook.set_xalign(0.0);
                hook.set_wrap(true);
                hook.add_css_class("muted");
                row.append(&hook);
            }
        }

        if let (Some(host), Some(view_id)) = (host, page_summary.page.view_id.as_deref()) {
            let button = action_button("Open Settings Page", Some("go-next-symbolic"));
            let host_copy = *host;
            let plugin_id = page_summary.plugin_id.clone();
            let view_id = view_id.to_string();
            let instance_key = page_summary.page.instance_key.clone();
            let requested_title = page_summary.page.requested_title.clone();
            let placement = page_summary
                .page
                .placement
                .unwrap_or(MzViewPlacement::Workbench);
            button.connect_clicked(move |_| {
                open_host_view(
                    &host_copy,
                    &plugin_id,
                    &view_id,
                    placement,
                    instance_key.as_deref(),
                    requested_title.as_deref(),
                    &[],
                );
            });
            row.append(&button);
        }

        row.append(&Separator::new(Orientation::Horizontal));
        box_.append(&row);
    }
    box_
}

fn action_button(label_text: &str, icon_name: Option<&str>) -> Button {
    let button = Button::with_label(label_text);
    button.set_halign(Align::Start);
    if let Some(icon_name) = icon_name {
        button.set_icon_name(icon_name);
    }
    button
}

fn open_host_view(
    host: &MzHostApi,
    plugin_id: &str,
    view_id: &str,
    placement: MzViewPlacement,
    instance_key: Option<&str>,
    requested_title: Option<&str>,
    payload: &[u8],
) {
    let Some(open) = host.open_view else {
        return;
    };
    let request = MzOpenViewRequest {
        plugin_id: str_to_mzstr(plugin_id),
        view_id: str_to_mzstr(view_id),
        placement,
        instance_key: instance_key.map(str_to_mzstr).unwrap_or_else(MzStr::empty),
        requested_title: requested_title
            .map(str_to_mzstr)
            .unwrap_or_else(MzStr::empty),
        payload: MzBytes {
            ptr: payload.as_ptr(),
            len: payload.len(),
        },
    };
    let _ = open(&request);
}

fn str_to_mzstr(value: &str) -> MzStr {
    MzStr {
        ptr: value.as_ptr(),
        len: value.len(),
    }
}

fn parse_plugin_instance_key(value: &str) -> Option<&str> {
    value.strip_prefix("plugin:")
}

fn toolbar_item_payload(
    item_id: &'static str,
    icon_name: Option<&'static str>,
    label: Option<&'static str>,
    command_id: &'static str,
    secondary: bool,
) -> Vec<u8> {
    MzToolbarItem::new(
        item_id,
        icon_name.map(str::to_string),
        label.map(str::to_string),
        command_id,
        secondary,
    )
    .to_bytes()
    .expect("toolbar item should serialize")
}

fn startup_tab_payload(
    group_id: &'static str,
    tab_id: &'static str,
    title: &'static str,
    view_id: &'static str,
    closable: bool,
    active: bool,
) -> Vec<u8> {
    let mut tab = MzStartupTab::new(group_id, tab_id, title, view_id);
    tab.closable = closable;
    tab.active = active;
    tab.to_bytes().expect("startup tab should serialize")
}

fn read_command_catalog(host: &MzHostApi) -> MzCommandCatalog {
    let Some(read) = host.read_command_catalog else {
        return MzCommandCatalog::default();
    };
    decode_snapshot(read())
}

fn read_view_catalog(host: &MzHostApi) -> MzViewCatalog {
    let Some(read) = host.read_view_catalog else {
        return MzViewCatalog::default();
    };
    decode_snapshot(read())
}

fn read_plugin_state(host: &MzHostApi) -> MzPluginSnapshot {
    let Some(read) = host.read_plugin_state else {
        return MzPluginSnapshot {
            activation_order: Vec::new(),
            plugins: Vec::new(),
        };
    };
    decode_snapshot(read())
}

fn decode_snapshot<T: serde::de::DeserializeOwned + Default>(bytes: MzBytes) -> T {
    if bytes.ptr.is_null() || bytes.len == 0 {
        return T::default();
    }
    serde_json::from_slice(unsafe { std::slice::from_raw_parts(bytes.ptr, bytes.len) })
        .unwrap_or_default()
}

fn read_settings_catalog(host: &MzHostApi) -> MzSettingsCatalog {
    let Some(read) = host.read_settings_catalog else {
        return MzSettingsCatalog::default();
    };
    let bytes = read();
    MzSettingsCatalog::from_bytes(unsafe { std::slice::from_raw_parts(bytes.ptr, bytes.len) })
        .unwrap_or_default()
}

fn read_diagnostic_catalog(host: &MzHostApi) -> MzDiagnosticCatalog {
    let Some(read) = host.read_diagnostic_catalog else {
        return MzDiagnosticCatalog::default();
    };
    let bytes = read();
    MzDiagnosticCatalog::from_bytes(unsafe { std::slice::from_raw_parts(bytes.ptr, bytes.len) })
        .unwrap_or_default()
}

fn read_about_catalog(host: &MzHostApi) -> MzAboutCatalog {
    let Some(read) = host.read_about_catalog else {
        return MzAboutCatalog::default();
    };
    decode_snapshot(read())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::PluginRuntime;

    #[test]
    fn base_plugin_registers_views_and_surfaces() {
        let runtime = PluginRuntime::activate(vec![load()]).expect("base plugin should activate");

        assert!(runtime
            .commands()
            .iter()
            .any(|command| command.command_id == "shell.about"));
        assert!(runtime
            .commands()
            .iter()
            .any(|command| command.command_id == "shell.plugins"));
        assert!(runtime
            .commands()
            .iter()
            .any(|command| command.command_id == "shell.settings"));
        assert!(runtime
            .surface_contributions()
            .iter()
            .any(|surface| surface.surface == Some(MzContributionSurface::AboutSections)));
        assert!(runtime
            .surface_contributions()
            .iter()
            .any(|surface| surface.surface == Some(MzContributionSurface::PluginSettingsPages)));
        assert!(runtime
            .view_factories()
            .iter()
            .any(|factory| factory.view_id == VIEW_WORKSPACE_HOME));
        assert!(runtime
            .view_factories()
            .iter()
            .any(|factory| factory.view_id == VIEW_WORKSPACE_SETTINGS));
        assert!(runtime
            .view_factories()
            .iter()
            .any(|factory| factory.view_id == VIEW_PANEL_EXTENSIONS));
    }
}
