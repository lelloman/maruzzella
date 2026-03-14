pub mod app;
pub mod commands;
pub mod layout;
pub mod product;
pub mod shell;
pub mod spec;
pub mod theme;

use gtk::prelude::*;
use gtk::Application;

pub fn build_application(application_id: &str) -> Application {
    let application = Application::builder()
        .application_id(application_id)
        .build();

    application.connect_activate(|application| {
        app::build(application);
    });

    application
}

pub fn run_default() {
    build_application("com.lelloman.maruzzella").run();
}
