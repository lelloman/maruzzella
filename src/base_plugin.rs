use std::path::PathBuf;

use maruzzella_api::{
    MzBytes, MzCommandSpec, MzHostApi, MzLogLevel, MzMenuItemSpec, MzPluginDescriptorView,
    MzPluginVTable, MzStatus, MzStr, MzSurfaceContribution, MzVersion, MZ_ABI_VERSION_V1,
};

use crate::plugins::{LoadedPlugin, PluginDescriptor, Version};

const BASE_PLUGIN_ID: &str = "maruzzella.base";

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
            parent_id: MzStr::from_static("maruzzella.menu.file.items"),
            title: MzStr::from_static("Plugins"),
            command_id: MzStr::from_static("shell.plugins"),
        },
        MzMenuItemSpec {
            plugin_id: MzStr::from_static(BASE_PLUGIN_ID),
            menu_id: MzStr::from_static("about"),
            parent_id: MzStr::from_static("maruzzella.menu.help.items"),
            title: MzStr::from_static("About Maruzzella"),
            command_id: MzStr::from_static("shell.about"),
        },
    ];

    let about_payload = maruzzella_api::MzAboutSection::new(
        "Maruzzella",
        "Core shell services provided by the built-in base plugin.",
    )
    .to_bytes()
    .expect("built-in about section should serialize");
    let about = MzSurfaceContribution {
        plugin_id: MzStr::from_static(BASE_PLUGIN_ID),
        surface_id: MzStr::from_static("maruzzella.about.sections"),
        contribution_id: MzStr::from_static("maruzzella.base.about"),
        payload: MzBytes {
            ptr: about_payload.as_ptr(),
            len: about_payload.len(),
        },
    };

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

    host.register_surface_contribution
        .expect("surface registrar")(&about)
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
