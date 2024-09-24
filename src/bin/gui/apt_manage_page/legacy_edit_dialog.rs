use crate::apt_package_row::AptPackageRow;
use adw::gio::SimpleAction;
use adw::prelude::*;
use apt_legacy_tools::LegacyAptSource;
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
use std::path::Path;

pub fn legacy_edit_dialog_fn(
    window: adw::ApplicationWindow,
    legacy_repo: &LegacyAptSource,
    reload_action: &gio::SimpleAction,
) {
                let repofile_path = Path::new(&legacy_repo.filepath);
                let repo_file_name = repofile_path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .trim_end_matches(".list")
                    .to_owned();
                
                let unofficial_source_add_dialog_child_box = Box::builder()
                    .hexpand(true)
                    .orientation(Orientation::Vertical)
                    .build();

                let unofficial_source_add_uri_entry = gtk::Entry::builder()
                    .build();

                let unofficial_source_add_uri_prefrencesgroup = adw::PreferencesGroup::builder()
                    .title(t!("unofficial_source_add_uri_prefrencesgroup_title"))
                    .build();

                unofficial_source_add_uri_prefrencesgroup.add(&unofficial_source_add_uri_entry);

                let unofficial_source_add_suites_entry = gtk::Entry::builder()
                    .build();

                let unofficial_source_add_suites_prefrencesgroup = adw::PreferencesGroup::builder()
                    .title(t!("unofficial_source_add_suites_prefrencesgroup_title"))
                    .build();

                unofficial_source_add_suites_prefrencesgroup.add(&unofficial_source_add_suites_entry);

                let unofficial_source_add_components_entry = gtk::Entry::builder()
                    .build();

                let unofficial_source_add_components_prefrencesgroup = adw::PreferencesGroup::builder()
                    .title(t!("unofficial_source_add_components_prefrencesgroup_title"))
                    .build();

                unofficial_source_add_components_prefrencesgroup.add(&unofficial_source_add_components_entry);

                let unofficial_source_add_legacy_options_entry = gtk::Entry::builder()
                    .build();

                let unofficial_source_add_legacy_options_prefrencesgroup = adw::PreferencesGroup::builder()
                    .title(t!("unofficial_source_add_legacy_options_prefrencesgroup_title"))
                    .build();
                
                unofficial_source_add_legacy_options_prefrencesgroup.add(&unofficial_source_add_legacy_options_entry);

                let unofficial_source_add_box2 = gtk::Box::builder()
                    .margin_top(10)
                    .orientation(Orientation::Horizontal)
                    .hexpand(true)
                    .spacing(5)
                    .build();

                let unofficial_source_add_is_source_label = gtk::Label::builder()
                    .label(t!("unofficial_source_add_is_legacy_source_label_label"))
                    .halign(Align::Start)
                    .valign(Align::Center)
                    .build();


                let unofficial_source_add_is_enabled_label = gtk::Label::builder()
                    .label(t!("unofficial_source_add_is_enabled_label_label"))
                    .halign(Align::Start)
                    .valign(Align::Center)
                    .build();
                
                let unofficial_source_add_is_source_switch = gtk::Switch::builder()
                    .halign(Align::Start)
                    .valign(Align::Center)
                    .build();

                let unofficial_source_add_is_enabled_switch = gtk::Switch::builder()
                    .halign(Align::Start)
                    .valign(Align::Center)
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
                    .heading(t!("unofficial_source_edit_dialog_heading").to_string() + " " + &repo_file_name)
                    .width_request(700)
                    .height_request(500)
                    .build();

                unofficial_source_add_dialog.add_response(
                    "unofficial_source_edit_dialog_edit",
                    &t!("unofficial_source_edit_dialog_add_edit").to_string(),
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
                    "unofficial_source_edit_dialog_edit",
                    adw::ResponseAppearance::Suggested,
                );

                //

                let unofficial_source_add_dialog_clone0 = unofficial_source_add_dialog.clone();
                let unofficial_source_add_uri_entry_clone0 = unofficial_source_add_uri_entry.clone();
                let unofficial_source_add_suites_entry_clone0 = unofficial_source_add_suites_entry.clone();
                let unofficial_source_add_components_entry_clone0 = unofficial_source_add_components_entry.clone();

                let add_button_update_state = move || {
                    if
                        !unofficial_source_add_uri_entry_clone0.text().is_empty() &&
                        !unofficial_source_add_suites_entry_clone0.text().is_empty() &&
                        !unofficial_source_add_components_entry_clone0.text().is_empty()
                    {
                        unofficial_source_add_dialog_clone0.set_response_enabled("unofficial_source_add_dialog_add", true);
                    } else {
                        unofficial_source_add_dialog_clone0.set_response_enabled("unofficial_source_add_dialog_add", false);
                    }
                };

                //

                for entry in [
                    &unofficial_source_add_uri_entry,
                    &unofficial_source_add_suites_entry,
                    &unofficial_source_add_components_entry,
                    &unofficial_source_add_legacy_options_entry,
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
                
                unofficial_source_add_box2.append(&unofficial_source_add_is_source_label);
                unofficial_source_add_box2.append(&unofficial_source_add_is_source_switch);
                unofficial_source_add_box2.append(&unofficial_source_add_is_enabled_label);
                unofficial_source_add_box2.append(&unofficial_source_add_is_enabled_switch);

                unofficial_source_add_dialog_child_box.append(&unofficial_source_add_uri_prefrencesgroup);
                unofficial_source_add_dialog_child_box.append(&unofficial_source_add_suites_prefrencesgroup);
                unofficial_source_add_dialog_child_box.append(&unofficial_source_add_components_prefrencesgroup);
                unofficial_source_add_dialog_child_box.append(&unofficial_source_add_box2);
                unofficial_source_add_dialog_child_box.append(&unofficial_source_add_legacy_options_prefrencesgroup);


                //

                unofficial_source_add_uri_entry.set_text(&legacy_repo.url);
                unofficial_source_add_suites_entry.set_text(&legacy_repo.suite);
                unofficial_source_add_components_entry.set_text(&legacy_repo.components);
                match &legacy_repo.options {
                    Some(t) => {
                        unofficial_source_add_legacy_options_entry.set_text(&t);
                    }
                    None => {}
                }
                unofficial_source_add_is_enabled_switch.set_active(legacy_repo.enabled);

                unofficial_source_add_is_source_switch.set_active(legacy_repo.is_source);

                //
                let legacy_repo_clone0 = legacy_repo.clone();

                let reload_action_clone0 = reload_action.clone();

                unofficial_source_add_dialog.clone()
                    .choose(None::<&gio::Cancellable>, move |choice| {
                        match choice.as_str() {
                            "unofficial_source_edit_dialog_edit" => {       
                                let new_repo = LegacyAptSource {
                                    url: unofficial_source_add_uri_entry.text().to_string(),
                                    is_source: unofficial_source_add_is_source_switch.is_active(),
                                    suite: unofficial_source_add_suites_entry.text().to_string(),
                                    components: unofficial_source_add_components_entry.text().to_string(),
                                    options: Some(unofficial_source_add_legacy_options_entry.text().to_string()),
                                    enabled: unofficial_source_add_is_enabled_switch.is_active(),
                                    ..legacy_repo_clone0
                                };
                                    /*match LegacyAptSource::save_to_file(new_repo.clone(), LegacyAptSource::get_legacy_sources().unwrap(), &format!("/tmp/{}.list", repo_file_name)) {
                                        Ok(_) => {
                                            match duct::cmd!("pkexec", "/usr/lib/pika/pikman-update-manager/scripts/modify_repo.sh", "legacy_move", repo_file_name).run() {
                                                Ok(_) => {}
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
                                            reload_action_clone0.activate(None);
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
                                        }
                                    }*/
                                    dbg!(&new_repo);
                                    LegacyAptSource::save_to_file(new_repo.clone(), LegacyAptSource::get_legacy_sources().unwrap(), &format!("/tmp/{}.list", repo_file_name)).unwrap();
                                }
                            _ => {}
                        }
                    });
}