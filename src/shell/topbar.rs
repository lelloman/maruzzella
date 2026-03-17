use gtk::gio;
use gtk::prelude::*;
use gtk::{Box as GtkBox, Button, Entry, Image, Label, Orientation, PopoverMenuBar};

use crate::commands::CommandRegistry;
use crate::spec::{command_name, menu_action_ref, MenuItemSpec, ShellSpec, ToolbarItemSpec};

pub struct TopBar {
    pub root: GtkBox,
}

pub fn build(spec: &ShellSpec) -> TopBar {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.add_css_class("topbar-shell");

    let masthead = GtkBox::new(Orientation::Horizontal, 12);
    masthead.add_css_class("topbar-masthead");

    let menu_model = build_menu_model(spec);
    let menu_bar = PopoverMenuBar::from_model(Some(&menu_model));
    menu_bar.add_css_class("menu-bar");
    masthead.append(&menu_bar);

    let masthead_spacer = GtkBox::new(Orientation::Horizontal, 0);
    masthead_spacer.set_hexpand(true);
    masthead.append(&masthead_spacer);
    root.append(&masthead);

    let toolbar = GtkBox::new(Orientation::Horizontal, 12);
    toolbar.add_css_class("studio-toolbar");

    let search_cluster = GtkBox::new(Orientation::Horizontal, 0);
    search_cluster.add_css_class("toolbar-search-cluster");
    search_cluster.set_hexpand(true);
    let search = Entry::new();
    search.add_css_class("toolbar-search");
    search.set_hexpand(true);
    search.set_placeholder_text(Some(&spec.search_placeholder));
    search_cluster.append(&search);
    toolbar.append(&search_cluster);

    let actions_group = GtkBox::new(Orientation::Horizontal, 8);
    actions_group.add_css_class("toolbar-actions");
    for item in spec.toolbar_items.iter().filter(|item| !item.secondary) {
        actions_group.append(&action_bar_item_button(item));
    }
    toolbar.append(&actions_group);

    let utility_group = GtkBox::new(Orientation::Horizontal, 6);
    utility_group.add_css_class("toolbar-utility-group");
    for item in spec.toolbar_items.iter().filter(|item| item.secondary) {
        utility_group.append(&action_bar_item_button(item));
    }
    toolbar.append(&utility_group);

    root.append(&toolbar);
    TopBar { root }
}

fn action_bar_item_button(item: &ToolbarItemSpec) -> Button {
    let action_ref = menu_action_ref(&item.id);
    match (&item.icon_name, &item.label) {
        (Some(icon_name), Some(label)) => toolbar_button(icon_name, label, &action_ref),
        (Some(icon_name), None) => icon_button(icon_name, &action_ref, &item.id),
        (None, Some(label)) => toolbar_button("applications-system-symbolic", label, &action_ref),
        (None, None) => toolbar_button("applications-system-symbolic", &item.id, &action_ref),
    }
}

fn toolbar_button(icon_name: &str, label: &str, action_name: &str) -> Button {
    let button = Button::new();
    button.add_css_class("toolbar-button");
    button.set_action_name(Some(action_name));

    let content = GtkBox::new(Orientation::Horizontal, 6);
    let icon = Image::from_icon_name(icon_name);
    icon.set_icon_size(gtk::IconSize::Normal);
    content.append(&icon);
    let text = Label::new(Some(label));
    text.add_css_class("toolbar-button-label");
    content.append(&text);
    button.set_child(Some(&content));
    button
}

fn icon_button(icon_name: &str, action_name: &str, tooltip: &str) -> Button {
    let button = Button::new();
    button.add_css_class("toolbar-icon-button");
    button.set_action_name(Some(action_name));
    button.set_tooltip_text(Some(tooltip));
    let icon = Image::from_icon_name(icon_name);
    icon.set_icon_size(gtk::IconSize::Normal);
    button.set_child(Some(&icon));
    button
}

fn build_menu_model(spec: &ShellSpec) -> gio::Menu {
    let menu = gio::Menu::new();
    for root in &spec.menu_roots {
        let submenu = submenu(
            &spec
                .menu_items
                .iter()
                .filter(|item| item.root_id == root.id)
                .cloned()
                .collect::<Vec<_>>(),
        );
        menu.append_submenu(Some(&root.label), &submenu);
    }
    menu
}

fn submenu(items: &[MenuItemSpec]) -> gio::Menu {
    let submenu = gio::Menu::new();
    for item in items {
        submenu.append(Some(&item.label), Some(&menu_action_ref(&item.id)));
    }
    submenu
}

pub fn install_actions(
    window: &gtk::ApplicationWindow,
    spec: &ShellSpec,
    registry: &CommandRegistry,
) {
    for command in &spec.commands {
        let simple = gio::SimpleAction::new(&command_name(&command.id), None);
        let handler = registry.handler_for(&command.id);
        let title = command.title.clone();
        simple.connect_activate(move |_, _| {
            if let Some(handler) = handler.as_ref() {
                handler(&[]);
            } else {
                eprintln!("unhandled command: {title}");
            }
        });
        window.add_action(&simple);
    }

    for action_id in spec
        .menu_items
        .iter()
        .map(|item| item.id.as_str())
        .chain(spec.toolbar_items.iter().map(|item| item.id.as_str()))
    {
        let Some(handler) = registry.handler_for(action_id) else {
            continue;
        };
        let simple = gio::SimpleAction::new(&command_name(action_id), None);
        simple.connect_activate(move |_, _| {
            handler(&[]);
        });
        window.add_action(&simple);
    }
}
