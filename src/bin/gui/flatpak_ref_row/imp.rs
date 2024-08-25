use std::{cell::RefCell, sync::OnceLock};

use adw::*;
use adw::{prelude::*, subclass::prelude::*};
use glib::{clone, subclass::Signal, Properties};
use gtk::*;
use pretty_bytes::converter::convert;
use std::env;

// ANCHOR: custom_button
// Object holding the state
#[derive(Properties, Default)]
#[properties(wrapper_type = super::FlatpakRefRow)]
pub struct FlatpakRefRow {
    #[property(get, set)]
    flatref_name: RefCell<String>,
    #[property(get, set)]
    flatref_arch: RefCell<String>,
    #[property(get, set)]
    flatref_ref_name: RefCell<String>,
    #[property(get, set)]
    flatref_summary: RefCell<String>,
    #[property(get, set)]
    flatref_remote_name: RefCell<String>,
    #[property(get, set)]
    flatref_installed_size_installed: RefCell<u64>,
    #[property(get, set)]
    flatref_installed_size_remote: RefCell<u64>,
    #[property(get, set)]
    flatref_download_size: RefCell<u64>,
    #[property(get, set)]
    flatref_ref_format: RefCell<String>,
    #[property(get, set)]
    flatref_is_system: RefCell<bool>,
    #[property(get, set)]
    flatref_marked: RefCell<bool>,
}
// ANCHOR_END: custom_button

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for FlatpakRefRow {
    const NAME: &'static str = "FlatpakRefRow";
    type Type = super::FlatpakRefRow;
    type ParentType = ExpanderRow;
}

// ANCHOR: object_impl
// Trait shared by all GObjects
#[glib::derived_properties]
impl ObjectImpl for FlatpakRefRow {
    fn signals() -> &'static [Signal] {
        static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
        SIGNALS.get_or_init(|| {
            vec![
                Signal::builder("checkbutton-toggled").build(),
                Signal::builder("checkbutton-untoggled").build(),
            ]
        })
    }
    fn constructed(&self) {
        let current_locale = match env::var_os("LANG") {
            Some(v) => v
                .into_string()
                .unwrap()
                .chars()
                .take_while(|&ch| ch != '.')
                .collect::<String>(),
            None => panic!("$LANG is not set"),
        };
        rust_i18n::set_locale(&current_locale);

        self.parent_constructed();

        // Bind label to number
        // `SYNC_CREATE` ensures that the label will be immediately set
        let obj = self.obj();

        let prefix_box = Box::new(Orientation::Vertical, 0);

        let expandable_box = Box::new(Orientation::Vertical, 0);

        obj.connect_flatref_name_notify(clone!(
            #[weak]
            prefix_box,
            #[weak]
            expandable_box,
            #[strong]
            obj,
            move |_| {
                remove_all_children_from_box(&prefix_box);
                remove_all_children_from_box(&expandable_box);
                //
                let flatref_name = obj.flatref_name();
                let flatref_arch = obj.flatref_arch();
                let flatref_ref_name = obj.flatref_ref_name();
                let flatref_summary = obj.flatref_summary();
                let flatref_remote_name = obj.flatref_remote_name();
                let flatref_installed_size_installed = obj.flatref_installed_size_installed();
                let flatref_installed_size_remote = obj.flatref_installed_size_remote();
                let flatref_download_size = obj.flatref_download_size();
                let flatref_ref_format = obj.flatref_download_size();
                let flatref_is_system = obj.flatref_is_system();
                let flatref_marked = obj.flatref_marked();
                //
                create_prefix_content(
                    &prefix_box,
                    &flatref_name,
                    &flatref_arch,
                    flatref_is_system,
                    &flatref_remote_name,
                );
                //
                create_expandable_content(
                    &obj,
                    &expandable_box,
                    flatref_ref_name,
                    flatref_summary,
                    flatref_download_size,
                    flatref_installed_size_remote,
                );
            }
        ));

        obj.add_prefix(&prefix_box);
        obj.add_row(&expandable_box);

        let suffix_toggle = CheckButton::builder()
            .tooltip_text(t!("mark_for_update"))
            .halign(Align::Center)
            .valign(Align::Center)
            .hexpand(false)
            .vexpand(false)
            .build();

        suffix_toggle.connect_toggled(clone!(
            #[weak]
            obj,
            #[weak]
            suffix_toggle,
            move |_| {
                if suffix_toggle.is_active() {
                    obj.emit_by_name::<()>("checkbutton-toggled", &[]);
                } else {
                    obj.emit_by_name::<()>("checkbutton-untoggled", &[]);
                }
            }
        ));

        obj.add_suffix(&suffix_toggle);

        let obj = self.obj();
        obj.bind_property("flatref_marked", &suffix_toggle, "active")
            .sync_create()
            .bidirectional()
            .build();

        // turn on by default
        obj.set_property("flatref_marked", true)
    }
}
// Trait shared by all widgets
impl WidgetImpl for FlatpakRefRow {}

