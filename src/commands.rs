use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::gio;
use gtk::ApplicationWindow;

use crate::app::{MaruzzellaHandle, WorkspaceSession};
use crate::base_plugin;
use crate::plugin_tabs::{
    last_active_plugin_tab, open_or_focus_plugin_view, GroupHandles, OpenPluginViewRequest,
    ShellState,
};
use crate::plugins::PluginHost;
use crate::spec::ShellSpec;
use crate::theme;

const BASE_ABOUT_VIEW_ID: &str = "maruzzella.base.workspace.about";
const BASE_PLUGINS_VIEW_ID: &str = "maruzzella.base.workspace.plugins";
const BASE_SETTINGS_VIEW_ID: &str = "maruzzella.base.workspace.settings";
const BASE_COMMANDS_VIEW_ID: &str = "maruzzella.base.workspace.commands";
const BASE_REGISTERED_VIEWS_VIEW_ID: &str = "maruzzella.base.workspace.registered_views";
const BASE_EDITOR_VIEW_ID: &str = base_plugin::VIEW_WORKSPACE_EDITOR;
const CMD_SWITCH_TO_WORKSPACE: &str = "shell.switch_to_workspace";

type CommandHandler = Rc<dyn Fn(&[u8])>;

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
        F: Fn(&[u8]) + 'static,
    {
        self.handlers
            .insert(command_id.to_string(), Rc::new(handler));
    }

    pub fn handler_for(&self, command_id: &str) -> Option<CommandHandler> {
        self.handlers.get(command_id).cloned()
    }
}

