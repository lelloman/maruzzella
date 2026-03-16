use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{
    Align, ApplicationWindow, Box as GtkBox, Dialog, Label, Orientation, ResponseType,
    ScrolledWindow, Separator,
};
use maruzzella_api::{MzAboutSection, MzContributionSurface, MzSettingsPage, MzViewPlacement};

use crate::plugins::{PluginHost, PluginRuntime};
use crate::shell::{tabbed_panel, workbench_custom::CustomWorkbenchGroupHandle};
use crate::spec::{plugin_tab, CommandSpec, ShellSpec, TabGroupSpec, WorkbenchNodeSpec};
use crate::theme;

type CommandHandler = Rc<dyn Fn()>;
type ShellState = Rc<RefCell<crate::layout::PersistedShell>>;
type GroupHandles = Rc<HashMap<String, CustomWorkbenchGroupHandle>>;

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
    window: &ApplicationWindow,
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

    let views_window = window.clone();
    let host_for_views = plugin_host.clone();
    let persistence_id_for_views = persistence_id.to_string();
    let state_for_views = shell_state.clone();
    let handles_for_views = group_handles.clone();
    registry.register("shell.browse_views", move || {
        present_views_dialog(
            &views_window,
            host_for_views.as_deref(),
            &persistence_id_for_views,
            state_for_views.as_ref(),
            handles_for_views.as_ref(),
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

    let scroller = ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .min_content_height(360)
        .build();
    let layout = GtkBox::new(Orientation::Vertical, 14);
    layout.add_css_class("plugin-detail-root");
    layout.set_margin_top(16);
    layout.set_margin_bottom(16);
    layout.set_margin_start(16);
    layout.set_margin_end(16);

    let loaded_count = host
        .and_then(|host| host.runtime().map(|runtime| runtime.plugins().len()))
        .unwrap_or_default();
    let diagnostic_count = host
        .map(|host| host.diagnostics().len())
        .unwrap_or_default();

    let hero = GtkBox::new(Orientation::Vertical, 8);
    hero.add_css_class("plugin-hero");

    let hero_title = Label::new(Some("Plugin Manager"));
    hero_title.set_xalign(0.0);
    hero_title.add_css_class("plugin-detail-name");
    hero.append(&hero_title);

    let hero_body = Label::new(Some(&format!(
        "{loaded_count} plugin(s) loaded, {diagnostic_count} diagnostic message(s) recorded for the current shell session."
    )));
    hero_body.set_xalign(0.0);
    hero_body.set_wrap(true);
    hero_body.add_css_class("plugin-detail-description");
    hero.append(&hero_body);
    layout.append(&hero);

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
            activation.add_css_class("plugin-detail-overview");
            layout.append(&activation);
            layout.append(&Separator::new(Orientation::Horizontal));

            for plugin in runtime.plugins() {
                let descriptor = plugin.descriptor();
                let card = GtkBox::new(Orientation::Vertical, 8);
                card.add_css_class("plugin-detail-root");

                let heading = GtkBox::new(Orientation::Horizontal, 8);
                let title = Label::new(Some(&descriptor.name));
                title.set_xalign(0.0);
                title.set_hexpand(true);
                title.add_css_class("plugin-detail-name");
                heading.append(&title);

                let status = Label::new(Some("Loaded"));
                status.set_halign(Align::End);
                status.add_css_class("status-badge");
                status.add_css_class("status-loaded");
                heading.append(&status);
                card.append(&heading);

                let plugin_id = Label::new(Some(&descriptor.id));
                plugin_id.set_xalign(0.0);
                plugin_id.add_css_class("mono");
                card.append(&plugin_id);

                let version = Label::new(Some(&format!("Version {}", descriptor.version)));
                version.set_xalign(0.0);
                version.add_css_class("muted");
                card.append(&version);

                if !descriptor.description.is_empty() {
                    let description = Label::new(Some(&descriptor.description));
                    description.set_xalign(0.0);
                    description.set_wrap(true);
                    description.add_css_class("plugin-detail-description");
                    card.append(&description);
                }

                let path = Label::new(Some(&format!("path {}", plugin.path().display())));
                path.set_xalign(0.0);
                path.add_css_class("mono");
                path.add_css_class("muted");
                path.set_wrap(true);
                card.append(&path);

                append_named_list(
                    &card,
                    "Dependencies",
                    if descriptor.dependencies.is_empty() {
                        vec!["none".to_string()]
                    } else {
                        descriptor
                            .dependencies
                            .iter()
                            .map(|dependency| {
                                format!(
                                    "{} {} [{}..{})",
                                    if dependency.required {
                                        "required"
                                    } else {
                                        "optional"
                                    },
                                    dependency.plugin_id,
                                    dependency.min_version,
                                    dependency.max_version_exclusive
                                )
                            })
                            .collect::<Vec<_>>()
                    },
                    true,
                );

                let settings_pages = settings_pages_for_plugin(runtime, &descriptor.id);
                if !settings_pages.is_empty() {
                    append_named_list(
                        &card,
                        "Settings Surfaces",
                        settings_pages
                            .into_iter()
                            .map(|page| {
                                format!(
                                    "{} / {}: {}",
                                    page.category.label(),
                                    page.title,
                                    page.summary
                                )
                            })
                            .collect::<Vec<_>>(),
                        false,
                    );
                }

                let plugin_views = views_for_plugin(runtime, &descriptor.id);
                if !plugin_views.is_empty() {
                    append_named_list(
                        &card,
                        "Registered Views",
                        plugin_views
                            .into_iter()
                            .map(|view| {
                                format!(
                                    "{} / {} ({})",
                                    view.placement.label(),
                                    view.title,
                                    view.view_id
                                )
                            })
                            .collect::<Vec<_>>(),
                        false,
                    );
                }

                let plugin_logs = logs_for_plugin(runtime, &descriptor.id);
                if !plugin_logs.is_empty() {
                    append_named_list(
                        &card,
                        "Runtime Logs",
                        plugin_logs
                            .into_iter()
                            .map(|entry| format!("[{:?}] {}", entry.level, entry.message))
                            .collect::<Vec<_>>(),
                        true,
                    );
                }

                layout.append(&card);
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

    scroller.set_child(Some(&layout));
    body.append(&scroller);
    dialog.connect_response(|dialog, _| {
        dialog.close();
    });
    dialog.present();
}

fn present_views_dialog(
    window: &ApplicationWindow,
    host: Option<&PluginHost>,
    persistence_id: &str,
    shell_state: Option<&ShellState>,
    group_handles: Option<&GroupHandles>,
) {
    let dialog = Dialog::builder()
        .transient_for(window)
        .modal(true)
        .title("Browse Views")
        .default_width(560)
        .default_height(420)
        .build();
    dialog.add_button("Close", ResponseType::Close);

    let body = dialog.content_area();
    body.set_spacing(12);

    let scroller = ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .min_content_height(360)
        .build();
    let layout = GtkBox::new(Orientation::Vertical, 14);
    layout.add_css_class("plugin-detail-root");
    layout.set_margin_top(16);
    layout.set_margin_bottom(16);
    layout.set_margin_start(16);
    layout.set_margin_end(16);

    let all_views = host
        .and_then(|host| host.runtime())
        .map(|runtime| sorted_views(runtime))
        .unwrap_or_default();

    let hero = GtkBox::new(Orientation::Vertical, 8);
    hero.add_css_class("plugin-hero");

    let hero_title = Label::new(Some("Registered Views"));
    hero_title.set_xalign(0.0);
    hero_title.add_css_class("plugin-detail-name");
    hero.append(&hero_title);

    let hero_body = Label::new(Some(&format!(
        "{} view factory(ies) available through the active plugin runtime.",
        all_views.len()
    )));
    hero_body.set_xalign(0.0);
    hero_body.set_wrap(true);
    hero_body.add_css_class("plugin-detail-description");
    hero.append(&hero_body);
    layout.append(&hero);

    if all_views.is_empty() {
        let empty = Label::new(Some(
            "No plugin views are currently registered. Load a plugin or enable the base runtime.",
        ));
        empty.set_xalign(0.0);
        empty.set_wrap(true);
        layout.append(&empty);
    } else {
        for placement in [
            MzViewPlacement::Workbench,
            MzViewPlacement::SidePanel,
            MzViewPlacement::BottomPanel,
            MzViewPlacement::Dialog,
        ] {
            let views = all_views
                .iter()
                .filter(|view| view.placement == placement)
                .collect::<Vec<_>>();
            if views.is_empty() {
                continue;
            }

            let section_title = Label::new(Some(placement.label()));
            section_title.set_xalign(0.0);
            section_title.add_css_class("section-title");
            layout.append(&section_title);

            for view in views {
                let card = GtkBox::new(Orientation::Vertical, 6);

                let title = Label::new(Some(&view.title));
                title.set_xalign(0.0);
                title.add_css_class("plugin-detail-name");
                card.append(&title);

                let plugin_line =
                    Label::new(Some(&format!("{}  ({})", view.plugin_id, view.view_id)));
                plugin_line.set_xalign(0.0);
                plugin_line.set_wrap(true);
                plugin_line.add_css_class("mono");
                card.append(&plugin_line);

                let focus_button = gtk::Button::with_label("Open");
                focus_button.set_halign(Align::Start);
                focus_button.add_css_class("toolbar-button");
                let view_id = view.view_id.clone();
                let runtime = host.and_then(|host| host.runtime()).cloned();
                let persistence_id = persistence_id.to_string();
                let state = shell_state.cloned();
                let handles = group_handles.cloned();
                let dialog_for_focus = dialog.clone();
                focus_button.connect_clicked(move |_| {
                    if let (Some(runtime), Some(state), Some(handles)) =
                        (runtime.as_ref(), state.as_ref(), handles.as_ref())
                    {
                        if open_or_focus_plugin_view(
                            runtime,
                            &persistence_id,
                            state,
                            handles,
                            &view_id,
                        ) {
                            dialog_for_focus.close();
                        }
                    }
                });
                focus_button.set_sensitive(host.and_then(|host| host.runtime()).is_some());
                card.append(&focus_button);

                layout.append(&card);
                layout.append(&Separator::new(Orientation::Horizontal));
            }
        }
    }

    scroller.set_child(Some(&layout));
    body.append(&scroller);
    dialog.connect_response(|dialog, _| {
        dialog.close();
    });
    dialog.present();
}

fn present_about_dialog(window: &ApplicationWindow, app_title: &str, host: Option<&PluginHost>) {
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

    let scroller = ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .min_content_height(300)
        .build();
    let layout = GtkBox::new(Orientation::Vertical, 12);
    layout.add_css_class("plugin-detail-root");
    layout.set_margin_top(16);
    layout.set_margin_bottom(16);
    layout.set_margin_start(16);
    layout.set_margin_end(16);

    let hero = GtkBox::new(Orientation::Vertical, 8);
    hero.add_css_class("plugin-hero");

    let title = Label::new(Some(app_title));
    title.set_xalign(0.0);
    title.add_css_class("plugin-detail-name");
    hero.append(&title);

    let version = Label::new(Some(&format!("Version {}", env!("CARGO_PKG_VERSION"))));
    version.set_xalign(0.0);
    version.add_css_class("plugin-detail-description");
    hero.append(&version);
    layout.append(&hero);

    for section in
        about_sections(host.and_then(|host| host.runtime().map(|runtime| runtime.as_ref())))
    {
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

    scroller.set_child(Some(&layout));
    body.append(&scroller);
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
            .filter(|contribution| {
                contribution.surface == Some(MzContributionSurface::AboutSections)
            })
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
                && contribution.surface == Some(MzContributionSurface::PluginSettingsPages)
        })
    {
        if let Ok(page) = MzSettingsPage::from_bytes(&contribution.payload) {
            pages.push(page);
        }
    }

    pages
}

fn views_for_plugin<'a>(
    runtime: &'a crate::plugins::PluginRuntime,
    plugin_id: &str,
) -> Vec<&'a crate::plugins::RegisteredViewFactory> {
    let mut views = runtime
        .view_factories()
        .iter()
        .filter(|view| view.plugin_id == plugin_id)
        .collect::<Vec<_>>();
    views.sort_by_key(|view| view_order_key(view.placement, &view.title));
    views
}

