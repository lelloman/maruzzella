#![allow(dead_code)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{
    Align, Box as GtkBox, Button, Entry, EventControllerMotion, Fixed, GestureClick, GestureDrag,
    Label, ListBox, Orientation, Overlay, PolicyType, Revealer, ScrolledWindow, Stack,
    StackTransitionType, TextBuffer, Widget,
};

use crate::plugins::PluginRuntime;
use crate::spec::TabSpec;

use super::tabbed_panel::{self, BuiltTabPage};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SplitPreviewSide {
    Left,
    Right,
    Bottom,
}

#[derive(Clone)]
pub struct CustomWorkbenchGroupHandle {
    group_id: String,
    root: Overlay,
    tab_strip: GtkBox,
    tab_scroller: ScrolledWindow,
    stack: Stack,
    preview: Revealer,
    preview_inner: GtkBox,
    drag_layer: Rc<RefCell<Fixed>>,
    headers: Rc<RefCell<HashMap<String, Widget>>>,
    tab_labels: Rc<RefCell<HashMap<String, Label>>>,
    page_names: Rc<RefCell<HashMap<String, String>>>,
    order: Rc<RefCell<Vec<String>>>,
    active_tab_id: Rc<RefCell<Option<String>>>,
    drag_state: Rc<RefCell<Option<DragState>>>,
    pointer_position: Rc<RefCell<Option<(f64, f64)>>>,
    external_pointer_position: Rc<RefCell<Option<Rc<RefCell<Option<(f64, f64)>>>>>>,
    external_coordinate_target: Rc<RefCell<Option<Widget>>>,
    split_hover: Rc<RefCell<Option<SplitPreviewSide>>>,
    split_drop_handler: Rc<RefCell<Option<Rc<dyn Fn(String, SplitPreviewSide)>>>>,
    tab_drop_handler: Rc<RefCell<Option<Rc<dyn Fn(String)>>>>,
    drag_hover_handler: Rc<RefCell<Option<Rc<dyn Fn(String, f64, f64, i32)>>>>,
    drag_end_handler: Rc<RefCell<Option<Rc<dyn Fn()>>>>,
    active_changed_handler: Rc<RefCell<Option<Rc<dyn Fn(String)>>>>,
    drop_placeholder: GtkBox,
}

pub struct BuiltCustomWorkbenchGroup {
    pub root: Overlay,
    pub handle: CustomWorkbenchGroupHandle,
    pub tab_labels: HashMap<String, Label>,
    pub close_buttons: HashMap<String, Button>,
    pub buffers: HashMap<String, TextBuffer>,
    pub lists: HashMap<String, ListBox>,
    pub entries: HashMap<String, Entry>,
    pub labels: HashMap<String, Label>,
}

struct DragState {
    tab_id: String,
    source_index: usize,
    current_index: usize,
    pointer_offset_x: f64,
    pointer_offset_y: f64,
    origin_left: f64,
    origin_top: f64,
    drag_widget: Widget,
    width: i32,
    height: i32,
}

impl CustomWorkbenchGroupHandle {
    pub fn group_id(&self) -> &str {
        &self.group_id
    }

    pub fn widget(&self) -> Overlay {
        self.root.clone()
    }

    pub fn set_active_tab(&self, tab_id: &str) {
        let Some(page_name) = self.page_names.borrow().get(tab_id).cloned() else {
            return;
        };
        self.stack.set_visible_child_name(&page_name);
        *self.active_tab_id.borrow_mut() = Some(tab_id.to_string());
        self.refresh_active_header_state();
        if let Some(handler) = self.active_changed_handler.borrow().as_ref().cloned() {
            handler(tab_id.to_string());
        }
    }

    pub fn active_tab_id(&self) -> Option<String> {
        self.active_tab_id.borrow().clone()
    }

    pub fn tab_ids(&self) -> Vec<String> {
        self.order.borrow().clone()
    }

    pub fn current_pointer_position(&self) -> Option<(f64, f64)> {
        *self.pointer_position.borrow()
    }

    pub fn current_external_pointer_position(&self) -> Option<(f64, f64)> {
        self.external_pointer_position
            .borrow()
            .as_ref()
            .and_then(|tracker| *tracker.borrow())
    }