pub fn shell_registry(
    window: &ApplicationWindow,
    spec: &ShellSpec,
    plugin_host: Option<Rc<PluginHost>>,
    persistence_id: &str,
    shell_state: Option<ShellState>,
    group_handles: Option<GroupHandles>,
) -> CommandRegistry {
    let mut registry = CommandRegistry::new();
    let window = window.clone();

    registry.register("shell.reload_theme", move |_| {
        theme::reload();
    });

    let window_for_workspace_switch = window.clone();
    registry.register(CMD_SWITCH_TO_WORKSPACE, move |payload| {
        eprintln!(
            "maruzzella: shell.switch_to_workspace received payload_len={}",
            payload.len()
        );
        let Some(handle) = (unsafe { window_for_workspace_switch.data::<MaruzzellaHandle>("maruzzella-handle") })
            .map(|ptr| unsafe { ptr.as_ref().clone() })
        else {
            eprintln!("maruzzella: shell.switch_to_workspace missing maruzzella-handle on window");
            return;
        };
        eprintln!("maruzzella: shell.switch_to_workspace invoking handle.switch_to_workspace");
        let result = handle.switch_to_workspace(WorkspaceSession {
            project_handle: Some(payload.to_vec()),
            shell_spec: None,
            window_policy: None,
        });
        eprintln!(
            "maruzzella: shell.switch_to_workspace result={}",
            match result {
                Ok(()) => "ok",
                Err(_) => "err",
            }
        );
    });

    let host_for_about = plugin_host.clone();
    let persistence_id_for_about = persistence_id.to_string();
    let state_for_about = shell_state.clone();
    let handles_for_about = group_handles.clone();
    registry.register("shell.about", move |_| {
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
    registry.register("shell.open_command_palette", move |_| {
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
    registry.register("shell.plugins", move |_| {
        open_base_view(
            host_for_plugins.as_deref(),
            &persistence_id_for_plugins,
            state_for_plugins.as_ref(),
            handles_for_plugins.as_ref(),
            BASE_PLUGINS_VIEW_ID,
            "Plugins",
        );
    });

    let host_for_settings = plugin_host.clone();
    let persistence_id_for_settings = persistence_id.to_string();
    let state_for_settings = shell_state.clone();
    let handles_for_settings = group_handles.clone();
    registry.register("shell.settings", move |_| {
        open_base_view(
            host_for_settings.as_deref(),
            &persistence_id_for_settings,
            state_for_settings.as_ref(),
            handles_for_settings.as_ref(),
            BASE_SETTINGS_VIEW_ID,
            "Settings",
        );
    });

    let host_for_views = plugin_host.clone();
    let persistence_id_for_views = persistence_id.to_string();
    let state_for_views = shell_state.clone();
    let handles_for_views = group_handles.clone();
    registry.register("shell.browse_views", move |_| {
        open_base_view(
            host_for_views.as_deref(),
            &persistence_id_for_views,
            state_for_views.as_ref(),
            handles_for_views.as_ref(),
            BASE_REGISTERED_VIEWS_VIEW_ID,
            "Registered Views",
        );
    });

    let host_for_new_buffer = plugin_host.clone();
    let persistence_id_for_new_buffer = persistence_id.to_string();
    let state_for_new_buffer = shell_state.clone();
    let handles_for_new_buffer = group_handles.clone();
    registry.register(base_plugin::CMD_NEW_BUFFER, move |_| {
        let Some(shell_state) = state_for_new_buffer.as_ref() else {
            return;
        };
        let Some(group_handles) = handles_for_new_buffer.as_ref() else {
            return;
        };
        let Some(runtime) = host_for_new_buffer.as_ref().and_then(|host| host.runtime()) else {
            return;
        };
        let document_id = next_untitled_document_id(shell_state);
        let mut request = OpenPluginViewRequest::new(
            BASE_EDITOR_VIEW_ID,
            maruzzella_api::MzViewPlacement::Workbench,
        );
        request.instance_key = Some(base_plugin::editor_instance_key(&document_id));
        request.payload = base_plugin::new_untitled_editor_payload(&document_id);
        request.requested_title = Some(untitled_title(&document_id));
        let _ = open_or_focus_plugin_view(
            runtime,
            &persistence_id_for_new_buffer,
            shell_state,
            group_handles,
            &request,
        );
    });

    let host_for_open_file = plugin_host.clone();
    let persistence_id_for_open_file = persistence_id.to_string();
    let state_for_open_file = shell_state.clone();
    let handles_for_open_file = group_handles.clone();
    let window_for_open_file = window.clone();
    registry.register(base_plugin::CMD_OPEN_FILE_EDITOR, move |payload| {
        let Some(shell_state) = state_for_open_file.as_ref() else {
            return;
        };
        let Some(group_handles) = handles_for_open_file.as_ref() else {
            return;
        };
        let Some(runtime) = host_for_open_file.as_ref().and_then(|host| host.runtime()) else {
            return;
        };
        if payload.is_empty() {
            open_file_picker(
                &window_for_open_file,
                runtime.clone(),
                persistence_id_for_open_file.clone(),
                shell_state.clone(),
                group_handles.clone(),
            );
            return;
        }
        let Ok(path) = std::str::from_utf8(payload) else {
            eprintln!("shell.open_file_editor payload must be valid UTF-8");
            return;
        };
        let path = path.trim();
        if path.is_empty() {
            return;
        }
        let Ok(document) = base_plugin::file_editor_payload_for_path(Path::new(path))
        else {
            eprintln!("shell.open_file_editor failed for path: {path}");
            return;
        };
        let _ = open_editor_document(
            runtime,
            &persistence_id_for_open_file,
            shell_state,
            group_handles,
            &document,
        );
    });

    let window_for_save_buffer = window.clone();
    let state_for_save_buffer = shell_state.clone();
    let handles_for_save_buffer = group_handles.clone();
    let persistence_id_for_save_buffer = persistence_id.to_string();
    registry.register(base_plugin::CMD_SAVE_BUFFER, move |_| {
        let Some(shell_state) = state_for_save_buffer.as_ref() else {
            return;
        };
        let Some(group_handles) = handles_for_save_buffer.as_ref() else {
            return;
        };
        let Some(active) = last_active_plugin_tab() else {
            return;
        };
        if !base_plugin::is_editor_view(Some(&active.plugin_view_id)) {
            return;
        }
        let Some(instance_key) = active.instance_key.as_deref() else {
            return;
        };
        match base_plugin::editor_document_for_instance_key(instance_key) {
            Some(document) if document.kind == base_plugin::EditorDocumentKind::Untitled => {
                save_editor_as_picker(
                    &window_for_save_buffer,
                    shell_state.clone(),
                    group_handles.clone(),
                    persistence_id_for_save_buffer.clone(),
                    instance_key.to_string(),
                    document.file_path.as_deref().map(str::to_string),
                );
            }
            Some(_) => {
                if let Err(error) = base_plugin::save_editor_by_instance_key(instance_key) {
                    eprintln!("shell.save_buffer failed: {error}");
                }
            }
            None => {}
        }
    });

    let window_for_save_as = window.clone();
    let state_for_save_as = shell_state.clone();
    let handles_for_save_as = group_handles.clone();
    let persistence_id_for_save_as = persistence_id.to_string();
    registry.register(base_plugin::CMD_SAVE_BUFFER_AS, move |_| {
        let Some(shell_state) = state_for_save_as.as_ref() else {
            return;
        };
        let Some(group_handles) = handles_for_save_as.as_ref() else {
            return;
        };
        let Some(active) = last_active_plugin_tab() else {
            return;
        };
        if !base_plugin::is_editor_view(Some(&active.plugin_view_id)) {
            return;
        }
        let Some(instance_key) = active.instance_key.as_deref() else {
            return;
        };
        let suggested_path = base_plugin::editor_document_for_instance_key(instance_key)
            .and_then(|document| document.file_path);
        save_editor_as_picker(
            &window_for_save_as,
            shell_state.clone(),
            group_handles.clone(),
            persistence_id_for_save_as.clone(),
            instance_key.to_string(),
            suggested_path,
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
            registry.register(&command_id.clone(), move |payload| {
                if let Err(status) = runtime.dispatch_command(&command_id, payload) {
                    eprintln!("plugin command failed: {command_id} ({status:?})");
                }
            });
        }
    }

    let mut invocation_ids = HashSet::new();
    for item in spec
        .menu_items
        .iter()
        .map(|item| (item.id.as_str(), item.command_id.as_str(), item.payload.clone()))
        .chain(
            spec.toolbar_items
                .iter()
                .map(|item| (item.id.as_str(), item.command_id.as_str(), item.payload.clone())),
        )
    {
        if !invocation_ids.insert(item.0.to_string()) {
            continue;
        }
        let action_id = item.0.to_string();
        let command_id = item.1.to_string();
        let payload = item.2;
        let Some(handler) = registry.handler_for(&command_id) else {
            continue;
        };
        registry.register(&action_id, move |_| {
            handler(&payload);
        });
    }

    registry
}

fn open_editor_document(
    runtime: &Rc<crate::plugins::PluginRuntime>,
    persistence_id: &str,
    shell_state: &ShellState,
    group_handles: &GroupHandles,
    document: &base_plugin::EditorDocumentPayload,
) -> Option<crate::plugin_tabs::OpenPluginViewOutcome> {
    let mut request = OpenPluginViewRequest::new(
        BASE_EDITOR_VIEW_ID,
        maruzzella_api::MzViewPlacement::Workbench,
    );
    request.instance_key = Some(base_plugin::editor_instance_key(&document.document_id));
    request.payload = base_plugin::editor_payload_to_bytes(document).ok()?;
    request.requested_title = Some(document.display_name.clone());
    open_or_focus_plugin_view(runtime, persistence_id, shell_state, group_handles, &request)
}

fn open_file_picker(
    window: &ApplicationWindow,
    runtime: Rc<crate::plugins::PluginRuntime>,
    persistence_id: String,
    shell_state: ShellState,
    group_handles: GroupHandles,
) {
    let dialog = gtk::FileDialog::builder()
        .title("Open File")
        .initial_folder(&gio::File::for_path(
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/".to_string()))),
        ))
        .build();
    dialog.open(Some(window), gio::Cancellable::NONE, move |result| {
        if let Ok(file) = result {
            if let Some(path) = file.path() {
                if let Ok(document) = base_plugin::file_editor_payload_for_path(&path) {
                    let _ = open_editor_document(
                        &runtime,
                        &persistence_id,
                        &shell_state,
                        &group_handles,
                        &document,
                    );
                }
            }
        }
    });
}

fn save_editor_as_picker(
    window: &ApplicationWindow,
    shell_state: ShellState,
    group_handles: GroupHandles,
    persistence_id: String,
    instance_key: String,
    suggested_path: Option<String>,
) {
    let mut builder = gtk::FileDialog::builder().title("Save Buffer As");
    if let Some(path) = suggested_path.as_deref() {
        let p = Path::new(path);
        if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
            builder = builder.initial_name(name);
        }
        if let Some(parent) = p.parent() {
            builder = builder.initial_folder(&gio::File::for_path(parent));
        }
    } else {
        builder = builder.initial_folder(&gio::File::for_path(
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/".to_string()))),
        ));
    }
    let dialog = builder.build();
    dialog.save(Some(window), gio::Cancellable::NONE, move |result| {
        if let Ok(file) = result {
            if let Some(path) = file.path() {
                if let Err(error) = save_editor_as_path(
                    &shell_state,
                    &group_handles,
                    &persistence_id,
                    &instance_key,
                    &path,
                ) {
                    eprintln!("shell.save_buffer_as failed: {error}");
                }
            }
        }
    });
}

fn save_editor_as_path(
    shell_state: &ShellState,
    group_handles: &GroupHandles,
    persistence_id: &str,
    instance_key: &str,
    path: &Path,
) -> Result<(), String> {
    let new_document = base_plugin::write_editor_contents_to_path(instance_key, path)?;
    let new_instance_key = base_plugin::editor_instance_key(&new_document.document_id);
    update_editor_tab_document(
        shell_state,
        group_handles,
        persistence_id,
        instance_key,
        &new_instance_key,
        &new_document,
    )?;
    let _ = base_plugin::replace_editor_document(instance_key, &new_instance_key, new_document)?;
    if let Some((group_id, tab_id)) = find_tab_by_instance_key(shell_state, &new_instance_key) {
        crate::plugin_tabs::remember_active_plugin_tab(shell_state, &group_id, &tab_id);
    }
    Ok(())
}

fn next_untitled_document_id(shell_state: &ShellState) -> String {
    let shell = shell_state.borrow();
    let mut next_index = 1usize;
    for tab in all_tabs(&shell.spec) {
        let Some(instance_key) = tab.instance_key.as_deref() else {
            continue;
        };
        let Some(document_id) = base_plugin::editor_document_id_from_instance_key(instance_key) else {
            continue;
        };
        if let Some(index) = document_id
            .strip_prefix("untitled:")
            .and_then(|value| value.parse::<usize>().ok())
        {
            next_index = next_index.max(index + 1);
        }
    }
    format!("untitled:{next_index}")
}

fn untitled_title(document_id: &str) -> String {
    document_id
        .strip_prefix("untitled:")
        .map(|index| format!("Untitled {index}"))
        .unwrap_or_else(|| "Untitled".to_string())
}

fn all_tabs<'a>(
    spec: &'a crate::spec::ShellSpec,
) -> Box<dyn Iterator<Item = &'a crate::spec::TabSpec> + 'a> {
    Box::new(
        spec.left_panel
            .tabs
            .iter()
            .chain(spec.right_panel.tabs.iter())
            .chain(spec.bottom_panel.tabs.iter())
            .chain(workbench_tabs(&spec.workbench)),
    )
}

fn workbench_tabs<'a>(
    node: &'a crate::spec::WorkbenchNodeSpec,
) -> Box<dyn Iterator<Item = &'a crate::spec::TabSpec> + 'a> {
    match node {
        crate::spec::WorkbenchNodeSpec::Group(group) => Box::new(group.tabs.iter()),
        crate::spec::WorkbenchNodeSpec::Split { children, .. } => {
            Box::new(children.iter().flat_map(|child| workbench_tabs(child)))
        }
    }
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

fn update_editor_tab_document(
    shell_state: &ShellState,
    group_handles: &GroupHandles,
    persistence_id: &str,
    old_instance_key: &str,
    new_instance_key: &str,
    document: &base_plugin::EditorDocumentPayload,
) -> Result<(), String> {
    let payload = base_plugin::editor_payload_to_bytes(document)?;
    let (group_id, tab_id) = find_tab_by_instance_key(shell_state, old_instance_key)
        .ok_or_else(|| "active editor tab was not found".to_string())?;

    {
        let mut shell = shell_state.borrow_mut();
        let tab = all_tabs_mut(&mut shell.spec)
            .find(|tab| tab.instance_key.as_deref() == Some(old_instance_key))
            .ok_or_else(|| "active editor tab was not found".to_string())?;
        tab.title = document.display_name.clone();
        tab.instance_key = Some(new_instance_key.to_string());
        tab.payload = payload;
        crate::layout::save(persistence_id, &shell.clone());
    }

    if let Some(handle) = group_handles.borrow().get(&group_id).cloned() {
        handle.set_tab_title(&tab_id, &document.display_name);
    }
    Ok(())
}

fn find_tab_by_instance_key(shell_state: &ShellState, instance_key: &str) -> Option<(String, String)> {
    let shell = shell_state.borrow();
    find_tab_in_group(&shell.spec.left_panel, instance_key)
        .or_else(|| find_tab_in_group(&shell.spec.right_panel, instance_key))
        .or_else(|| find_tab_in_group(&shell.spec.bottom_panel, instance_key))
        .or_else(|| find_tab_in_workbench(&shell.spec.workbench, instance_key))
}

fn find_tab_in_group(group: &crate::spec::TabGroupSpec, instance_key: &str) -> Option<(String, String)> {
    group
        .tabs
        .iter()
        .find(|tab| tab.instance_key.as_deref() == Some(instance_key))
        .map(|tab| (group.id.clone(), tab.id.clone()))
}

fn find_tab_in_workbench(
    node: &crate::spec::WorkbenchNodeSpec,
    instance_key: &str,
) -> Option<(String, String)> {
    match node {
        crate::spec::WorkbenchNodeSpec::Group(group) => find_tab_in_group(group, instance_key),
        crate::spec::WorkbenchNodeSpec::Split { children, .. } => children
            .iter()
            .find_map(|child| find_tab_in_workbench(child, instance_key)),
    }
}

fn all_tabs_mut<'a>(
    spec: &'a mut crate::spec::ShellSpec,
) -> Box<dyn Iterator<Item = &'a mut crate::spec::TabSpec> + 'a> {
    Box::new(
        spec.left_panel
            .tabs
            .iter_mut()
            .chain(spec.right_panel.tabs.iter_mut())
            .chain(spec.bottom_panel.tabs.iter_mut())
            .chain(workbench_tabs_mut(&mut spec.workbench)),
    )
}

fn workbench_tabs_mut<'a>(
    node: &'a mut crate::spec::WorkbenchNodeSpec,
) -> Box<dyn Iterator<Item = &'a mut crate::spec::TabSpec> + 'a> {
    match node {
        crate::spec::WorkbenchNodeSpec::Group(group) => Box::new(group.tabs.iter_mut()),
        crate::spec::WorkbenchNodeSpec::Split { children, .. } => {
            Box::new(children.iter_mut().flat_map(|child| workbench_tabs_mut(child)))
        }
    }
}
