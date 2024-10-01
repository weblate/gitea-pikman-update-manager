mod apt_package_row;
mod apt_update_page;
mod apt_manage_page;
mod build_ui;
mod config;
mod flatpak_ref_row;
mod flatpak_update_page;
mod flatpak_manage_page;

use crate::config::APP_ID;
use adw::prelude::*;
use adw::*;
use build_ui::build_ui;
use gdk::Display;
use gtk::*;
use std::boxed::Box;
use std::env;

// Init translations for current crate.
#[macro_use]
extern crate rust_i18n;
i18n!("locales", fallback = "en_US");

/// main function
fn main() {
    let current_locale = match env::var_os("LANG") {
        Some(v) => v
            .into_string()
            .unwrap()
            .chars()
            .take_while(|&ch| ch != '.')
            .collect::<String>(),
        None => panic!("$LANG is not set"),
    };
    rust_i18n::set_locale(&current_locale);
    let application = adw::Application::new(Some(APP_ID), gio::ApplicationFlags::HANDLES_COMMAND_LINE);
    application.connect_startup(|app| {
        // The CSS "magic" happens here.
        let provider = CssProvider::new();
        provider.load_from_string(include_str!("style.css"));
        // We give the CssProvided to the default screen so the CSS rules we added
        // can be applied to our window.
        gtk::style_context_add_provider_for_display(
            &Display::default().expect("Could not connect to a display."),
            &provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        build_ui(&app);
    });

    //if get_current_username().unwrap() == "pikaos" {
    //    application.run();
    //} else {
    //    println!("Error: This program can only be run via pikaos user");
    //    std::process::exit(1)
    //}
    application.run();
}