    pub fn set_external_pointer_tracker(&self, tracker: Rc<RefCell<Option<(f64, f64)>>>) {
        *self.external_pointer_position.borrow_mut() = Some(tracker);
    }

    pub fn set_external_coordinate_target<W: IsA<Widget>>(&self, target: &W) {
        *self.external_coordinate_target.borrow_mut() = Some(target.clone().upcast::<Widget>());
    }

    pub fn set_drag_layer(&self, drag_layer: &Fixed) {
        *self.drag_layer.borrow_mut() = drag_layer.clone();
    }

    pub fn strip_band_height(&self) -> f64 {
        self.tab_scroller.height() as f64 + 8.0
    }

    pub fn insertion_index_for_local_x(&self, dragged_tab_id: &str, local_x: f64) -> usize {
        self.insertion_index(dragged_tab_id, local_x)
    }

    pub fn bounds_in<W: IsA<Widget>>(&self, target: &W) -> Option<(f64, f64, f64, f64)> {
        let bounds = self.root.compute_bounds(target)?;
        Some((
            bounds.x() as f64,
            bounds.y() as f64,
            bounds.width() as f64,
            bounds.height() as f64,
        ))
    }

    pub fn append_page(&self, page: BuiltTabPage, make_active: bool) {
        let tab_id = page.tab_id.clone();
        let page_name = page_name(&tab_id);
        let header = page.tab_header.clone();
        let drag_title = page.tab_label.text().to_string();

        page.widget.set_hexpand(true);
        page.widget.set_vexpand(true);
        page.widget.set_halign(Align::Fill);
        page.widget.set_valign(Align::Fill);
        self.stack.add_named(&page.widget, Some(&page_name));
        self.tab_strip.append(&header);
        self.install_header_controllers(&header, &tab_id, &drag_title);

        self.headers.borrow_mut().insert(tab_id.clone(), header);
        self.tab_labels
            .borrow_mut()
            .insert(tab_id.clone(), page.tab_label.clone());
        self.page_names
            .borrow_mut()
            .insert(tab_id.clone(), page_name);
        self.order.borrow_mut().push(tab_id.clone());

        if make_active || self.active_tab_id.borrow().is_none() {
            self.set_active_tab(&tab_id);
        } else {
            self.refresh_active_header_state();
        }
    }

    pub fn remove_tab(&self, tab_id: &str) {
        if let Some(header) = self.headers.borrow_mut().remove(tab_id) {
            self.tab_strip.remove(&header);
        }
        self.tab_labels.borrow_mut().remove(tab_id);
        if let Some(page_name) = self.page_names.borrow_mut().remove(tab_id) {
            if let Some(child) = self.stack.child_by_name(&page_name) {
                self.stack.remove(&child);
            }
        }

        let mut order = self.order.borrow_mut();
        order.retain(|candidate| candidate != tab_id);
        let next_active = if self.active_tab_id.borrow().as_deref() == Some(tab_id) {
            order.first().cloned()
        } else {
            self.active_tab_id.borrow().clone()
        };
        drop(order);

        *self.active_tab_id.borrow_mut() = next_active.clone();
        if let Some(next_active) = next_active {
            self.set_active_tab(&next_active);
        } else {
            self.refresh_active_header_state();
        }
    }

    pub fn show_split_preview(&self, side: SplitPreviewSide) {
        self.preview_inner.remove_css_class("split-preview-left");
        self.preview_inner.remove_css_class("split-preview-right");
        self.preview_inner.remove_css_class("split-preview-bottom");
        self.preview_inner.add_css_class("workbench-split-preview");
        *self.split_hover.borrow_mut() = Some(side);
        match side {
            SplitPreviewSide::Left => {
                self.preview_inner.add_css_class("split-preview-left");
                self.preview.set_halign(Align::Start);
                self.preview.set_valign(Align::Fill);
                self.preview_inner.set_size_request(180, -1);
            }
            SplitPreviewSide::Right => {
                self.preview_inner.add_css_class("split-preview-right");
                self.preview.set_halign(Align::End);
                self.preview.set_valign(Align::Fill);
                self.preview_inner.set_size_request(180, -1);
            }
            SplitPreviewSide::Bottom => {
                self.preview_inner.add_css_class("split-preview-bottom");
                self.preview.set_halign(Align::Fill);
                self.preview.set_valign(Align::End);
                self.preview_inner.set_size_request(-1, 140);
            }
        }
        self.preview.set_reveal_child(true);
    }

