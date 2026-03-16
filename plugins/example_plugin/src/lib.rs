use gtk::glib::translate::IntoGlibPtr;
use gtk::prelude::*;
use gtk::{Box as GtkBox, Button, Label, Orientation};
use maruzzella_sdk::{
    export_plugin, CommandSpec, HostApi, MenuItemSpec, MzStatusCode, Plugin, PluginDependency,
    PluginDescriptor, SurfaceContributionSpec, Version, ViewFactorySpec,
};
use serde::{Deserialize, Serialize};

struct ExamplePlugin;

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

        let mut config = host
            .read_config()
            .ok()
            .and_then(|bytes| serde_json::from_slice::<ExamplePluginConfig>(&bytes).ok())
            .unwrap_or_default();
        config.launches += 1;
        let config_bytes = serde_json::to_vec(&config).map_err(|_| MzStatusCode::InternalError)?;
        host.write_config(&config_bytes)?;

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
            "maruzzella.menu.file.items",
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
            "com.example.hello.settings.general",
            "Example Plugin Settings",
            format!(
                "This plugin has been registered {} time(s) for the current persistence namespace.",
                config.launches
            ),
        ))?;

        host.register_view_factory(ViewFactorySpec::new(
            "com.example.hello",
            "com.example.hello.welcome",
            create_example_view,
        ))?;

        Ok(())
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
