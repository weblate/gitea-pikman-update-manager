use adw::gio::SimpleAction;
use adw::prelude::*;
use gtk::glib::*;
use gtk::*;
use pika_unixsocket_tools::pika_unixsocket_tools::{
    start_socket_server, start_socket_server_no_log,
};
use pretty_bytes::converter::convert;
use rust_apt::cache::Upgrade;
use rust_apt::new_cache;
use serde::Serialize;
use serde_json::Value;
use std::path::Path;
use std::process::Command;
use std::{thread};
use std::cell::RefCell;
use std::rc::Rc;
use tokio::runtime::Runtime;

struct AptChangesInfo {
    package_count_upgrade: u64,
    package_count_install: u64,
    package_count_downgrade: u64,
    package_count_remove: u64,
    total_download_size: u64,
    total_installed_size: i64,
}
#[derive(Serialize)]
struct Exclusions {
    exclusions: Vec<Value>,
}

impl AptChangesInfo {
    fn add_upgrade(&mut self) {
        self.package_count_upgrade += 1;
    }
    fn add_install(&mut self) {
        self.package_count_install += 1;
    }
    fn add_downgrade(&mut self) {
        self.package_count_downgrade += 1;
    }
    fn add_remove(&mut self) {
        self.package_count_remove += 1;
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

pub fn apt_process_update(
    excluded_updates_vec: &Vec<String>,
    window: adw::ApplicationWindow,
    retry_signal_action: &SimpleAction,
) {
    let excluded_updates_alert_dialog = adw::MessageDialog::builder()
        .transient_for(&window)
        .heading(t!("excluded_updates_alert_dialog_heading"))
        .body(t!("excluded_updates_alert_dialog_body"))
        .build();

    excluded_updates_alert_dialog.add_response(
        "excluded_updates_alert_dialog_cancel",
        &t!("excluded_updates_alert_dialog_cancel_label").to_string(),
    );

    excluded_updates_alert_dialog.add_response(
        "excluded_updates_alert_continue",
        &t!("excluded_updates_alert_continue_label").to_string(),
    );

    excluded_updates_alert_dialog.set_response_appearance(
        "excluded_updates_alert_continue",
        adw::ResponseAppearance::Destructive,
    );

    excluded_updates_alert_dialog.set_default_response(Some("excluded_updates_alert_continue"));

    let excluded_updates_alert_dialog_action =
        SimpleAction::new("excluded_updates_alert_dialog_action", None);

    excluded_updates_alert_dialog_action.connect_activate(
        clone!(#[weak] window, #[weak] retry_signal_action, #[strong] excluded_updates_vec, move |_, _| {
            apt_confirm_window(&excluded_updates_vec, window, &retry_signal_action)
        }),
    );

    if excluded_updates_vec.is_empty() {
        excluded_updates_alert_dialog_action.activate(None);
    } else {
        excluded_updates_alert_dialog.choose(None::<&gio::Cancellable>, move |choice| {
            if choice == "excluded_updates_alert_continue" {
                excluded_updates_alert_dialog_action.activate(None);
            }
        });
    }
}

fn apt_confirm_window(
    excluded_updates_vec: &Vec<String>,
    window: adw::ApplicationWindow,
    retry_signal_action: &SimpleAction,
) {
    let to_be_removed_packages_vec: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    // Emulate Apt Full Upgrade to get transaction info
    let mut apt_changes_struct = AptChangesInfo {
        package_count_upgrade: 0,
        package_count_install: 0,
        package_count_downgrade: 0,
        package_count_remove: 0,
        total_download_size: 0,
        total_installed_size: 0,
    };

    let apt_cache = new_cache!().unwrap();
    let apt_upgrade_cache = new_cache!().unwrap();

    apt_cache.upgrade(Upgrade::FullUpgrade).unwrap();

    for change in apt_cache.get_changes(false) {
        if !excluded_updates_vec
            .iter()
            .any(|e| change.name().contains(e))
        {
            let pkg = apt_upgrade_cache.get(change.name()).unwrap();
            if change.marked_upgrade() || change.marked_install() || change.marked_downgrade() {
                pkg.mark_install(true, false);
            } else if change.marked_delete() {
                pkg.mark_delete(false);
                to_be_removed_packages_vec.borrow_mut().push(pkg.name().to_owned());
            }
            pkg.protect();
        }
    }

    apt_upgrade_cache.resolve(true).unwrap();

    println!("{}", t!("gui_changes_emu_msg_0"));
    for change in apt_upgrade_cache.get_changes(false) {
        if change.is_installed() {
            apt_changes_struct
                .decrease_total_installed_size_by(change.installed().unwrap().installed_size());
        }
        if change.marked_upgrade() && change.is_installed() {
            println!("{}: {}", t!("gui_changes_emu_msg_upgrading"), change.name());
            apt_changes_struct.add_upgrade();
            apt_changes_struct.increase_total_download_size_by(change.candidate().unwrap().size());
            apt_changes_struct
                .increase_total_installed_size_by(change.candidate().unwrap().installed_size());
        } else if change.marked_install() || change.marked_upgrade() && !change.is_installed() {
            println!(
                "{}: {}",
                t!("gui_changes_emu_msg_installing"),
                change.name()
            );
            apt_changes_struct.add_install();
            apt_changes_struct.increase_total_download_size_by(change.candidate().unwrap().size());
            apt_changes_struct
                .increase_total_installed_size_by(change.candidate().unwrap().installed_size());
        } else if change.marked_downgrade() {
            println!(
                "{}: {}",
                t!("gui_changes_emu_msg_downgrading"),
                change.name()
            );
            apt_changes_struct.add_downgrade();
            apt_changes_struct.increase_total_download_size_by(change.candidate().unwrap().size());
            apt_changes_struct
                .increase_total_installed_size_by(change.candidate().unwrap().installed_size());
        } else if change.marked_delete() {
            println!("{}: {}", t!("gui_changes_emu_msg_removing"), change.name());
            apt_changes_struct.add_remove();
        }
    }

    let apt_confirm_dialog_child_box = Box::builder()
        .orientation(Orientation::Vertical)
        .build();

    let apt_update_dialog_badges_size_group = SizeGroup::new(SizeGroupMode::Both);
    let apt_update_dialog_badges_size_group0 = SizeGroup::new(SizeGroupMode::Both);
    let apt_update_dialog_badges_size_group1 = SizeGroup::new(SizeGroupMode::Both);

    apt_confirm_dialog_child_box.append(&create_color_badge(
        &t!("package_count_upgrade_badge_label"),
        &apt_changes_struct.package_count_upgrade.to_string(),
        "background-accent-bg",
        &apt_update_dialog_badges_size_group,
        &apt_update_dialog_badges_size_group0,
        &apt_update_dialog_badges_size_group1,
    ));

    apt_confirm_dialog_child_box.append(&create_color_badge(
        &t!("package_count_install_badge_label"),
        &apt_changes_struct.package_count_install.to_string(),
        "background-accent-bg",
        &apt_update_dialog_badges_size_group,
        &apt_update_dialog_badges_size_group0,
        &apt_update_dialog_badges_size_group1,
    ));

    apt_confirm_dialog_child_box.append(&create_color_badge(
        &t!("package_count_downgrade_badge_label"),
        &apt_changes_struct.package_count_downgrade.to_string(),
        "background-accent-bg",
        &apt_update_dialog_badges_size_group,
        &apt_update_dialog_badges_size_group0,
        &apt_update_dialog_badges_size_group1,
    ));

    apt_confirm_dialog_child_box.append(&create_color_badge(
        &t!("package_count_remove_badge_label"),
        &apt_changes_struct.package_count_remove.to_string(),
        "background-accent-bg",
        &apt_update_dialog_badges_size_group,
        &apt_update_dialog_badges_size_group0,
        &apt_update_dialog_badges_size_group1,
    ));

    apt_confirm_dialog_child_box.append(&create_color_badge(
        &t!("total_download_size_badge_label"),
        &convert(apt_changes_struct.total_download_size as f64),
        "background-accent-bg",
        &apt_update_dialog_badges_size_group,
        &apt_update_dialog_badges_size_group0,
        &apt_update_dialog_badges_size_group1,
    ));

    apt_confirm_dialog_child_box.append(&create_color_badge(
        &t!("total_installed_size_badge_label"),
        &convert(apt_changes_struct.total_installed_size as f64),
        "background-accent-bg",
        &apt_update_dialog_badges_size_group,
        &apt_update_dialog_badges_size_group0,
        &apt_update_dialog_badges_size_group1,
    ));

    let apt_confirm_dialog = adw::MessageDialog::builder()
        .transient_for(&window)
        .heading(t!("apt_confirm_dialog_heading"))
        .body(t!("apt_confirm_dialog_body"))
        .extra_child(&apt_confirm_dialog_child_box)
        .build();

    apt_confirm_dialog.add_response(
        "apt_confirm_dialog_cancel",
        &t!("apt_confirm_dialog_cancel_label").to_string(),
    );

    apt_confirm_dialog.add_response(
        "apt_confirm_dialog_confirm",
        &t!("apt_confirm_dialog_confirm_label").to_string(),
    );

    apt_confirm_dialog.set_response_appearance(
        "apt_confirm_dialog_confirm",
        adw::ResponseAppearance::Destructive,
    );

    apt_confirm_dialog.set_default_response(Some("apt_confirm_dialog_confirm"));
    apt_confirm_dialog.set_close_response("apt_confirm_dialog_cancel");

    let json_file_path = "/tmp/pika-apt-exclusions.json";

    if Path::new(json_file_path).exists() {
        std::fs::remove_file(json_file_path).expect("Failed to remove old json file");
    }

    if !excluded_updates_vec.is_empty() {
        let exclusions_array = Exclusions {
            exclusions: excluded_updates_vec
                .into_iter()
                .map(|i| serde_json::from_str(format!("{{\"package\":\"{}\"}}", i).as_str()))
                .collect::<Result<Vec<Value>, _>>()
                .unwrap(),
        };

        std::fs::write(
            json_file_path,
            serde_json::to_string_pretty(&exclusions_array).unwrap(),
        )
        .expect("Failed to write to json file");
    }

    let apt_confirm_start_signal_action = SimpleAction::new("apt_confirm_start", None);

    apt_confirm_start_signal_action.connect_activate(clone!(#[weak] window, #[strong] retry_signal_action, #[strong] apt_confirm_dialog, move |_, _| {
        let retry_signal_action0 = retry_signal_action.clone();
        apt_confirm_dialog.clone().choose(None::<&gio::Cancellable>, move |choice| {
        if choice == "apt_confirm_dialog_confirm" {
            apt_full_upgrade_from_socket(window, &retry_signal_action0);
        }
    });
    }));

