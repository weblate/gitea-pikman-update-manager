use crate::flatpak_ref_row::FlatpakRefRow;
use adw::gio::SimpleAction;
use adw::prelude::*;
use gtk::glib::*;
use gtk::*;
use libflatpak::prelude::*;
use libflatpak::Transaction;
use pretty_bytes::converter::convert;
use serde::Serialize;
use serde_json::Value;
use std::cell::RefCell;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::{fs, thread};
use tokio::runtime::Runtime;

struct FlatpakChangesInfo {
    system_flatref_count: u64,
    user_flatref_count: u64,
    total_download_size: u64,
    total_installed_size: i64,
}
/*#[derive(Serialize)]
struct Exclusions {
    exclusions: Vec<Value>,
}*/

impl FlatpakChangesInfo {
    fn add_system(&mut self) {
        self.system_flatref_count += 1;
    }
    fn add_user(&mut self) {
        self.user_flatref_count += 1;
    }

    fn increase_total_download_size_by(&mut self, value: u64) {
        self.total_download_size += value;
    }

    fn increase_total_installed_size_by(&mut self, value: u64) {
        self.total_installed_size += value as i64;
    }

    fn decrease_total_installed_size_by(&mut self, value: u64) {
        self.total_installed_size -= value as i64;
    }
}