    pub fn set_tab_title(&self, tab_id: &str, title: &str) {
        if let Some(label) = self.tab_labels.borrow().get(tab_id) {
            label.set_text(title);
        }
    }

    pub fn hide_split_preview(&self) {
        *self.split_hover.borrow_mut() = None;
        self.preview.set_reveal_child(false);
    }

    pub fn set_split_drop_handler<F>(&self, handler: F)
    where
        F: Fn(String, SplitPreviewSide) + 'static,
    {
        *self.split_drop_handler.borrow_mut() = Some(Rc::new(handler));
    }

    pub fn set_tab_drop_handler<F>(&self, handler: F)
    where
        F: Fn(String) + 'static,
    {
        *self.tab_drop_handler.borrow_mut() = Some(Rc::new(handler));
    }

    pub fn set_drag_hover_handler<F>(&self, handler: F)
    where
        F: Fn(String, f64, f64, i32) + 'static,
    {
        *self.drag_hover_handler.borrow_mut() = Some(Rc::new(handler));
    }

    pub fn set_drag_end_handler<F>(&self, handler: F)
    where
        F: Fn() + 'static,
    {
        *self.drag_end_handler.borrow_mut() = Some(Rc::new(handler));
    }

    pub fn set_active_changed_handler<F>(&self, handler: F)
    where
        F: Fn(String) + 'static,
    {
        *self.active_changed_handler.borrow_mut() = Some(Rc::new(handler));
    }

    pub fn show_drop_placeholder(&self, index: usize, width: i32) {
        self.drop_placeholder.set_size_request(width.max(48), 28);
        if self.drop_placeholder.parent().is_none() {
            self.tab_strip.append(&self.drop_placeholder);
        }
        if index == 0 {
            self.tab_strip
                .reorder_child_after(&self.drop_placeholder, None::<&Widget>);
        } else {
            let previous_id = self.order.borrow().get(index.saturating_sub(1)).cloned();
            if let Some(previous_id) = previous_id {
                if let Some(previous_header) = self.headers.borrow().get(&previous_id).cloned() {
                    self.tab_strip
                        .reorder_child_after(&self.drop_placeholder, Some(&previous_header));
                }
            }
        }
    }

    pub fn hide_drop_placeholder(&self) {
        if self.drop_placeholder.parent().is_some() {
            self.tab_strip.remove(&self.drop_placeholder);
        }
    }

    fn refresh_active_header_state(&self) {
        let active = self.active_tab_id.borrow().clone();
        for (tab_id, header) in self.headers.borrow().iter() {
            if active.as_deref() == Some(tab_id.as_str()) {
                header.add_css_class("active");
            } else {
                header.remove_css_class("active");
            }
        }
    }

    fn install_header_controllers(&self, header: &Widget, tab_id: &str, drag_title: &str) {
        install_header_activation(
            header,
            tab_id,
            self.stack.clone(),
            self.headers.clone(),
            self.active_tab_id.clone(),
            self.active_changed_handler.clone(),
        );

        let drag = GestureDrag::new();
        let handle = self.clone();
        let tab_id_string = tab_id.to_string();
        let drag_title = drag_title.to_string();
        drag.connect_drag_begin(move |_, start_x, start_y| {
            handle.begin_drag(&tab_id_string, &drag_title, start_x, start_y);
        });

        let handle = self.clone();
        drag.connect_drag_update(move |_, offset_x, offset_y| {
            handle.update_drag(offset_x, offset_y);
        });

        let handle = self.clone();
        drag.connect_drag_end(move |_, _, _| {
            handle.finish_drag();
        });
        header.add_controller(drag);
    }

