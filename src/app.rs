use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box as GtkBox, Orientation, Paned};

use crate::shell::topbar;
use crate::shell::workbench_custom;
use crate::spec::{default_shell_spec, ShellSpec, SplitAxis, TabGroupSpec, WorkbenchNodeSpec};
use crate::theme;

pub fn build(application: &Application) {
    theme::load();

    let spec = default_shell_spec();
    let window = ApplicationWindow::builder()
        .application(application)
        .title(&spec.title)
        .default_width(1600)
        .default_height(980)
        .build();
    window.add_css_class("app-window");
    topbar::install_actions(&window, &spec);

    let root = GtkBox::new(Orientation::Vertical, 0);
    root.add_css_class("app-root");
    root.append(&topbar::build(&spec).root);
    root.append(&build_shell(&spec));
    window.set_child(Some(&root));
    window.present();
}

fn build_shell(spec: &ShellSpec) -> gtk::Widget {
    let left = build_group(&spec.left_panel);
    let right = build_group(&spec.right_panel);
    let bottom = build_group(&spec.bottom_panel);
    let workbench = build_workbench_node(&spec.workbench);

    let horizontal = Paned::new(Orientation::Horizontal);
    horizontal.set_wide_handle(true);
    horizontal.set_resize_start_child(true);
    horizontal.set_resize_end_child(true);
    horizontal.set_start_child(Some(&left.root));
    horizontal.set_end_child(Some(&workbench));
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
    outer.upcast::<gtk::Widget>()
}

fn build_group(group: &TabGroupSpec) -> workbench_custom::BuiltCustomWorkbenchGroup {
    workbench_custom::build_group(&group.id, &group.tabs, group.active_tab_id.as_deref())
}

fn build_workbench_node(node: &WorkbenchNodeSpec) -> gtk::Widget {
    match node {
        WorkbenchNodeSpec::Group(group) => build_group(group).root.upcast::<gtk::Widget>(),
        WorkbenchNodeSpec::Split { axis, children } => {
            let mut child_widgets = children.iter().map(build_workbench_node).collect::<Vec<_>>();
            let first = child_widgets.remove(0);
            let mut current = first;
            for child in child_widgets {
                let paned = Paned::new(match axis {
                    SplitAxis::Horizontal => Orientation::Horizontal,
                    SplitAxis::Vertical => Orientation::Vertical,
                });
                paned.set_wide_handle(true);
                paned.set_resize_start_child(true);
                paned.set_resize_end_child(true);
                paned.set_start_child(Some(&current));
                paned.set_end_child(Some(&child));
                current = paned.upcast::<gtk::Widget>();
            }
            current
        }
    }
}
