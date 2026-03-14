use gtk::gdk::Display;
use gtk::{CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION};

const STYLE: &str = include_str!("../resources/style.css");

pub fn load() {
    let provider = CssProvider::new();
    provider.load_from_data(STYLE);

    if let Some(display) = Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
