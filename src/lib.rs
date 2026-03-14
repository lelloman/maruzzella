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
    build_application_with_activate(application_id, |application| {
        app::build(application);
    })
}

pub fn build_application_with_activate<F>(application_id: &str, activate: F) -> Application
where
    F: Fn(&Application) + 'static,
{
    let application = Application::builder()
        .application_id(application_id)
        .build();

    application.connect_activate(activate);

    application
}

pub fn run_default() {
    build_application("com.lelloman.maruzzella").run();
}