    let to_be_removed_packages_borrow = to_be_removed_packages_vec.borrow();
    if to_be_removed_packages_borrow.is_empty() {
        apt_confirm_start_signal_action.activate(None);
    } else {
        let apt_remove_confirm_text_buffer = TextBuffer::builder()
            .text(to_be_removed_packages_borrow.iter().map(|x| x.to_string() + "\n").collect::<String>() + "\n")
            .build();

        let apt_remove_confirm_text_view = TextView::builder()
            .buffer(&apt_remove_confirm_text_buffer)
            .hexpand(true)
            .vexpand(true)
            .margin_top(15)
            .margin_bottom(15)
            .margin_start(15)
            .margin_end(15)
            .editable(false)
            .build();

        let apt_remove_confirm_dialog = adw::MessageDialog::builder()
            .transient_for(&window)
            .heading(t!("apt_remove_confirm_dialog_heading"))
            .body(t!("apt_remove_confirm_dialog_body"))
            .extra_child(&apt_remove_confirm_text_view)
            .build();

        apt_remove_confirm_dialog.add_response(
            "apt_remove_confirm_dialog_cancel",
            &t!("apt_remove_confirm_dialog_cancel_label").to_string(),
        );

        apt_remove_confirm_dialog.add_response(
            "apt_remove_confirm_dialog_confirm",
            &t!("apt_remove_confirm_dialog_confirm_label").to_string(),
        );

        apt_remove_confirm_dialog.set_response_appearance(
            "apt_remove_confirm_dialog_confirm",
            adw::ResponseAppearance::Destructive,
        );

        apt_remove_confirm_dialog.set_default_response(Some("apt_remove_confirm_dialog_confirm"));
        apt_remove_confirm_dialog.set_close_response("apt_remove_confirm_dialog_cancel");

        apt_remove_confirm_dialog.choose(None::<&gio::Cancellable>, move |choice| {
        if choice == "apt_remove_confirm_dialog_confirm" {
            apt_confirm_start_signal_action.activate(None);
        }
        });
    }
}