pub fn flatpak_process_update(
    system_refs_for_upgrade_vec_opt: Option<&Vec<FlatpakRefRow>>,
    user_refs_for_upgrade_vec_opt: Option<&Vec<FlatpakRefRow>>,
    system_refs_for_upgrade_vec_all: &Vec<FlatpakRefRow>,
    user_refs_for_upgrade_vec_all: &Vec<FlatpakRefRow>,
    window: adw::ApplicationWindow,
    retry_signal_action: &SimpleAction,
) {
    let cancellable = libflatpak::gio::Cancellable::NONE;
    // Emulate Flatpak Full Upgrade to get transaction info
    let mut flatpak_changes_struct = FlatpakChangesInfo {
        system_flatref_count: 0,
        user_flatref_count: 0,
        total_download_size: 0,
        total_installed_size: 0,
    };

    let mut system_refs_for_upgrade_vec = Vec::new();

    match system_refs_for_upgrade_vec_opt {
        Some(t) => {
            for flatpak_row in t {
                flatpak_changes_struct.add_system();
                //
                let installed_size_installed = flatpak_row.flatref_installed_size_installed();
                let installed_size_remote = flatpak_row.flatref_installed_size_installed();
                let installed_download_size = flatpak_row.flatref_download_size();
                let ref_format = flatpak_row.flatref_ref_format();
                //
                flatpak_changes_struct.decrease_total_installed_size_by(installed_size_installed);
                flatpak_changes_struct.increase_total_installed_size_by(installed_size_remote);
                //
                flatpak_changes_struct.increase_total_download_size_by(installed_download_size);
                //
                system_refs_for_upgrade_vec.push(ref_format);
            }
        }
        None => {
            for flatpak_row in system_refs_for_upgrade_vec_all {
                flatpak_changes_struct.add_system();
                //
                let installed_size_installed = flatpak_row.flatref_installed_size_installed();
                let installed_size_remote = flatpak_row.flatref_installed_size_installed();
                let installed_download_size = flatpak_row.flatref_download_size();
                let ref_format = flatpak_row.flatref_ref_format();
                //
                flatpak_changes_struct.decrease_total_installed_size_by(installed_size_installed);
                flatpak_changes_struct.increase_total_installed_size_by(installed_size_remote);
                //
                flatpak_changes_struct.increase_total_download_size_by(installed_download_size);
                //
                system_refs_for_upgrade_vec.push(ref_format);
            }
        }
    };

    let mut user_refs_for_upgrade_vec = Vec::new();

    match user_refs_for_upgrade_vec_opt {
        Some(t) => {
            for flatpak_row in t {
                flatpak_changes_struct.add_user();
                //
                let installed_size_installed = flatpak_row.flatref_installed_size_installed();
                let installed_size_remote = flatpak_row.flatref_installed_size_installed();
                let installed_download_size = flatpak_row.flatref_download_size();
                let ref_format = flatpak_row.flatref_ref_format();
                //
                flatpak_changes_struct.decrease_total_installed_size_by(installed_size_installed);
                flatpak_changes_struct.increase_total_installed_size_by(installed_size_remote);
                //
                flatpak_changes_struct.increase_total_download_size_by(installed_download_size);
                //
                user_refs_for_upgrade_vec.push(ref_format);
            }
        }
        None => {
            for flatpak_row in user_refs_for_upgrade_vec_all {
                flatpak_changes_struct.add_user();
                //
                let installed_size_installed = flatpak_row.flatref_installed_size_installed();
                let installed_size_remote = flatpak_row.flatref_installed_size_installed();
                let installed_download_size = flatpak_row.flatref_download_size();
                let ref_format = flatpak_row.flatref_ref_format();
                //
                flatpak_changes_struct.decrease_total_installed_size_by(installed_size_installed);
                flatpak_changes_struct.increase_total_installed_size_by(installed_size_remote);
                //
                flatpak_changes_struct.increase_total_download_size_by(installed_download_size);
                //
                user_refs_for_upgrade_vec.push(ref_format);
            }
        }
    };

    let flatpak_confirm_dialog_child_box =
        Box::builder().orientation(Orientation::Vertical).build();

    let flatpak_update_dialog_badges_size_group = SizeGroup::new(SizeGroupMode::Both);
    let flatpak_update_dialog_badges_size_group0 = SizeGroup::new(SizeGroupMode::Both);
    let flatpak_update_dialog_badges_size_group1 = SizeGroup::new(SizeGroupMode::Both);

    flatpak_confirm_dialog_child_box.append(&create_color_badge(
        &t!("system_flatref_count_badge_label"),
        &flatpak_changes_struct.system_flatref_count.to_string(),
        "background-accent-bg",
        &flatpak_update_dialog_badges_size_group,
        &flatpak_update_dialog_badges_size_group0,
        &flatpak_update_dialog_badges_size_group1,
    ));

    flatpak_confirm_dialog_child_box.append(&create_color_badge(
        &t!("user_flatref_count_badge_label"),
        &flatpak_changes_struct.user_flatref_count.to_string(),
        "background-accent-bg",
        &flatpak_update_dialog_badges_size_group,
        &flatpak_update_dialog_badges_size_group0,
        &flatpak_update_dialog_badges_size_group1,
    ));

    flatpak_confirm_dialog_child_box.append(&create_color_badge(
        &t!("total_download_size_badge_label"),
        &convert(flatpak_changes_struct.total_download_size as f64),
        "background-accent-bg",
        &flatpak_update_dialog_badges_size_group,
        &flatpak_update_dialog_badges_size_group0,
        &flatpak_update_dialog_badges_size_group1,
    ));

    flatpak_confirm_dialog_child_box.append(&create_color_badge(
        &t!("total_installed_size_badge_label"),
        &convert(flatpak_changes_struct.total_installed_size as f64),
        "background-accent-bg",
        &flatpak_update_dialog_badges_size_group,
        &flatpak_update_dialog_badges_size_group0,
        &flatpak_update_dialog_badges_size_group1,
    ));

    let flatpak_confirm_dialog = adw::MessageDialog::builder()
        .transient_for(&window)
        .heading(t!("flatpak_confirm_dialog_heading"))
        .body(t!("flatpak_confirm_dialog_body"))
        .extra_child(&flatpak_confirm_dialog_child_box)
        .build();

    flatpak_confirm_dialog.add_response(
        "flatpak_confirm_dialog_cancel",
        &t!("flatpak_confirm_dialog_cancel_label").to_string(),
    );

    flatpak_confirm_dialog.add_response(
        "flatpak_confirm_dialog_confirm",
        &t!("flatpak_confirm_dialog_confirm_label").to_string(),
    );

    flatpak_confirm_dialog.set_response_appearance(
        "flatpak_confirm_dialog_confirm",
        adw::ResponseAppearance::Destructive,
    );

    flatpak_confirm_dialog.set_default_response(Some("flatpak_confirm_dialog_confirm"));
    flatpak_confirm_dialog.set_close_response("flatpak_confirm_dialog_cancel");

    let retry_signal_action0 = retry_signal_action.clone();
    flatpak_confirm_dialog
        .clone()
        .choose(None::<&gio::Cancellable>, move |choice| {
            if choice == "flatpak_confirm_dialog_confirm" {
                flatpak_run_transactions(
                    system_refs_for_upgrade_vec,
                    user_refs_for_upgrade_vec,
                    window,
                    &retry_signal_action0,
                );
            }
        });
}