    fn begin_drag(&self, tab_id: &str, drag_title: &str, start_x: f64, start_y: f64) {
        let Some(source_index) = self
            .order
            .borrow()
            .iter()
            .position(|candidate| candidate == tab_id)
        else {
            return;
        };
        let Some(header) = self.headers.borrow().get(tab_id).cloned() else {
            return;
        };
        if let Some(picked) = header.pick(start_x, start_y, gtk::PickFlags::DEFAULT) {
            if widget_or_ancestor_has_css_class(&picked, "tab-close-button") {
                return;
            }
        }

        let adjustment = self.tab_scroller.hadjustment().value();
        let scroller_allocation = self.tab_scroller.allocation();
        let header_allocation = header.allocation();
        let origin_left =
            scroller_allocation.x() as f64 + header_allocation.x() as f64 - adjustment;
        let origin_top = scroller_allocation.y() as f64 + header_allocation.y() as f64;
        let width = header.width().max(120);
        let height = header.height().max(32);
        let drag_widget = build_drag_widget(drag_title);
        drag_widget.set_size_request(width, height);
        drag_widget.add_css_class("dragging");
        self.drag_layer.borrow().put(&drag_widget, 0.0, 0.0);
        self.position_drag_widget_at(&drag_widget, origin_left, origin_top);
        drag_widget.set_visible(true);
        header.set_opacity(0.35);
        self.hide_split_preview();

        *self.drag_state.borrow_mut() = Some(DragState {
            tab_id: tab_id.to_string(),
            source_index,
            current_index: source_index,
            pointer_offset_x: start_x,
            pointer_offset_y: start_y,
            origin_left,
            origin_top,
            drag_widget,
            width,
            height,
        });
    }

    fn update_drag(&self, offset_x: f64, offset_y: f64) {
        let (
            tab_id,
            _width,
            height,
            pointer_offset_x,
            pointer_offset_y,
            origin_left,
            origin_top,
            drag_widget,
        ) = {
            let state_ref = self.drag_state.borrow();
            let Some(state) = state_ref.as_ref() else {
                return;
            };
            (
                state.tab_id.clone(),
                state.width,
                state.height,
                state.pointer_offset_x,
                state.pointer_offset_y,
                state.origin_left,
                state.origin_top,
                state.drag_widget.clone(),
            )
        };

        let (pointer_x, pointer_y, current_left, current_top, local_x, local_y) =
            if let (Some((x, y)), Some(target)) = (
                self.current_external_pointer_position(),
                self.external_coordinate_target.borrow().clone(),
            ) {
                let Some((group_x, group_y, _, _)) = self.bounds_in(&target) else {
                    return;
                };
                (
                    x,
                    y,
                    x - pointer_offset_x,
                    y - pointer_offset_y,
                    x - group_x,
                    y - group_y,
                )
            } else if let Some((x, y)) = *self.pointer_position.borrow() {
                (x, y, x - pointer_offset_x, y - pointer_offset_y, x, y)
            } else {
                (
                    origin_left + pointer_offset_x + offset_x,
                    origin_top + pointer_offset_y + offset_y,
                    origin_left + offset_x,
                    origin_top + offset_y,
                    pointer_offset_x + offset_x,
                    pointer_offset_y + offset_y,
                )
            };
        let target_center = local_x;
        let target_index = self.insertion_index(&tab_id, target_center);

        self.position_drag_widget_at(&drag_widget, current_left, current_top);
        self.autoscroll_for_x(pointer_x);
        if let Some(handler) = self.drag_hover_handler.borrow().as_ref().cloned() {
            handler(tab_id.clone(), pointer_x, pointer_y, height);
        }
        if self.is_in_reorder_band(local_y) && self.is_inside_horizontal_bounds(local_x) {
            self.hide_split_preview();
            self.reorder_to_index(&tab_id, target_index);
        } else if self.split_drop_handler.borrow().is_some() {
            if let Some(side) = self.split_side_for_pointer(local_x, local_y, height) {
                self.show_split_preview(side);
            } else {
                self.hide_split_preview();
            }
        } else {
            self.hide_split_preview();
        }
        if let Some(state) = self.drag_state.borrow_mut().as_mut() {
            state.current_index = target_index;
        }
    }

