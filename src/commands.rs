use std::collections::HashMap;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{ApplicationWindow, Box as GtkBox, Dialog, Label, Orientation, ResponseType, Separator};
use maruzzella_api::{MzAboutSection, MzSettingsPage};

use crate::plugins::PluginHost;
use crate::spec::{CommandSpec, ShellSpec};
use crate::theme;

type CommandHandler = Rc<dyn Fn()>;

#[derive(Clone, Default)]
pub struct CommandRegistry {
    handlers: HashMap<String, CommandHandler>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<F>(&mut self, command_id: &str, handler: F)
    where
        F: Fn() + 'static,
    {
        self.handlers.insert(command_id.to_string(), Rc::new(handler));
    }

    pub fn handler_for(&self, command_id: &str) -> Option<CommandHandler> {
        self.handlers.get(command_id).cloned()
    }
}

pub fn shell_registry(
    window: &ApplicationWindow,
    spec: &ShellSpec,
    plugin_host: Option<Rc<PluginHost>>,
) -> CommandRegistry {
    let mut registry = CommandRegistry::new();

    registry.register("shell.reload_theme", move || {
        theme::reload();
    });

    let about_window = window.clone();
    let app_title = spec.title.clone();
    let host_for_about = plugin_host.clone();
    registry.register("shell.about", move || {
        present_about_dialog(&about_window, &app_title, host_for_about.as_deref());
    });

    let palette_window = window.clone();
    let commands = spec.commands.clone();
    registry.register("shell.open_command_palette", move || {
        present_command_palette(&palette_window, &commands);
    });

    let plugins_window = window.clone();
    let host_for_plugins = plugin_host.clone();
    registry.register("shell.plugins", move || {
        present_plugins_dialog(&plugins_window, host_for_plugins.as_deref());
    });

    if let Some(plugin_host) = plugin_host {
        let Some(plugin_runtime) = plugin_host.runtime().cloned() else {
            return registry;
        };
        for command in &spec.commands {
            if registry.handler_for(&command.id).is_some() {
                continue;
            }
            let command_id = command.id.clone();
            let runtime = plugin_runtime.clone();
            registry.register(&command_id.clone(), move || {
                if let Err(status) = runtime.dispatch_command(&command_id, &[]) {
                    eprintln!("plugin command failed: {command_id} ({status:?})");
                }
            });
        }
    }

    registry
}

fn present_command_palette(window: &ApplicationWindow, commands: &[CommandSpec]) {
    let dialog = Dialog::builder()
        .transient_for(window)
        .modal(true)
        .title("Command Palette")
        .default_width(520)
        .default_height(360)
        .build();
    dialog.add_button("Close", ResponseType::Close);

    let body = dialog.content_area();
    body.set_spacing(12);

    let layout = GtkBox::new(Orientation::Vertical, 10);
    let summary = Label::new(Some("Registered shell commands"));
    summary.set_xalign(0.0);
    summary.add_css_class("section-title");
    layout.append(&summary);

    for command in commands {
        let label = Label::new(Some(&format!("{}  ({})", command.title, command.id)));
        label.set_xalign(0.0);
        label.add_css_class("mono");
        layout.append(&label);
    }

    body.append(&layout);
    dialog.connect_response(|dialog, _| {
        dialog.close();
    });
    dialog.present();
}

