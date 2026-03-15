use gtk::gio;
use gtk::prelude::*;
use gtk::{
    Align, Box as GtkBox, Button, Entry, Image, Label, Orientation, PopoverMenuBar,
};

use crate::commands::CommandRegistry;
use crate::spec::{command_name, menu_action_ref, MenuItemSpec, ShellSpec, ToolbarItemSpec};

pub struct TopBar {
    pub root: GtkBox,
}

pub fn build(spec: &ShellSpec) -> TopBar {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.add_css_class("topbar-shell");

    let menu_model = build_menu_model(spec);
    let menu_bar = PopoverMenuBar::from_model(Some(&menu_model));
    menu_bar.add_css_class("menu-bar");
    root.append(&menu_bar);

    let toolbar = GtkBox::new(Orientation::Horizontal, 10);
    toolbar.add_css_class("studio-toolbar");

    let primary_group = GtkBox::new(Orientation::Horizontal, 10);
    primary_group.add_css_class("toolbar-group");
    primary_group.add_css_class("toolbar-group-primary");

    let title_chip = GtkBox::new(Orientation::Horizontal, 0);
    title_chip.add_css_class("toolbar-title-chip");
    let title = Label::new(Some(&spec.title));
    title.add_css_class("toolbar-title");
    title_chip.append(&title);
    primary_group.append(&title_chip);

    let search = Entry::new();
    search.add_css_class("toolbar-search");
    search.set_placeholder_text(Some(&spec.search_placeholder));
    primary_group.append(&search);
    toolbar.append(&primary_group);

    let spacer = GtkBox::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    toolbar.append(&spacer);

    let actions_group = GtkBox::new(Orientation::Horizontal, 6);
    actions_group.add_css_class("toolbar-group");
    actions_group.add_css_class("toolbar-group-actions");

    let leading = spec.toolbar_items.iter().filter(|item| !item.secondary);
    for item in leading {
        actions_group.append(&action_bar_item_button(item));
    }
    toolbar.append(&actions_group);

    let utility_group = GtkBox::new(Orientation::Horizontal, 4);
    utility_group.add_css_class("toolbar-group");
    utility_group.add_css_class("toolbar-utility-group");
    for item in spec.toolbar_items.iter().filter(|item| item.secondary) {
        utility_group.append(&action_bar_item_button(item));
    }
    toolbar.append(&utility_group);

    let status_cluster = GtkBox::new(Orientation::Horizontal, 6);
    status_cluster.set_halign(Align::End);
    status_cluster.add_css_class("toolbar-status-cluster");
    let meta = Label::new(Some(&spec.status_text));
    meta.add_css_class("toolbar-status");
    status_cluster.append(&meta);
    toolbar.append(&status_cluster);

    root.append(&toolbar);
    TopBar { root }
}

fn action_bar_item_button(item: &ToolbarItemSpec) -> Button {
    let action_ref = menu_action_ref(&item.command_id);
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
        submenu.append(Some(&item.label), Some(&menu_action_ref(&item.command_id)));
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
                handler();
            } else {
                eprintln!("unhandled command: {title}");
            }
        });
        window.add_action(&simple);
    }
}