fn sorted_views(
    runtime: &crate::plugins::PluginRuntime,
) -> Vec<&crate::plugins::RegisteredViewFactory> {
    let mut views = runtime.view_factories().iter().collect::<Vec<_>>();
    views.sort_by_key(|view| {
        (
            view_order_key(view.placement, &view.title),
            view.plugin_id.clone(),
            view.view_id.clone(),
        )
    });
    views
}

fn view_order_key(placement: MzViewPlacement, title: &str) -> (usize, String) {
    let rank = match placement {
        MzViewPlacement::Workbench => 0,
        MzViewPlacement::SidePanel => 1,
        MzViewPlacement::BottomPanel => 2,
        MzViewPlacement::Dialog => 3,
    };
    (rank, title.to_string())
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

fn append_named_list(container: &GtkBox, title_text: &str, rows: Vec<String>, mono: bool) {
    let title = Label::new(Some(title_text));
    title.set_xalign(0.0);
    title.add_css_class("section-title");
    container.append(&title);

    for row in rows {
        let label = Label::new(Some(&row));
        label.set_xalign(0.0);
        label.set_wrap(true);
        if mono {
            label.add_css_class("mono");
        } else {
            label.add_css_class("plugin-detail-overview");
        }
        container.append(&label);
    }
}

fn focus_plugin_view(
    shell_state: &ShellState,
    group_handles: &GroupHandles,
    plugin_view_id: &str,
) -> bool {
    let Some((group_id, tab_id)) = find_plugin_view_tab(&shell_state.borrow().spec, plugin_view_id)
    else {
        return false;
    };
    let Some(handle) = group_handles.get(&group_id) else {
        return false;
    };
    handle.set_active_tab(&tab_id);
    true
}

fn open_or_focus_plugin_view(
    runtime: &Rc<PluginRuntime>,
    persistence_id: &str,
    shell_state: &ShellState,
    group_handles: &GroupHandles,
    plugin_view_id: &str,
) -> bool {
    if focus_plugin_view(shell_state, group_handles, plugin_view_id) {
        return true;
    }

    let Some(view) = runtime
        .view_factories()
        .iter()
        .find(|view| view.view_id == plugin_view_id)
    else {
        return false;
    };
    let Some(group_id) = target_group_id_for_placement(view.placement, group_handles) else {
        return false;
    };
    let Some(handle) = group_handles.get(group_id) else {
        return false;
    };

    let tab = {
        let mut shell = shell_state.borrow_mut();
        let tab = plugin_tab(
            &next_dynamic_tab_id(&shell.spec, &view.view_id),
            group_id,
            &view.title,
            &view.view_id,
            "Plugin view opened from the shell view browser.",
            true,
        );
        if let Some(group) = find_group_mut(&mut shell.spec, group_id) {
            group.tabs.push(tab.clone());
            group.active_tab_id = Some(tab.id.clone());
        } else {
            return false;
        }
        crate::layout::save(persistence_id, &shell.clone());
        tab
    };

    let page = tabbed_panel::build_tab_page("workbench", &tab, Some(runtime));
    handle.append_page(page, true);
    true
}

fn find_plugin_view_tab(spec: &ShellSpec, plugin_view_id: &str) -> Option<(String, String)> {
    find_plugin_view_in_group(&spec.left_panel, plugin_view_id)
        .or_else(|| find_plugin_view_in_group(&spec.right_panel, plugin_view_id))
        .or_else(|| find_plugin_view_in_group(&spec.bottom_panel, plugin_view_id))
        .or_else(|| find_plugin_view_in_workbench(&spec.workbench, plugin_view_id))
}

fn find_plugin_view_in_workbench(
    node: &WorkbenchNodeSpec,
    plugin_view_id: &str,
) -> Option<(String, String)> {
    match node {
        WorkbenchNodeSpec::Group(group) => find_plugin_view_in_group(group, plugin_view_id),
        WorkbenchNodeSpec::Split { children, .. } => children
            .iter()
            .find_map(|child| find_plugin_view_in_workbench(child, plugin_view_id)),
    }
}

fn find_plugin_view_in_group(
    group: &TabGroupSpec,
    plugin_view_id: &str,
) -> Option<(String, String)> {
    group.tabs.iter().find_map(|tab| {
        (tab.plugin_view_id.as_deref() == Some(plugin_view_id))
            .then(|| (group.id.clone(), tab.id.clone()))
    })
}

fn target_group_id_for_placement<'a>(
    placement: MzViewPlacement,
    group_handles: &'a GroupHandles,
) -> Option<&'a str> {
    let preferred = match placement {
        MzViewPlacement::Workbench => "workbench-a",
        MzViewPlacement::SidePanel => "panel-left",
        MzViewPlacement::BottomPanel => "panel-bottom",
        MzViewPlacement::Dialog => return None,
    };
    if group_handles.contains_key(preferred) {
        Some(preferred)
    } else {
        None
    }
}

