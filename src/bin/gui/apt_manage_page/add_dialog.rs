use crate::apt_package_row::AptPackageRow;
use adw::gio::SimpleAction;
use adw::prelude::*;
use apt_deb822_tools::Deb822Repository;
use regex::Regex;
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

pub fn add_dialog_fn(
        window: adw::ApplicationWindow,
        reload_action: &gio::SimpleAction,
        apt_retry_signal_action: &gio::SimpleAction,
    )
    {
    let unofficial_source_add_dialog_child_box = Box::builder()
                    .hexpand(true)
                    .orientation(Orientation::Vertical)
                    .build();
                
                let unofficial_source_add_name_entry = gtk::Entry::builder()
                    .placeholder_text("WineHQ Debian")
                    .build();

                let unofficial_source_add_name_prefrencesgroup = adw::PreferencesGroup::builder()
                    .title(t!("unofficial_source_add_name_prefrencesgroup_title"))
                    .build();

                unofficial_source_add_name_prefrencesgroup.add(&unofficial_source_add_name_entry);

                let unofficial_source_add_uri_entry = gtk::Entry::builder()
                    .placeholder_text("https://dl.winehq.org/wine-builds/debian")
                    .build();

                let unofficial_source_add_uri_prefrencesgroup = adw::PreferencesGroup::builder()
                    .title(t!("unofficial_source_add_uri_prefrencesgroup_title"))
                    .build();

                unofficial_source_add_uri_prefrencesgroup.add(&unofficial_source_add_uri_entry);

                let unofficial_source_add_suites_entry = gtk::Entry::builder()
                    .placeholder_text("trixie bookworm sid")
                    .build();

                let unofficial_source_add_suites_prefrencesgroup = adw::PreferencesGroup::builder()
                    .title(t!("unofficial_source_add_suites_prefrencesgroup_title"))
                    .build();

                unofficial_source_add_suites_prefrencesgroup.add(&unofficial_source_add_suites_entry);

                let unofficial_source_add_components_entry = gtk::Entry::builder()
                    .placeholder_text("main proprietary")
                    .build();

                let unofficial_source_add_components_prefrencesgroup = adw::PreferencesGroup::builder()
                    .title(t!("unofficial_source_add_components_prefrencesgroup_title"))
                    .build();

                unofficial_source_add_components_prefrencesgroup.add(&unofficial_source_add_components_entry);

                let unofficial_source_add_signed_entry = gtk::Entry::builder()
                    .sensitive(false)
                    .build();

                let unofficial_source_add_signed_prefrencesgroup = adw::PreferencesGroup::builder()
                    .title(t!("unofficial_source_add_signed_prefrencesgroup_title"))
                    .build();
                
                unofficial_source_add_signed_prefrencesgroup.add(&unofficial_source_add_signed_entry);

                let unofficial_source_add_archs_entry = gtk::Entry::builder()
                    .placeholder_text("amd64 arm64 i386")
                    .build();

                let unofficial_source_add_archs_prefrencesgroup = adw::PreferencesGroup::builder()
                    .title(t!("unofficial_source_add_archs_prefrencesgroup_title"))
                    .build();

                unofficial_source_add_archs_prefrencesgroup.add(&unofficial_source_add_archs_entry);

                let unofficial_source_add_box2 = gtk::Box::builder()
                    .margin_top(10)
                    .orientation(Orientation::Horizontal)
                    .hexpand(true)
                    .spacing(5)
                    .build();

                let unofficial_source_add_is_source_label = gtk::Label::builder()
                    .label(t!("unofficial_source_add_is_source_label_label"))
                    .halign(Align::Start)
                    .valign(Align::Center)
                    .build();
                
                let unofficial_source_add_is_source_switch = gtk::Switch::builder()
                    .halign(Align::Start)
                    .valign(Align::Center)
                    .build();

                let unofficial_source_signed_keyring_checkbutton = gtk::CheckButton::builder()
                    .halign(Align::Start)
                    .valign(Align::Center)
                    .label(t!("unofficial_source_signed_keyring_checkbutton_label"))
                    .active(true)
                    .build();

                let unofficial_source_signed_file_checkbutton = gtk::CheckButton::builder()
                    .halign(Align::Start)
                    .valign(Align::Center)
                    .label(t!("unofficial_source_signed_file_checkbutton_label"))
                    .group(&unofficial_source_signed_keyring_checkbutton)
                    .build();

                let unofficial_source_signed_url_checkbutton = gtk::CheckButton::builder()
                    .halign(Align::Start)
                    .valign(Align::Center)
                    .label(t!("unofficial_source_signed_url_checkbutton_label"))
                    .group(&unofficial_source_signed_keyring_checkbutton)
                    .build();

                //
                let unofficial_source_add_dialog_child_clamp = adw::Clamp::builder()
                    .child(&unofficial_source_add_dialog_child_box)
                    .maximum_size(500)
                    .build();

                let unofficial_source_add_viewport = gtk::ScrolledWindow::builder()
                    .hexpand(true)
                    .vexpand(true)
                    .child(&unofficial_source_add_dialog_child_clamp)
                    .hscrollbar_policy(PolicyType::Never)
                    .build();

                let unofficial_source_add_dialog = adw::MessageDialog::builder()
                    .transient_for(&window)
                    .extra_child(&unofficial_source_add_viewport)
                    .heading(t!("unofficial_source_add_dialog_heading"))
                    .width_request(700)
                    .height_request(500)
                    .build();

                unofficial_source_add_dialog.add_response(
                    "unofficial_source_add_dialog_add",
                    &t!("unofficial_source_add_dialog_add_label").to_string(),
                );
                
                unofficial_source_add_dialog.add_response(
                    "unofficial_source_add_dialog_cancel",
                    &t!("unofficial_source_add_dialog_cancel_label").to_string(),
                    );

                unofficial_source_add_dialog.set_response_enabled("unofficial_source_add_dialog_add", false);
                
                unofficial_source_add_dialog.set_response_appearance(
                    "unofficial_source_add_dialog_cancel",
                    adw::ResponseAppearance::Destructive,
                );

                unofficial_source_add_dialog.set_response_appearance(
                    "unofficial_source_add_dialog_add",
                    adw::ResponseAppearance::Suggested,
                );

                //

                let unofficial_source_add_dialog_clone0 = unofficial_source_add_dialog.clone();
                let unofficial_source_add_name_entry_clone0 = unofficial_source_add_name_entry.clone();
                let unofficial_source_add_uri_entry_clone0 = unofficial_source_add_uri_entry.clone();
                let unofficial_source_add_suites_entry_clone0 = unofficial_source_add_suites_entry.clone();
                let unofficial_source_add_components_entry_clone0 = unofficial_source_add_components_entry.clone();
                let unofficial_source_add_signed_entry_clone0 = unofficial_source_add_signed_entry.clone();
                let unofficial_source_signed_keyring_checkbutton_clone0 = unofficial_source_signed_keyring_checkbutton.clone();

                let add_button_update_state = move || {
                    if
                        !unofficial_source_add_name_entry_clone0.text().is_empty() &&
                        !unofficial_source_add_uri_entry_clone0.text().is_empty() &&
                        !unofficial_source_add_suites_entry_clone0.text().is_empty() &&
                        !unofficial_source_add_components_entry_clone0.text().is_empty()
                    {
                        if unofficial_source_signed_keyring_checkbutton_clone0.is_active() {
                            unofficial_source_add_dialog_clone0.set_response_enabled("unofficial_source_add_dialog_add", true);
                        } else if !unofficial_source_add_signed_entry_clone0.text().is_empty() {
                            unofficial_source_add_dialog_clone0.set_response_enabled("unofficial_source_add_dialog_add", true);
                        } else {
                            unofficial_source_add_dialog_clone0.set_response_enabled("unofficial_source_add_dialog_add", false);
                        }
                    } else {
                        unofficial_source_add_dialog_clone0.set_response_enabled("unofficial_source_add_dialog_add", false);
                    }
                };

                //

                for entry in [
                    &unofficial_source_add_name_entry,
                    &unofficial_source_add_uri_entry,
                    &unofficial_source_add_suites_entry,
                    &unofficial_source_add_components_entry,
                    &unofficial_source_add_signed_entry,
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

                unofficial_source_signed_keyring_checkbutton.connect_toggled(clone!(
                    #[weak]
                    unofficial_source_add_signed_entry,
                    #[strong]
                    add_button_update_state,
                    move |checkbutton|
                        {
                            if checkbutton.is_active() {
                                unofficial_source_add_signed_entry.set_sensitive(false);
                                unofficial_source_add_signed_entry.set_placeholder_text(Some(""));
                                add_button_update_state();
                            }
                        }
                    )
                );

                unofficial_source_signed_file_checkbutton.connect_toggled(clone!(
                    #[weak]
                    unofficial_source_add_signed_entry,
                    #[strong]
                    add_button_update_state,
                    move |checkbutton|
                        {
                            if checkbutton.is_active() {
                                unofficial_source_add_signed_entry.set_sensitive(true);
                                unofficial_source_add_signed_entry.set_placeholder_text(Some("/etc/apt/keyrings/winehq-archive.key"));
                                add_button_update_state();
                            }
                        }
                    )
                );

                unofficial_source_signed_url_checkbutton.connect_toggled(clone!(
                    #[weak]
                    unofficial_source_add_signed_entry,
                    #[strong]
                    add_button_update_state,
                    move |checkbutton|
                        {
                            if checkbutton.is_active() {
                                unofficial_source_add_signed_entry.set_sensitive(true);
                                unofficial_source_add_signed_entry.set_placeholder_text(Some("https://dl.winehq.org/wine-builds/winehq.key"));
                                add_button_update_state();
                            }
                        }
                    )
                );
                
                unofficial_source_add_box2.append(&unofficial_source_add_is_source_label);
                unofficial_source_add_box2.append(&unofficial_source_add_is_source_switch);
                unofficial_source_add_box2.append(&unofficial_source_signed_keyring_checkbutton);
                unofficial_source_add_box2.append(&unofficial_source_signed_file_checkbutton);
                unofficial_source_add_box2.append(&unofficial_source_signed_url_checkbutton);

                unofficial_source_add_dialog_child_box.append(&unofficial_source_add_name_prefrencesgroup);
                unofficial_source_add_dialog_child_box.append(&unofficial_source_add_uri_prefrencesgroup);
                unofficial_source_add_dialog_child_box.append(&unofficial_source_add_suites_prefrencesgroup);
                unofficial_source_add_dialog_child_box.append(&unofficial_source_add_components_prefrencesgroup);
                unofficial_source_add_dialog_child_box.append(&unofficial_source_add_archs_prefrencesgroup);
                unofficial_source_add_dialog_child_box.append(&unofficial_source_add_box2);
                unofficial_source_add_dialog_child_box.append(&unofficial_source_add_signed_prefrencesgroup);

                let reload_action_clone0 = reload_action.clone();
                let apt_retry_signal_action_clone0 = apt_retry_signal_action.clone();

                unofficial_source_add_dialog.clone()
                    .choose(None::<&gio::Cancellable>, move |choice| {
                        match choice.as_str() {
                            "unofficial_source_add_dialog_add" => {
                                let non_alphanum_regex = Regex::new(r"[^a-zA-Z0-9]").unwrap();
                                let sign_method = if unofficial_source_signed_file_checkbutton.is_active() {
                                    1
                                } else if unofficial_source_signed_url_checkbutton.is_active() {
                                    2
                                } else {
                                    0
                                };
                                let repo_file_name = non_alphanum_regex.replace_all(unofficial_source_add_name_entry.text().as_str(), "_").to_string().to_lowercase();
                                let new_repo = Deb822Repository {
                                    repolib_name: Some(unofficial_source_add_name_entry.text().to_string()),
                                    filepath: format!("/etc/apt/sources.list.d/{}.source", repo_file_name),
                                    uris: Some(unofficial_source_add_uri_entry.text().to_string()),
                                    types: if unofficial_source_add_is_source_switch.is_active() {
                                        Some("deb deb-src".to_string())
                                    } else {
                                        Some("deb".to_string())
                                    },
                                    suites: Some(unofficial_source_add_suites_entry.text().to_string()),
                                    components: Some(unofficial_source_add_components_entry.text().to_string()),
                                    architectures: if unofficial_source_add_archs_entry.text().is_empty() {
                                        None
                                    } else {
                                        Some(unofficial_source_add_archs_entry.text().to_string())
                                    },
                                    signed_by: match sign_method {
                                        1 => Some(unofficial_source_add_signed_entry.text().to_string()),
                                        2 => Some(format!("/etc/apt/keyrings/{}.gpg.key", repo_file_name)),
                                        _ => None
                                    },
                                    ..Default::default()
                                };
                                if sign_method == 2 {
                                            match Deb822Repository::write_to_file(new_repo.clone(), format!("/tmp/{}.sources", repo_file_name).into()) {
                                                Ok(_) => {
                                                    match duct::cmd!("pkexec", "/usr/lib/pika/pikman-update-manager/scripts/modify_repo.sh", "deb822_move_with_wget", &repo_file_name, &unofficial_source_add_signed_entry.text().to_string(), &format!("/etc/apt/keyrings/{}.gpg.key", &repo_file_name)).run() {
                                                        Ok(_) => {
                                                            reload_action_clone0.activate(None);
                                                            apt_retry_signal_action_clone0.activate(None);
                                                            
                                                        }
                                                        Err(e) => {
                                                            let apt_src_create_error_dialog = adw::MessageDialog::builder()
                                                                .heading(t!("apt_src_create_error_dialog_heading"))
                                                                .body(e.to_string())
                                                                .build();
                                                            apt_src_create_error_dialog.add_response(
                                                                "apt_src_create_error_dialog_ok",
                                                                &t!("apt_src_create_error_dialog_ok_label").to_string(),
                                                                );
                                                            apt_src_create_error_dialog.present();
                                                            reload_action_clone0.activate(None);
                                                            apt_retry_signal_action_clone0.activate(None);
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    let apt_src_create_error_dialog = adw::MessageDialog::builder()
                                                        .heading(t!("apt_src_create_error_dialog_heading"))
                                                        .body(e.to_string())
                                                        .build();
                                                    apt_src_create_error_dialog.add_response(
                                                        "apt_src_create_error_dialog_ok",
                                                        &t!("apt_src_create_error_dialog_ok_label").to_string(),
                                                        );
                                                    apt_src_create_error_dialog.present();
                                                }
                                            }
                                } else {
                                    match Deb822Repository::write_to_file(new_repo.clone(), format!("/tmp/{}.sources", repo_file_name).into()) {
                                        Ok(_) => {
                                            match duct::cmd!("pkexec", "/usr/lib/pika/pikman-update-manager/scripts/modify_repo.sh", "deb822_move", repo_file_name).run() {
                                                Ok(_) => {
                                                    reload_action_clone0.activate(None);
                                                    apt_retry_signal_action_clone0.activate(None);
                                                }
                                                Err(e) => {
                                                    let apt_src_create_error_dialog = adw::MessageDialog::builder()
                                                        .heading(t!("apt_src_create_error_dialog_heading"))
                                                        .body(e.to_string())
                                                        .build();
                                                    apt_src_create_error_dialog.add_response(
                                                        "apt_src_create_error_dialog_ok",
                                                        &t!("apt_src_create_error_dialog_ok_label").to_string(),
                                                        );
                                                    apt_src_create_error_dialog.present();
                                                    reload_action_clone0.activate(None);
                                                    apt_retry_signal_action_clone0.activate(None);
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            let apt_src_create_error_dialog = adw::MessageDialog::builder()
                                                .heading(t!("apt_src_create_error_dialog_heading"))
                                                .body(e.to_string())
                                                .build();
                                            apt_src_create_error_dialog.add_response(
                                                "apt_src_create_error_dialog_ok",
                                                &t!("apt_src_create_error_dialog_ok_label").to_string(),
                                                );
                                            apt_src_create_error_dialog.present();
                                            reload_action_clone0.activate(None);
                                            apt_retry_signal_action_clone0.activate(None);
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    });
}