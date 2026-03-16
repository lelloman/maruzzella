use std::ffi::c_void;
use std::path::PathBuf;

use gtk::glib::translate::IntoGlibPtr;
use gtk::prelude::*;
use gtk::{Align, Box as GtkBox, Label, Orientation, Separator};
use maruzzella_api::{
    MzAboutSection, MzBytes, MzCommandSpec, MzContributionSurface, MzHostApi, MzLogLevel,
    MzMenuItemSpec, MzMenuSurface, MzPluginDescriptorView, MzPluginVTable, MzSettingsPage,
    MzStatus, MzStr, MzSurfaceContribution, MzVersion, MzViewFactorySpec, MzViewRequest,
    MZ_ABI_VERSION_V1,
};

use crate::plugins::{LoadedPlugin, PluginDescriptor, Version};

const BASE_PLUGIN_ID: &str = "maruzzella.base";

const VIEW_WORKSPACE_HOME: &str = "maruzzella.base.workspace.home";
const VIEW_WORKSPACE_QUEUE: &str = "maruzzella.base.workspace.queue";
const VIEW_WORKSPACE_SURFACES: &str = "maruzzella.base.workspace.surfaces";
const VIEW_WORKSPACE_OPS: &str = "maruzzella.base.workspace.ops";
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
    ];

    let menu_items = [
        MzMenuItemSpec {
            plugin_id: MzStr::from_static(BASE_PLUGIN_ID),
            menu_id: MzStr::from_static("plugins"),
            parent_id: MzStr::from_static(MzMenuSurface::FileItems.as_str()),
            title: MzStr::from_static("Plugins"),
            command_id: MzStr::from_static("shell.plugins"),
        },
        MzMenuItemSpec {
            plugin_id: MzStr::from_static(BASE_PLUGIN_ID),
            menu_id: MzStr::from_static("about"),
            parent_id: MzStr::from_static(MzMenuSurface::HelpItems.as_str()),
            title: MzStr::from_static("About Maruzzella"),
            command_id: MzStr::from_static("shell.about"),
        },
    ];

    let about_payload = MzAboutSection::new(
        "Maruzzella",
        "Core shell services and the default workspace slice are provided by the built-in base plugin.",
    )
    .to_bytes()
    .expect("built-in about section should serialize");
    let settings_payload = MzSettingsPage::new(
        "Workspace Defaults",
        "Default shell areas are now base-plugin-backed views rather than placeholder ProductSpec tabs.",
    )
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
    let view_factories = [
        view_factory(VIEW_WORKSPACE_HOME),
        view_factory(VIEW_WORKSPACE_QUEUE),
        view_factory(VIEW_WORKSPACE_SURFACES),
        view_factory(VIEW_WORKSPACE_OPS),
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
    MzViewFactorySpec {
        plugin_id: MzStr::from_static(BASE_PLUGIN_ID),
        view_id: MzStr::from_static(view_id),
        create: create_base_view,
    }
}

extern "C" fn create_base_view(
    _host: *const MzHostApi,
    request: *const MzViewRequest,
) -> *mut c_void {
    let Some(request) = (unsafe { request.as_ref() }) else {
        return std::ptr::null_mut();
    };
    let Ok(view_id) = decode_str(request.view_id) else {
        return std::ptr::null_mut();
    };

    let widget = match view_id.as_str() {
        VIEW_WORKSPACE_HOME => workspace_home_view(),
        VIEW_WORKSPACE_QUEUE => workspace_queue_view(),
        VIEW_WORKSPACE_SURFACES => workspace_surfaces_view(),
        VIEW_WORKSPACE_OPS => workspace_ops_view(),
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
            "Plugin-owned settings summaries shown in the Plugins dialog",
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
        "The base plugin and example plugin already contribute commands, menus, settings summaries, and views through the same runtime.",
        Some(("Platform proven", "status-running")),
    ));
    root.append(&section(
        "Enabled capabilities",
        &[
            "Dynamic plugin loading and dependency ordering",
            "Plugin commands dispatched from GTK actions",
            "Plugin settings summaries in the Plugins dialog",
            "Plugin-owned view factories mounted into shell tabs",
        ],
    ));
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
            .any(|factory| factory.view_id == VIEW_PANEL_EXTENSIONS));
    }
}
