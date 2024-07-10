use rust_apt::new_cache;
use rust_apt::cache::Upgrade;
use adw::prelude::*;
use gtk::glib::*;
use gtk::*;
use pretty_bytes::converter::convert;
use serde_json::{Value};
use std::{fs::*};
use std::path::Path;

struct AptChangesInfo {
    package_count: u64,
    total_download_size: u64,
    total_installed_size: u64
}

impl AptChangesInfo {
    fn add_package(&mut self) {
        self.package_count += 1;
    }

    fn increase_total_download_size_by(&mut self, value: u64) {
        self.total_download_size += value;
    }

    fn increase_total_installed_size_by(&mut self, value: u64) {
        self.total_installed_size += value;
    }
}

pub fn apt_process_update(excluded_updates_vec: &Vec<String>, window: adw::ApplicationWindow) {
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

    let excluded_updates_alert_dialog_action = gio::SimpleAction::new("excluded_updates_alert_dialog_action", None);

    excluded_updates_alert_dialog_action.connect_activate(clone!(@weak window, @strong excluded_updates_vec => move |_, _| {
        apt_confirm_window(&excluded_updates_vec, window)
    }));

    if excluded_updates_vec.is_empty() {
        excluded_updates_alert_dialog_action.activate(None);
    } else {
        excluded_updates_alert_dialog
            .choose(None::<&gio::Cancellable>, move |choice| {
                if choice == "excluded_updates_alert_continue" {
                    excluded_updates_alert_dialog_action.activate(None);
                }
            });
    }
}

fn apt_confirm_window(excluded_updates_vec: &Vec<String>, window: adw::ApplicationWindow) {
    // Emulate Apt Full Upgrade to get transaction info
    let mut apt_changes_struct = AptChangesInfo {
        package_count: 0,
        total_download_size: 0,
        total_installed_size: 0
    };

    let apt_cache = new_cache!().unwrap();

    apt_cache.upgrade(Upgrade::FullUpgrade).unwrap();

    for change in apt_cache.get_changes(false) {
        if !excluded_updates_vec.iter().any(|e| change.name().contains(e)) {
            apt_changes_struct.add_package();
            apt_changes_struct.increase_total_download_size_by(change.candidate().unwrap().size());
            apt_changes_struct.increase_total_installed_size_by(change.candidate().unwrap().installed_size());
        }
    }

    let apt_confirm_dialog_child_box = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .build();

    let apt_update_dialog_badges_size_group = gtk::SizeGroup::new(SizeGroupMode::Both);
    let apt_update_dialog_badges_size_group0 = gtk::SizeGroup::new(SizeGroupMode::Both);
    let apt_update_dialog_badges_size_group1 = gtk::SizeGroup::new(SizeGroupMode::Both);

    apt_confirm_dialog_child_box.append(
        &create_color_badge(
            &t!("package_count_badge_label"),
            &apt_changes_struct.package_count.to_string(),
            "background-accent-bg",
            &apt_update_dialog_badges_size_group,
            &apt_update_dialog_badges_size_group0,
            &apt_update_dialog_badges_size_group1
        )
    );

    apt_confirm_dialog_child_box.append(
        &create_color_badge(
            &t!("total_download_size_badge_label"),
            &convert(apt_changes_struct.total_download_size as f64),
            "background-accent-bg",
            &apt_update_dialog_badges_size_group,
            &apt_update_dialog_badges_size_group0,
            &apt_update_dialog_badges_size_group1
        )
    );


    apt_confirm_dialog_child_box.append(
        &create_color_badge(
            &t!("total_installed_size_badge_label"),
            &convert(apt_changes_struct.total_installed_size as f64),
            "background-accent-bg",
            &apt_update_dialog_badges_size_group,
            &apt_update_dialog_badges_size_group0,
            &apt_update_dialog_badges_size_group1
        )
    );

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


    if !excluded_updates_vec.is_empty() {
        let excluded_updates_values: Vec<Value> = excluded_updates_vec.into_iter()
            .map(|i| serde_json::from_str(format!("{{\"package\":\"{}\"}}", i).as_str()))
            .collect::<Result<Vec<Value>,_>>()
            .unwrap();

        let excluded_updates_values_json = Value::Array(excluded_updates_values);

        let json_file_path = "/tmp/pika-apt-exclusions.json";

        if Path::new(json_file_path).exists() {
            std::fs::remove_file(json_file_path).expect("Failed to remove old json file");
        }

        std::fs::write(json_file_path, serde_json::to_string_pretty(&excluded_updates_values_json).unwrap()).expect("Failed to write to json file");
    }

    apt_confirm_dialog.present();
}

fn create_color_badge(
    label0_text: &str,
    label1_text: &str,
    css_style: &str,
    group_size: &gtk::SizeGroup,
    group_size0: &gtk::SizeGroup,
    group_size1: &gtk::SizeGroup,
) -> gtk::ListBox {
    let badge_box = gtk::Box::builder().build();

    let label0 = gtk::Label::builder()
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

    let label_seprator = gtk::Separator::builder().build();

    let label1 = gtk::Label::builder()
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
    badge_box.append(&label_seprator);
    badge_box.append(&label1);

    let boxedlist = gtk::ListBox::builder()
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