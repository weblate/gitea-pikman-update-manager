use adw::gio::SimpleAction;
use adw::prelude::*;
use apt_deb822_tools::Deb822Repository;
use gtk::glib::clone;
use gtk::*;
use std::path::Path;

pub fn deb822_edit_dialog_fn(
    window: adw::ApplicationWindow,
    deb822_repo: &Deb822Repository,
    reload_action: &gio::SimpleAction,
    apt_retry_signal_action: &SimpleAction,
) {
    let repofile_path = Path::new(&deb822_repo.filepath);
    let repo_file_name = repofile_path
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .trim_end_matches(".sources")
        .to_owned();

    let unofficial_source_add_dialog_child_box = Box::builder()
        .hexpand(true)
        .orientation(Orientation::Vertical)
        .build();

    let unofficial_source_add_name_entry = gtk::Entry::builder().build();

    let unofficial_source_add_name_prefrencesgroup = adw::PreferencesGroup::builder()
        .title(t!("unofficial_source_add_name_prefrencesgroup_title"))
        .build();

    unofficial_source_add_name_prefrencesgroup.add(&unofficial_source_add_name_entry);

    let unofficial_source_add_uri_entry = gtk::Entry::builder().build();

    let unofficial_source_add_uri_prefrencesgroup = adw::PreferencesGroup::builder()
        .title(t!("unofficial_source_add_uri_prefrencesgroup_title"))
        .build();

    unofficial_source_add_uri_prefrencesgroup.add(&unofficial_source_add_uri_entry);

    let unofficial_source_add_suites_entry = gtk::Entry::builder().build();

    let unofficial_source_add_suites_prefrencesgroup = adw::PreferencesGroup::builder()
        .title(t!("unofficial_source_add_suites_prefrencesgroup_title"))
        .build();

    unofficial_source_add_suites_prefrencesgroup.add(&unofficial_source_add_suites_entry);

    let unofficial_source_add_components_entry = gtk::Entry::builder().build();

    let unofficial_source_add_components_prefrencesgroup = adw::PreferencesGroup::builder()
        .title(t!("unofficial_source_add_components_prefrencesgroup_title"))
        .build();

    unofficial_source_add_components_prefrencesgroup.add(&unofficial_source_add_components_entry);

    let unofficial_source_add_signed_entry = gtk::Entry::builder().sensitive(false).build();

    let unofficial_source_add_signed_prefrencesgroup = adw::PreferencesGroup::builder()
        .title(t!("unofficial_source_add_signed_prefrencesgroup_title"))
        .build();

    unofficial_source_add_signed_prefrencesgroup.add(&unofficial_source_add_signed_entry);

    let unofficial_source_add_archs_entry = gtk::Entry::builder().build();

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
    let unofficial_source_add_name_entry_clone0 = unofficial_source_add_name_entry.clone();
    let unofficial_source_add_uri_entry_clone0 = unofficial_source_add_uri_entry.clone();
    let unofficial_source_add_suites_entry_clone0 = unofficial_source_add_suites_entry.clone();
    let unofficial_source_add_components_entry_clone0 =
        unofficial_source_add_components_entry.clone();
    let unofficial_source_add_signed_entry_clone0 = unofficial_source_add_signed_entry.clone();
    let unofficial_source_signed_keyring_checkbutton_clone0 =
        unofficial_source_signed_keyring_checkbutton.clone();

    let add_button_update_state = move || {
        if !unofficial_source_add_name_entry_clone0.text().is_empty()
            && !unofficial_source_add_uri_entry_clone0.text().is_empty()
            && !unofficial_source_add_suites_entry_clone0.text().is_empty()
            && !unofficial_source_add_components_entry_clone0
                .text()
                .is_empty()
        {
            if unofficial_source_signed_keyring_checkbutton_clone0.is_active() {
                unofficial_source_add_dialog_clone0
                    .set_response_enabled("unofficial_source_add_dialog_add", true);
            } else if !unofficial_source_add_signed_entry_clone0.text().is_empty() {
                unofficial_source_add_dialog_clone0
                    .set_response_enabled("unofficial_source_add_dialog_add", true);
            } else {
                unofficial_source_add_dialog_clone0
                    .set_response_enabled("unofficial_source_add_dialog_add", false);
            }
        } else {
            unofficial_source_add_dialog_clone0
                .set_response_enabled("unofficial_source_add_dialog_add", false);
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
            move |_| {
                add_button_update_state();
            }
        ));
    }

    //

    unofficial_source_signed_keyring_checkbutton.connect_toggled(clone!(
        #[weak]
        unofficial_source_add_signed_entry,
        #[strong]
        add_button_update_state,
        move |checkbutton| {
            if checkbutton.is_active() {
                unofficial_source_add_signed_entry.set_sensitive(false);
                add_button_update_state();
            }
        }
    ));

    unofficial_source_signed_file_checkbutton.connect_toggled(clone!(
        #[weak]
        unofficial_source_add_signed_entry,
        #[strong]
        add_button_update_state,
        move |checkbutton| {
            if checkbutton.is_active() {
                unofficial_source_add_signed_entry.set_sensitive(true);
                add_button_update_state();
            }
        }
    ));

    unofficial_source_add_box2.append(&unofficial_source_add_is_source_label);
    unofficial_source_add_box2.append(&unofficial_source_add_is_source_switch);
    unofficial_source_add_box2.append(&unofficial_source_add_is_enabled_label);
    unofficial_source_add_box2.append(&unofficial_source_add_is_enabled_switch);
    unofficial_source_add_box2.append(&unofficial_source_signed_keyring_checkbutton);
    unofficial_source_add_box2.append(&unofficial_source_signed_file_checkbutton);

    unofficial_source_add_dialog_child_box.append(&unofficial_source_add_name_prefrencesgroup);
    unofficial_source_add_dialog_child_box.append(&unofficial_source_add_uri_prefrencesgroup);
    unofficial_source_add_dialog_child_box.append(&unofficial_source_add_suites_prefrencesgroup);
    unofficial_source_add_dialog_child_box
        .append(&unofficial_source_add_components_prefrencesgroup);
    unofficial_source_add_dialog_child_box.append(&unofficial_source_add_archs_prefrencesgroup);
    unofficial_source_add_dialog_child_box.append(&unofficial_source_add_box2);
    unofficial_source_add_dialog_child_box.append(&unofficial_source_add_signed_prefrencesgroup);

    //

    match &deb822_repo.repolib_name {
        Some(t) => {
            unofficial_source_add_name_entry.set_text(&t);
        }
        None => {}
    }
    match &deb822_repo.uris {
        Some(t) => {
            unofficial_source_add_uri_entry.set_text(&t);
        }
        None => {}
    }
    match &deb822_repo.suites {
        Some(t) => {
            unofficial_source_add_suites_entry.set_text(&t);
        }
        None => {}
    }
    match &deb822_repo.components {
        Some(t) => {
            unofficial_source_add_components_entry.set_text(&t);
        }
        None => {}
    }
    match &deb822_repo.signed_by {
        Some(t) => {
            unofficial_source_signed_file_checkbutton.set_active(true);
            unofficial_source_add_signed_entry.set_text(&t);
        }
        None => unofficial_source_signed_keyring_checkbutton.set_active(true),
    }
    match &deb822_repo.architectures {
        Some(t) => {
            unofficial_source_add_archs_entry.set_text(&t);
        }
        None => {}
    }
    match &deb822_repo.enabled {
        Some(t) => {
            unofficial_source_add_is_enabled_switch.set_active(match t.to_lowercase().as_str() {
                "yes" => true,
                "true" => true,
                "no" => false,
                "false" => false,
                _ => true,
            });
        }
        None => {
            unofficial_source_add_is_enabled_switch.set_active(true);
        }
    }

    match &deb822_repo.types {
        Some(t) => {
            unofficial_source_add_is_source_switch.set_active(t.contains("deb-src"));
        }
        None => {}
    }

    //
    let deb822_repo_clone0 = deb822_repo.clone();

    let reload_action_clone0 = reload_action.clone();
    let apt_retry_signal_action_clone0 = apt_retry_signal_action.clone();

    unofficial_source_add_dialog
        .clone()
        .choose(None::<&gio::Cancellable>, move |choice| {
            match choice.as_str() {
                "unofficial_source_edit_dialog_edit" => {
                    let sign_method = if unofficial_source_signed_file_checkbutton.is_active() {
                        1
                    } else {
                        0
                    };
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
                            _ => None,
                        },
                        enabled: match unofficial_source_add_is_enabled_switch.is_active() {
                            true => Some("yes".to_string()),
                            false => Some("no".to_string()),
                        },
                        ..deb822_repo_clone0
                    };
                    match Deb822Repository::write_to_file(
                        new_repo.clone(),
                        format!("/tmp/{}.sources", repo_file_name).into(),
                    ) {
                        Ok(_) => {
                            match duct::cmd!(
                                "pkexec",
                                "/usr/lib/pika/pikman-update-manager/scripts/modify_repo.sh",
                                "deb822_move",
                                repo_file_name
                            )
                            .run()
                            {
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
                _ => {}
            }
        });
}
