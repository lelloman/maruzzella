mod app;
mod shell;
mod studio;
mod theme;

use gtk::prelude::*;
use gtk::Application;

fn main() {
    let application = Application::builder()
        .application_id("com.lelloman.maruzzella")
        .build();

    application.connect_activate(|application| {
        app::build(application);
    });

    application.run();
}