fn next_dynamic_tab_id(spec: &ShellSpec, view_id: &str) -> String {
    let base = format!("plugin-{}", view_id.replace('.', "-"));
    if !tab_id_exists(spec, &base) {
        return base;
    }
    let mut index = 2usize;
    loop {
        let candidate = format!("{base}-{index}");
        if !tab_id_exists(spec, &candidate) {
            return candidate;
        }
        index += 1;
    }
}

fn tab_id_exists(spec: &ShellSpec, tab_id: &str) -> bool {
    all_tabs(spec).any(|tab| tab.id == tab_id)
}

fn all_tabs<'a>(spec: &'a ShellSpec) -> Box<dyn Iterator<Item = &'a crate::spec::TabSpec> + 'a> {
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
    node: &'a WorkbenchNodeSpec,
) -> Box<dyn Iterator<Item = &'a crate::spec::TabSpec> + 'a> {
    match node {
        WorkbenchNodeSpec::Group(group) => Box::new(group.tabs.iter()),
        WorkbenchNodeSpec::Split { children, .. } => {
            Box::new(children.iter().flat_map(|child| workbench_tabs(child)))
        }
    }
}

fn find_group_mut<'a>(spec: &'a mut ShellSpec, group_id: &str) -> Option<&'a mut TabGroupSpec> {
    if spec.left_panel.id == group_id {
        return Some(&mut spec.left_panel);
    }
    if spec.right_panel.id == group_id {
        return Some(&mut spec.right_panel);
    }
    if spec.bottom_panel.id == group_id {
        return Some(&mut spec.bottom_panel);
    }
    find_group_mut_in_workbench(&mut spec.workbench, group_id)
}

fn find_group_mut_in_workbench<'a>(
    node: &'a mut WorkbenchNodeSpec,
    group_id: &str,
) -> Option<&'a mut TabGroupSpec> {
    match node {
        WorkbenchNodeSpec::Group(group) => (group.id == group_id).then_some(group),
        WorkbenchNodeSpec::Split { children, .. } => children
            .iter_mut()
            .find_map(|child| find_group_mut_in_workbench(child, group_id)),
    }
}
