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


pub fn apt_manage_page(
    window: adw::ApplicationWindow,
    retry_signal_action: &SimpleAction,
) -> gtk::Box {

    let deb822_sources = Deb822Repository::get_deb822_sources().unwrap();

    let mut unofficial_deb822_sources = deb822_sources.clone();

    let system_source = deb822_sources.iter().filter(|x| {
        match &x.repolib_id {
            Some(t) => {
                t == "system"
            }
            None => false
        }
    }).next().unwrap();
    
    unofficial_deb822_sources.retain(|x| {
        match &x.repolib_id {
            Some(t) => {
                !(t == "system")
            }
            None => true
        }
    });

    let legacy_apt_repos = apt_legacy_tools::LegacyAptSource::get_legacy_sources();

    let main_box = Box::builder()
        .hexpand(true)
        .vexpand(true)
        .orientation(Orientation::Vertical)
        .build();

    //

    let system_mirror_label0 = gtk::Label::builder()
        .label(t!("system_mirror_label0_label"))
        .halign(gtk::Align::Start)
        .valign(gtk::Align::Start)
        .hexpand(true)
        .margin_top(15)
        .margin_start(15)
        .margin_end(15)
        .margin_bottom(5)
        .build();
    system_mirror_label0.add_css_class("heading");

    let system_mirror_label1 = gtk::Label::builder()
        .label(t!("system_mirror_label1_label"))
        .halign(gtk::Align::Start)
        .valign(gtk::Align::Start)
        .hexpand(true)
        .margin_start(15)
        .margin_end(15)
        .build();

    let system_mirror_entry = gtk::Entry::builder()
        .placeholder_text(system_source.repolib_default_mirror.as_deref().unwrap())
        .text(system_source.uris.as_deref().unwrap())
        .valign(gtk::Align::Start)
        .margin_top(5)
        .margin_bottom(5)
        .margin_start(15)
        .margin_end(15)
        .build();

    //

    let unofficial_sources_label0 = gtk::Label::builder()
        .label(t!("unofficial_sources_label"))
        .halign(gtk::Align::Start)
        .valign(gtk::Align::Start)
        .hexpand(true)
        .margin_top(15)
        .margin_start(15)
        .margin_end(15)
        .margin_bottom(5)
        .build();
    unofficial_sources_label0.add_css_class("heading");

    let unofficial_sources_label1 = gtk::Label::builder()
        .label(t!("unofficial_sources_label1_label"))
        .halign(gtk::Align::Start)
        .valign(gtk::Align::Start)
        .hexpand(true)
        .margin_start(15)
        .margin_end(15)
        .build();

    let unofficial_sources_list_store = gio::ListStore::new::<BoxedAnyObject>();

    enum AptSourceConfig {
        Legacy(apt_legacy_tools::LegacyAptSource),
        DEB822(apt_deb822_tools::Deb822Repository)
    }

    for deb822_source in unofficial_deb822_sources {
        unofficial_sources_list_store.append(&BoxedAnyObject::new(AptSourceConfig::DEB822(deb822_source)));
    };

    match legacy_apt_repos {
        Ok(vec) => {
            for legacy_repo in vec {
                unofficial_sources_list_store.append(&BoxedAnyObject::new(AptSourceConfig::Legacy(legacy_repo)));
            };
        }
        Err(_) => {}
    }

    let unofficial_sources_selection_model = SingleSelection::new(Some(unofficial_sources_list_store));

    /*let unofficial_sources_item_factory = SignalListItemFactory::new();

    unofficial_sources_item_factory.connect_setup(|_item_factory, list_item| {
        let label = gtk::Label::new(Some("DDD"));

        let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();

        list_item.set_child(Some(&label));
    });

    unofficial_sources_item_factory.connect_bind(|_item_factory, list_item| {
        let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();

        let list_item_item = list_item.item().unwrap().to_value().get::<String>().unwrap();
        let list_item_label = list_item.child().unwrap().downcast::<gtk::Label>().unwrap();

        list_item_label.set_label(&list_item_item);
    });
    */

    let unofficial_sources_columnview = ColumnView::builder()
        .vexpand(true)
        .model(&unofficial_sources_selection_model)
        .build();

    //

    let unofficial_sources_columnview_factory0 = gtk::SignalListItemFactory::new();
    
    unofficial_sources_columnview_factory0.connect_setup(move |_factory, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let row = Label::builder()
            .halign(Align::Start)
            .build();
        item.set_child(Some(&row));
    });

    unofficial_sources_columnview_factory0.connect_bind(move |_factory, item| {
        let item: &ListItem = item.downcast_ref::<gtk::ListItem>().unwrap();
        let child = item.child().and_downcast::<Label>().unwrap();
        let entry: BoxedAnyObject = item.item().and_downcast::<BoxedAnyObject>().unwrap();
        let entry_borrow = entry.borrow::<AptSourceConfig>();
        let repo_name = match entry_borrow.deref() {
            AptSourceConfig::DEB822(src) => {
                match &src.repolib_name {
                    Some(name) => name,
                    None => match(&src.uris, &src.suites, &src.components) {
                        (Some(uris),Some(suites),Some(components)) => {
                            &format!("{} {} {}", uris, suites, components)
                        }
                        (_,_,_) => {
                            &t!("apt_source_parse_error").to_string()
                        }
                    }
                    
                    
                }
            }
            AptSourceConfig::Legacy(src) => {
                &format!("{} {} {} {}",
                    if src.is_source {
                        "(Legacy Src)"
                    } else {
                        "(Legacy)"
                    },
                    &src.url,
                    &src.suite,
                    &src.components
                )
            }
        };
        child.set_label(&repo_name);
    });
    
    let unofficial_sources_columnview_col0 = gtk::ColumnViewColumn::builder()
        .title(t!("unofficial_sources_columnview_col0_title"))
        .factory(&unofficial_sources_columnview_factory0)
        .expand(true)
        .build();

    //

    let unofficial_sources_columnview_factory1 = gtk::SignalListItemFactory::new();
    
    unofficial_sources_columnview_factory1.connect_setup(move |_factory, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let row = Label::builder()
            .halign(Align::Start)
            .build();
        item.set_child(Some(&row));
    });

    unofficial_sources_columnview_factory1.connect_bind(move |_factory, item| {
        let item: &ListItem = item.downcast_ref::<gtk::ListItem>().unwrap();
        let child = item.child().and_downcast::<Label>().unwrap();
        let entry: BoxedAnyObject = item.item().and_downcast::<BoxedAnyObject>().unwrap();
        let entry_borrow = entry.borrow::<AptSourceConfig>();
        let repo_enabled = match entry_borrow.deref() {
            AptSourceConfig::DEB822(src) => {
                match &src.enabled {
                    Some(t) => match t.to_lowercase().as_str() {
                        "yes" => true,
                        "true" => true,
                        "no" => false,
                        "false" => false,
                        _ => true,
                    }
                    None => true,
                }
            }
            AptSourceConfig::Legacy(src) => {
                src.enabled
            }
        };
        if repo_enabled {
            child.set_label(&t!("apt_repo_enabled"));
        } else {
            child.set_label(&t!("apt_repo_disabled"));
        }
    });
    
    let unofficial_sources_columnview_col1 = gtk::ColumnViewColumn::builder()
        .title(t!("unofficial_sources_columnview_col1_title"))
        .factory(&unofficial_sources_columnview_factory1)
        .build();

    //

    unofficial_sources_selection_model.connect_selected_item_notify(|selection| {
        //let selection = selection.selected_item().unwrap();
        //let entry  = selection.downcast_ref::<BoxedAnyObject>().unwrap();
        //let r: Ref<AptSourceConfig> = entry.borrow();
        //println!("{}", r.col2.to_string())
    });
    unofficial_sources_columnview.append_column(&unofficial_sources_columnview_col0);
    unofficial_sources_columnview.append_column(&unofficial_sources_columnview_col1);

    let unofficial_sources_box = Box::builder()
        .orientation(Orientation::Vertical)
        .margin_bottom(3)
        .margin_top(3)
        .margin_end(3)
        .margin_start(3)
        .build();

    let unofficial_sources_viewport = ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .has_frame(true)
        .margin_bottom(15)
        .margin_top(15)
        .margin_end(15)
        .margin_start(15)
        .child(&unofficial_sources_box)
        .height_request(390)
        .build();
    unofficial_sources_viewport.add_css_class("round-all-scroll");

    //

    let unofficial_sources_edit_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .homogeneous(true)
        .build();
    unofficial_sources_edit_box.add_css_class("linked");

    let unofficial_source_edit_button = Button::builder()
        .icon_name("document-edit-symbolic")
        .tooltip_text(t!("unofficial_source_edit_button"))
        //.halign(Align::End)
        .valign(Align::End)
        .build();

    let unofficial_source_add_button = Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text(t!("unofficial_source_add_button"))
        //.halign(Align::End)
        .valign(Align::End)
        .build();

    unofficial_source_add_button.connect_clicked(clone!(
        #[strong]
        window,
            move
            |_|
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
                    "unofficial_source_add_dialog_add",
                    adw::ResponseAppearance::Suggested,
                );

                unofficial_source_add_dialog.set_response_appearance(
                    "unofficial_source_add_dialog_cancel",
                    adw::ResponseAppearance::Destructive,
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

                unofficial_source_add_dialog.clone()
                    .choose(None::<&gio::Cancellable>, move |choice| {
                        match choice.as_str() {
                            "unofficial_source_add_dialog_add" => {
                                println!("add");
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
                                    let key_download_cmd = duct::cmd!("pkexec", "/usr/lib/pika/pikman-update-manager/scripts/wget.sh", &unofficial_source_add_signed_entry.text().to_string(), &format!("/etc/apt/keyrings/{}.gpg.key", repo_file_name))
                                        .run();
                                    match key_download_cmd {
                                        Ok(_) => {}
                                        Err(e) => {
                                            let key_download_error_dialog = adw::MessageDialog::builder()
                                                .heading(t!("key_download_error_dialog_heading"))
                                                .body(e.to_string())
                                                .build();
                                            key_download_error_dialog.add_response(
                                                "key_download_error_dialog_ok",
                                                &t!("key_download_error_dialog_ok_label").to_string(),
                                                );
                                            key_download_error_dialog.present();
                                        }
                                    }
                                }
                            }
                            "apt_update_dialog_ignore" => {
                                unofficial_source_add_dialog.close();
                            }
                            _ => {}
                        }
                    });
            }
        )
    );

    //

    unofficial_sources_edit_box.append(&unofficial_source_add_button);
    unofficial_sources_edit_box.append(&unofficial_source_edit_button);

    unofficial_sources_box.append(&unofficial_sources_columnview);
    unofficial_sources_box.append(&unofficial_sources_edit_box);

    //

    main_box.append(&system_mirror_label0);
    main_box.append(&system_mirror_label1);
    main_box.append(&system_mirror_entry);
    //
    main_box.append(&unofficial_sources_label0);
    main_box.append(&unofficial_sources_label1);
    main_box.append(&unofficial_sources_viewport);

    main_box
}