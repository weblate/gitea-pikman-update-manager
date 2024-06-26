use std::{cell::RefCell, default, sync::OnceLock};

use adw::*;
use adw::{prelude::*, subclass::prelude::*};
use glib::{subclass::Signal, Properties};
use gtk::{Align, glib, Orientation, SizeGroupMode, SelectionMode};
use gtk::Orientation::Horizontal;
use crate::apt_update_page::AptPackageSocket;

// ANCHOR: custom_button
// Object holding the state
#[derive(Properties, Default)]
#[properties(wrapper_type = super::AptPackageRow)]
pub struct AptPackageRow {
    #[property(get, set)]
    package_name: RefCell<String>,
    #[property(get, set)]
    package_arch: RefCell<String>,
    #[property(get, set)]
    package_installed_version: RefCell<String>,
    #[property(get, set)]
    package_candidate_version: RefCell<String>
}
// ANCHOR_END: custom_button

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for AptPackageRow {
    const NAME: &'static str = "AptPackageRow";
    type Type = super::AptPackageRow;
    type ParentType = adw::ActionRow;
}

// ANCHOR: object_impl
// Trait shared by all GObjects
#[glib::derived_properties]
impl ObjectImpl for AptPackageRow {
    fn signals() -> &'static [Signal] {
        static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
        SIGNALS.get_or_init(|| vec![Signal::builder("row-deleted").build()])
    }
    fn constructed(&self) {
        self.parent_constructed();

        // Bind label to number
        // `SYNC_CREATE` ensures that the label will be immediately set
        let obj = self.obj();

        let prefix_box = gtk::Box::new(Orientation::Horizontal, 0);
        prefix_box.append(&create_version_badge("1.0-100-pika1".to_string(), "1.1-101-pika1".to_string()));
        obj.add_prefix(&prefix_box);

        // Bind label to number
        // `SYNC_CREATE` ensures that the label will be immediately set
        //let obj = self.obj();
        //obj.bind_property("package", &basic_expander_row_package_label, "label")
        //    .sync_create()
        //    .bidirectional()
        //    .build();
    }
}
// Trait shared by all widgets
impl WidgetImpl for AptPackageRow {}

// Trait shared by all buttons
// Trait shared by all buttons

impl ListBoxRowImpl for AptPackageRow {}
impl PreferencesRowImpl for AptPackageRow {}
impl ActionRowImpl for AptPackageRow {}

fn create_version_badge(installed_version: String, candidate_version: String) -> gtk::ListBox {
    let (base_version, installed_diff, candidate_diff) = get_diff_by_prefix(installed_version, candidate_version);

    let badge_box = gtk::Box::builder()
        .halign(Align::Start)
        .hexpand(false)
        .orientation(Orientation::Horizontal)
        .margin_start(5)
        .margin_end(5)
        .margin_bottom(5)
        .margin_top(5)
        .build();

    let installed_version_box = gtk::Box::builder()
        .halign(Align::Start)
        .hexpand(false)
        .orientation(Orientation::Horizontal)
        .build();

    let installed_version_base_version_label = gtk::Label::builder()
        .label(&base_version)
        .valign(Align::Center)
        .halign(Align::Start)
        .hexpand(false)
        .vexpand(true)
        .build();

    let installed_diff_label = gtk::Label::builder()
        .label(installed_diff)
        .valign(Align::Center)
        .halign(Align::Start)
        .hexpand(false)
        .vexpand(true)
        .build();
    installed_diff_label.add_css_class("destructive-color-text");

    installed_version_box.append(&installed_version_base_version_label.clone());
    installed_version_box.append(&installed_diff_label);

    let label_separator = gtk::Separator::builder()
        .margin_start(5)
        .margin_end(5)
        .margin_bottom(5)
        .margin_top(5).build();

    let candidate_version_box = gtk::Box::builder()
        .halign(Align::Start)
        .hexpand(false)
        .orientation(Orientation::Horizontal)
        .build();

    let candidate_version_base_version_label = gtk::Label::builder()
        .label(base_version)
        .valign(Align::Center)
        .halign(Align::Start)
        .hexpand(false)
        .vexpand(true)
        .build();

    let candidate_diff_label = gtk::Label::builder()
        .label(candidate_diff)
        .valign(Align::Center)
        .halign(Align::Start)
        .hexpand(false)
        .vexpand(true)
        .build();
    candidate_diff_label.add_css_class("success-color-text");

    candidate_version_box.append(&candidate_version_base_version_label);
    candidate_version_box.append(&candidate_diff_label);

    badge_box.append(&installed_version_box);
    badge_box.append(&label_separator);
    badge_box.append(&candidate_version_box);

    let boxedlist = gtk::ListBox::builder()
        .selection_mode(SelectionMode::None)
        .halign(Align::Start)
        .valign(Align::End)
        .build();

    boxedlist.add_css_class("boxed-list");
    boxedlist.append(&badge_box);
    boxedlist
}

pub fn get_diff_by_prefix(xs: String, ys: String) -> (String, String, String) {
    let mut count = String::new();
    for (x,y) in xs.chars().zip(ys.chars()) {
        if x == y {
            count.push(x)
        } else {
            break
        }
    }
    let count_clone0 = count.clone();
    return(count_clone0, xs.trim_start_matches(&count.as_str()).to_string(), ys.trim_start_matches(&count.as_str()).to_string())
}