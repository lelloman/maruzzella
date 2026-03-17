use gtk::glib::translate::IntoGlibPtr;
use gtk::prelude::*;
use gtk::{Align, Box as GtkBox, Button, Label, Orientation};
use maruzzella_sdk::{
    export_plugin, CommandSpec, HostApi, MenuItemSpec, MzConfigContract, MzMenuSurface,
    MzSettingsCategory, MzStatusCode, MzViewPlacement, Plugin, PluginDependency,
    PluginDescriptor, SurfaceContributionSpec, Version, ViewFactorySpec,
};
use serde::{Deserialize, Serialize};

struct ExamplePlugin;

const CONFIG_SCHEMA_VERSION: u32 = 1;
const CONFIG_MIGRATION_HOOK: &str = "com.example.hello.config.v1";

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct ExamplePluginConfig {
    launches: u32,
}

extern "C" fn show_example_plugin(
    payload: maruzzella_sdk::ffi::MzBytes,
) -> maruzzella_sdk::ffi::MzStatus {
    let _ = payload;
    maruzzella_sdk::ffi::MzStatus::OK
}

impl Plugin for ExamplePlugin {
    fn descriptor() -> PluginDescriptor {
        static DEPENDENCIES: &[PluginDependency] = &[PluginDependency::required(
            "maruzzella.base",
            Version::new(1, 0, 0),
            Version::new(2, 0, 0),
        )];

        PluginDescriptor::new(
            "com.example.hello",
            "Example Hello Plugin",
            Version::new(0, 1, 0),
        )
        .with_description("Reference dynamic plugin built against maruzzella_sdk")
        .with_dependencies(DEPENDENCIES)
    }

    fn register(host: &HostApi<'_>) -> Result<(), MzStatusCode> {
        host.log(
            maruzzella_sdk::ffi::MzLogLevel::Info,
            "Registering example Maruzzella plugin",
        );

        let mut config = host.read_json_config::<ExamplePluginConfig>()?;
        config.launches += 1;
        host.write_json_config(&config, Some(CONFIG_SCHEMA_VERSION))?;

        host.register_command(
            CommandSpec::new(
                "com.example.hello",
                "example.hello.show",
                "Show Example Plugin",
            )
            .with_handler(show_example_plugin),
        )?;

        host.register_menu_item(MenuItemSpec::new(
            "com.example.hello",
            "example-plugin",
            MzMenuSurface::FileItems,
            "Example Plugin",
            "example.hello.show",
        ))?;

        host.register_surface_contribution(SurfaceContributionSpec::about_section(
            "com.example.hello",
            "com.example.hello.about",
            "Example Plugin",
            "Loaded from a dynamic library",
        ))?;

        host.register_surface_contribution(SurfaceContributionSpec::settings_page(
            "com.example.hello",
            "com.example.hello.settings.summary",
            "general",
            "Example Plugin Settings",
            format!(
                "This plugin has been registered {} time(s) for the current persistence namespace.",
                config.launches
            ),
            MzSettingsCategory::Integrations,
        ))?;

        host.register_surface_contribution(SurfaceContributionSpec::settings_page_with_view(
            "com.example.hello",
            "com.example.hello.settings.editor",
            maruzzella_sdk::ffi::MzSettingsPage::new(
                "editor",
                "Launch Counter",
                "Open a plugin-owned settings view backed by persisted config.",
                MzSettingsCategory::Integrations,
            )
            .with_config(
                MzConfigContract::new(CONFIG_SCHEMA_VERSION)
                    .with_migration_hook(CONFIG_MIGRATION_HOOK),
            )
            .with_view("com.example.hello.settings", MzViewPlacement::Workbench)
            .with_instance_key("plugin:com.example.hello")
            .with_requested_title("Example Plugin Settings"),
        ))?;

        host.register_view_factory(ViewFactorySpec::new(
            "com.example.hello",
            "com.example.hello.welcome",
            "Example Welcome View",
            MzViewPlacement::Workbench,
            create_example_view,
        ))?;

        host.register_view_factory(ViewFactorySpec::new(
            "com.example.hello",
            "com.example.hello.settings",
            "Example Plugin Settings",
            MzViewPlacement::Workbench,
            create_example_settings_view,
        ))?;

        Ok(())
    }
}

fn load_config(host: &maruzzella_sdk::ffi::MzHostApi) -> ExamplePluginConfig {
    let Some(read) = host.read_config_record else {
        return ExamplePluginConfig::default();
    };
    let bytes = read();
    if bytes.ptr.is_null() || bytes.len == 0 {
        return ExamplePluginConfig::default();
    }
    let record = maruzzella_sdk::MzConfigRecord::from_bytes(unsafe {
        std::slice::from_raw_parts(bytes.ptr, bytes.len)
    })
    .unwrap_or_default();
    serde_json::from_slice(&record.payload)
        .unwrap_or_default()
}

