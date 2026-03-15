use std::collections::HashMap;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{ApplicationWindow, Box as GtkBox, Dialog, Label, Orientation, ResponseType, Separator};
use maruzzella_api::MzAboutSection;

use crate::plugins::PluginRuntime;
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
    plugin_runtime: Option<Rc<PluginRuntime>>,
) -> CommandRegistry {
    let mut registry = CommandRegistry::new();

    registry.register("shell.reload_theme", move || {
        theme::load();
    });

    let about_window = window.clone();
    let app_title = spec.title.clone();
    let runtime_for_about = plugin_runtime.clone();
    registry.register("shell.about", move || {
        present_about_dialog(&about_window, &app_title, runtime_for_about.as_deref());
    });

    let palette_window = window.clone();
    let commands = spec.commands.clone();
    registry.register("shell.open_command_palette", move || {
        present_command_palette(&palette_window, &commands);
    });

    let plugins_window = window.clone();
    let runtime_for_plugins = plugin_runtime.clone();
    registry.register("shell.plugins", move || {
        present_plugins_dialog(&plugins_window, runtime_for_plugins.as_deref());
    });

    if let Some(plugin_runtime) = plugin_runtime {
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

fn present_plugins_dialog(window: &ApplicationWindow, runtime: Option<&PluginRuntime>) {
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

    if let Some(runtime) = runtime {
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

            layout.append(&Separator::new(Orientation::Horizontal));
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
    runtime: Option<&PluginRuntime>,
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

    for section in about_sections(runtime) {
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

fn about_sections(runtime: Option<&PluginRuntime>) -> Vec<MzAboutSection> {
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