    fn finish_drag(&self) {
        let Some(state) = self.drag_state.borrow_mut().take() else {
            return;
        };
        let split_hover = *self.split_hover.borrow();
        if let Some(header) = self.headers.borrow().get(&state.tab_id) {
            header.set_opacity(1.0);
        }
        if let Some(handler) = self.drag_end_handler.borrow().as_ref().cloned() {
            handler();
        }
        self.hide_split_preview();
        self.drag_layer.borrow().remove(&state.drag_widget);
        if let Some(side) = split_hover {
            if let Some(handler) = self.split_drop_handler.borrow().as_ref().cloned() {
                handler(state.tab_id, side);
            }
        } else if let Some(handler) = self.tab_drop_handler.borrow().as_ref().cloned() {
            handler(state.tab_id);
        }
    }

    fn reorder_to_index(&self, tab_id: &str, target_index: usize) {
        let mut order = self.order.borrow_mut();
        let Some(current_index) = order.iter().position(|candidate| candidate == tab_id) else {
            return;
        };
        if current_index == target_index {
            return;
        }

        let tab_id_string = order.remove(current_index);
        let insert_at = target_index.min(order.len());
        order.insert(insert_at, tab_id_string);
        drop(order);

        let Some(header) = self.headers.borrow().get(tab_id).cloned() else {
            return;
        };
        if insert_at == 0 {
            self.tab_strip.reorder_child_after(&header, None::<&Widget>);
        } else {
            let order = self.order.borrow();
            let previous_id = order[insert_at - 1].clone();
            drop(order);
            if let Some(previous_header) = self.headers.borrow().get(&previous_id).cloned() {
                self.tab_strip
                    .reorder_child_after(&header, Some(&previous_header));
            }
        }
    }

    fn insertion_index(&self, dragged_tab_id: &str, target_center: f64) -> usize {
        let visible_tabs = self.visible_order_without_dragged(dragged_tab_id);

        for (index, candidate) in visible_tabs.iter().enumerate() {
            let header = {
                let headers = self.headers.borrow();
                headers.get(candidate).cloned()
            };
            let Some(header) = header else {
                continue;
            };
            let midpoint = self.header_midpoint(&header);
            if target_center < midpoint {
                return index;
            }
        }
        visible_tabs.len()
    }

    fn visible_order_without_dragged(&self, dragged_tab_id: &str) -> Vec<String> {
        self.order
            .borrow()
            .iter()
            .filter(|candidate| candidate.as_str() != dragged_tab_id)
            .cloned()
            .collect::<Vec<_>>()
    }

    fn header_x(&self, index: usize) -> f64 {
        let adjustment = self.tab_scroller.hadjustment().value();
        let mut x = -adjustment;
        let order = self.order.borrow();
        for candidate in order.iter().take(index) {
            if let Some(header) = self.headers.borrow().get(candidate) {
                x += header.width() as f64;
            }
        }
        x
    }

    fn header_midpoint(&self, header: &Widget) -> f64 {
        let adjustment = self.tab_scroller.hadjustment().value();
        let allocation = header.allocation();
        allocation.x() as f64 - adjustment + (allocation.width() as f64 / 2.0)
    }

    fn position_drag_widget_at(&self, drag_widget: &Widget, x: f64, y: f64) {
        self.drag_layer
            .borrow()
            .move_(drag_widget, x.max(0.0), y.max(0.0));
    }

    fn autoscroll_for_x(&self, x: f64) {
        let adjustment = self.tab_scroller.hadjustment();
        let page_size = adjustment.page_size();
        let lower = adjustment.lower();
        let upper = adjustment.upper();
        let edge = 36.0;
        let step = 24.0;

        if x < edge {
            adjustment.set_value((adjustment.value() - step).max(lower));
        } else if x > page_size - edge {
            adjustment.set_value((adjustment.value() + step).min((upper - page_size).max(lower)));
        }
    }

    fn is_in_reorder_band(&self, pointer_y: f64) -> bool {
        pointer_y >= 0.0 && pointer_y <= self.tab_scroller.height() as f64 + 8.0
    }

    fn is_inside_horizontal_bounds(&self, pointer_x: f64) -> bool {
        pointer_x >= 0.0 && pointer_x <= self.root.width() as f64
    }

