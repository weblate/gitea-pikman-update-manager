use adw::gio::SimpleAction;
use adw::prelude::*;
use gtk::glib::*;
use gtk::*;
use std::cell::RefCell;
use std::rc::Rc;

pub fn main_update_page(
    apt_update_button: &Rc<RefCell<Button>>,
    initiated_by_main: &Rc<RefCell<bool>>,
    theme_changed_action: &gio::SimpleAction,
) -> gtk::Box {
    let main_box = Box::builder()
        .hexpand(true)
        .vexpand(true)
        .orientation(Orientation::Vertical)
        .build();

    let bottom_icon = gtk::Image::builder()
        .pixel_size(128)
        .halign(Align::Center)
        .hexpand(true)
        .icon_name("tux-symbolic")
        .margin_start(10)
        .margin_end(10)
        .margin_bottom(20)
        .margin_top(20)
        .build();

    let update_badge_box = gtk::Box::builder()
        .hexpand(true)
        .vexpand(true)
        .orientation(Orientation::Vertical)
        .build();

    let bottom_bar = Box::builder().valign(Align::End).build();

    let update_button = Button::builder()
        .halign(Align::End)
        .valign(Align::Center)
        .hexpand(true)
        .margin_start(10)
        .margin_end(30)
        .margin_bottom(15)
        .label(t!("update_button_label"))
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

    main_box.append(&update_badge_box);
    main_box.append(&bottom_icon);
    main_box.append(&bottom_bar);

    main_box
}
