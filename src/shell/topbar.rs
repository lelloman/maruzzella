use gtk::gio;
use gtk::prelude::*;
use gtk::{
    Box as GtkBox, Button, Entry, EventControllerMotion, Fixed, Image, Label, Orientation,
    Overlay, PopoverMenuBar,
};

use crate::commands::CommandRegistry;
use crate::spec::{
    command_name, menu_action_ref, MenuItemSpec, ShellSpec, ToolbarDisplayMode, ToolbarItemSpec,
};

struct IconButtonTooltip {
    button: Button,
    label: Label,
}

pub struct TopBar {
    pub root: GtkBox,
    pub search: Entry,
    tooltips: Vec<IconButtonTooltip>,
}

impl TopBar {
    pub fn install_tooltip_overlay(&self, app_overlay: &Overlay) {
        if self.tooltips.is_empty() {
            return;
        }

        let fixed = Fixed::new();
        fixed.set_can_target(false);
        fixed.set_overflow(gtk::Overflow::Visible);

        for tooltip in &self.tooltips {
            tooltip.label.set_visible(false);
            tooltip.label.set_can_target(false);
            fixed.put(&tooltip.label, 0.0, 0.0);
        }

        app_overlay.add_overlay(&fixed);

        for tooltip in &self.tooltips {
            let button = tooltip.button.clone();
            let label = tooltip.label.clone();
            let fixed_ref = fixed.clone();

            let hover = EventControllerMotion::new();
            let label_enter = label.clone();
            let button_enter = button.clone();
            let fixed_enter = fixed_ref.clone();
            hover.connect_enter(move |_, _, _| {
                if let Some((bx, by)) = button_enter.translate_coordinates(&fixed_enter, 0.0, 0.0)
                {
                    let bw = button_enter.width() as f64;
                    let bh = button_enter.height() as f64;
                    let lw = label_enter.preferred_size().1.width() as f64;
                    let x = bx + (bw - lw) / 2.0;
                    let y = by + bh + 4.0;
                    fixed_enter.move_(&label_enter, x, y);
                }
                label_enter.set_visible(true);
            });
            let label_leave = label.clone();
            hover.connect_leave(move |_| {
                label_leave.set_visible(false);
            });
            button.add_controller(hover);
        }
    }
}

pub fn build(spec: &ShellSpec) -> TopBar {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.add_css_class("topbar-shell");

    let masthead = GtkBox::new(Orientation::Horizontal, 12);
    masthead.add_css_class("topbar-masthead");

    let menu_model = build_menu_model(spec);
    let menu_bar = PopoverMenuBar::from_model(Some(&menu_model));
    menu_bar.add_css_class("menu-bar");
    menu_bar.set_hexpand(true);
    masthead.append(&menu_bar);
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

    let mut tooltips = Vec::new();

    let actions_group = GtkBox::new(Orientation::Horizontal, 8);
    actions_group.add_css_class("toolbar-actions");
    for item in spec.toolbar_items.iter().filter(|item| !item.secondary) {
        actions_group.append(&action_bar_item_button(item, &mut tooltips));
    }
    toolbar.append(&actions_group);

    let utility_group = GtkBox::new(Orientation::Horizontal, 6);
    utility_group.add_css_class("toolbar-utility-group");
    for item in spec.toolbar_items.iter().filter(|item| item.secondary) {
        utility_group.append(&action_bar_item_button(item, &mut tooltips));
    }
    toolbar.append(&utility_group);

    root.append(&toolbar);
    TopBar {
        root,
        search,
        tooltips,
    }
}

fn action_bar_item_button(
    item: &ToolbarItemSpec,
    tooltips: &mut Vec<IconButtonTooltip>,
) -> Button {
    let action_ref = menu_action_ref(&item.id);
    match item.display_mode {
        ToolbarDisplayMode::IconOnly => {
            let icon_name = item
                .icon_name
                .as_deref()
                .unwrap_or_else(|| panic!("toolbar item '{}' is IconOnly but has no icon", item.id));
            let tooltip = item.label.as_deref().unwrap_or(&item.id);
            icon_button(icon_name, &action_ref, tooltip, tooltips)
        }
        ToolbarDisplayMode::IconAndText => {
            let label = item.label.as_deref().unwrap_or(&item.id);
            let icon_name = item
                .icon_name
                .as_deref()
                .unwrap_or("applications-system-symbolic");
            toolbar_button(icon_name, label, &action_ref)
        }
        ToolbarDisplayMode::TextOnly => {
            let label = item.label.as_deref().unwrap_or(&item.id);
            text_button(label, &action_ref)
        }
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

fn text_button(label: &str, action_name: &str) -> Button {
    let button = Button::new();
    button.add_css_class("toolbar-button");
    button.set_action_name(Some(action_name));
    let text = Label::new(Some(label));
    text.add_css_class("toolbar-button-label");
    button.set_child(Some(&text));
    button
}

fn icon_button(
    icon_name: &str,
    action_name: &str,
    tooltip: &str,
    tooltips: &mut Vec<IconButtonTooltip>,
) -> Button {
    let button = Button::new();
    button.add_css_class("toolbar-icon-button");
    button.set_action_name(Some(action_name));

    let icon = Image::from_icon_name(icon_name);
    icon.set_icon_size(gtk::IconSize::Normal);
    button.set_child(Some(&icon));

    let tip_label = Label::new(Some(tooltip));
    tip_label.add_css_class("icon-button-tooltip-label");

    tooltips.push(IconButtonTooltip {
        button: button.clone(),
        label: tip_label,
    });

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
    let mut section = gio::Menu::new();
    for item in items {
        if item.command_id.is_empty() {
            if section.n_items() > 0 {
                submenu.append_section(None, &section);
                section = gio::Menu::new();
            }
            continue;
        }
        section.append(Some(&item.label), Some(&menu_action_ref(&item.id)));
    }
    if section.n_items() > 0 {
        submenu.append_section(None, &section);
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
