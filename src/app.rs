use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box as GtkBox, Orientation, Paned};

use crate::shell::workbench_custom::{self, SplitPreviewSide};
use crate::studio::text_tab;
use crate::theme;

pub fn build(application: &Application) {
    theme::load();

    let window = ApplicationWindow::builder()
        .application(application)
        .title("Maruzzella")
        .default_width(1600)
        .default_height(980)
        .build();
    window.add_css_class("app-window");

    let root = GtkBox::new(Orientation::Vertical, 0);
    root.add_css_class("app-root");

    let shell = build_shell();
    root.append(&shell);
    window.set_child(Some(&root));
    window.present();
}

fn build_shell() -> gtk::Widget {
    let left_tabs = vec![
        text_tab("navigation", "tool-left", "Navigation", "Anonymous shell navigation goes here.", false),
        text_tab("library", "tool-left", "Library", "A product can mount its own content here.", false),
    ];
    let right_tabs = vec![
        text_tab("inspector", "tool-right", "Inspector", "Selection-aware details live here.", false),
        text_tab("outline", "tool-right", "Outline", "Structure and metadata panels fit here.", false),
    ];
    let bottom_tabs = vec![
        text_tab("logs", "tool-bottom", "Logs", "Runtime output, tasks, and traces.", false),
        text_tab("problems", "tool-bottom", "Problems", "Validation and build output.", false),
    ];
    let editor_a_tabs = vec![
        text_tab("overview", "workbench-a", "Overview", "Maruzzella is a neutral desktop shell host.", false),
        text_tab("notes", "workbench-a", "Notes", "Drop any product-specific editor or view into the center workbench.", true),
        text_tab("scratch", "workbench-a", "Scratch", "This area is now fully custom and no longer backed by GtkNotebook.", true),
    ];
    let editor_b_tabs = vec![
        text_tab("automation", "workbench-b", "Automation", "Tooling and workflows can sit in adjacent workbench groups.", false),
        text_tab("chat", "workbench-b", "Chat", "This is only placeholder content for the extraction pass.", true),
    ];

    let left = workbench_custom::build_group("tool-left", &left_tabs, Some("navigation"));
    let right = workbench_custom::build_group("tool-right", &right_tabs, Some("inspector"));
    let bottom = workbench_custom::build_group("tool-bottom", &bottom_tabs, Some("logs"));
    let group_a = workbench_custom::build_group("workbench-a", &editor_a_tabs, Some("overview"));
    let group_b = workbench_custom::build_group("workbench-b", &editor_b_tabs, Some("automation"));

    let center_split = Paned::new(Orientation::Horizontal);
    center_split.set_wide_handle(true);
    center_split.set_resize_start_child(true);
    center_split.set_resize_end_child(true);
    center_split.set_start_child(Some(&group_a.root));
    center_split.set_end_child(Some(&group_b.root));
    center_split.set_position(760);

    let horizontal = Paned::new(Orientation::Horizontal);
    horizontal.set_wide_handle(true);
    horizontal.set_resize_start_child(true);
    horizontal.set_resize_end_child(true);
    horizontal.set_start_child(Some(&left.root));
    horizontal.set_end_child(Some(&center_split));
    horizontal.set_position(280);

    let vertical = Paned::new(Orientation::Vertical);
    vertical.set_wide_handle(true);
    vertical.set_resize_start_child(true);
    vertical.set_resize_end_child(true);
    vertical.set_start_child(Some(&horizontal));
    vertical.set_end_child(Some(&bottom.root));
    vertical.set_position(720);

    let outer = Paned::new(Orientation::Horizontal);
    outer.set_wide_handle(true);
    outer.set_resize_start_child(true);
    outer.set_resize_end_child(true);
    outer.set_start_child(Some(&vertical));
    outer.set_end_child(Some(&right.root));
    outer.set_position(1260);

    group_a.handle.set_split_drop_handler(|tab_id, side| {
        eprintln!("split drop requested: {tab_id} -> {:?}", side);
    });
    group_b.handle.set_split_drop_handler(|tab_id, side| {
        eprintln!("split drop requested: {tab_id} -> {:?}", side);
    });

    group_a.handle.set_tab_drop_handler(|tab_id| {
        eprintln!("tab drop finished in workbench-a: {tab_id}");
    });
    group_b.handle.set_tab_drop_handler(|tab_id| {
        eprintln!("tab drop finished in workbench-b: {tab_id}");
    });

    let _ = SplitPreviewSide::Left;
    outer.upcast::<gtk::Widget>()
}
