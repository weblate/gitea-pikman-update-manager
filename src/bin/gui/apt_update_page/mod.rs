mod process;

use crate::apt_package_row::AptPackageRow;
use adw::gio::SimpleAction;
use adw::prelude::*;
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

#[derive(Clone)]
pub struct AptPackageSocket {
    pub name: String,
    pub arch: String,
    pub installed_version: String,
    pub candidate_version: String,
    pub description: String,
    pub source_uri: String,
    pub maintainer: String,
    pub size: u64,
    pub installed_size: u64,
    pub is_last: bool,
}
pub fn apt_update_page(
    window: adw::ApplicationWindow,
    retry_signal_action: &SimpleAction,
    flatpak_retry_signal_action: &SimpleAction,
    flatpak_ran_once: Rc<RefCell<bool>>,
    update_sys_tray: &SimpleAction,
    apt_update_count: &Rc<RefCell<i32>>,
    flatpak_update_count: &Rc<RefCell<i32>>,
) -> gtk::Box {
    let (update_percent_sender, update_percent_receiver) = async_channel::unbounded::<String>();
    let update_percent_sender = update_percent_sender.clone();
    let (update_status_sender, update_status_receiver) = async_channel::unbounded::<String>();
    let update_status_sender = update_status_sender.clone();
    let update_status_sender_clone0 = update_status_sender.clone();
    let (get_upgradable_sender, get_upgradable_receiver) = async_channel::unbounded();
    let get_upgradable_sender = get_upgradable_sender.clone();

    (*apt_update_count.borrow_mut() = 0);

    let excluded_updates_vec: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));

    thread::spawn(move || {
        Runtime::new().unwrap().block_on(start_socket_server_no_log(
            update_percent_sender,
            "/tmp/pika_apt_update_percent.sock",
        ));
    });

    thread::spawn(move || {
        Runtime::new().unwrap().block_on(start_socket_server(
            update_status_sender,
            "/tmp/pika_apt_update_status.sock",
            "/tmp/pika-apt-update.log",
        ));
    });

    thread::spawn(move || {
        let apt_update_command = Command::new("pkexec")
            .args([
                "/usr/lib/pika/pikman-update-manager/scripts/apt_update",
            ])
            .status()
            .unwrap();
        match apt_update_command.code().unwrap() {
            0 => update_status_sender_clone0
                .send_blocking("FN_OVERRIDE_SUCCESSFUL".to_owned())
                .unwrap(),
            53 => {}
            _ => {
                update_status_sender_clone0
                    .send_blocking(t!("update_status_error_perms").to_string())
                    .unwrap();
                update_status_sender_clone0
                    .send_blocking("FN_OVERRIDE_FAILED".to_owned())
                    .unwrap()
            }
        }
    });

    let main_box = Box::builder()
        .hexpand(true)
        .vexpand(true)
        .orientation(Orientation::Vertical)
        .build();

    let searchbar = SearchEntry::builder()
        .search_delay(500)
        .margin_top(15)
        .margin_bottom(15)
        .margin_end(30)
        .margin_start(30)
        .build();
    searchbar.add_css_class("rounded-all-25");

    let packages_boxedlist = ListBox::builder()
        .selection_mode(SelectionMode::None)
        .sensitive(false)
        .build();
    packages_boxedlist.add_css_class("boxed-list");
    packages_boxedlist.add_css_class("round-all-scroll");

    let packages_viewport = ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .has_frame(true)
        .margin_bottom(15)
        .margin_top(15)
        .margin_end(15)
        .margin_start(15)
        .height_request(390)
        .child(&packages_boxedlist)
        .build();
    packages_viewport.add_css_class("round-all-scroll");

    let packages_no_viewport_page = adw::StatusPage::builder()
        .icon_name("emblem-default-symbolic")
        .title(t!("apt_packages_no_viewport_page_title"))
        .hexpand(true)
        .vexpand(true)
        .build();

    let packages_ignored_viewport_page = adw::StatusPage::builder()
        .icon_name("dialog-warning-symbolic")
        .title(t!("apt_ignored_viewport_page_title"))
        .hexpand(true)
        .vexpand(true)
        .build();    

    let viewport_bin = adw::Bin::builder()
        .child(&packages_no_viewport_page)
        .build();

    let apt_update_dialog_child_box = Box::builder().orientation(Orientation::Vertical).build();

    let apt_update_dialog_progress_bar =
        ProgressBar::builder().show_text(true).hexpand(true).build();

    let apt_update_dialog_spinner = Spinner::builder()
        .hexpand(true)
        .valign(Align::Start)
        .halign(Align::Center)
        .spinning(true)
        .height_request(128)
        .width_request(128)
        .build();

    apt_update_dialog_child_box.append(&apt_update_dialog_spinner);
    apt_update_dialog_child_box.append(&apt_update_dialog_progress_bar);

    let apt_update_dialog = adw::MessageDialog::builder()
        .transient_for(&window)
        .extra_child(&apt_update_dialog_child_box)
        .heading(t!("apt_update_dialog_heading"))
        .width_request(500)
        .build();

    apt_update_dialog.add_response(
        "apt_update_dialog_retry",
        &t!("apt_update_dialog_retry_label").to_string(),
    );

    apt_update_dialog.add_response(
        "apt_update_dialog_ignore",
        &t!("apt_update_dialog_ignore_label").to_string(),
    );

    apt_update_dialog.set_response_appearance(
        "apt_update_dialog_retry",
        adw::ResponseAppearance::Suggested,
    );

    apt_update_dialog.set_response_enabled("apt_update_dialog_retry", false);
    apt_update_dialog.set_response_enabled("apt_update_dialog_ignore", false);

    

    if window.is_visible() {
        let retry_signal_action0 = retry_signal_action.clone();
        let viewport_bin = viewport_bin.clone();
        let flatpak_retry_signal_action = flatpak_retry_signal_action.clone();
        let flatpak_ran_once = flatpak_ran_once.clone();

        apt_update_dialog
            .clone()
            .choose(None::<&gio::Cancellable>, move |choice| {
                match choice.as_str() {
                    "apt_update_dialog_retry" => {
                        retry_signal_action0.activate(None);
                    }
                    "apt_update_dialog_ignore" => {
                        viewport_bin.set_child(Some(&packages_ignored_viewport_page));
                        let mut flatpak_ran_once_borrow = flatpak_ran_once.borrow_mut();
                        if *flatpak_ran_once_borrow != true {
                            flatpak_retry_signal_action.activate(None);
                            *flatpak_ran_once_borrow = true;
                        }
                    }
                    _ => {}
                }
            });
    }

    let bottom_bar = Box::builder().valign(Align::End).build();

    let select_button = Button::builder()
        .halign(Align::End)
        .valign(Align::Center)
        .hexpand(true)
        .margin_start(10)
        .margin_end(10)
        .margin_bottom(15)
        .label(t!("select_button_deselect_all"))
        .build();

    select_button.connect_clicked(clone!(
        #[weak]
        select_button,
        #[weak]
        packages_boxedlist,
        move |_| {
            let select_button_label = select_button.label().unwrap();
            let value_to_mark = if select_button_label == t!("select_button_select_all").to_string()
            {
                true
            } else if select_button_label == t!("select_button_deselect_all").to_string() {
                false
            } else {
                panic!("Unexpected label on selection button")
            };
            set_all_apt_row_marks_to(&packages_boxedlist, value_to_mark)
        }
    ));

    let update_button = Button::builder()
        .halign(Align::End)
        .valign(Align::Center)
        .hexpand(false)
        .sensitive(false)
        .margin_start(10)
        .margin_end(30)
        .margin_bottom(15)
        .label(t!("update_button_label"))
        .build();
    update_button.add_css_class("destructive-action");

    update_button.connect_clicked(clone!(
        #[weak]
        window,
        #[weak]
        retry_signal_action,
        #[strong]
        excluded_updates_vec,
        move |_| {
            process::apt_process_update(
                &excluded_updates_vec.borrow(),
                window,
                &retry_signal_action,
            );
        }
    ));

    bottom_bar.append(&select_button);
    bottom_bar.append(&update_button);

    let update_percent_server_context = MainContext::default();
    // The main loop executes the asynchronous block
    update_percent_server_context.spawn_local(clone!(
        #[weak]
        apt_update_dialog_progress_bar,
        async move {
            while let Ok(state) = update_percent_receiver.recv().await {
                match state.parse::<f64>() {
                    Ok(p) => apt_update_dialog_progress_bar.set_fraction(p / 100.0),
                    Err(_) => {}
                }
            }
        }
    ));

    let update_status_server_context = MainContext::default();
    // The main loop executes the asynchronous block
    update_status_server_context.spawn_local(clone!(
        #[weak]
        apt_update_dialog,
        #[weak]
        apt_update_dialog_child_box,
        #[weak]
        flatpak_retry_signal_action,
        async move {
            while let Ok(state) = update_status_receiver.recv().await {
                match state.as_ref() {
                    "FN_OVERRIDE_SUCCESSFUL" => {
                        let get_upgradable_sender = get_upgradable_sender.clone();
                        thread::spawn(move || {
                            // Create upgradable list cache
                            let upgradable_cache = new_cache!().unwrap();
                            //
                            upgradable_cache.upgrade(Upgrade::FullUpgrade).unwrap();

                            upgradable_cache.resolve(true).unwrap();

                            let mut upgradeable_iter =
                                upgradable_cache.get_changes(false).peekable();
                            while let Some(pkg) = upgradeable_iter.next() {
                                if !pkg.marked_delete() {
                                    let candidate_version_pkg = pkg.candidate().unwrap();
                                    let package_struct = AptPackageSocket {
                                        name: pkg.name().to_string(),
                                        arch: pkg.arch().to_string(),
                                        installed_version: match pkg.installed() {
                                            Some(t) => t.version().to_string(),
                                            _ => {
                                                t!("installed_version_to_be_installed").to_string()
                                            }
                                        },
                                        candidate_version: candidate_version_pkg
                                            .version()
                                            .to_string(),
                                        description: match candidate_version_pkg.description() {
                                            Some(s) => s,
                                            _ => t!("apt_pkg_property_unknown").to_string(),
                                        },
                                        source_uri: candidate_version_pkg
                                            .uris()
                                            .collect::<Vec<String>>()
                                            .join("\n"),
                                        maintainer: match candidate_version_pkg
                                            .get_record(RecordField::Maintainer)
                                        {
                                            Some(s) => s,
                                            _ => t!("apt_pkg_property_unknown").to_string(),
                                        },
                                        size: candidate_version_pkg.size(),
                                        installed_size: candidate_version_pkg.installed_size(),
                                        is_last: upgradeable_iter.peek().is_none(),
                                    };
                                    get_upgradable_sender.send_blocking(package_struct).unwrap()
                                }
                            }
                        });
                        apt_update_dialog.close();
                        let mut flatpak_ran_once_borrow = flatpak_ran_once.borrow_mut();
                        if *flatpak_ran_once_borrow != true {
                            flatpak_retry_signal_action.activate(None);
                            *flatpak_ran_once_borrow = true;
                        }
                    }
                    "FN_OVERRIDE_FAILED" => {
                        apt_update_dialog_child_box.set_visible(false);
                        apt_update_dialog.set_extra_child(Some(
                            &Image::builder()
                                .pixel_size(128)
                                .icon_name("dialog-error-symbolic")
                                .halign(Align::Center)
                                .build(),
                        ));
                        apt_update_dialog
                            .set_title(Some(&t!("apt_update_dialog_status_failed").to_string()));
                        apt_update_dialog.set_response_enabled("apt_update_dialog_retry", true);
                        apt_update_dialog.set_response_enabled("apt_update_dialog_ignore", true);
                    }
                    _ => apt_update_dialog.set_body(&state),
                }
            }
        }
    ));

    let get_upgradable_server_context = MainContext::default();
    // The main loop executes the asynchronous block
    get_upgradable_server_context.spawn_local(clone!(
        #[strong]
        select_button,
        #[strong]
        update_button,
        #[strong]
        packages_boxedlist,
        #[strong]
        packages_viewport,
        #[strong]
        viewport_bin,
        #[strong]
        excluded_updates_vec,
        #[strong]
        update_sys_tray,
        #[strong]
        apt_update_count,
        #[strong]
        flatpak_update_count,
        async move {
            while let Ok(state) = get_upgradable_receiver.recv().await {
                viewport_bin.set_child(Some(&packages_viewport));
                update_button.set_sensitive(true);
                let apt_row = AptPackageRow::new(state.clone());
                apt_row.connect_closure(
                    "checkbutton-toggled",
                    false,
                    closure_local!(
                        #[strong]
                        select_button,
                        #[strong]
                        update_button,
                        #[strong]
                        packages_boxedlist,
                        #[strong]
                        excluded_updates_vec,
                        move |apt_row: AptPackageRow| {
                            if is_widget_select_all_ready(&packages_boxedlist) {
                                select_button
                                    .set_label(&t!("select_button_select_all").to_string());
                            } else {
                                select_button
                                    .set_label(&t!("select_button_deselect_all").to_string());
                            }
                            update_button
                                .set_sensitive(!is_all_children_unmarked(&packages_boxedlist));
                            excluded_updates_vec
                                .borrow_mut()
                                .retain(|x| x != &apt_row.package_name());
                        }
                    ),
                );
                apt_row.connect_closure(
                    "checkbutton-untoggled",
                    false,
                    closure_local!(
                        #[strong]
                        select_button,
                        #[strong]
                        update_button,
                        #[strong]
                        packages_boxedlist,
                        #[strong]
                        excluded_updates_vec,
                        move |apt_row: AptPackageRow| {
                            select_button.set_label(&t!("select_button_select_all").to_string());
                            update_button
                                .set_sensitive(!is_all_children_unmarked(&packages_boxedlist));
                            excluded_updates_vec
                                .borrow_mut()
                                .push(apt_row.package_name())
                        }
                    ),
                );
                packages_boxedlist.append(&apt_row);
                (*apt_update_count.borrow_mut() += 1);
                if state.is_last {
                    packages_boxedlist.set_sensitive(true);
                    update_sys_tray.activate(Some(&glib::Variant::array_from_fixed_array(&[*apt_update_count.borrow(),*flatpak_update_count.borrow()])));
                }
            }
        }
    ));

    searchbar.connect_search_changed(clone!(
        #[weak]
        searchbar,
        #[weak]
        packages_boxedlist,
        move |_| {
            let mut counter = packages_boxedlist.first_child();
            while let Some(row) = counter {
                if row.widget_name() == "AptPackageRow" {
                    if !searchbar.text().is_empty() {
                        if row
                            .property::<String>("package-name")
                            .to_lowercase()
                            .contains(&searchbar.text().to_string().to_lowercase())
                        {
                            row.set_property("visible", true);
                            searchbar.grab_focus();
                        } else {
                            row.set_property("visible", false);
                        }
                    } else {
                        row.set_property("visible", true);
                    }
                }
                counter = row.next_sibling();
            }
        }
    ));

    main_box.append(&searchbar);
    main_box.append(&viewport_bin);
    main_box.append(&bottom_bar);

    main_box
}