fn apt_full_upgrade_from_socket(
    window: adw::ApplicationWindow,
    retry_signal_action: &SimpleAction,
) {
    let (upgrade_percent_sender, upgrade_percent_receiver) = async_channel::unbounded::<String>();
    let upgrade_percent_sender = upgrade_percent_sender.clone();
    let (upgrade_status_sender, upgrade_status_receiver) = async_channel::unbounded::<String>();
    let upgrade_status_sender = upgrade_status_sender.clone();
    let upgrade_status_sender_clone0 = upgrade_status_sender.clone();

    let log_file_path = format!(
        "/tmp/pika-apt-upgrade_{}.log",
        chrono::offset::Local::now().format("%Y-%m-%d_%H:%M")
    );
    let log_file_path_clone0 = log_file_path.clone();

    thread::spawn(move || {
        Runtime::new().unwrap().block_on(start_socket_server_no_log(
            upgrade_percent_sender,
            "/tmp/pika_apt_upgrade_percent.sock",
        ));
    });

    thread::spawn(move || {
        Runtime::new().unwrap().block_on(start_socket_server(
            upgrade_status_sender,
            "/tmp/pika_apt_upgrade_status.sock",
            &log_file_path,
        ));
    });

    thread::spawn(move || {
        let apt_upgrade_command = Command::new("pkexec")
            .args([
                "/home/ward/RustroverProjects/pikman-update-manager/target/debug/apt_full_upgrade",
            ])
            .status()
            .unwrap();
        match apt_upgrade_command.code().unwrap() {
            0 => upgrade_status_sender_clone0
                .send_blocking("FN_OVERRIDE_SUCCESSFUL".to_owned())
                .unwrap(),
            53 => {}
            _ => {
                upgrade_status_sender_clone0
                    .send_blocking(t!("upgrade_status_error_perms").to_string())
                    .unwrap();
                upgrade_status_sender_clone0
                    .send_blocking("FN_OVERRIDE_FAILED".to_owned())
                    .unwrap()
            }
        }
    });

    let apt_upgrade_dialog_child_box = Box::builder()
        .orientation(Orientation::Vertical)
        .build();

    let apt_upgrade_dialog_progress_bar = ProgressBar::builder()
        .show_text(true)
        .hexpand(true)
        .build();

    let apt_upgrade_dialog_spinner = Spinner::builder()
        .hexpand(true)
        .valign(Align::Start)
        .halign(Align::Center)
        .spinning(true)
        .height_request(128)
        .width_request(128)
        .build();

    apt_upgrade_dialog_child_box.append(&apt_upgrade_dialog_spinner);
    apt_upgrade_dialog_child_box.append(&apt_upgrade_dialog_progress_bar);

    let apt_upgrade_dialog = adw::MessageDialog::builder()
        .transient_for(&window)
        .extra_child(&apt_upgrade_dialog_child_box)
        .heading(t!("apt_upgrade_dialog_heading"))
        .width_request(500)
        .build();

    apt_upgrade_dialog.add_response(
        "apt_upgrade_dialog_ok",
        &t!("apt_upgrade_dialog_ok_label").to_string(),
    );

    let apt_upgrade_dialog_child_box_done = Box::builder()
        .orientation(Orientation::Vertical)
        .build();

    let apt_upgrade_log_image = Image::builder()
        .pixel_size(128)
        .halign(Align::Center)
        .build();

    let apt_upgrade_log_button = Button::builder()
        .label(t!("apt_upgrade_dialog_open_log_file_label"))
        .halign(Align::Center)
        .margin_start(15)
        .margin_end(15)
        .margin_top(15)
        .margin_bottom(15)
        .build();

    apt_upgrade_dialog_child_box_done.append(&apt_upgrade_log_image);
    apt_upgrade_dialog_child_box_done.append(&apt_upgrade_log_button);

    apt_upgrade_dialog.set_response_enabled("apt_upgrade_dialog_ok", false);
    apt_upgrade_dialog.set_close_response("apt_upgrade_dialog_ok");

    let upgrade_percent_server_context = MainContext::default();
    // The main loop executes the asynchronous block
    upgrade_percent_server_context.spawn_local(clone!(#[weak] apt_upgrade_dialog_progress_bar, async move {
        while let Ok(state) = upgrade_percent_receiver.recv().await {
            match state.as_ref() {
                "FN_OVERRIDE_SUCCESSFUL" => {}
                _ => {
                    match state.parse::<f64>() {
                        Ok(p) => apt_upgrade_dialog_progress_bar.set_fraction(p/100.0),
                        Err(_) => {}
                    }
                }
            }
        }
        }));

    let upgrade_status_server_context = MainContext::default();
    // The main loop executes the asynchronous block
    upgrade_status_server_context.spawn_local(
        clone!(#[weak] apt_upgrade_dialog, #[weak] apt_upgrade_dialog_child_box, #[strong] apt_upgrade_dialog_child_box_done, #[strong] apt_upgrade_log_image, async move {
        while let Ok(state) = upgrade_status_receiver.recv().await {
            match state.as_ref() {
                "FN_OVERRIDE_SUCCESSFUL" => {
                        apt_upgrade_dialog_child_box.set_visible(false);
                        apt_upgrade_log_image.set_icon_name(Some("face-cool-symbolic"));
                        apt_upgrade_dialog.set_extra_child(Some(&apt_upgrade_dialog_child_box_done));
                        apt_upgrade_dialog.set_title(Some(&t!("apt_upgrade_dialog_status_successful").to_string()));
                        apt_upgrade_dialog.set_response_enabled("apt_upgrade_dialog_ok", true);
                    }
                "FN_OVERRIDE_FAILED" => {
                        apt_upgrade_dialog_child_box.set_visible(false);
                        apt_upgrade_log_image.set_icon_name(Some("dialog-error-symbolic"));
                        apt_upgrade_dialog.set_extra_child(Some(&apt_upgrade_dialog_child_box_done));
                        apt_upgrade_dialog.set_title(Some(&t!("apt_upgrade_dialog_status_failed").to_string()));
                        apt_upgrade_dialog.set_response_enabled("apt_upgrade_dialog_ok", true);
                        apt_upgrade_dialog.set_response_enabled("apt_upgrade_dialog_open_log_file", true);
                    }
                _ => apt_upgrade_dialog.set_body(&state)
            }
        }
        }),
    );

    let retry_signal_action0 = retry_signal_action.clone();

    apt_upgrade_log_button.connect_clicked(move |_| {
        let _ = Command::new("xdg-open")
            .arg(log_file_path_clone0.to_owned())
            .spawn();
    });

    apt_upgrade_dialog.choose(None::<&gio::Cancellable>, move |choice| {
        match choice.as_str() {
            "apt_upgrade_dialog_ok" => {
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
