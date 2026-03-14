pub mod tabbed_panel;
pub mod workbench_custom;

use gtk::prelude::*;
use gtk::{Box as GtkBox, Label, Orientation, PolicyType, ScrolledWindow, Separator, Widget};

pub fn pane_container(title: &str, pane_class: &str) -> (GtkBox, GtkBox) {
    let pane = GtkBox::new(Orientation::Vertical, 0);
    pane.add_css_class("workspace-pane");
    pane.add_css_class(pane_class);

    let header = GtkBox::new(Orientation::Horizontal, 0);
    header.add_css_class("panel-header");

    let title_label = Label::new(Some(title));
    title_label.set_xalign(0.0);
    title_label.add_css_class("panel-title");
    header.append(&title_label);

    pane.append(&header);
    pane.append(&Separator::new(Orientation::Horizontal));

    let content = GtkBox::new(Orientation::Vertical, 0);
    content.set_hexpand(true);
    content.set_vexpand(true);
    pane.append(&content);

    (pane, content)
}

pub fn bare_pane_container(pane_class: &str) -> (GtkBox, GtkBox) {
    let pane = GtkBox::new(Orientation::Vertical, 0);
    pane.add_css_class("workspace-pane");
    pane.add_css_class(pane_class);

    let content = GtkBox::new(Orientation::Vertical, 0);
    content.set_hexpand(true);
    content.set_vexpand(true);
    pane.append(&content);

    (pane, content)
}

pub fn section_title(text: &str) -> Label {
    let label = Label::new(Some(text));
    label.set_xalign(0.0);
    label.add_css_class("section-title");
    label
}

pub fn scrolled<W: IsA<Widget>>(widget: &W) -> ScrolledWindow {
    let scrolled = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .hscrollbar_policy(PolicyType::Automatic)
        .vscrollbar_policy(PolicyType::Automatic)
        .child(widget)
        .build();
    scrolled.add_css_class("panel-scroller");
    scrolled
}