fn present_plugins_dialog(window: &ApplicationWindow, host: Option<&PluginHost>) {
    let dialog = Dialog::builder()
        .transient_for(window)
        .modal(true)
        .title("Plugins")
        .default_width(560)
        .default_height(420)
        .build();
    dialog.add_button("Close", ResponseType::Close);

    let body = dialog.content_area();
    body.set_spacing(12);

    let layout = GtkBox::new(Orientation::Vertical, 12);
    let summary = Label::new(Some("Loaded plugins"));
    summary.set_xalign(0.0);
    summary.add_css_class("section-title");
    layout.append(&summary);

    if let Some(host) = host {
        for diagnostic in host.diagnostics() {
            let line = Label::new(Some(&format!(
                "[{:?}] {}{}",
                diagnostic.level,
                diagnostic
                    .plugin_id
                    .as_deref()
                    .map(|plugin_id| format!("{plugin_id}: "))
                    .unwrap_or_default(),
                diagnostic.message
            )));
            line.set_xalign(0.0);
            line.set_wrap(true);
            line.add_css_class("mono");
            layout.append(&line);
        }
        if !host.diagnostics().is_empty() {
            layout.append(&Separator::new(Orientation::Horizontal));
        }

        if let Some(runtime) = host.runtime() {
        let activation = Label::new(Some(&format!(
            "Activation order: {}",
            runtime.activation_order().join(" -> ")
        )));
        activation.set_xalign(0.0);
        activation.set_wrap(true);
        activation.add_css_class("muted");
        layout.append(&activation);
        layout.append(&Separator::new(Orientation::Horizontal));

        for plugin in runtime.plugins() {
            let descriptor = plugin.descriptor();

            let title = Label::new(Some(&format!(
                "{} ({})",
                descriptor.name, descriptor.id
            )));
            title.set_xalign(0.0);
            title.add_css_class("mono");

            let version = Label::new(Some(&format!("version {}", descriptor.version)));
            version.set_xalign(0.0);
            version.add_css_class("muted");

            layout.append(&title);
            layout.append(&version);

            if !descriptor.description.is_empty() {
                let description = Label::new(Some(&descriptor.description));
                description.set_xalign(0.0);
                description.set_wrap(true);
                layout.append(&description);
            }

            let path = Label::new(Some(&format!("path {}", plugin.path().display())));
            path.set_xalign(0.0);
            path.add_css_class("mono");
            path.add_css_class("muted");
            path.set_wrap(true);
            layout.append(&path);

            if descriptor.dependencies.is_empty() {
                let deps = Label::new(Some("Dependencies: none"));
                deps.set_xalign(0.0);
                deps.add_css_class("muted");
                layout.append(&deps);
            } else {
                let deps_title = Label::new(Some("Dependencies"));
                deps_title.set_xalign(0.0);
                deps_title.add_css_class("section-title");
                layout.append(&deps_title);

                for dependency in &descriptor.dependencies {
                    let deps = Label::new(Some(&format!(
                        "{} {} [{}..{})",
                        if dependency.required { "required" } else { "optional" },
                        dependency.plugin_id,
                        dependency.min_version,
                        dependency.max_version_exclusive
                    )));
                    deps.set_xalign(0.0);
                    deps.add_css_class("mono");
                    deps.set_wrap(true);
                    layout.append(&deps);
                }
            }

            for page in settings_pages_for_plugin(runtime, &descriptor.id) {
                let page_title = Label::new(Some(&format!("Settings: {}", page.title)));
                page_title.set_xalign(0.0);
                page_title.add_css_class("section-title");
                layout.append(&page_title);

                let page_summary = Label::new(Some(&page.summary));
                page_summary.set_xalign(0.0);
                page_summary.set_wrap(true);
                layout.append(&page_summary);
            }

            let plugin_logs = logs_for_plugin(runtime, &descriptor.id);
            if !plugin_logs.is_empty() {
                let logs_title = Label::new(Some("Runtime logs"));
                logs_title.set_xalign(0.0);
                logs_title.add_css_class("section-title");
                layout.append(&logs_title);

                for entry in plugin_logs {
                    let log_line = Label::new(Some(&format!(
                        "[{:?}] {}",
                        entry.level, entry.message
                    )));
                    log_line.set_xalign(0.0);
                    log_line.add_css_class("mono");
                    log_line.set_wrap(true);
                    layout.append(&log_line);
                }
            }

            layout.append(&Separator::new(Orientation::Horizontal));
        }
        } else {
            let empty_runtime = Label::new(Some("No active plugin runtime."));
            empty_runtime.set_xalign(0.0);
            layout.append(&empty_runtime);
        }
    } else {
        let empty = Label::new(Some("No plugin runtime is active."));
        empty.set_xalign(0.0);
        layout.append(&empty);
    }

    body.append(&layout);
    dialog.connect_response(|dialog, _| {
        dialog.close();
    });
    dialog.present();
}

fn present_about_dialog(
    window: &ApplicationWindow,
    app_title: &str,
    host: Option<&PluginHost>,
) {
    let dialog = Dialog::builder()
        .transient_for(window)
        .modal(true)
        .title(format!("About {app_title}"))
        .default_width(560)
        .default_height(420)
        .build();
    dialog.add_button("Close", ResponseType::Close);

    let body = dialog.content_area();
    body.set_spacing(12);

    let layout = GtkBox::new(Orientation::Vertical, 12);

    let title = Label::new(Some(app_title));
    title.set_xalign(0.0);
    title.add_css_class("section-title");
    layout.append(&title);

    let version = Label::new(Some(&format!("Version {}", env!("CARGO_PKG_VERSION"))));
    version.set_xalign(0.0);
    version.add_css_class("muted");
    layout.append(&version);

    for section in about_sections(host.and_then(|host| host.runtime().map(|runtime| runtime.as_ref()))) {
        layout.append(&Separator::new(Orientation::Horizontal));

        let section_title = Label::new(Some(&section.title));
        section_title.set_xalign(0.0);
        section_title.add_css_class("section-title");
        layout.append(&section_title);

        let section_body = Label::new(Some(&section.body));
        section_body.set_xalign(0.0);
        section_body.set_wrap(true);
        layout.append(&section_body);
    }

    body.append(&layout);
    dialog.connect_response(|dialog, _| {
        dialog.close();
    });
    dialog.present();
}

fn about_sections(runtime: Option<&crate::plugins::PluginRuntime>) -> Vec<MzAboutSection> {
    let mut sections = Vec::new();

    if let Some(runtime) = runtime {
        for contribution in runtime
            .surface_contributions()
            .iter()
            .filter(|contribution| contribution.surface_id == "maruzzella.about.sections")
        {
            if let Ok(section) = MzAboutSection::from_bytes(&contribution.payload) {
                sections.push(section);
            }
        }
    }

    if sections.is_empty() {
        sections.push(MzAboutSection::new(
            "Maruzzella",
            "Neutral GTK desktop shell host",
        ));
    }

    sections
}

fn settings_pages_for_plugin(
    runtime: &crate::plugins::PluginRuntime,
    plugin_id: &str,
) -> Vec<MzSettingsPage> {
    let mut pages = Vec::new();

    for contribution in runtime
        .surface_contributions()
        .iter()
        .filter(|contribution| {
            contribution.plugin_id == plugin_id
                && contribution.surface_id == "maruzzella.plugins.settings_pages"
        })
    {
        if let Ok(page) = MzSettingsPage::from_bytes(&contribution.payload) {
            pages.push(page);
        }
    }

    pages
}

fn logs_for_plugin<'a>(
    runtime: &'a crate::plugins::PluginRuntime,
    plugin_id: &str,
) -> Vec<&'a crate::plugins::PluginLogEntry> {
    runtime
        .logs()
        .iter()
        .filter(|entry| entry.plugin_id == plugin_id)
        .collect()
}
