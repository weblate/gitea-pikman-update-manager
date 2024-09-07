use crate::apt_package_row::AptPackageRow;
use adw::gio::SimpleAction;
use adw::prelude::*;
use apt_deb822_tools::Deb822Repository;
use gtk::glib::*;
use gtk::*;
use pika_unixsocket_tools::pika_unixsocket_tools::*;
use rust_apt::cache::*;
use rust_apt::new_cache;
use rust_apt::records::RecordField;
use std::cell::RefCell;
use std::process::Command;
use std::rc::Rc;
use std::thread;
use tokio::runtime::Runtime;


pub fn apt_manage_page(
    window: adw::ApplicationWindow,
    retry_signal_action: &SimpleAction,
) -> gtk::Box {

    let deb822_sources = Deb822Repository::get_deb822_sources().unwrap();

    let system_source = deb822_sources.iter().filter(|x| {
        match &x.repolib_id {
            Some(t) => {
                if t == "system" {
                    true
                } else {
                    false
                }
            }
            None => false
        }
    }).next().unwrap();

    let main_box = Box::builder()
        .hexpand(true)
        .vexpand(true)
        .orientation(Orientation::Vertical)
        .build();

    let system_mirror_entry = gtk::Entry::builder()
        .placeholder_text(system_source.repolib_default_mirror.as_deref().unwrap())
        .text(system_source.uris.as_deref().unwrap())
        .build();

    main_box.append(&system_mirror_entry);

    main_box
}