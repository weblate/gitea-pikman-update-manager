use adw::prelude::*;
use gtk::glib::*;
use gtk::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::build_ui::create_color_badge;

pub fn main_update_page(
    apt_update_button: &Rc<RefCell<Button>>,
    initiated_by_main: &Rc<RefCell<bool>>,
    theme_changed_action: &gio::SimpleAction,
    update_sys_tray: &gio::SimpleAction,
) -> gtk::Box {
    let main_box = Box::builder()
        .hexpand(true)
        .vexpand(true)
        .orientation(Orientation::Vertical)
        .build();

    let header_box = Box::builder()
        .hexpand(true)
        .vexpand(true)
        .halign(Align::Center)
        .orientation(Orientation::Horizontal)
        .build();

    let header_label = gtk::Label::builder()
        .label(t!("main_page_header_label_no_label"))
        .build();
    header_label.add_css_class("size-35-bold-text");

    let updater_icon = gtk::Image::builder()
        .pixel_size(128)
        .icon_name("update-none")
        .margin_start(5)
        .margin_end(5)
        .margin_bottom(5)
        .margin_top(5)
        .build();

    let update_badge_box = gtk::Box::builder()
        .hexpand(true)
        .vexpand(true)
        .halign(Align::Center)
        .valign(Align::Center)
        .orientation(Orientation::Vertical)
        .build();

    let update_badge_box_size_group = SizeGroup::new(SizeGroupMode::Both);
    let update_badge_box_size_group0 = SizeGroup::new(SizeGroupMode::Both);
    let update_badge_box_size_group1 = SizeGroup::new(SizeGroupMode::Both);

    update_badge_box.append(&create_color_badge(
        &t!("update_badge_box_total_label"),
        &t!("pikman_indicator_flatpak_item_label_calculating"),
        "background-accent-bg",
        &theme_changed_action,
        &update_badge_box_size_group,
        &update_badge_box_size_group0,
        &update_badge_box_size_group1,
    ));
    //
    update_badge_box.append(&create_color_badge(
        &t!("update_badge_box_apt_label"),
        &t!("pikman_indicator_flatpak_item_label_calculating"),
        "background-accent-bg",
        &theme_changed_action,
        &update_badge_box_size_group,
        &update_badge_box_size_group0,
        &update_badge_box_size_group1,
    ));
    //
    update_badge_box.append(&create_color_badge(
        &t!("update_badge_box_flatpak_label"),
        &t!("pikman_indicator_flatpak_item_label_calculating"),
        "background-accent-bg",
        &theme_changed_action,
        &update_badge_box_size_group,
        &update_badge_box_size_group0,
        &update_badge_box_size_group1,
    ));

    update_sys_tray.connect_activate(clone!(
        #[strong]
        update_badge_box,
        #[strong]
        updater_icon,
        #[strong]
        header_label,
        #[strong]
        theme_changed_action,
        move |_, param| {
            let array: &[i32] = param.unwrap().fixed_array().unwrap();
            let vec = array.to_vec();
            let apt_update_count = vec[0];
            let flatpak_update_count = vec[1];
            let total_count = apt_update_count + flatpak_update_count;
            if total_count > 1 {
                updater_icon.set_icon_name(Some("update-high".into()));
                header_label.set_label(&t!("main_page_header_label_yes_label"));
            } else {
                updater_icon.set_icon_name(Some("update-none".into()));
                header_label.set_label(&t!("main_page_header_label_no_label"));
            }
            //
            while let Some(widget) = update_badge_box.last_child() {
                update_badge_box.remove(&widget);
            }
            //
            update_badge_box.append(&create_color_badge(
                &t!("update_badge_box_total_label"),
                &match total_count {
                    -2 => t!("pikman_indicator_flatpak_item_label_calculating").into(),
                    _ => total_count.to_string(),
                },
                "background-accent-bg",
                &theme_changed_action,
                &update_badge_box_size_group,
                &update_badge_box_size_group0,
                &update_badge_box_size_group1,
            ));
            //
            update_badge_box.append(&create_color_badge(
                &t!("update_badge_box_apt_label"),
                &match apt_update_count {
                    -1 => t!("pikman_indicator_flatpak_item_label_calculating").into(),
                    _ => apt_update_count.to_string(),
                },
                "background-accent-bg",
                &theme_changed_action,
                &update_badge_box_size_group,
                &update_badge_box_size_group0,
                &update_badge_box_size_group1,
            ));
            //
            update_badge_box.append(&create_color_badge(
                &t!("update_badge_box_flatpak_label"),
                &match flatpak_update_count {
                    -1 => t!("pikman_indicator_flatpak_item_label_calculating").into(),
                    _ => flatpak_update_count.to_string(),
                },
                "background-accent-bg",
                &theme_changed_action,
                &update_badge_box_size_group,
                &update_badge_box_size_group0,
                &update_badge_box_size_group1,
            ));
        }
    ));

    let bottom_bar = Box::builder().valign(Align::End).build();

    let update_button = Button::builder()
        .halign(Align::End)
        .valign(Align::Center)
        .hexpand(true)
        .margin_start(10)
        .margin_end(30)
        .margin_bottom(15)
        .label(t!("update_button_main_label"))
        .build();
    update_button.add_css_class("destructive-action");

    update_button.connect_clicked(clone!(
        #[strong]
        initiated_by_main,
        #[strong]
        apt_update_button,
        move |_| {
            *initiated_by_main.borrow_mut() = true;
            apt_update_button.borrow().emit_clicked();
        }
    ));

    bottom_bar.append(&update_button);

    header_box.append(&updater_icon);
    header_box.append(&header_label);

    main_box.append(&header_box);
    main_box.append(&update_badge_box);
    main_box.append(&bottom_bar);

    main_box
}