fn is_widget_select_all_ready(parent_listbox: &impl IsA<ListBox>) -> bool {
    let mut is_ready = false;
    let mut child_counter = parent_listbox.borrow().first_child();
    while let Some(child) = child_counter {
        let next_child = child.next_sibling();
        let downcast = child.downcast::<AptPackageRow>().unwrap();
        if !downcast.package_marked() {
            is_ready = true;
            break;
        }
        child_counter = next_child
    }
    is_ready
}

fn is_all_children_unmarked(parent_listbox: &impl IsA<ListBox>) -> bool {
    let mut is_all_unmarked = true;
    let mut child_counter = parent_listbox.borrow().first_child();
    while let Some(child) = child_counter {
        let next_child = child.next_sibling();
        let downcast = child.downcast::<AptPackageRow>().unwrap();
        if downcast.package_marked() {
            is_all_unmarked = false;
            break;
        }
        child_counter = next_child
    }
    is_all_unmarked
}

fn set_all_apt_row_marks_to(parent_listbox: &impl IsA<ListBox>, value: bool) {
    let mut child_counter = parent_listbox.borrow().first_child();
    while let Some(child) = child_counter {
        let next_child = child.next_sibling();
        let downcast = child.downcast::<AptPackageRow>().unwrap();
        downcast.set_package_marked(value);
        child_counter = next_child
    }
}
