use crate::apt_package_row::AptPackageRow;
use adw::gio::SimpleAction;
use adw::prelude::*;
use apt_deb822_tools::Deb822Repository;
use libflatpak::builders::RemoteRefBuilder;
use regex::{bytes, Regex};
use gtk::glib::{property::PropertyGet, clone, BoxedAnyObject};
use gtk::*;
use std::cell::Ref;
use std::ops::Deref;
use pika_unixsocket_tools::pika_unixsocket_tools::*;
use rust_apt::cache::*;
use rust_apt::new_cache;
use rust_apt::records::RecordField;
use std::cell::RefCell;
use std::process::Command;
use std::rc::Rc;
use std::thread;
use tokio::runtime::Runtime;
use libflatpak::prelude::*;
use libflatpak::InstalledRef;

pub fn add_dialog_fn(
        window: adw::ApplicationWindow,
        reload_action: &gio::SimpleAction
    )
    {
                let flatpak_remote_add_dialog_child_box = Box::builder()
                    .hexpand(true)
                    .orientation(Orientation::Vertical)
                    .build();
                
                let flatpak_remote_add_name_entry = gtk::Entry::builder()
                    .placeholder_text("Flathub")
                    .build();

                let flatpak_remote_add_name_prefrencesgroup = adw::PreferencesGroup::builder()
                    .title(t!("flatpak_remote_add_name_prefrencesgroup_title"))
                    .build();

                flatpak_remote_add_name_prefrencesgroup.add(&flatpak_remote_add_name_entry);

                let flatpak_remote_add_url_entry = gtk::Entry::builder()
                    .placeholder_text("https://dl.flathub.org/repo/flathub.flatpakrepo")
                    .build();

                let flatpak_remote_add_url_prefrencesgroup = adw::PreferencesGroup::builder()
                    .title(t!("flatpak_remote_add_url_prefrencesgroup_title"))
                    .build();

                flatpak_remote_add_url_prefrencesgroup.add(&flatpak_remote_add_url_entry);

                let flatpak_remote_add_box2 = gtk::Box::builder()
                    .margin_top(10)
                    .orientation(Orientation::Horizontal)
                    .hexpand(true)
                    .spacing(5)
                    .build();

                let flatpak_remote_user_togglebutton = gtk::ToggleButton::builder()
                    .valign(Align::Center)
                    .hexpand(true)
                    .label(t!("flatpak_remotes_columnview_user"))
                    .active(true)
                    .build();

                let flatpak_remote_system_togglebutton = gtk::ToggleButton::builder()
                    .valign(Align::Center)
                    .hexpand(true)
                    .label(t!("flatpak_remotes_columnview_system"))
                    .group(&flatpak_remote_user_togglebutton)
                    .build();

                //
                let flatpak_remote_add_dialog_child_clamp = adw::Clamp::builder()
                    .child(&flatpak_remote_add_dialog_child_box)
                    .maximum_size(500)
                    .build();

                let flatpak_remote_add_viewport = gtk::ScrolledWindow::builder()
                    .hexpand(true)
                    .vexpand(true)
                    .child(&flatpak_remote_add_dialog_child_clamp)
                    .hscrollbar_policy(PolicyType::Never)
                    .build();

                let flatpak_remote_add_dialog = adw::MessageDialog::builder()
                    .transient_for(&window)
                    .extra_child(&flatpak_remote_add_viewport)
                    .heading(t!("flatpak_remote_add_dialog_heading"))
                    .width_request(700)
                    .height_request(400)
                    .build();

                flatpak_remote_add_dialog.add_response(
                    "flatpak_remote_add_dialog_add",
                    &t!("flatpak_remote_add_dialog_add_label").to_string(),
                );
                
                flatpak_remote_add_dialog.add_response(
                    "flatpak_remote_add_dialog_cancel",
                    &t!("flatpak_remote_add_dialog_cancel_label").to_string(),
                    );

                flatpak_remote_add_dialog.set_response_enabled("flatpak_remote_add_dialog_add", false);
                
                flatpak_remote_add_dialog.set_response_appearance(
                    "flatpak_remote_add_dialog_cancel",
                    adw::ResponseAppearance::Destructive,
                );

                flatpak_remote_add_dialog.set_response_appearance(
                    "flatpak_remote_add_dialog_add",
                    adw::ResponseAppearance::Suggested,
                );

                //

                let flatpak_remote_add_dialog_clone0 = flatpak_remote_add_dialog.clone();
                let flatpak_remote_add_name_entry_clone0 = flatpak_remote_add_name_entry.clone();
                let flatpak_remote_add_url_entry_clone0 = flatpak_remote_add_url_entry.clone();


                let add_button_update_state = move || {
                    if
                        !flatpak_remote_add_name_entry_clone0.text().is_empty() &&
                        !flatpak_remote_add_url_entry_clone0.text().is_empty()
                    {
                        flatpak_remote_add_dialog_clone0.set_response_enabled("flatpak_remote_add_dialog_add", true);
                    } else {
                        flatpak_remote_add_dialog_clone0.set_response_enabled("flatpak_remote_add_dialog_add", false);
                    }
                };

                //

                for entry in [
                    &flatpak_remote_add_name_entry,
                    &flatpak_remote_add_url_entry,
                ] {
                    entry.connect_text_notify(clone!(
                        #[strong]
                        add_button_update_state,
                        move |_|
                            {
                                add_button_update_state();
                            }
                        )
                    );
                }
                
                //
                
                flatpak_remote_add_box2.append(&flatpak_remote_user_togglebutton);
                flatpak_remote_add_box2.append(&flatpak_remote_system_togglebutton);

                flatpak_remote_add_dialog_child_box.append(&flatpak_remote_add_name_prefrencesgroup);
                flatpak_remote_add_dialog_child_box.append(&flatpak_remote_add_url_prefrencesgroup);
                flatpak_remote_add_dialog_child_box.append(&flatpak_remote_add_box2);

                let reload_action_clone0 = reload_action.clone();

                flatpak_remote_add_dialog.clone()
                    .choose(None::<&gio::Cancellable>, move |choice| {
                        match choice.as_str() {
                            "flatpak_remote_add_dialog_add" => {
                                let cancellable_no = libflatpak::gio::Cancellable::NONE;          

                                let flatpak_installation = match flatpak_remote_system_togglebutton.is_active() {
                                    true => libflatpak::Installation::new_system(cancellable_no).unwrap(),
                                    false => libflatpak::Installation::new_user(cancellable_no).unwrap(),
                                };

                                match libflatpak::Remote::from_file(&flatpak_remote_add_name_entry.text(), &get_data_from_url(&flatpak_remote_add_url_entry.text()).unwrap()) {
                                    Ok(remote) => {
                                        match libflatpak::Installation::add_remote(&flatpak_installation, &remote, true, cancellable_no) {
                                            Ok(_) => {
                                                reload_action_clone0.activate(None);
                                            }
                                            Err(e) => {
                                                let flatpak_remote_add_error_dialog = adw::MessageDialog::builder()
                                                    .heading(t!("flatpak_remote_add_error_dialog_heading"))
                                                    .body(e.to_string())
                                                    .build();
                                                flatpak_remote_add_error_dialog.add_response(
                                                    "flatpak_remote_add_error_dialog_ok",
                                                    &t!("flatpak_remote_add_error_dialog_ok_label").to_string(),
                                                    );
                                                flatpak_remote_add_error_dialog.present();
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        let flatpak_remote_add_error_dialog = adw::MessageDialog::builder()
                                                    .heading(t!("flatpak_remote_add_error_dialog_heading"))
                                                    .body(e.to_string())
                                                    .build();
                                        flatpak_remote_add_error_dialog.add_response(
                                            "flatpak_remote_add_error_dialog_ok",
                                            &t!("flatpak_remote_add_error_dialog_ok_label").to_string(),
                                        );
                                    }
                                }
                            }
                            _ => {}
                        }
                    });
}

fn get_data_from_url(url: &str) -> Result<libflatpak::glib::Bytes, reqwest::Error> {
    let data = reqwest::blocking::get(url)?
        .text()
        .unwrap();
        
    let bytes = data.as_bytes();

    let glib_bytes = libflatpak::glib::Bytes::from(bytes);
    Ok(glib_bytes)
}