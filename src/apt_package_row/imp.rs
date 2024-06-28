use std::{cell::RefCell, default, sync::OnceLock};

use adw::*;
use adw::{prelude::*, subclass::prelude::*};
use glib::{subclass::Signal, Properties, clone};
use gtk::{*};
use gtk::Orientation::Horizontal;
use crate::apt_update_page::AptPackageSocket;
use std::env;

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
    type ParentType = adw::ExpanderRow;
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
        let current_locale = match env::var_os("LANG") {
            Some(v) => v.into_string().unwrap().chars()
                .take_while(|&ch| ch != '.')
                .collect::<String>(),
            None => panic!("$LANG is not set"),
        };
        rust_i18n::set_locale(&current_locale);

        self.parent_constructed();

        // Bind label to number
        // `SYNC_CREATE` ensures that the label will be immediately set
        let obj = self.obj();

        let prefix_box = gtk::Box::new(Orientation::Vertical, 0);

        let expandable_box = gtk::Box::new(Orientation::Vertical, 0);
        expandable_box.add_css_class("linked");

        obj.connect_package_name_notify(clone!(@weak prefix_box, @weak expandable_box, @strong obj => move |obj| {
            remove_all_children_from_box(&prefix_box);
            remove_all_children_from_box(&expandable_box);
            //
            let package_name = obj.package_name();
            let package_arch = obj.package_arch();
            let package_installed_version= obj.package_installed_version();
            let package_candidate_version= obj.package_candidate_version();
            //
            create_prefix_content(&prefix_box, &package_arch, &package_installed_version, &package_candidate_version);
            //
            let expandable_page_selection_box = gtk::Box::builder()
                .orientation(Orientation::Horizontal)
                .hexpand(false)
                .vexpand(false)
                .halign(Align::Start)
                .valign(Align::Start)
                .build();
            expandable_page_selection_box.add_css_class("linked");
            let description_page_button = gtk::ToggleButton::builder()
                .label("Description")
                .active(true)
                .build();
            let changelog_page_button = gtk::ToggleButton::builder()
                .label("Changelog")
                .group(&description_page_button)
                .build();
            expandable_page_selection_box.append(&description_page_button);
            expandable_page_selection_box.append(&changelog_page_button);
            expandable_box.append(&expandable_page_selection_box);
        }));

        obj.add_prefix(&prefix_box);
        obj.add_row(&expandable_box);

        //let obj = self.obj();
        //obj.bind_property("package-name", &package_label, "label")
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
impl ExpanderRowImpl for AptPackageRow {}

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
        .tooltip_text(t!("installed_version_badge_text"))
        .build();

    let installed_version_base_version_label = gtk::Label::builder()
        .label(format!("{}: {}", t!("installed_version_badge_text"), &base_version))
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
        .build();

    let candidate_version_box = gtk::Box::builder()
        .halign(Align::Start)
        .hexpand(false)
        .orientation(Orientation::Horizontal)
        .tooltip_text(t!("candidate_version_badge_text"))
        .build();

    let candidate_version_base_version_label = gtk::Label::builder()
        .label(format!("{}: {}", t!("candidate_version_badge_text"), &base_version))
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
        .margin_start(5)
        .margin_end(5)
        .margin_bottom(10)
        .build();

    boxedlist.add_css_class("boxed-list");
    boxedlist.append(&badge_box);
    boxedlist
}

fn create_arch_badge(arch: String) -> gtk::ListBox {
    let arch_label = gtk::Label::builder()
        .halign(Align::Start)
        .hexpand(false)
        .label(format!("{}: {}", t!("arch_label_label"), arch))
        .margin_start(5)
        .margin_end(5)
        .margin_bottom(5)
        .margin_top(5)
        .build();

    let boxedlist = gtk::ListBox::builder()
        .selection_mode(SelectionMode::None)
        .halign(Align::Start)
        .valign(Align::End)
        .margin_start(5)
        .margin_end(5)
        .margin_bottom(10)
        .build();

    boxedlist.add_css_class("boxed-list");
    boxedlist.append(&arch_label);
    boxedlist
}

fn remove_all_children_from_box(parent: &gtk::Box) {
    while let Some(child) = parent.last_child() {
        parent.remove(&child);
    }
}

fn create_prefix_content(prefix_box: &gtk::Box, package_name: &str ,package_arch: &str, package_installed_version: &str, package_candidate_version: &str) {
    let package_label = gtk::Label::builder()
        .halign(Align::Start)
        .margin_start(5)
        .margin_end(5)
        .margin_bottom(5)
        .margin_top(5)
        .label(package_name)
        .build();
    package_label.add_css_class("size-20-bold-text");
    let version_box = gtk::Box::new(Orientation::Horizontal, 0);
    version_box.append(&create_version_badge(package_installed_version, package_candidate_version));
    version_box.append(&create_arch_badge(package_arch));
    prefix_box.append(&package_label);
    prefix_box.append(&version_box);
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