fn save_config(
    host: &maruzzella_sdk::ffi::MzHostApi,
    config: &ExamplePluginConfig,
) -> Result<(), MzStatusCode> {
    let Some(write) = host.write_config_record else {
        return Err(MzStatusCode::NotFound);
    };
    let payload = maruzzella_sdk::MzConfigRecord::new(
        serde_json::to_vec(config).map_err(|_| MzStatusCode::InternalError)?,
    )
    .with_schema_version(CONFIG_SCHEMA_VERSION)
    .to_bytes()
    .map_err(|_| MzStatusCode::InternalError)?;
    let status = write(maruzzella_sdk::ffi::MzBytes {
        ptr: payload.as_ptr(),
        len: payload.len(),
    });
    if status.is_ok() {
        Ok(())
    } else {
        Err(status.code)
    }
}

extern "C" fn create_example_settings_view(
    host: *const maruzzella_sdk::ffi::MzHostApi,
    _request: *const maruzzella_sdk::ffi::MzViewRequest,
) -> *mut std::ffi::c_void {
    let Some(host) = (unsafe { host.as_ref() }) else {
        return std::ptr::null_mut();
    };
    if !gtk::is_initialized_main_thread() && gtk::init().is_err() {
        return std::ptr::null_mut();
    }

    let root = GtkBox::new(Orientation::Vertical, 12);
    root.set_margin_top(18);
    root.set_margin_bottom(18);
    root.set_margin_start(18);
    root.set_margin_end(18);

    let title = Label::new(Some("Example Plugin Settings"));
    title.set_xalign(0.0);
    title.add_css_class("title-3");

    let body = Label::new(Some(
        "This settings view is contributed by the plugin itself and persists state through the Maruzzella host config API.",
    ));
    body.set_xalign(0.0);
    body.set_wrap(true);

    let launches = Label::new(None);
    launches.set_xalign(0.0);
    launches.add_css_class("monospace");

    let refresh_launches = {
        let launches = launches.clone();
        let host_copy = *host;
        move || {
            let config = load_config(&host_copy);
            launches.set_label(&format!("launches = {}", config.launches));
        }
    };
    refresh_launches();

    let increment = Button::with_label("Increment");
    increment.set_halign(Align::Start);
    {
        let host_copy = *host;
        let refresh = refresh_launches.clone();
        increment.connect_clicked(move |_| {
            let mut config = load_config(&host_copy);
            config.launches += 1;
            let _ = save_config(&host_copy, &config);
            refresh();
        });
    }

    let reset = Button::with_label("Reset Counter");
    reset.set_halign(Align::Start);
    {
        let host_copy = *host;
        let refresh = refresh_launches.clone();
        reset.connect_clicked(move |_| {
            let config = ExamplePluginConfig::default();
            let _ = save_config(&host_copy, &config);
            refresh();
        });
    }

    root.append(&title);
    root.append(&body);
    root.append(&launches);
    root.append(&increment);
    root.append(&reset);

    unsafe {
        <gtk::Widget as IntoGlibPtr<*mut gtk::ffi::GtkWidget>>::into_glib_ptr(root.upcast())
            as *mut std::ffi::c_void
    }
}

extern "C" fn create_example_view(
    _host: *const maruzzella_sdk::ffi::MzHostApi,
    _request: *const maruzzella_sdk::ffi::MzViewRequest,
) -> *mut std::ffi::c_void {
    if !gtk::is_initialized_main_thread() && gtk::init().is_err() {
        return std::ptr::null_mut();
    }

    let root = GtkBox::new(Orientation::Vertical, 12);
    root.set_margin_top(18);
    root.set_margin_bottom(18);
    root.set_margin_start(18);
    root.set_margin_end(18);

    let title = Label::new(Some("Example plugin view"));
    title.set_xalign(0.0);
    title.add_css_class("title-3");

    let body = Label::new(Some(
        "This widget was created inside plugins/example_plugin and mounted into a Maruzzella tab.",
    ));
    body.set_xalign(0.0);
    body.set_wrap(true);

    let button = Button::with_label("Run Example Command");
    button.set_halign(gtk::Align::Start);
    button.connect_clicked(|_| {
        let _ = show_example_plugin(maruzzella_sdk::ffi::MzBytes::empty());
    });

    root.append(&title);
    root.append(&body);
    root.append(&button);

    unsafe {
        <gtk::Widget as IntoGlibPtr<*mut gtk::ffi::GtkWidget>>::into_glib_ptr(root.upcast())
            as *mut std::ffi::c_void
    }
}

export_plugin!(ExamplePlugin);
