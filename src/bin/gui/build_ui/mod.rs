use crate::apt_update_page;
use crate::config::{APP_GITHUB, APP_ICON, APP_ID, VERSION};
use crate::flatpak_update_page;
use adw::prelude::*;
use adw::*;
use gtk::glib::{clone, MainContext};
use gtk::License;
use std::cell::RefCell;
use std::process::Command;
use std::rc::Rc;
use std::thread;

pub fn build_ui(app: &Application) {
    // setup glib
    glib::set_prgname(Some(t!("application_name").to_string()));
    glib::set_application_name(&t!("application_name").to_string());
    let glib_settings = gio::Settings::new(APP_ID);

    let internet_connected = Rc::new(RefCell::new(false));
    let (internet_loop_sender, internet_loop_receiver) = async_channel::unbounded();
    let internet_loop_sender = internet_loop_sender.clone();

    thread::spawn(move || loop {
        match Command::new("ping").arg("google.com").arg("-c 1").output() {
            Ok(t) if t.status.success() => internet_loop_sender
                .send_blocking(true)
                .expect("The channel needs to be open"),
            _ => internet_loop_sender
                .send_blocking(false)
                .expect("The channel needs to be open"),
        };
        thread::sleep(std::time::Duration::from_secs(5));
    });

    let window_banner = Banner::builder().revealed(false).build();

    let internet_connected_status = internet_connected.clone();

    let internet_loop_context = MainContext::default();
    // The main loop executes the asynchronous block
    internet_loop_context.spawn_local(clone!(
        #[weak]
        window_banner,
        async move {
            while let Ok(state) = internet_loop_receiver.recv().await {
                let banner_text = t!("banner_text_no_internet").to_string();
                if state == true {
                    *internet_connected_status.borrow_mut() = true;
                    if window_banner.title() == banner_text {
                        window_banner.set_revealed(false)
                    }
                } else {
                    *internet_connected_status.borrow_mut() = false;
                    window_banner.set_title(&banner_text);
                    window_banner.set_revealed(true)
                }
            }
        }
    ));

    let window_headerbar = HeaderBar::builder()
        .title_widget(&WindowTitle::builder().title(t!("application_name")).build())
        .show_title(false)
        .build();

    let window_breakpoint = adw::Breakpoint::new(BreakpointCondition::new_length(
        BreakpointConditionLengthType::MaxWidth,
        800.0,
        LengthUnit::Px,
    ));

    let window_adw_stack = gtk::Stack::builder()
        .hhomogeneous(true)
        .vhomogeneous(true)
        .transition_type(gtk::StackTransitionType::SlideUpDown)
        .build();

    let window_toolbar = ToolbarView::builder()
        .content(&window_adw_stack)
        .top_bar_style(ToolbarStyle::Flat)
        .bottom_bar_style(ToolbarStyle::Flat)
        .build();

    let window_adw_view_switcher_sidebar = gtk::StackSidebar::builder()
        .vexpand(true)
        .hexpand(true)
        .margin_start(5)
        .margin_end(5)
        .stack(&window_adw_stack)
        .build();

    let window_adw_view_switcher_sidebar_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    window_adw_view_switcher_sidebar_box.append(&WindowTitle::builder().title(t!("application_name")).margin_top(20).margin_bottom(20).margin_start(5).margin_end(5).build());
    window_adw_view_switcher_sidebar_box.append(&window_adw_view_switcher_sidebar);

    let window_adw_view_sidebar_navigation_page = adw::NavigationPage::new(&window_adw_view_switcher_sidebar_box, "sidebar_view");

    let sidebar_toggle_button = gtk::ToggleButton::builder()
        .icon_name("view-right-pane-symbolic")
        .visible(false)
        .build();

    let window_content_page_split_view = adw::OverlaySplitView::builder()
        .vexpand(true)
        .hexpand(true)
        .content(&window_toolbar)
        .sidebar(&window_adw_view_sidebar_navigation_page)
        .max_sidebar_width(300.0)
        .min_sidebar_width(300.0)
        .enable_hide_gesture(true)
        .enable_show_gesture(true)
        .build();

    let _sidebar_toggle_button_binding = window_content_page_split_view
        .bind_property("show_sidebar", &sidebar_toggle_button, "active")
        .sync_create()
        .bidirectional()
        .build();

    window_breakpoint.add_setter(
        &window_content_page_split_view,
        "collapsed",
        Some(&true.to_value()),
    );
    window_breakpoint.add_setter(
        &sidebar_toggle_button,
        "visible",
        Some(&true.to_value()),
    );
    window_breakpoint.add_setter(
        &window_headerbar,
        "show_title",
        Some(&true.to_value()),
    );

    window_headerbar.pack_end(&sidebar_toggle_button);

    window_toolbar.add_top_bar(&window_headerbar);
    window_toolbar.add_top_bar(&window_banner);

    // create the main Application window
    let window = ApplicationWindow::builder()
        // The text on the titlebar
        .title(t!("application_name"))
        // link it to the application "app"
        .application(app)
        // Add the box called "window_box" to it
        // Application icon
        .icon_name(APP_ICON)
        // Minimum Size/Default
        .default_width(glib_settings.int("window-width"))
        .default_height(glib_settings.int("window-height"))
        //
        .width_request(700)
        .height_request(500)
        .content(&window_content_page_split_view)
        // Startup
        .startup_id(APP_ID)
        // build the window
        .build();

    window.add_breakpoint(window_breakpoint);

    if glib_settings.boolean("is-maximized") == true {
        window.maximize()
    }

    window.connect_close_request(move |window| {
        if let Some(application) = window.application() {
            let size = window.default_size();
            let _ = glib_settings.set_int("window-width", size.0);
            let _ = glib_settings.set_int("window-height", size.1);
            let _ = glib_settings.set_boolean("is-maximized", window.is_maximized());
            application.remove_window(window);
        }
        glib::Propagation::Proceed
    });

    let credits_button = gtk::Button::builder()
        .icon_name("dialog-information-symbolic")
        .build();

    let refresh_button = gtk::Button::builder()
        .icon_name("view-refresh-symbolic")
        .tooltip_text(t!("refresh_button_tooltip_text"))
        .build();

    let credits_window = AboutWindow::builder()
        .application_icon(APP_ICON)
        .application_name(t!("application_name"))
        .transient_for(&window)
        .version(VERSION)
        .hide_on_close(true)
        .developer_name(t!("developer_name"))
        .license_type(License::Mpl20)
        .issue_url(APP_GITHUB.to_owned() + "/issues")
        .build();

    window_headerbar.pack_end(&refresh_button);
    window_headerbar.pack_end(&credits_button);
    credits_button.connect_clicked(move |_| credits_window.present());

    // show the window

    window.present();

    // Flatpak Update Page

    let flatpak_retry_signal_action = gio::SimpleAction::new("retry", None);

    let flatpak_update_view_stack_bin = Bin::builder()
        .build();

    flatpak_retry_signal_action.connect_activate(clone!(
        #[weak]
        window,
        #[strong]
        flatpak_retry_signal_action,
        #[strong]
        flatpak_update_view_stack_bin,
        move |_, _| {
            flatpak_update_view_stack_bin.set_child(Some(&flatpak_update_page::flatpak_update_page(
                window,
                &flatpak_retry_signal_action,
            )));
        }
    ));

    // Apt Update Page
    let apt_retry_signal_action = gio::SimpleAction::new("retry", None);

    let flatpak_ran_once = Rc::new(RefCell::new(false));

    let apt_update_view_stack_bin = Bin::builder().build();

    apt_retry_signal_action.connect_activate(clone!(
            #[weak]
            window,
            #[strong]
            apt_retry_signal_action,
            #[strong]
            flatpak_retry_signal_action,
            #[strong]
            apt_update_view_stack_bin,
            #[weak]
            flatpak_ran_once,
            move |_, _| {
               apt_update_view_stack_bin.set_child(Some(&apt_update_page::apt_update_page(
                    window,
                    &apt_retry_signal_action,
                    &flatpak_retry_signal_action,
                    flatpak_ran_once,
                )));
            }
        ));

    apt_update_view_stack_bin.set_child(Some(&apt_update_page::apt_update_page(
        window.clone(),
        &apt_retry_signal_action,
        &flatpak_retry_signal_action,
        flatpak_ran_once,
    )));

    // Add to stack switcher

    window_adw_stack.add_titled(
        &apt_update_view_stack_bin,
        Some("apt_update_page"),
        &t!("apt_update_page_title"),
    );

    window_adw_stack.add_titled(
        &flatpak_update_view_stack_bin,
        Some("flatpak_update_page"),
        &t!("flatpak_update_page_title"),
    );

    // Refresh button

    refresh_button.connect_clicked(clone!(
        #[weak]
        apt_retry_signal_action,
        #[weak]
        flatpak_retry_signal_action,
        #[weak]
        window_adw_stack,
        move |_| {
            match window_adw_stack.visible_child_name().unwrap().as_str() {
                "apt_update_page" => apt_retry_signal_action.activate(None),
                "flatpak_update_page" => flatpak_retry_signal_action.activate(None),
                _ => {}
            }
        }
    ));
}
