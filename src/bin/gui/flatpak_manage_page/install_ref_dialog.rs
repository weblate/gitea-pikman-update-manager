use crate::apt_package_row::AptPackageRow;
use adw::gio::SimpleAction;
use adw::prelude::*;
use apt_deb822_tools::Deb822Repository;
use libflatpak::builders::RemoteRefBuilder;
use regex::{bytes, Regex};
use gtk::glib::{property::PropertyGet, clone, BoxedAnyObject, MainContext};
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
use std::io::Write;
use libflatpak::InstalledRef;
use std::fs::OpenOptions;
use pretty_bytes::converter::convert;
use configparser::ini::Ini;


pub fn install_ref_dialog_fn(
        window: adw::ApplicationWindow,
        reload_action: &gio::SimpleAction,
        flatpak_retry_signal_action: &SimpleAction,
    )
    {
                let flatpak_ref_install_dialog_child_box = Box::builder()
                    .hexpand(true)
                    .orientation(Orientation::Vertical)
                    .build();

                let flatpak_ref_install_flatref_path_file_dialog_filter = FileFilter::new();
                flatpak_ref_install_flatref_path_file_dialog_filter.add_pattern("*.flatpakref");

                #[allow(deprecated)]
                let flatpak_ref_install_flatref_path_file_dialog = gtk::FileChooserNative::builder()
                    .title(t!("flatpak_ref_install_flatref_path_file_dialog_title"))
                    .accept_label(t!("flatpak_ref_install_flatref_path_file_dialog_accept_label"))
                    .cancel_label(t!("flatpak_ref_install_flatref_path_file_dialog_cancel_label"))
                    .action(gtk::FileChooserAction::Open)
                    .filter(&flatpak_ref_install_flatref_path_file_dialog_filter)
                    .build(); 

                let flatpak_ref_install_flatref_path_entry = gtk::Entry::builder()
                    .placeholder_text("/home/andy/Downloads/com.visualstudio.code.flatpakref")
                    .hexpand(true)
                    .build();

                let flatpak_ref_install_flatref_path_entry_open_file_dialog = gtk::Button::builder()
                        .tooltip_text(t!("flatpak_ref_install_flatref_path_entry_open_file_dialog_text"))
                        .halign(gtk::Align::End)
                        .icon_name("document-open-symbolic")
                        .build();

                let flatpak_ref_install_flatref_path_entry_box = gtk::Box::builder()
                    .orientation(gtk::Orientation::Horizontal)
                    .hexpand(true)
                    .build();
                flatpak_ref_install_flatref_path_entry_box.add_css_class("linked");

                flatpak_ref_install_flatref_path_entry_open_file_dialog.connect_clicked(clone!(
                    #[strong]
                    flatpak_ref_install_flatref_path_file_dialog,
                    move |_| {
                        flatpak_ref_install_flatref_path_file_dialog.set_visible(true);
                    }
                ));

                flatpak_ref_install_flatref_path_entry_box.append(&flatpak_ref_install_flatref_path_entry);
                flatpak_ref_install_flatref_path_entry_box.append(&flatpak_ref_install_flatref_path_entry_open_file_dialog);

                let flatpak_ref_install_flatref_path_prefrencesgroup = adw::PreferencesGroup::builder()
                    .title(t!("flatpak_ref_install_flatref_path_prefrencesgroup_title"))
                    .build();

                #[allow(deprecated)]
                flatpak_ref_install_flatref_path_file_dialog.connect_response(
                    clone!(
                        #[weak]
                        flatpak_ref_install_flatref_path_entry,
                        move |dialog, response| 
                        {
                            if response == gtk::ResponseType::Accept {
                                match dialog.file() {
                                    Some(f) => {
                                        match f.path() {
                                            Some(p) => flatpak_ref_install_flatref_path_entry.set_text(p.to_str().unwrap()),
                                            None => {}
                                        }
                                        
                                    }
                                    None => {}
                                }
                            }
                        }
                    )
                );   

                flatpak_ref_install_flatref_path_prefrencesgroup.add(&flatpak_ref_install_flatref_path_entry_box);

                let flatpak_ref_install_box2 = gtk::Box::builder()
                    .margin_top(10)
                    .orientation(Orientation::Horizontal)
                    .hexpand(true)
                    .spacing(5)
                    .build();

                let flatpak_ref_install_label0 = gtk::Label::builder()
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
                let flatpak_ref_install_dialog_child_clamp = adw::Clamp::builder()
                    .child(&flatpak_ref_install_dialog_child_box)
                    .maximum_size(500)
                    .build();

                let flatpak_ref_install_viewport = gtk::ScrolledWindow::builder()
                    .hexpand(true)
                    .vexpand(true)
                    .child(&flatpak_ref_install_dialog_child_clamp)
                    .hscrollbar_policy(PolicyType::Never)
                    .build();

                let flatpak_ref_install_dialog = adw::MessageDialog::builder()
                    .transient_for(&window)
                    .extra_child(&flatpak_ref_install_viewport)
                    .heading(t!("flatpak_ref_install_dialog_heading"))
                    .width_request(700)
                    .height_request(400)
                    .build();

                flatpak_ref_install_flatref_path_file_dialog.set_transient_for(Some(&flatpak_ref_install_dialog));

                flatpak_ref_install_dialog.add_response(
                    "flatpak_ref_install_dialog_add",
                    &t!("flatpak_ref_install_dialog_add_label").to_string(),
                );
                
                flatpak_ref_install_dialog.add_response(
                    "flatpak_ref_install_dialog_cancel",
                    &t!("flatpak_ref_install_dialog_cancel_label").to_string(),
                    );

                flatpak_ref_install_dialog.set_response_enabled("flatpak_ref_install_dialog_add", false);
                
                flatpak_ref_install_dialog.set_response_appearance(
                    "flatpak_ref_install_dialog_cancel",
                    adw::ResponseAppearance::Destructive,
                );

                flatpak_ref_install_dialog.set_response_appearance(
                    "flatpak_ref_install_dialog_add",
                    adw::ResponseAppearance::Suggested,
                );

                //

                let flatpak_ref_install_dialog_clone0 = flatpak_ref_install_dialog.clone();
                let flatpak_ref_install_flatref_path_entry_clone0 = flatpak_ref_install_flatref_path_entry.clone();
                let flatpak_ref_install_label0_clone0 = flatpak_ref_install_label0.clone();

                let tbi_remote_name = Rc::new(RefCell::new(None));
                let tbi_remote_url = Rc::new(RefCell::new(None));

                let tbi_remote_name_clone0 = tbi_remote_name.clone();
                let tbi_remote_url_clone0 = tbi_remote_url.clone();

                let add_button_update_state = move || {
                    if
                        !flatpak_ref_install_flatref_path_entry_clone0.text().is_empty()
                    {
                        match std::fs::read_to_string(flatpak_ref_install_flatref_path_entry_clone0.text()) {
                            Ok(t) => {
                                let mut flatref_file = Ini::new();
                                match flatref_file.read(t) {
                                    Ok(_) => {
                                        let ref_name = flatref_file.get("Flatpak Ref", "Name");
                                        let ref_remote_name = flatref_file.get("Flatpak Ref", "SuggestRemoteName");
                                        let ref_remote_url = flatref_file.get("Flatpak Ref", "RuntimeRepo");
                                        match (ref_name, ref_remote_name.clone()) {
                                            (Some(name), Some(remote_name)) => {
                                                flatpak_ref_install_label0_clone0.set_label(&strfmt::strfmt(
                                                    &t!("flatpak_ref_install_label").to_string(),
                                                    &std::collections::HashMap::from([
                                                        (
                                                            "NAME".to_string(),
                                                            name,
                                                        ),
                                                        (
                                                            "REMOTE".to_string(),
                                                            remote_name,
                                                        ),
                                                    ]),
                                                )
                                                .unwrap());
                                                {
                                                    *tbi_remote_name_clone0.borrow_mut() = ref_remote_name;
                                                    *tbi_remote_url_clone0.borrow_mut() = ref_remote_url;
                                                }
                                            }
                                            (_, _) => {
                                                flatpak_ref_install_dialog_clone0.set_response_enabled("flatpak_ref_install_dialog_add", false);
                                                flatpak_ref_install_label0_clone0.set_label("");
                                            }
                                        }
                                        flatpak_ref_install_dialog_clone0.set_response_enabled("flatpak_ref_install_dialog_add", true);
                                    }
                                    Err(_) => {
                                        flatpak_ref_install_dialog_clone0.set_response_enabled("flatpak_ref_install_dialog_add", false);
                                        flatpak_ref_install_label0_clone0.set_label("");
                                    }
                                }
                            }
                            Err(_) => {
                                flatpak_ref_install_dialog_clone0.set_response_enabled("flatpak_ref_install_dialog_add", false);
                                flatpak_ref_install_label0_clone0.set_label("");
                            }
                        }
                    } else {
                        flatpak_ref_install_dialog_clone0.set_response_enabled("flatpak_ref_install_dialog_add", false);
                        flatpak_ref_install_label0_clone0.set_label("");
                    }
                };

                //

                for entry in [
                    &flatpak_ref_install_flatref_path_entry,
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
                
                flatpak_ref_install_box2.append(&flatpak_remote_user_togglebutton);
                flatpak_ref_install_box2.append(&flatpak_remote_system_togglebutton);

                flatpak_ref_install_dialog_child_box.append(&flatpak_ref_install_flatref_path_prefrencesgroup);
                flatpak_ref_install_dialog_child_box.append(&flatpak_ref_install_box2);
                flatpak_ref_install_dialog_child_box.append(&flatpak_ref_install_label0);

                let reload_action_clone0 = reload_action.clone();
                let flatpak_retry_signal_action_clone0 = flatpak_retry_signal_action.clone();
                let tbi_remote_name_clone0 = tbi_remote_name.clone();
                let tbi_remote_url_clone0 = tbi_remote_url.clone();

                flatpak_ref_install_dialog.clone()
                    .choose(None::<&gio::Cancellable>, move |choice| {
                        match choice.as_str() {
                            "flatpak_ref_install_dialog_add" => {
                                match (tbi_remote_name_clone0.borrow().deref(), tbi_remote_url_clone0.borrow().deref()) {
                                    (Some(remote_name), Some(remote_url)) => {
                                        add_flatpakref_remote(&reload_action_clone0, &remote_name, &remote_url, flatpak_remote_system_togglebutton.is_active());
                                    }
                                    (_,_) => {}
                                }
                                run_flatpak_ref_install_transaction(&flatpak_retry_signal_action_clone0, &reload_action_clone0, flatpak_remote_system_togglebutton.is_active(), window, &flatpak_ref_install_flatref_path_entry.text());
                            }
                            _ => {}
                        }
                    });
}

pub fn run_flatpak_ref_install_transaction(flatpak_retry_signal_action: &gio::SimpleAction, retry_signal_action: &gio::SimpleAction, is_system: bool, window: adw::ApplicationWindow, flatref_path: &str) {
    let (transaction_percent_sender, transaction_percent_receiver) =
    async_channel::unbounded::<u32>();
    let transaction_percent_sender = transaction_percent_sender.clone();
    let (transaction_status_sender, transaction_status_receiver) =
        async_channel::unbounded::<String>();
    let transaction_status_sender = transaction_status_sender.clone();

    let flatref_path_clone0 = flatref_path.clone().to_owned();

    thread::spawn(move || {
        let cancellable_no = libflatpak::gio::Cancellable::NONE;

        let transaction_status_sender0 = transaction_status_sender.clone();
        let transaction_percent_sender0 = transaction_percent_sender.clone();

        let transaction_run_closure =
            move |transaction: &libflatpak::Transaction,
                transaction_operation: &libflatpak::TransactionOperation,
                transaction_progress: &libflatpak::TransactionProgress| {
                let transaction_status_sender = transaction_status_sender0.clone();
                let transaction_percent_sender = transaction_percent_sender0.clone();
                transaction_progress.connect_changed(clone!(@strong transaction_progress, @strong transaction_operation => move |_| {
                let status_message = format!("{}: {}\n{}: {}\n{}: {}/{}\n{}: {}", t!("flatpak_ref"), transaction_operation.get_ref().unwrap_or(libflatpak::glib::GString::from_string_unchecked("Unknown".to_owned())), t!("flatpak_status") ,transaction_progress.status().unwrap_or(libflatpak::glib::GString::from_string_unchecked("Unknown".to_owned())), t!("flatpak_transaction_bytes_transferred"), convert(transaction_progress.bytes_transferred() as f64), convert(transaction_operation.download_size() as f64), t!("flatpak_transaction_installed_size"), convert(transaction_operation.installed_size() as f64));
                transaction_status_sender.send_blocking(status_message).expect("transaction_status_receiver closed!");
                transaction_percent_sender.send_blocking(transaction_progress.progress().try_into().unwrap_or(0)).expect("transaction_percent_receiver closed!");
            }));
            };

        //

        let (flatpak_installation, flatpak_transaction) = match is_system {
            true => {
                        let installation = libflatpak::Installation::new_system(cancellable_no).unwrap();
                        let transaction = libflatpak::Transaction::for_installation(&installation, cancellable_no).unwrap();
                        (installation, transaction)
            }
            false => {
                let installation = libflatpak::Installation::new_user(cancellable_no).unwrap();
                let transaction = libflatpak::Transaction::for_installation(&installation, cancellable_no).unwrap();
                (installation, transaction)
            }
        };

        flatpak_transaction.connect_new_operation(transaction_run_closure.clone());

        match get_data_from_filepath(&flatref_path_clone0) {
            Ok(t) => {
                flatpak_transaction.add_install_flatpakref(&t).unwrap();
            }
            Err(e) => {
                transaction_status_sender
                    .send_blocking(e.to_string())
                    .expect("transaction_sync_status_receiver closed");
                transaction_status_sender
                    .send_blocking("FN_OVERRIDE_FAILED".to_owned())
                    .expect("transaction_sync_status_receiver closed");
                panic!("{}", e);
            }
        }

        //
    
        match flatpak_transaction.run(cancellable_no) {
            Ok(_) => {
                transaction_status_sender
                    .send_blocking("FN_OVERRIDE_SUCCESSFUL".to_owned())
                    .expect("transaction_sync_status_receiver closed");
            }
            Err(e) => {
                transaction_status_sender
                    .send_blocking(e.to_string())
                    .expect("transaction_sync_status_receiver closed");
                transaction_status_sender
                    .send_blocking("FN_OVERRIDE_FAILED".to_owned())
                    .expect("transaction_sync_status_receiver closed");
                panic!("{}", e);
            }
        }
    });

    let log_file_path = format!(
        "/tmp/pika-flatpak-transaction_{}.log",
        chrono::offset::Local::now().format("%Y-%m-%d_%H:%M")
    );

    let log_file_path_clone0 = log_file_path.clone();

    if !std::path::Path::new(&log_file_path).exists() {
        match std::fs::File::create(&log_file_path) {
            Ok(_) => {}
            Err(_) => {
                eprintln!("Warning: {} file couldn't be created", log_file_path);
            }
        };
    }

    let flatpak_transaction_dialog_child_box =
        Box::builder().orientation(Orientation::Vertical).build();

    let flatpak_transaction_dialog_progress_bar =
        ProgressBar::builder().show_text(true).hexpand(true).build();

    let flatpak_transaction_dialog_spinner = Spinner::builder()
        .hexpand(true)
        .valign(Align::Start)
        .halign(Align::Center)
        .spinning(true)
        .height_request(128)
        .width_request(128)
        .build();

    flatpak_transaction_dialog_child_box.append(&flatpak_transaction_dialog_spinner);
    flatpak_transaction_dialog_child_box.append(&flatpak_transaction_dialog_progress_bar);

    let flatpak_transaction_dialog = 
            adw::MessageDialog::builder()
            .transient_for(&window)
            .extra_child(&flatpak_transaction_dialog_child_box)
            .heading(t!("flatpak_transaction_dialog_heading"))
            .width_request(500)
            .build();

    flatpak_transaction_dialog.add_response(
        "flatpak_transaction_dialog_ok",
        &t!("flatpak_transaction_dialog_ok_label").to_string(),
    );

    let flatpak_transaction_dialog_child_box_done =
        Box::builder().orientation(Orientation::Vertical).build();

    let flatpak_transaction_log_image = Image::builder()
        .pixel_size(128)
        .halign(Align::Center)
        .build();

    let flatpak_transaction_log_button = Button::builder()
        .label(t!("flatpak_transaction_dialog_open_log_file_label"))
        .halign(Align::Center)
        .margin_start(15)
        .margin_end(15)
        .margin_top(15)
        .margin_bottom(15)
        .build();

    flatpak_transaction_dialog_child_box_done.append(&flatpak_transaction_log_image);
    flatpak_transaction_dialog_child_box_done.append(&flatpak_transaction_log_button);

    flatpak_transaction_dialog.set_response_enabled("flatpak_transaction_dialog_ok", false);
    flatpak_transaction_dialog.set_close_response("flatpak_transaction_dialog_ok");

    let transaction_percent_server_context = MainContext::default();
    // The main loop executes the asynchronous block
    transaction_percent_server_context.spawn_local(clone!(
        #[weak]
        flatpak_transaction_dialog_progress_bar,
        async move {
            while let Ok(state) = transaction_percent_receiver.recv().await {
                flatpak_transaction_dialog_progress_bar.set_fraction((state as f32 / 100.0).into());
            }
        }
    ));

    let transaction_status_server_context = MainContext::default();
    // The main loop executes the asynchronous block
    transaction_status_server_context.spawn_local(clone!(
        #[weak]
        flatpak_transaction_dialog,
        #[weak]
        flatpak_transaction_dialog_child_box,
        #[strong]
        flatpak_transaction_dialog_child_box_done,
        #[strong]
        flatpak_transaction_log_image,
        async move {
            while let Ok(state) = transaction_status_receiver.recv().await {
                match state.as_ref() {
                    "FN_OVERRIDE_SUCCESSFUL" => {
                        flatpak_transaction_dialog_child_box.set_visible(false);
                        flatpak_transaction_log_image.set_icon_name(Some("face-cool-symbolic"));
                        flatpak_transaction_dialog
                            .set_extra_child(Some(&flatpak_transaction_dialog_child_box_done));
                        flatpak_transaction_dialog.set_title(Some(
                            &t!("flatpak_transaction_dialog_status_successful").to_string(),
                        ));
                        flatpak_transaction_dialog
                            .set_response_enabled("flatpak_transaction_dialog_ok", true);
                    }
                    "FN_OVERRIDE_FAILED" => {
                        flatpak_transaction_dialog_child_box.set_visible(false);
                        flatpak_transaction_log_image.set_icon_name(Some("dialog-error-symbolic"));
                        flatpak_transaction_dialog
                            .set_extra_child(Some(&flatpak_transaction_dialog_child_box_done));
                        flatpak_transaction_dialog.set_title(Some(
                            &t!("flatpak_transaction_dialog_status_failed").to_string(),
                        ));
                        flatpak_transaction_dialog
                            .set_response_enabled("flatpak_transaction_dialog_ok", true);
                        flatpak_transaction_dialog
                            .set_response_enabled("flatpak_transaction_dialog_open_log_file", true);
                    }
                    _ => {
                        flatpak_transaction_dialog.set_body(&state);
                        let mut log_file = OpenOptions::new()
                            .write(true)
                            .append(true)
                            .open(&log_file_path)
                            .unwrap();

                        if let Err(e) = writeln!(
                            log_file,
                            "[{}] {}",
                            chrono::offset::Local::now().format("%Y/%m/%d_%H:%M"),
                            state
                        ) {
                            eprintln!("Couldn't write to file: {}", e);
                        }
                    }
                }
            }
        }
    ));

    let retry_signal_action0 = retry_signal_action.clone();
    let flatpak_retry_signal_action0 = flatpak_retry_signal_action.clone();

    flatpak_transaction_log_button.connect_clicked(move |_| {
        let _ = Command::new("xdg-open")
            .arg(log_file_path_clone0.to_owned())
            .spawn();
    });

    flatpak_transaction_dialog.choose(None::<&gio::Cancellable>, move |choice| {
        match choice.as_str() {
            "flatpak_transaction_dialog_ok" => {
                retry_signal_action0.activate(None);
                flatpak_retry_signal_action0.activate(None);
            }
            _ => {}
        }
    });
}

fn get_data_from_filepath(filepath: &str) -> Result<libflatpak::glib::Bytes, std::io::Error> {
    let data = std::fs::read_to_string(filepath)?;
        
    let bytes = data.as_bytes();

    let glib_bytes = libflatpak::glib::Bytes::from(bytes);
    Ok(glib_bytes)
}

fn add_flatpakref_remote(reload_action: &gio::SimpleAction, remote_name: &str, remote_url: &str, is_system: bool) {
    let flatpak_installation = match is_system {
        true => "--system",
        false => "--user"
    };    

    match duct::cmd!("flatpak", "remote-add",  "--if-not-exists", &flatpak_installation, remote_name, remote_url).run() {
        Ok(_) => {
            reload_action.activate(None);
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