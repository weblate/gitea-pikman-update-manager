use adw::gio::SimpleAction;
use adw::prelude::*;
use apt_deb822_tools::Deb822Repository;
use gtk::glib::{clone, BoxedAnyObject};
use gtk::*;
use std::cell::Ref;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::Mutex;

mod add_dialog;
mod deb822_edit_dialog;
mod legacy_edit_dialog;

enum AptSourceConfig {
    Legacy(apt_legacy_tools::LegacyAptSource),
    DEB822(apt_deb822_tools::Deb822Repository),
}

pub fn apt_manage_page(
    window: adw::ApplicationWindow,
    glib_settings: &gio::Settings,
    apt_retry_signal_action: &SimpleAction,
    thread_sleep_sender: &std::sync::mpsc::Sender<()>,
    automatically_check_for_updates_arc: &Arc<AtomicBool>,
    update_interval_arc: &Arc<Mutex<i32>>,
) -> gtk::Box {
    let retry_signal_action = gio::SimpleAction::new("apt_manage_retry", None);

    let deb822_sources = Deb822Repository::get_deb822_sources().unwrap();

    let system_source = deb822_sources
        .iter()
        .filter(|x| match &x.repolib_id {
            Some(t) => t == "system",
            None => false,
        })
        .next()
        .unwrap();

    let system_mirror_refcell = Rc::new(RefCell::new(
        system_source
            .repolib_default_mirror
            .as_deref()
            .unwrap()
            .to_string(),
    ));

    let main_box = Box::builder()
        .hexpand(true)
        .vexpand(true)
        .orientation(Orientation::Vertical)
        .build();

    //

    let mirror_entry_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .margin_top(15)
        .margin_bottom(5)
        .margin_start(15)
        .margin_end(15)
        .hexpand(true)
        .valign(gtk::Align::Start)
        .build();
    mirror_entry_box.add_css_class("linked");

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
        .placeholder_text(system_mirror_refcell.borrow().to_string())
        .text(system_source.uris.as_deref().unwrap())
        .hexpand(true)
        .build();

    let system_mirror_save_button = gtk::Button::builder()
        .tooltip_text(t!("system_mirror_save_button_tooltip_text"))
        .sensitive(false)
        .halign(gtk::Align::End)
        .icon_name("object-select-symbolic")
        .build();

    system_mirror_entry.connect_changed(clone!(
        #[weak]
        system_mirror_save_button,
        #[strong]
        system_mirror_refcell,
        move |entry| {
            system_mirror_save_button.set_sensitive(
                !(entry.text().to_string() == system_mirror_refcell.borrow().to_string()),
            );
        }
    ));

    system_mirror_save_button.connect_clicked(clone!(
        #[weak]
        system_mirror_entry,
        #[strong]
        system_mirror_refcell,
        #[strong]
        system_source,
        #[strong]
        retry_signal_action,
        move |button| {
            let new_repo = Deb822Repository {
                uris: (Some(system_mirror_entry.text().to_string())),
                ..system_source.clone()
            };
            match Deb822Repository::write_to_file(
                new_repo.clone(),
                std::path::Path::new("/tmp/system.sources").to_path_buf(),
            ) {
                Ok(_) => {
                    match duct::cmd!(
                        "pkexec",
                        "/usr/lib/pika/pikman-update-manager/scripts/modify_repo.sh",
                        "deb822_move",
                        "system",
                        "system"
                    )
                    .run()
                    {
                        Ok(_) => {
                            retry_signal_action.activate(None);
                            *system_mirror_refcell.borrow_mut() =
                                system_mirror_entry.text().to_string();
                            button.set_sensitive(false);
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
                            retry_signal_action.activate(None);
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
        }
    ));
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

    let unofficial_sources_selection_model_rc: Rc<RefCell<gtk::SingleSelection>> =
        Rc::new(RefCell::default());

    let unofficial_sources_selection_model_rc_clone0 =
        Rc::clone(&unofficial_sources_selection_model_rc);

    let unofficial_sources_columnview_bin = adw::Bin::new();

    let unofficial_sources_columnview_bin_clone0 = unofficial_sources_columnview_bin.clone();

    retry_signal_action.connect_activate(clone!(
        #[weak]
        unofficial_sources_columnview_bin_clone0,
        move |_, _| {
            let mut unofficial_deb822_sources = Deb822Repository::get_deb822_sources().unwrap();

            unofficial_deb822_sources.retain(|x| match &x.repolib_id {
                Some(t) => !(t == "system"),
                None => true,
            });

            let legacy_apt_repos = apt_legacy_tools::LegacyAptSource::get_legacy_sources();

            let unofficial_sources_list_store = gio::ListStore::new::<BoxedAnyObject>();

            for deb822_source in unofficial_deb822_sources {
                unofficial_sources_list_store
                    .append(&BoxedAnyObject::new(AptSourceConfig::DEB822(deb822_source)));
            }

            match legacy_apt_repos {
                Ok(vec) => {
                    for legacy_repo in vec {
                        unofficial_sources_list_store
                            .append(&BoxedAnyObject::new(AptSourceConfig::Legacy(legacy_repo)));
                    }
                }
                Err(_) => {}
            }

            let unofficial_sources_selection_model =
                SingleSelection::new(Some(unofficial_sources_list_store));

            (*unofficial_sources_selection_model_rc_clone0.borrow_mut() =
                unofficial_sources_selection_model.clone());

            let unofficial_sources_columnview = ColumnView::builder()
                .vexpand(true)
                .model(&unofficial_sources_selection_model)
                .build();

            //

            let unofficial_sources_columnview_factory0 = gtk::SignalListItemFactory::new();

            unofficial_sources_columnview_factory0.connect_setup(move |_factory, item| {
                let item = item.downcast_ref::<gtk::ListItem>().unwrap();
                let row = Label::builder().halign(Align::Start).build();
                item.set_child(Some(&row));
            });

            unofficial_sources_columnview_factory0.connect_bind(move |_factory, item| {
                let item: &ListItem = item.downcast_ref::<gtk::ListItem>().unwrap();
                let child = item.child().and_downcast::<Label>().unwrap();
                let entry: BoxedAnyObject = item.item().and_downcast::<BoxedAnyObject>().unwrap();
                let entry_borrow = entry.borrow::<AptSourceConfig>();
                let repo_name = match entry_borrow.deref() {
                    AptSourceConfig::DEB822(src) => match &src.repolib_name {
                        Some(name) => name,
                        None => match (&src.uris, &src.suites, &src.components) {
                            (Some(uris), Some(suites), Some(components)) => {
                                &format!("{} {} {}", uris, suites, components)
                            }
                            (_, _, _) => &t!("apt_source_parse_error").to_string(),
                        },
                    },
                    AptSourceConfig::Legacy(src) => &format!(
                        "{} {} {} {}",
                        if src.is_source {
                            "(Legacy Src)"
                        } else {
                            "(Legacy)"
                        },
                        &src.url,
                        &src.suite,
                        &src.components
                    ),
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
                let row = Label::builder().halign(Align::Start).build();
                item.set_child(Some(&row));
            });

            unofficial_sources_columnview_factory1.connect_bind(move |_factory, item| {
                let item: &ListItem = item.downcast_ref::<gtk::ListItem>().unwrap();
                let child = item.child().and_downcast::<Label>().unwrap();
                let entry: BoxedAnyObject = item.item().and_downcast::<BoxedAnyObject>().unwrap();
                let entry_borrow = entry.borrow::<AptSourceConfig>();
                let repo_enabled = match entry_borrow.deref() {
                    AptSourceConfig::DEB822(src) => match &src.enabled {
                        Some(t) => match t.to_lowercase().as_str() {
                            "yes" => true,
                            "true" => true,
                            "no" => false,
                            "false" => false,
                            _ => true,
                        },
                        None => true,
                    },
                    AptSourceConfig::Legacy(src) => src.enabled,
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
            unofficial_sources_columnview.append_column(&unofficial_sources_columnview_col0);
            unofficial_sources_columnview.append_column(&unofficial_sources_columnview_col1);
            unofficial_sources_columnview_bin_clone0
                .set_child(Some(&unofficial_sources_columnview));
        }
    ));

    retry_signal_action.activate(None);

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
        .tooltip_text(t!("unofficial_source_edit_button_tooltip_text"))
        //.halign(Align::End)
        .valign(Align::End)
        .build();

    let unofficial_source_add_button = Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text(t!("unofficial_source_add_button_tooltip_text"))
        //.halign(Align::End)
        .valign(Align::End)
        .build();

    let unofficial_source_remove_button = Button::builder()
        .icon_name("edit-delete-symbolic")
        .tooltip_text(t!("unofficial_source_remove_button_tooltip_text"))
        //.halign(Align::End)
        .valign(Align::End)
        .build();

    unofficial_source_add_button.connect_clicked(clone!(
        #[strong]
        window,
        #[strong]
        retry_signal_action,
        #[strong]
        apt_retry_signal_action,
        move |_| {
            add_dialog::add_dialog_fn(
                window.clone(),
                &retry_signal_action,
                &apt_retry_signal_action,
            );
        }
    ));

    unofficial_source_edit_button.connect_clicked(clone!(
        #[strong]
        window,
        #[strong]
        unofficial_sources_selection_model_rc,
        #[strong]
        retry_signal_action,
        #[strong]
        apt_retry_signal_action,
        move |_| {
            let unofficial_sources_selection_model = unofficial_sources_selection_model_rc.borrow();
            let selection = unofficial_sources_selection_model.selected_item().unwrap();
            let item = selection.downcast_ref::<BoxedAnyObject>().unwrap();
            let apt_src: Ref<AptSourceConfig> = item.borrow();
            match apt_src.deref() {
                AptSourceConfig::DEB822(src) => {
                    deb822_edit_dialog::deb822_edit_dialog_fn(
                        window.clone(),
                        src,
                        &retry_signal_action,
                        &apt_retry_signal_action,
                    );
                }
                AptSourceConfig::Legacy(list) => legacy_edit_dialog::legacy_edit_dialog_fn(
                    window.clone(),
                    list,
                    &retry_signal_action,
                    &apt_retry_signal_action,
                ),
            };
        }
    ));

    unofficial_source_remove_button.connect_clicked(clone!(
        #[strong]
        window,
        #[strong]
        unofficial_sources_selection_model_rc,
        #[strong]
        retry_signal_action,
        #[strong]
        apt_retry_signal_action,
        move |_| {
            {
                let mut _command = duct::cmd!("");
                {
                    let unofficial_sources_selection_model =
                        unofficial_sources_selection_model_rc.borrow();
                    let selection = unofficial_sources_selection_model.selected_item().unwrap();
                    let item = selection.downcast_ref::<BoxedAnyObject>().unwrap();
                    let apt_src: Ref<AptSourceConfig> = item.borrow();
                    match apt_src.deref() {
                        AptSourceConfig::DEB822(src) => match &src.signed_by {
                            Some(t) => {
                                _command = duct::cmd!(
                                    "pkexec",
                                    "/usr/lib/pika/pikman-update-manager/scripts/modify_repo.sh",
                                    "delete_deb822",
                                    &src.filepath,
                                    t
                                )
                            }
                            None => {
                                _command = duct::cmd!(
                                    "pkexec",
                                    "/usr/lib/pika/pikman-update-manager/scripts/modify_repo.sh",
                                    "delete_legacy",
                                    &src.filepath
                                )
                            }
                        },
                        AptSourceConfig::Legacy(list) => {
                            _command = duct::cmd!(
                                "pkexec",
                                "/usr/lib/pika/pikman-update-manager/scripts/modify_repo.sh",
                                "delete_legacy",
                                &list.filepath
                            )
                        }
                    };
                }
                let apt_src_remove_warning_dialog = adw::MessageDialog::builder()
                    .heading(t!("apt_src_remove_warning_dialog_heading"))
                    .body(t!("apt_src_remove_warning_dialog_body"))
                    .transient_for(&window)
                    .build();
                apt_src_remove_warning_dialog.add_response(
                    "apt_src_remove_warning_dialog_cancel",
                    &t!("apt_src_remove_warning_dialog_cancel_label").to_string(),
                );
                apt_src_remove_warning_dialog.add_response(
                    "apt_src_remove_warning_dialog_ok",
                    &t!("apt_src_remove_warning_dialog_ok_label").to_string(),
                );
                apt_src_remove_warning_dialog.set_response_appearance(
                    "apt_src_remove_warning_dialog_ok",
                    adw::ResponseAppearance::Destructive,
                );
                let retry_signal_action_clone0 = retry_signal_action.clone();
                let apt_retry_signal_action_clone0 = apt_retry_signal_action.clone();
                apt_src_remove_warning_dialog.clone().choose(
                    None::<&gio::Cancellable>,
                    move |choice| match choice.as_str() {
                        "apt_src_remove_warning_dialog_ok" => match _command.run() {
                            Ok(_) => {
                                retry_signal_action_clone0.activate(None);
                                apt_retry_signal_action_clone0.activate(None)
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
                                retry_signal_action_clone0.activate(None);
                                apt_retry_signal_action_clone0.activate(None)
                            }
                        },
                        _ => {}
                    },
                );
            }
        }
    ));

    //

    let retry_interval_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .halign(Align::Start)
        .valign(Align::Center)
        .margin_start(10)
        .margin_end(10)
        .margin_bottom(10)
        .build();

    let retry_interval_labal = gtk::Label::builder()
        .label(t!("retry_interval_labal_label"))
        .margin_start(2)
        .margin_end(5)
        .halign(Align::Start)
        .valign(Align::Center)
        .build();

    let retry_interval_switch = gtk::Switch::builder()
        .active(glib_settings.boolean("check-for-updates"))
        .margin_end(5)
        .halign(Align::Start)
        .valign(Align::Center)
        .build();

    let retry_interval_spinrow = adw::SpinRow::builder()
        .title(t!("retry_interval_spinrow_title"))
        .subtitle(t!("retry_interval_spinrow_title"))
        .activatable(false)
        .selectable(false)
        .climb_rate(1.0)
        .adjustment(&gtk::Adjustment::new(
            (glib_settings.int("update-interval") as f64) / 3600000.0,
            1.0,
            24.0,
            1.0,
            0.0,
            0.0,
        ))
        .halign(Align::Start)
        .valign(Align::Center)
        .build();

    retry_interval_spinrow.connect_value_notify(clone!(
        #[strong]
        glib_settings,
        #[strong]
        automatically_check_for_updates_arc,
        #[strong]
        update_interval_arc,
        #[strong]
        thread_sleep_sender,
        move |spinrow| {
            match glib_settings.set_int("update-interval", (spinrow.value() * 3600000.0) as i32) {
                Ok(_) => {
                    {
                        automatically_check_for_updates_arc.store(
                            glib_settings.boolean("check-for-updates"),
                            std::sync::atomic::Ordering::Relaxed,
                        );
                        let mut update_interval_arc_gaurd = loop {
                            if let Ok(guard) = update_interval_arc.lock() {
                                break guard;
                            }
                        };
                        *update_interval_arc_gaurd = glib_settings.int("update-interval");
                    }
                    thread_sleep_sender.send(()).unwrap();
                }
                Err(_) => {
                    spinrow.set_value(glib_settings.int("update-interval") as f64 / 3600000.0);
                }
            }
        }
    ));

    retry_interval_switch.connect_state_set(clone!(
        #[strong]
        glib_settings,
        #[strong]
        automatically_check_for_updates_arc,
        #[strong]
        update_interval_arc,
        #[strong]
        thread_sleep_sender,
        move |switch, state| {
            match glib_settings.set_boolean("check-for-updates", state) {
                Ok(_) => {
                    {
                        automatically_check_for_updates_arc.store(
                            glib_settings.boolean("check-for-updates"),
                            std::sync::atomic::Ordering::Relaxed,
                        );
                        let mut update_interval_arc_gaurd = loop {
                            if let Ok(guard) = update_interval_arc.lock() {
                                break guard;
                            }
                        };
                        *update_interval_arc_gaurd = glib_settings.int("update-interval");
                    }
                    thread_sleep_sender.send(()).unwrap();
                }
                Err(_) => {
                    switch.set_active(!state);
                }
            }
            glib::Propagation::Proceed
        }
    ));

    retry_interval_spinrow.add_css_class("disable-outline");

    unofficial_sources_edit_box.append(&unofficial_source_add_button);
    unofficial_sources_edit_box.append(&unofficial_source_edit_button);
    unofficial_sources_edit_box.append(&unofficial_source_remove_button);

    unofficial_sources_box.append(&unofficial_sources_columnview_bin);
    unofficial_sources_box.append(&unofficial_sources_edit_box);

    //

    retry_interval_box.append(&retry_interval_labal);
    retry_interval_box.append(&retry_interval_switch);
    retry_interval_box.append(&retry_interval_spinrow);

    //

    main_box.append(&system_mirror_label0);
    main_box.append(&system_mirror_label1);
    main_box.append(&mirror_entry_box);

    mirror_entry_box.append(&system_mirror_entry);
    mirror_entry_box.append(&system_mirror_save_button);
    //
    main_box.append(&unofficial_sources_label0);
    main_box.append(&unofficial_sources_label1);
    main_box.append(&unofficial_sources_viewport);
    main_box.append(&retry_interval_box);

    main_box
}
