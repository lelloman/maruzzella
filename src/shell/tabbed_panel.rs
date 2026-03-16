use std::collections::HashMap;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{
    Box as GtkBox, Button, Entry, Label, ListBox, Notebook, Orientation, SelectionMode, TextBuffer,
    TextView,
};

use crate::plugins::PluginRuntime;
use crate::spec::{PanelContentKind, TabSpec};

use super::{bare_pane_container, scrolled, section_title};

pub struct BuiltTabPage {
    pub tab_id: String,
    pub widget: gtk::Widget,
    pub tab_label: Label,
    pub tab_header: gtk::Widget,
    pub close_button: Option<Button>,
    pub buffer: Option<TextBuffer>,
    pub list: Option<ListBox>,
    pub entry: Option<Entry>,
    pub labels: HashMap<String, Label>,
}

pub struct BuiltNotebook {
    pub root: GtkBox,
    pub notebook: Notebook,
    pub page_indexes: HashMap<String, u32>,
    pub tab_labels: HashMap<String, Label>,
    pub close_buttons: HashMap<String, Button>,
    pub buffers: HashMap<String, TextBuffer>,
    pub lists: HashMap<String, ListBox>,
    pub entries: HashMap<String, Entry>,
    pub labels: HashMap<String, Label>,
}

pub fn build(
    css_class: &str,
    tabs: &[TabSpec],
    plugin_runtime: Option<Rc<PluginRuntime>>,
) -> BuiltNotebook {
    let (root, content) = bare_pane_container(css_class);
    let notebook = Notebook::new();
    notebook.add_css_class("workbench-tabs");
    match css_class {
        "workbench" => notebook.add_css_class("editor-tabs"),
        "console-pane" => notebook.add_css_class("bottom-tabs"),
        _ => notebook.add_css_class("tool-tabs"),
    }

    let mut page_indexes = HashMap::new();
    let mut tab_labels = HashMap::new();
    let mut close_buttons = HashMap::new();
    let mut buffers = HashMap::new();
    let mut lists = HashMap::new();
    let mut entries = HashMap::new();
    let mut labels = HashMap::new();

    for (index, tab) in tabs.iter().enumerate() {
        page_indexes.insert(tab.id.to_string(), index as u32);
        let page = build_tab_page(css_class, tab, plugin_runtime.as_ref());
        page.widget
            .set_widget_name(&format!("tab-page:{}", page.tab_id));
        notebook.append_page(&page.widget, Some(&page.tab_header));
        tab_labels.insert(page.tab_id.clone(), page.tab_label);
        if let Some(close_button) = page.close_button {
            close_buttons.insert(tab.id.to_string(), close_button);
        }
        if let Some(buffer) = page.buffer {
            buffers.insert(tab.id.to_string(), buffer);
        }
        if let Some(list) = page.list {
            lists.insert(tab.id.to_string(), list);
        }
        if let Some(entry) = page.entry {
            entries.insert(tab.id.to_string(), entry);
        }
        labels.extend(page.labels);
    }
    content.append(&notebook);

    BuiltNotebook {
        root,
        notebook,
        page_indexes,
        tab_labels,
        close_buttons,
        buffers,
        lists,
        entries,
        labels,
    }
}