    fn split_side_for_pointer(
        &self,
        pointer_x: f64,
        pointer_y: f64,
        drag_height: i32,
    ) -> Option<SplitPreviewSide> {
        let width = self.root.width().max(1) as f64;
        let height = self.root.height().max(1) as f64;
        let strip_band = self.tab_scroller.height() as f64;
        let left_right_edge = (width * 0.22).min(120.0);
        let bottom_edge = ((height - strip_band) * 0.24).min(140.0);
        let pointer_bottom = pointer_y + drag_height as f64;

        if pointer_x < 0.0 || pointer_x > width || pointer_y < 0.0 || pointer_y > height {
            return None;
        }
        if pointer_y <= strip_band + 8.0 {
            return None;
        }
        if pointer_x <= left_right_edge {
            return Some(SplitPreviewSide::Left);
        }
        if pointer_x >= width - left_right_edge {
            return Some(SplitPreviewSide::Right);
        }
        if pointer_bottom >= height - bottom_edge {
            return Some(SplitPreviewSide::Bottom);
        }
        None
    }
}

pub fn build_group(
    group_id: &str,
    tabs: &[TabSpec],
    active_tab_id: Option<&str>,
    show_tab_strip: bool,
    plugin_runtime: Option<Rc<PluginRuntime>>,
) -> BuiltCustomWorkbenchGroup {
    let overlay = Overlay::new();
    overlay.set_halign(Align::Fill);
    overlay.set_valign(Align::Fill);
    overlay.set_hexpand(true);
    overlay.set_vexpand(true);
    overlay.add_css_class("custom-workbench-group");

    let root = GtkBox::new(Orientation::Vertical, 0);
    root.set_halign(Align::Fill);
    root.set_valign(Align::Fill);
    root.set_hexpand(true);
    root.set_vexpand(true);
    root.add_css_class("workspace-pane");
    root.add_css_class("workbench");

    let tab_strip = GtkBox::new(Orientation::Horizontal, 0);
    tab_strip.add_css_class("workbench-tab-strip");

    let tab_scroller = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(false)
        .hscrollbar_policy(PolicyType::Automatic)
        .vscrollbar_policy(PolicyType::Never)
        .min_content_height(36)
        .child(&tab_strip)
        .build();
    tab_scroller.add_css_class("workbench-tab-strip-scroller");
    tab_scroller.set_visible(show_tab_strip);

    let stack = Stack::new();
    stack.set_halign(Align::Fill);
    stack.set_valign(Align::Fill);
    stack.set_hexpand(true);
    stack.set_vexpand(true);
    stack.set_transition_type(StackTransitionType::None);
    stack.add_css_class("workbench-stack");

    root.append(&tab_scroller);
    root.append(&stack);
    overlay.set_child(Some(&root));

    let preview_inner = GtkBox::new(Orientation::Vertical, 0);
    preview_inner.add_css_class("workbench-split-preview");
    let drop_placeholder = GtkBox::new(Orientation::Horizontal, 0);
    drop_placeholder.add_css_class("tab-drag-gap");

    let preview = Revealer::new();
    preview.set_can_target(false);
    preview.set_reveal_child(false);
    preview.set_child(Some(&preview_inner));
    overlay.add_overlay(&preview);

    let drag_layer = Fixed::new();
    drag_layer.set_hexpand(true);
    drag_layer.set_vexpand(true);
    drag_layer.set_can_target(false);
    overlay.add_overlay(&drag_layer);

    let pointer_position = Rc::new(RefCell::new(None::<(f64, f64)>));
    let motion = EventControllerMotion::new();
    {
        let pointer_position = pointer_position.clone();
        motion.connect_motion(move |_, x, y| {
            *pointer_position.borrow_mut() = Some((x, y));
        });
    }
    {
        let pointer_position = pointer_position.clone();
        motion.connect_leave(move |_| {
            *pointer_position.borrow_mut() = None;
        });
    }
    overlay.add_controller(motion);

    let handle = CustomWorkbenchGroupHandle {
        group_id: group_id.to_string(),
        root: overlay.clone(),
        tab_strip,
        tab_scroller,
        stack,
        preview,
        preview_inner,
        drag_layer: Rc::new(RefCell::new(drag_layer)),
        headers: Rc::new(RefCell::new(HashMap::new())),
        tab_labels: Rc::new(RefCell::new(HashMap::new())),
        page_names: Rc::new(RefCell::new(HashMap::new())),
        order: Rc::new(RefCell::new(Vec::new())),
        active_tab_id: Rc::new(RefCell::new(None)),
        drag_state: Rc::new(RefCell::new(None)),
        pointer_position,
        external_pointer_position: Rc::new(RefCell::new(None)),
        external_coordinate_target: Rc::new(RefCell::new(None)),
        split_hover: Rc::new(RefCell::new(None)),
        split_drop_handler: Rc::new(RefCell::new(None)),
        tab_drop_handler: Rc::new(RefCell::new(None)),
        drag_hover_handler: Rc::new(RefCell::new(None)),
        drag_end_handler: Rc::new(RefCell::new(None)),
        active_changed_handler: Rc::new(RefCell::new(None)),
        drop_placeholder,
    };

    let mut tab_labels = HashMap::new();
    let mut close_buttons = HashMap::new();
    let mut buffers = HashMap::new();
    let mut lists = HashMap::new();
    let mut entries = HashMap::new();
    let mut labels = HashMap::new();

    for tab in tabs {
        let page = tabbed_panel::build_tab_page("workbench", tab, plugin_runtime.as_ref());
        tab_labels.insert(page.tab_id.clone(), page.tab_label.clone());
        if let Some(close_button) = page.close_button.clone() {
            close_buttons.insert(page.tab_id.clone(), close_button);
        }
        if let Some(buffer) = page.buffer.clone() {
            buffers.insert(page.tab_id.clone(), buffer);
        }
        if let Some(list) = page.list.clone() {
            lists.insert(page.tab_id.clone(), list);
        }
        if let Some(entry) = page.entry.clone() {
            entries.insert(page.tab_id.clone(), entry);
        }
        labels.extend(page.labels.clone());
        handle.append_page(page, false);
    }

    if let Some(active_tab_id) = active_tab_id {
        handle.set_active_tab(active_tab_id);
    } else if let Some(first_tab_id) = handle.tab_ids().first().cloned() {
        handle.set_active_tab(&first_tab_id);
    }

    BuiltCustomWorkbenchGroup {
        root: overlay,
        handle,
        tab_labels,
        close_buttons,
        buffers,
        lists,
        entries,
        labels,
    }
}

