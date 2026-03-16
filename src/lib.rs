use std::path::{Path, PathBuf};

pub mod app;
pub mod base_plugin;
pub mod commands;
pub mod layout;
mod plugin_tabs;
pub mod plugins;
pub mod product;
pub mod shell;
pub mod spec;
pub mod theme;

use gtk::prelude::*;
use gtk::Application;

pub use plugins::{
    diagnostic_for_load_error, diagnostic_for_runtime_error, load_plugin, load_static_plugin,
    resolve_load_order, LoadedPlugin, PluginDependencySpec, PluginDescriptor, PluginDiagnostic,
    PluginDiagnosticLevel, PluginHost, PluginLoadError, PluginLogEntry, PluginResolveError,
    PluginRuntime, PluginRuntimeError, RegisteredCommand, RegisteredMenuItem,
    RegisteredSurfaceContribution, RegisteredViewFactory, Version as PluginVersion,
};
pub use product::{default_product_spec, BrandingSpec, LayoutContribution, ProductSpec};
pub use spec::{
    plugin_tab, plugin_tab_with_instance, text_tab, BottomPanelLayout, CommandSpec, MenuItemSpec,
    MenuRootSpec, PanelContentKind, ShellSpec, SplitAxis, TabGroupSpec, TabSpec, ToolbarItemSpec,
    WorkbenchNodeSpec,
};
pub use theme::{ThemeDensity, ThemePalette, ThemeSpec, ThemeStylesheet, ThemeTypography};

#[derive(Clone, Debug)]
pub struct MaruzzellaConfig {
    pub application_id: String,
    pub persistence_id: String,
    pub product: ProductSpec,
    pub theme: ThemeSpec,
    pub plugin_paths: Vec<PathBuf>,
    pub builtin_plugins: Vec<fn() -> Result<plugins::LoadedPlugin, plugins::PluginLoadError>>,
}

impl Default for MaruzzellaConfig {
    fn default() -> Self {
        Self::new("com.lelloman.maruzzella")
    }
}

impl MaruzzellaConfig {
    pub fn new(application_id: &str) -> Self {
        Self {
            application_id: application_id.to_string(),
            persistence_id: "maruzzella".to_string(),
            product: default_product_spec(),
            theme: ThemeSpec::default(),
            plugin_paths: Vec::new(),
            builtin_plugins: Vec::new(),
        }
    }

    pub fn with_persistence_id(mut self, persistence_id: &str) -> Self {
        self.persistence_id = persistence_id.to_string();
        self
    }

    pub fn with_product(mut self, product: ProductSpec) -> Self {
        self.product = product;
        self
    }

    pub fn with_theme(mut self, theme: ThemeSpec) -> Self {
        self.theme = theme;
        self
    }

    pub fn with_plugin_path(mut self, path: impl AsRef<Path>) -> Self {
        self.plugin_paths.push(path.as_ref().to_path_buf());
        self
    }

    pub fn with_plugin_paths<I, P>(mut self, paths: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.plugin_paths = paths
            .into_iter()
            .map(|path| path.as_ref().to_path_buf())
            .collect();
        self
    }

    pub fn with_builtin_plugin(
        mut self,
        loader: fn() -> Result<plugins::LoadedPlugin, plugins::PluginLoadError>,
    ) -> Self {
        self.builtin_plugins.push(loader);
        self
    }

    pub fn with_builtin_plugins<I>(mut self, loaders: I) -> Self
    where
        I: IntoIterator<Item = fn() -> Result<plugins::LoadedPlugin, plugins::PluginLoadError>>,
    {
        self.builtin_plugins = loaders.into_iter().collect();
        self
    }
}

pub fn build_application(config: MaruzzellaConfig) -> Application {
    let config_for_activate = config.clone();
    build_application_with_activate(&config.application_id, move |application| {
        app::build(application, &config_for_activate);
    })
}

pub fn build_application_with_activate<F>(application_id: &str, activate: F) -> Application
where
    F: Fn(&Application) + 'static,
{
    let application = Application::builder()
        .application_id(application_id)
        .build();

    application.connect_activate(activate);

    application
}

pub fn run_default() {
    run(MaruzzellaConfig::default());
}

pub fn run(config: MaruzzellaConfig) {
    build_application(config).run();
}