pub fn build_tab_page(
    css_class: &str,
    tab: &TabSpec,
    plugin_runtime: Option<&Rc<PluginRuntime>>,
) -> BuiltTabPage {
    let mut buffer = None;
    let mut list = None;
    let mut entry = None;
    let mut labels = HashMap::new();
    let mut close_button = None;
    let widget = if let Some(plugin_view_id) = tab.plugin_view_id.as_deref() {
        build_plugin_widget(tab, plugin_view_id, plugin_runtime)
    } else {
        match tab.content_kind {
            PanelContentKind::NavigationList | PanelContentKind::IdentityList => {
                let built_list = ListBox::new();
                built_list.set_selection_mode(SelectionMode::Single);
                built_list.add_css_class("dense-list");
                list = Some(built_list.clone());
                scrolled(&built_list).upcast::<gtk::Widget>()
            }
            PanelContentKind::InspectorDetails => {
                let inspector_box = GtkBox::new(Orientation::Vertical, 10);
                inspector_box.add_css_class("inspector-content");

                let summary_title = section_title("Selection");
                let summary = value_label("No identity selected");
                let identifiers_title = section_title("Identifiers");
                let identity_hash = value_label("-");
                let destination = value_label("-");
                let runtime_title = section_title("Runtime");
                let status = value_label("Idle");

                labels.insert("selection.summary".to_string(), summary.clone());
                labels.insert("selection.identity_hash".to_string(), identity_hash.clone());
                labels.insert("selection.destination".to_string(), destination.clone());
                labels.insert("selection.status".to_string(), status.clone());

                inspector_box.append(&summary_title);
                inspector_box.append(&summary);
                inspector_box.append(&identifiers_title);
                inspector_box.append(&field("Identity hash", &identity_hash));
                inspector_box.append(&field("Destination", &destination));
                inspector_box.append(&runtime_title);
                inspector_box.append(&field("Status", &status));

                scrolled(&inspector_box).upcast::<gtk::Widget>()
            }
            PanelContentKind::CommandList => {
                let box_ = GtkBox::new(Orientation::Vertical, 8);
                box_.add_css_class("inspector-content");
                let search = Entry::new();
                search.set_placeholder_text(Some("Filter commands"));
                search.add_css_class("command-entry");
                let built_list = ListBox::new();
                built_list.set_selection_mode(SelectionMode::None);
                built_list.add_css_class("dense-list");
                entry = Some(search.clone());
                list = Some(built_list.clone());
                box_.append(&search);
                box_.append(&scrolled(&built_list));
                box_.upcast::<gtk::Widget>()
            }
            PanelContentKind::TextBuffer => {
                let built_buffer = TextBuffer::new(None);
                built_buffer.set_text(&tab.placeholder);
                let view = TextView::builder()
                    .editable(false)
                    .monospace(true)
                    .buffer(&built_buffer)
                    .build();
                if css_class == "console-pane" {
                    view.add_css_class("console-view");
                }
                buffer = Some(built_buffer);
                scrolled(&view).upcast::<gtk::Widget>()
            }
        }
    };
    let tab_label = Label::new(Some(&tab.title));
    tab_label.add_css_class("tab-label");
    let tab_header = GtkBox::new(Orientation::Horizontal, 6);
    tab_header.add_css_class("tab-header");
    tab_header.append(&tab_label);
    if tab.closable {
        let button = Button::new();
        button.set_icon_name("window-close-symbolic");
        button.add_css_class("tab-close-button");
        button.set_focus_on_click(false);
        tab_header.append(&button);
        close_button = Some(button);
    }

    BuiltTabPage {
        tab_id: tab.id.clone(),
        widget,
        tab_label,
        tab_header: tab_header.upcast::<gtk::Widget>(),
        close_button,
        buffer,
        list,
        entry,
        labels,
    }
}

fn build_plugin_widget(
    tab: &TabSpec,
    plugin_view_id: &str,
    plugin_runtime: Option<&Rc<PluginRuntime>>,
) -> gtk::Widget {
    let Some(plugin_runtime) = plugin_runtime else {
        return plugin_fallback_widget(
            &format!(
                "Plugin view '{plugin_view_id}' is configured for this tab, but no plugin runtime is active.\n\n{}",
                tab.placeholder
            ),
        );
    };

    match plugin_runtime.create_view(plugin_view_id, tab.instance_key.as_deref(), &tab.payload) {
        Ok(widget) => widget,
        Err(error) => plugin_fallback_widget(&format!(
            "Failed to build plugin view '{plugin_view_id}': {error:?}\n\n{}",
            tab.placeholder
        )),
    }
}

fn plugin_fallback_widget(message: &str) -> gtk::Widget {
    let buffer = TextBuffer::new(None);
    buffer.set_text(message);
    let view = TextView::builder()
        .editable(false)
        .monospace(true)
        .buffer(&buffer)
        .build();
    scrolled(&view).upcast::<gtk::Widget>()
}

fn field(label_text: &str, value: &Label) -> GtkBox {
    let row = GtkBox::new(Orientation::Vertical, 2);
    let label = Label::new(Some(label_text));
    label.set_xalign(0.0);
    label.add_css_class("muted");
    row.append(&label);
    row.append(value);
    row
}

fn value_label(text: &str) -> Label {
    let label = Label::new(Some(text));
    label.set_xalign(0.0);
    label.add_css_class("mono");
    label.add_css_class("inspector-value");
    label.set_wrap(true);
    label
}
