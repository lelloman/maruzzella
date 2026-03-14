use std::path::{Path, PathBuf};

pub mod app;
pub mod commands;
pub mod layout;
pub mod plugins;
pub mod product;
pub mod shell;
pub mod spec;
pub mod theme;

use gtk::prelude::*;
use gtk::Application;

pub use product::{default_product_spec, BrandingSpec, LayoutContribution, ProductSpec};
pub use plugins::{
    load_plugin, resolve_load_order, LoadedPlugin, PluginDependencySpec, PluginDescriptor,
    PluginLoadError, PluginLogEntry, PluginResolveError, PluginRuntime, PluginRuntimeError,
    RegisteredCommand, RegisteredMenuItem, RegisteredSurfaceContribution,
    RegisteredViewFactory, Version as PluginVersion,
};
pub use spec::{
    text_tab, CommandSpec, MenuItemSpec, MenuRootSpec, PanelContentKind, ShellSpec, SplitAxis,
    TabGroupSpec, TabSpec, ToolbarItemSpec, WorkbenchNodeSpec,
};

#[derive(Clone, Debug)]
pub struct MaruzzellaConfig {
    pub application_id: String,
    pub persistence_id: String,
    pub product: ProductSpec,
    pub plugin_paths: Vec<PathBuf>,
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
            plugin_paths: Vec::new(),
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