fn flatpak_run_transactions(
    system_refs_for_upgrade_vec: Vec<String>,
    user_refs_for_upgrade_vec: Vec<String>,
    window: adw::ApplicationWindow,
    retry_signal_action: &SimpleAction,
) {
    let (transaction_percent_sender, transaction_percent_receiver) =
        async_channel::unbounded::<u32>();
    let transaction_percent_sender = transaction_percent_sender.clone();
    let (transaction_status_sender, transaction_status_receiver) =
        async_channel::unbounded::<String>();
    let transaction_status_sender = transaction_status_sender.clone();

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

        let flatpak_system_installation =
            libflatpak::Installation::new_system(cancellable_no).unwrap();
        let flatpak_system_transaction =
            libflatpak::Transaction::for_installation(&flatpak_system_installation, cancellable_no)
                .unwrap();

        for ref_format in system_refs_for_upgrade_vec {
            flatpak_system_transaction
                .add_update(&ref_format, &[], None)
                .unwrap();
        }

        flatpak_system_transaction.connect_new_operation(transaction_run_closure.clone());

        match flatpak_system_transaction.run(cancellable_no) {
            Ok(_) => {}
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

        let flatpak_user_installation = libflatpak::Installation::new_user(cancellable_no).unwrap();
        let flatpak_user_transaction =
            libflatpak::Transaction::for_installation(&flatpak_user_installation, cancellable_no)
                .unwrap();

        flatpak_user_transaction.connect_new_operation(transaction_run_closure);

        for ref_format in user_refs_for_upgrade_vec {
            flatpak_user_transaction
                .add_update(&ref_format, &[], None)
                .unwrap();
        }

        match flatpak_user_transaction.run(cancellable_no) {
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

    if !Path::new(&log_file_path).exists() {
        match fs::File::create(&log_file_path) {
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

    let flatpak_transaction_dialog = adw::MessageDialog::builder()
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

    flatpak_transaction_log_button.connect_clicked(move |_| {
        let _ = Command::new("xdg-open")
            .arg(log_file_path_clone0.to_owned())
            .spawn();
    });

    flatpak_transaction_dialog.choose(None::<&gio::Cancellable>, move |choice| {
        match choice.as_str() {
            "flatpak_transaction_dialog_ok" => {
                retry_signal_action0.activate(None);
            }
            _ => {}
        }
    });
}

fn create_color_badge(
    label0_text: &str,
    label1_text: &str,
    css_style: &str,
    group_size: &SizeGroup,
    group_size0: &SizeGroup,
    group_size1: &SizeGroup,
) -> ListBox {
    let badge_box = Box::builder().build();

    let label0 = Label::builder()
        .label(label0_text)
        .margin_start(5)
        .margin_end(5)
        .margin_bottom(1)
        .margin_top(1)
        .valign(Align::Center)
        .halign(Align::Center)
        .hexpand(true)
        .vexpand(true)
        .build();
    group_size0.add_widget(&label0);

    let label_separator = Separator::builder().build();

    let label1 = Label::builder()
        .label(label1_text)
        .margin_start(3)
        .margin_end(0)
        .margin_bottom(1)
        .margin_top(1)
        .valign(Align::Center)
        .halign(Align::Center)
        .hexpand(true)
        .vexpand(true)
        .build();
    group_size1.add_widget(&label1);

    label1.add_css_class(css_style);

    badge_box.append(&label0);
    badge_box.append(&label_separator);
    badge_box.append(&label1);

    let boxedlist = ListBox::builder()
        .selection_mode(SelectionMode::None)
        .halign(Align::Center)
        .margin_start(10)
        .margin_end(10)
        .margin_bottom(10)
        .margin_top(10)
        .build();

    boxedlist.add_css_class("boxed-list");
    boxedlist.append(&badge_box);
    group_size.add_widget(&boxedlist);
    boxedlist
}
