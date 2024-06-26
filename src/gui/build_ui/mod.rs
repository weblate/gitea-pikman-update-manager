use adw::prelude::*;
use adw::*;
use crate::apt_update_page;
use crate::apt_update_page::apt_update_page;
use crate::config::{APP_ICON, APP_ID};

pub fn build_ui(app: &adw::Application) {
    // setup glib
    gtk::glib::set_prgname(Some(t!("app_name").to_string()));
    glib::set_application_name(&t!("app_name").to_string());

    // create the main Application window
    let window = adw::ApplicationWindow::builder()
        // The text on the titlebar
        .title(t!("app_name"))
        // link it to the application "app"
        .application(app)
        // Add the box called "window_box" to it
        // Application icon
        .icon_name(APP_ICON)
        // Minimum Size/Default
        .width_request(700)
        .height_request(500)
        .deletable(false)
        // Startup
        .startup_id(APP_ID)
        // build the window
        .build();

    apt_update_page::apt_update_page();

    // show the window
    window.present()
}