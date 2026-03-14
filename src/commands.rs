use std::collections::HashMap;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{ApplicationWindow, Box as GtkBox, Dialog, Label, Orientation, ResponseType};

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
    registry.register("shell.about", move || {
        let dialog = gtk::AboutDialog::builder()
            .transient_for(&about_window)
            .modal(true)
            .program_name(&app_title)
            .comments("Neutral GTK desktop shell host")
            .website("https://example.invalid/maruzzella")
            .version(env!("CARGO_PKG_VERSION"))
            .build();
        dialog.present();
    });

    let palette_window = window.clone();
    let commands = spec.commands.clone();
    registry.register("shell.open_command_palette", move || {
        present_command_palette(&palette_window, &commands);
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