// Trait shared by all buttons
// Trait shared by all buttons

impl ListBoxRowImpl for FlatpakRefRow {}
impl PreferencesRowImpl for FlatpakRefRow {}
impl ExpanderRowImpl for FlatpakRefRow {}

fn create_remote_badge(remote_name: &str) -> ListBox {
    let remote_label = Label::builder()
        .halign(Align::Start)
        .hexpand(false)
        .label(format!("{}: {}", t!("remote_label_label"), remote_name))
        .margin_start(5)
        .margin_end(5)
        .margin_bottom(5)
        .margin_top(5)
        .build();

    let boxedlist = ListBox::builder()
        .selection_mode(SelectionMode::None)
        .halign(Align::Start)
        .valign(Align::End)
        .margin_start(5)
        .margin_end(5)
        .margin_bottom(10)
        .build();

    boxedlist.add_css_class("boxed-list");
    boxedlist.append(&remote_label);
    boxedlist
}
fn create_arch_badge(arch: &str) -> ListBox {
    let arch_label = Label::builder()
        .halign(Align::Start)
        .hexpand(false)
        .label(format!("{}: {}", t!("arch_label_label"), arch))
        .margin_start(5)
        .margin_end(5)
        .margin_bottom(5)
        .margin_top(5)
        .build();

    let boxedlist = ListBox::builder()
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

fn create_system_badge(is_system: bool) -> ListBox {
    let system_label = Label::builder()
        .halign(Align::Start)
        .hexpand(false)
        .label(match is_system {
            true => "System",
            false => "User",
        })
        .margin_start(5)
        .margin_end(5)
        .margin_bottom(5)
        .margin_top(5)
        .build();

    let boxedlist = ListBox::builder()
        .selection_mode(SelectionMode::None)
        .halign(Align::Start)
        .valign(Align::End)
        .margin_start(5)
        .margin_end(5)
        .margin_bottom(10)
        .build();

    boxedlist.add_css_class("boxed-list");
    boxedlist.append(&system_label);
    boxedlist
}

fn remove_all_children_from_box(parent: &gtk::Box) {
    while let Some(child) = parent.last_child() {
        parent.remove(&child);
    }
}

fn create_prefix_content(
    prefix_box: &gtk::Box,
    flatref_name: &str,
    flatref_arch: &str,
    flatref_is_system: bool,
    flatref_remote_name: &str,
) {
    let package_label = Label::builder()
        .halign(Align::Start)
        .margin_start(5)
        .margin_end(5)
        .margin_bottom(5)
        .margin_top(5)
        .label(flatref_name)
        .build();
    package_label.add_css_class("size-20-bold-text");
    let prefix_badge_box = Box::new(Orientation::Horizontal, 0);
    prefix_badge_box.append(&create_remote_badge(flatref_remote_name));
    prefix_badge_box.append(&create_arch_badge(flatref_arch));
    prefix_badge_box.append(&create_system_badge(flatref_is_system));
    prefix_box.append(&package_label);
    prefix_box.append(&prefix_badge_box);
}

fn create_expandable_content(
    flatpak_package_row: &impl IsA<ExpanderRow>,
    expandable_box: &gtk::Box,
    flatref_ref_name: String,
    flatref_summary: String,
    flatref_download_size: u64,
    flatref_installed_size_remote: u64,
) {
    let expandable_page_selection_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .hexpand(false)
        .vexpand(false)
        .halign(Align::Start)
        .valign(Align::Start)
        .margin_bottom(10)
        .margin_top(10)
        .margin_start(10)
        .margin_end(10)
        .build();
    expandable_page_selection_box.add_css_class("linked");
    //
    let summary_page_button = ToggleButton::builder()
        .label(t!("summary_button_label"))
        .active(true)
        .build();
    let extra_info_page_button = ToggleButton::builder()
        .label(t!("extra_info_page_button_label"))
        .group(&summary_page_button)
        .build();
    expandable_page_selection_box.append(&summary_page_button);
    expandable_page_selection_box.append(&extra_info_page_button);
    //
    expandable_box.append(&expandable_page_selection_box);
    //
    let expandable_bin = Bin::builder().hexpand(true).vexpand(true).build();
    //
    summary_page_button.connect_clicked(clone!(
        #[strong]
        expandable_bin,
        #[strong]
        summary_page_button,
        move |_| {
            if summary_page_button.is_active() {
                expandable_bin.set_child(Some(&summary_stack_page(&flatref_summary)));
            }
        }
    ));

    extra_info_page_button.connect_clicked(clone!(
        #[strong]
        expandable_bin,
        #[strong]
        extra_info_page_button,
        move |_| {
            if extra_info_page_button.is_active() {
                expandable_bin.set_child(Some(&extra_info_stack_page(
                    &flatref_ref_name,
                    flatref_download_size,
                    flatref_installed_size_remote,
                )));
            }
        }
    ));

    flatpak_package_row.connect_expanded_notify(clone!(
        #[strong]
        expandable_bin,
        #[strong]
        expandable_box,
        #[strong]
        flatpak_package_row,
        #[strong]
        summary_page_button,
        move |_| {
            if flatpak_package_row.property("expanded") {
                summary_page_button.set_active(true);
                summary_page_button.emit_by_name::<()>("clicked", &[]);
                expandable_box.append(&expandable_bin)
            } else {
                expandable_box.remove(&expandable_bin)
            }
        }
    ));
}

fn summary_stack_page(flatref_summary: &str) -> gtk::Box {
    let summary_content_box = Box::builder()
        .hexpand(true)
        .vexpand(true)
        .orientation(Orientation::Vertical)
        .build();
    let summary_text_buffer = TextBuffer::builder()
        .text(flatref_summary.to_owned() + "\n")
        .build();
    let summary_text_view = TextView::builder()
        .buffer(&summary_text_buffer)
        .hexpand(true)
        .vexpand(true)
        .margin_top(0)
        .margin_bottom(10)
        .margin_start(15)
        .margin_end(15)
        .editable(false)
        .build();
    summary_content_box.append(&summary_text_view);
    summary_content_box
}

fn extra_info_stack_page(
    flatref_ref_name: &str,
    flatref_download_size: u64,
    flatref_installed_size_remote: u64,
) -> gtk::Box {
    let extra_info_badges_content_box = Box::builder()
        .hexpand(true)
        .vexpand(true)
        .orientation(Orientation::Vertical)
        .build();
    let extra_info_badges_size_group = SizeGroup::new(SizeGroupMode::Both);
    let extra_info_badges_size_group0 = SizeGroup::new(SizeGroupMode::Both);
    let extra_info_badges_size_group1 = SizeGroup::new(SizeGroupMode::Both);
    let package_size = flatref_download_size as f64;
    let package_installed_size = flatref_installed_size_remote as f64;
    extra_info_badges_content_box.append(&create_color_badge(
        &t!("flatpak_extra_info_ref_name").to_string(),
        flatref_ref_name,
        "background-accent-bg",
        &extra_info_badges_size_group,
        &extra_info_badges_size_group0,
        &extra_info_badges_size_group1,
    ));
    extra_info_badges_content_box.append(&create_color_badge(
        &t!("flatpak_extra_info_download_size").to_string(),
        &convert(package_size),
        "background-accent-bg",
        &extra_info_badges_size_group,
        &extra_info_badges_size_group0,
        &extra_info_badges_size_group1,
    ));
    extra_info_badges_content_box.append(&create_color_badge(
        &t!("flatpak_extra_info_installed_size").to_string(),
        &convert(package_installed_size),
        "background-accent-bg",
        &extra_info_badges_size_group,
        &extra_info_badges_size_group0,
        &extra_info_badges_size_group1,
    ));
    extra_info_badges_content_box
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
        .halign(Align::Start)
        .valign(Align::Start)
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
