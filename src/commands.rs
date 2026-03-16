use std::collections::HashMap;
use std::rc::Rc;

use gtk::ApplicationWindow;

use crate::plugin_tabs::{
    open_or_focus_plugin_view, GroupHandles, OpenPluginViewRequest, ShellState,
};
use crate::plugins::PluginHost;
use crate::spec::ShellSpec;
use crate::theme;

const BASE_ABOUT_VIEW_ID: &str = "maruzzella.base.workspace.about";
const BASE_PLUGINS_VIEW_ID: &str = "maruzzella.base.workspace.plugins";
const BASE_COMMANDS_VIEW_ID: &str = "maruzzella.base.workspace.commands";
const BASE_REGISTERED_VIEWS_VIEW_ID: &str = "maruzzella.base.workspace.registered_views";

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
        self.handlers
            .insert(command_id.to_string(), Rc::new(handler));
    }

    pub fn handler_for(&self, command_id: &str) -> Option<CommandHandler> {
        self.handlers.get(command_id).cloned()
    }
}

pub fn shell_registry(
    _window: &ApplicationWindow,
    spec: &ShellSpec,
    plugin_host: Option<Rc<PluginHost>>,
    persistence_id: &str,
    shell_state: Option<ShellState>,
    group_handles: Option<GroupHandles>,
) -> CommandRegistry {
    let mut registry = CommandRegistry::new();

    registry.register("shell.reload_theme", move || {
        theme::reload();
    });

    let host_for_about = plugin_host.clone();
    let persistence_id_for_about = persistence_id.to_string();
    let state_for_about = shell_state.clone();
    let handles_for_about = group_handles.clone();
    registry.register("shell.about", move || {
        open_base_view(
            host_for_about.as_deref(),
            &persistence_id_for_about,
            state_for_about.as_ref(),
            handles_for_about.as_ref(),
            BASE_ABOUT_VIEW_ID,
            "About",
        );
    });

    let host_for_commands = plugin_host.clone();
    let persistence_id_for_commands = persistence_id.to_string();
    let state_for_commands = shell_state.clone();
    let handles_for_commands = group_handles.clone();
    registry.register("shell.open_command_palette", move || {
        open_base_view(
            host_for_commands.as_deref(),
            &persistence_id_for_commands,
            state_for_commands.as_ref(),
            handles_for_commands.as_ref(),
            BASE_COMMANDS_VIEW_ID,
            "Command Palette",
        );
    });

    let host_for_plugins = plugin_host.clone();
    let persistence_id_for_plugins = persistence_id.to_string();
    let state_for_plugins = shell_state.clone();
    let handles_for_plugins = group_handles.clone();
    registry.register("shell.plugins", move || {
        open_base_view(
            host_for_plugins.as_deref(),
            &persistence_id_for_plugins,
            state_for_plugins.as_ref(),
            handles_for_plugins.as_ref(),
            BASE_PLUGINS_VIEW_ID,
            "Plugins",
        );
    });

    let host_for_views = plugin_host.clone();
    let persistence_id_for_views = persistence_id.to_string();
    let state_for_views = shell_state.clone();
    let handles_for_views = group_handles.clone();
    registry.register("shell.browse_views", move || {
        open_base_view(
            host_for_views.as_deref(),
            &persistence_id_for_views,
            state_for_views.as_ref(),
            handles_for_views.as_ref(),
            BASE_REGISTERED_VIEWS_VIEW_ID,
            "Registered Views",
        );
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

fn open_base_view(
    host: Option<&PluginHost>,
    persistence_id: &str,
    shell_state: Option<&ShellState>,
    group_handles: Option<&GroupHandles>,
    view_id: &str,
    title: &str,
) {
    let Some(runtime) = host.and_then(|host| host.runtime()) else {
        return;
    };
    let Some(shell_state) = shell_state else {
        return;
    };
    let Some(group_handles) = group_handles else {
        return;
    };

    let mut request =
        OpenPluginViewRequest::new(view_id, maruzzella_api::MzViewPlacement::Workbench);
    request.requested_title = Some(title.to_string());
    let _ = open_or_focus_plugin_view(
        runtime,
        persistence_id,
        shell_state,
        group_handles,
        &request,
    );
}