fn install_header_activation(
    header: &Widget,
    tab_id: &str,
    stack: Stack,
    headers: Rc<RefCell<HashMap<String, Widget>>>,
    active_tab_id: Rc<RefCell<Option<String>>>,
    active_changed_handler: Rc<RefCell<Option<Rc<dyn Fn(String)>>>>,
) {
    let gesture = GestureClick::new();
    let tab_id = tab_id.to_string();
    gesture.connect_pressed(move |_, _, _, _| {
        *active_tab_id.borrow_mut() = Some(tab_id.clone());
        stack.set_visible_child_name(&page_name(&tab_id));
        for (candidate, header) in headers.borrow().iter() {
            if candidate == &tab_id {
                header.add_css_class("active");
            } else {
                header.remove_css_class("active");
            }
        }
        if let Some(handler) = active_changed_handler.borrow().as_ref().cloned() {
            handler(tab_id.clone());
        }
    });
    header.add_controller(gesture);
}

fn page_name(tab_id: &str) -> String {
    format!("custom-workbench-page:{tab_id}")
}

fn build_drag_widget(title: &str) -> Widget {
    let label = Label::new(Some(title));
    label.add_css_class("tab-label");

    let container = GtkBox::new(Orientation::Horizontal, 6);
    container.add_css_class("tab-header");
    container.add_css_class("drag-preview");
    container.append(&label);
    container.upcast::<Widget>()
}

fn widget_or_ancestor_has_css_class(widget: &Widget, css_class: &str) -> bool {
    let mut current = Some(widget.clone());
    while let Some(candidate) = current {
        if candidate.has_css_class(css_class) {
            return true;
        }
        current = candidate.parent();
    }
    false
}
