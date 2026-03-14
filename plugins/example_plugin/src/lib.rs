use maruzzella_sdk::{
    export_plugin, CommandSpec, HostApi, MenuItemSpec, MzStatusCode, Plugin, PluginDependency,
    PluginDescriptor, SurfaceContributionSpec, Version,
};

struct ExamplePlugin;

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

        host.register_command(CommandSpec::new(
            "com.example.hello",
            "example.hello.show",
            "Show Example Plugin",
        ))?;

        host.register_menu_item(MenuItemSpec::new(
            "com.example.hello",
            "example-plugin",
            "maruzzella.menu.file.items",
            "Example Plugin",
            "example.hello.show",
        ))?;

        host.register_surface_contribution(SurfaceContributionSpec::new(
            "com.example.hello",
            "maruzzella.about.sections",
            "com.example.hello.about",
            br#"{"title":"Example Plugin","body":"Loaded from a dynamic library"}"#,
        ))?;

        Ok(())
    }
}

export_plugin!(ExamplePlugin);
