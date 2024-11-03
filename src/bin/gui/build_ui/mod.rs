use crate::apt_manage_page::apt_manage_page;
use crate::apt_update_page;
use crate::config::{APP_GITHUB, APP_ICON, APP_ID, VERSION};
use crate::flatpak_manage_page::flatpak_manage_page;
use crate::flatpak_update_page;
use crate::main_update_page::main_update_page;
use adw::prelude::*;
use adw::*;
use gtk::glib::{clone, MainContext};
use gtk::License;
use ksni;
use std::cell::RefCell;
use std::process::Command;
use std::rc::Rc;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

pub fn get_current_font() -> String {
    let settings = gtk::Settings::default().unwrap();
    settings.gtk_font_name().unwrap().to_string()
}

#[derive(Debug)]
struct PikmanTray {
    icon_name: Option<String>,
    apt_item_label: Option<String>,
    flatpak_item_label: Option<String>,
    action_sender: &'static async_channel::Sender<String>,
}

impl ksni::Tray for PikmanTray {
    fn icon_name(&self) -> String {
        match &self.icon_name {
            Some(t) => t.into(),
            None => "help-about".into(),
        }
    }
    fn title(&self) -> String {
        t!("application_name").to_string()
    }
    // NOTE: On some system trays, `id` is a required property to avoid unexpected behaviors
    fn id(&self) -> String {
        env!("CARGO_PKG_NAME").into()
    }
    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        vec![
            StandardItem {
                label: match &self.apt_item_label {
                    Some(t) => t,
                    None => "?",
                }
                .into(),
                icon_name: "application-vnd.debian.binary-package".into(),
                enabled: false,
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: match &self.flatpak_item_label {
                    Some(t) => t,
                    None => "?",
                }
                .into(),
                icon_name: "application-vnd.flatpak".into(),
                enabled: false,
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: t!("pikman_indicator_open_item_label").into(),
                icon_name: "view-paged-symbolic".into(),
                activate: Box::new(|_| {
                    self.action_sender
                        .send_blocking(String::from("open"))
                        .unwrap()
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: t!("pikman_indicator_exit_item_label").into(),
                icon_name: "application-exit-symbolic".into(),
                activate: Box::new(|_| std::process::exit(0)),
                ..Default::default()
            }
            .into(),
        ]
    }
}

enum ConstantLoopMessage {
    InternetStatus(bool),
    RefreshRequest,
}

pub fn build_ui(app: &Application) {
    // setup glib
    glib::set_prgname(Some(t!("application_name").to_string()));
    glib::set_application_name(&t!("application_name").to_string());
    let glib_settings = gio::Settings::new(APP_ID);

    let automatically_check_for_updates_arc =
        Arc::new(AtomicBool::new(glib_settings.boolean("check-for-updates")));
    let update_interval_arc = Arc::new(Mutex::new(glib_settings.int("update-interval")));
    let internet_connected = Rc::new(RefCell::new(false));

    let (thread_sleep_sender, thread_sleep_receiver) = std::sync::mpsc::channel::<()>();

    let (constant_loop_sender, constant_loop_receiver) = async_channel::unbounded();
    let constant_loop_sender_clone0 = constant_loop_sender.clone();
    let constant_loop_sender_clone1 = constant_loop_sender.clone();

    let (gsettings_change_sender, gsettings_change_receiver) = async_channel::unbounded();
    let gsettings_change_sender_clone0 = gsettings_change_sender.clone();

    let refresh_button = gtk::Button::builder()
        .icon_name("view-refresh-symbolic")
        .tooltip_text(t!("refresh_button_tooltip_text"))
        .build();

    // Systray

    let apt_update_count = Rc::new(RefCell::new(0));
    let flatpak_update_count = Rc::new(RefCell::new(0));

    let update_sys_tray = gio::SimpleAction::new("sys_tray", Some(glib::VariantTy::ARRAY));
    let theme_changed_action = gio::SimpleAction::new("theme_changed", None);

    let (tray_service_sender, tray_service_receiver) = async_channel::unbounded();
    let tray_service_sender = tray_service_sender.clone();

    let tray_service = ksni::TrayService::new(PikmanTray {
        action_sender: Box::leak(Box::new(tray_service_sender)),
        icon_name: None,
        apt_item_label: None,
        flatpak_item_label: None,
    });
    let tray_handle = tray_service.handle();

    tray_service.spawn();

    update_sys_tray.connect_activate(clone!(
        #[strong]
        tray_handle,
        /*#[strong]
        refresh_button,*/
        move |_, param| {
            let array: &[i32] = param.unwrap().fixed_array().unwrap();
            let vec = array.to_vec();
            let apt_update_count = vec[0];
            let flatpak_update_count = vec[1];
            let tray_icon = if apt_update_count + flatpak_update_count > 1 {
                Some("update-high".into())
            } else {
                Some("update-none".into())
            };
            tray_handle.update(|tray: &mut PikmanTray| {
                tray.icon_name = tray_icon;
                tray.apt_item_label = Some(
                    strfmt::strfmt(
                        &t!("pikman_indicator_apt_count_item_label").to_string(),
                        &std::collections::HashMap::from([(
                            "NUM".to_string(),
                            match apt_update_count {
                                -1 => t!("pikman_indicator_flatpak_item_label_calculating").into(),
                                _ => apt_update_count.to_string(),
                            },
                        )]),
                    )
                    .unwrap(),
                );
                tray.flatpak_item_label = Some(
                    strfmt::strfmt(
                        &t!("pikman_indicator_flatpak_count_item_label").to_string(),
                        &std::collections::HashMap::from([(
                            "NUM".to_string(),
                            match flatpak_update_count {
                                -1 => t!("pikman_indicator_flatpak_item_label_calculating").into(),
                                _ => flatpak_update_count.to_string(),
                            },
                        )]),
                    )
                    .unwrap(),
                );
            });
            /*if apt_update_count == -1 || flatpak_update_count == -1 {
                refresh_button.set_sensitive(false);
                refresh_button.set_tooltip_text(Some(
                    &t!("pikman_indicator_flatpak_item_label_calculating").to_string(),
                ));
            } else {
                refresh_button.set_sensitive(true);
                refresh_button
                    .set_tooltip_text(Some(&t!("refresh_button_tooltip_text").to_string()));
            }*/
        }
    ));
    update_sys_tray.activate(Some(&glib::Variant::array_from_fixed_array(&[-1, -1])));

    // internet check loop
    thread::spawn(move || {
        let mut last_result = false;
        loop {
            if last_result == true {
                std::thread::sleep(std::time::Duration::from_secs(60));
            }

            let check_internet_connection_cli = Command::new("ping")
                .arg("iso.pika-os.com")
                .arg("-c 1")
                .output()
                .expect("failed to execute process");
            if check_internet_connection_cli.status.success() {
                constant_loop_sender_clone0
                    .send_blocking(ConstantLoopMessage::InternetStatus(true))
                    .expect("The channel needs to be open.");
                last_result = true
            } else {
                constant_loop_sender_clone0
                    .send_blocking(ConstantLoopMessage::InternetStatus(false))
                    .expect("The channel needs to be open.");
                last_result = false
            }
        }
    });

    {
        let automatically_check_for_updates_arc = automatically_check_for_updates_arc.clone();
        let update_interval_arc = update_interval_arc.clone();

        // update interval loop
        thread::spawn(move || {
            loop {
                let local_interval: i32;
                let automatically_check_for_updates =
                    automatically_check_for_updates_arc.load(std::sync::atomic::Ordering::Relaxed);
                if automatically_check_for_updates {
                    let update_interval = match update_interval_arc.lock() {
                        Ok(t) => t,
                        Err(_) => {
                            continue;
                        }
                    };
                    local_interval = *update_interval;
                    std::mem::drop(update_interval);
                    //println!("Sleeping on auto update check: {}", local_interval);
                    if let Ok(_) = thread_sleep_receiver
                        .recv_timeout(std::time::Duration::from_millis(local_interval as u64))
                    {
                        //println!("Sleeping on auto was interrupted was interrupted");
                        continue;
                    }
                    //println!("Starting Refresh Request");
                    constant_loop_sender_clone1
                        .send_blocking(ConstantLoopMessage::RefreshRequest)
                        .expect("The channel needs to be open.");
                }
            }
        });
    }

    let window_banner = Banner::builder().revealed(false).build();

    let internet_connected_status = internet_connected.clone();

    let window_headerbar = HeaderBar::builder()
        .title_widget(&WindowTitle::builder().title(t!("application_name")).build())
        .show_title(false)
        .build();

    let window_breakpoint = adw::Breakpoint::new(BreakpointCondition::new_length(
        BreakpointConditionLengthType::MaxWidth,
        1200.0,
        LengthUnit::Sp,
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

    let window_adw_view_switcher_sidebar_box = gtk::Box::new(gtk::Orientation::Vertical, 0);

    let null_toggle_button: gtk::ToggleButton = gtk::ToggleButton::new();

    let sidebar_toggle_button = gtk::ToggleButton::builder()
        .icon_name("view-right-pane-symbolic")
        .visible(false)
        .build();

    let window_adw_view_switcher_sidebar_toolbar = ToolbarView::builder()
        .content(&window_adw_view_switcher_sidebar_box)
        .top_bar_style(ToolbarStyle::Flat)
        .bottom_bar_style(ToolbarStyle::Flat)
        .build();

    window_adw_view_switcher_sidebar_toolbar.add_top_bar(
        &HeaderBar::builder()
            .title_widget(&WindowTitle::builder().title(t!("application_name")).build())
            .show_title(true)
            .build(),
    );

    let window_content_page_split_view = adw::OverlaySplitView::builder()
        .content(&window_toolbar)
        .sidebar(&window_adw_view_switcher_sidebar_toolbar)
        .max_sidebar_width(300.0)
        .min_sidebar_width(290.0)
        .sidebar_width_unit(adw::LengthUnit::Px)
        .sidebar_width_fraction(0.2)
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
    window_breakpoint.add_setter(&sidebar_toggle_button, "visible", Some(&true.to_value()));
    window_breakpoint.add_setter(&window_headerbar, "show_title", Some(&true.to_value()));

    window_headerbar.pack_end(&sidebar_toggle_button);

    window_toolbar.add_top_bar(&window_headerbar);
    window_toolbar.add_top_bar(&window_banner);

    let window_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    window_box.append(&window_content_page_split_view);

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
        .width_request(1140)
        .height_request(780)
        .content(&window_box)
        // Startup
        .startup_id(APP_ID)
        // build the window
        .hide_on_close(true)
        //
        .build();

    window.add_breakpoint(window_breakpoint);

    if glib_settings.boolean("is-maximized") == true {
        window.maximize()
    }

    {
        let glib_settings = glib_settings.clone();

        window.connect_close_request(move |window| {
            let size = window.default_size();
            let _ = glib_settings.set_int("window-width", size.0);
            let _ = glib_settings.set_int("window-height", size.1);
            let _ = glib_settings.set_boolean("is-maximized", window.is_maximized());
            glib::Propagation::Proceed
        });
    }

    let credits_button = gtk::Button::builder()
        .icon_name("dialog-information-symbolic")
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

    //window.present();

    // Theme update actions
    {
        let setting = gtk::Settings::default().unwrap();

        setting.connect_gtk_application_prefer_dark_theme_notify(clone!(
            #[strong]
            theme_changed_action,
            move |_| {
                let theme_changed_action = theme_changed_action.clone();
                glib::timeout_add_seconds_local(5, move || {
                    theme_changed_action.activate(None);
                    glib::ControlFlow::Break
                });
            }
        ));
        setting.connect_gtk_font_name_notify(clone!(
            #[strong]
            theme_changed_action,
            move |_| {
                let theme_changed_action = theme_changed_action.clone();
                glib::timeout_add_seconds_local(5, move || {
                    theme_changed_action.activate(None);
                    glib::ControlFlow::Break
                });
            }
        ));
    }

    thread::spawn(move || {
        let context = glib::MainContext::default();
        let main_loop = glib::MainLoop::new(Some(&context), false);
        let gsettings = gtk::gio::Settings::new("org.gnome.desktop.interface");
        gsettings.connect_changed(
            Some("accent-color"),
            clone!(
                #[strong]
                gsettings_change_sender_clone0,
                move |_, _| {
                    let gsettings_change_sender_clone0 = gsettings_change_sender_clone0.clone();
                    glib::timeout_add_seconds_local(5, move || {
                        gsettings_change_sender_clone0.send_blocking(()).unwrap();
                        glib::ControlFlow::Break
                    });
                }
            ),
        );
        main_loop.run()
    });

    let gsettings_changed_context = MainContext::default();
    // The main loop executes the asynchronous block
    gsettings_changed_context.spawn_local(clone!(
        #[strong]
        theme_changed_action,
        async move {
            while let Ok(()) = gsettings_change_receiver.recv().await {
                theme_changed_action.activate(None);
            }
        }
    ));

    // Update buttons

    let apt_update_button = Rc::new(RefCell::new(gtk::Button::new()));
    let flatpak_update_button = Rc::new(RefCell::new(gtk::Button::new()));

    // Flatpak Update Page

    let flatpak_retry_signal_action = gio::SimpleAction::new("retry", None);

    let flatpak_update_view_stack_bin = Bin::builder().build();

    flatpak_retry_signal_action.connect_activate(clone!(
        #[weak]
        window,
        #[strong]
        flatpak_update_button,
        #[strong]
        flatpak_retry_signal_action,
        #[strong]
        flatpak_update_view_stack_bin,
        #[strong]
        update_sys_tray,
        #[strong]
        apt_update_count,
        #[strong]
        flatpak_update_count,
        #[strong]
        theme_changed_action,
        move |_, _| {
            (*flatpak_update_button.borrow_mut() = gtk::Button::new());
            flatpak_update_view_stack_bin.set_child(Some(
                &flatpak_update_page::flatpak_update_page(
                    window,
                    &flatpak_update_button,
                    &flatpak_retry_signal_action,
                    &theme_changed_action,
                    &update_sys_tray,
                    &apt_update_count,
                    &flatpak_update_count,
                ),
            ));
        }
    ));

    // Apt Update Page
    let apt_retry_signal_action = gio::SimpleAction::new("retry", None);

    let flatpak_ran_once = Rc::new(RefCell::new(false));
    let initiated_by_main = Rc::new(RefCell::new(false));

    let apt_update_view_stack_bin = Bin::builder().build();

    apt_retry_signal_action.connect_activate(clone!(
        #[strong]
        window,
        #[strong]
        apt_update_button,
        #[strong]
        flatpak_update_button,
        #[strong]
        flatpak_retry_signal_action,
        #[strong]
        apt_update_view_stack_bin,
        #[strong]
        flatpak_ran_once,
        #[strong]
        initiated_by_main,
        #[strong]
        update_sys_tray,
        #[strong]
        apt_update_count,
        #[strong]
        flatpak_update_count,
        #[strong]
        theme_changed_action,
        move |action, _| {
            (*apt_update_button.borrow_mut() = gtk::Button::new());
            apt_update_view_stack_bin.set_child(Some(&apt_update_page::apt_update_page(
                window.clone(),
                &apt_update_button,
                &flatpak_update_button,
                &action,
                &flatpak_retry_signal_action,
                &theme_changed_action,
                flatpak_ran_once.clone(),
                initiated_by_main.clone(),
                &update_sys_tray,
                &apt_update_count,
                &flatpak_update_count,
            )));
        }
    ));

    apt_update_view_stack_bin.set_child(Some(&apt_update_page::apt_update_page(
        window.clone(),
        &apt_update_button,
        &flatpak_update_button,
        &apt_retry_signal_action,
        &flatpak_retry_signal_action,
        &theme_changed_action,
        flatpak_ran_once.clone(),
        initiated_by_main.clone(),
        &update_sys_tray,
        &apt_update_count,
        &flatpak_update_count,
    )));

    // Add to stack switcher

    window_adw_stack.add_titled(
        &main_update_page(
            &apt_update_button,
            &initiated_by_main,
            &theme_changed_action,
        ),
        Some("main_update_page"),
        &t!("main_update_page_title"),
    );

    let main_update_page_toggle_button = add_content_button(
        &window_adw_stack,
        true,
        "main_update_page".to_string(),
        t!("main_update_page_title").to_string(),
        &null_toggle_button,
    );
    window_adw_view_switcher_sidebar_box.append(&main_update_page_toggle_button);

    window_adw_stack.add_titled(
        &apt_update_view_stack_bin,
        Some("apt_update_page"),
        &t!("apt_update_page_title"),
    );

    let apt_update_page_toggle_button = add_content_button(
        &window_adw_stack,
        false,
        "apt_update_page".to_string(),
        t!("apt_update_page_title").to_string(),
        &null_toggle_button,
    );
    window_adw_view_switcher_sidebar_box.append(&apt_update_page_toggle_button);

    window_adw_stack.add_titled(
        &flatpak_update_view_stack_bin,
        Some("flatpak_update_page"),
        &t!("flatpak_update_page_title"),
    );

    let flatpak_update_page_toggle_button = add_content_button(
        &window_adw_stack,
        false,
        "flatpak_update_page".to_string(),
        t!("flatpak_update_page_title").to_string(),
        &null_toggle_button,
    );
    window_adw_view_switcher_sidebar_box.append(&flatpak_update_page_toggle_button);

    window_adw_stack.add_titled(
        &apt_manage_page(
            window.clone(),
            &glib_settings,
            &apt_retry_signal_action,
            &thread_sleep_sender,
            &automatically_check_for_updates_arc,
            &update_interval_arc,
        ),
        Some("apt_manage_page"),
        &t!("apt_manage_page_title"),
    );

    let apt_manage_page_toggle_button = add_content_button(
        &window_adw_stack,
        false,
        "apt_manage_page".to_string(),
        t!("apt_manage_page_title").to_string(),
        &null_toggle_button,
    );
    window_adw_view_switcher_sidebar_box.append(&apt_manage_page_toggle_button);

    let flatpak_entry_signal_action =
        gio::SimpleAction::new("entry-change", Some(glib::VariantTy::STRING));

    let flatpak_flatref_install_button = gtk::Button::builder()
        .icon_name("document-open-symbolic")
        .tooltip_text(t!("flatpak_flatref_install_button_tooltip_text"))
        //.halign(Align::End)
        .valign(gtk::Align::End)
        .build();

    window_adw_stack.add_titled(
        &flatpak_manage_page(
            window.clone(),
            &flatpak_retry_signal_action,
            &flatpak_entry_signal_action,
            &flatpak_flatref_install_button,
        ),
        Some("flatpak_manage_page"),
        &t!("flatpak_manage_page_title"),
    );

    let flatpak_manage_page_toggle_button = add_content_button(
        &window_adw_stack,
        false,
        "flatpak_manage_page".to_string(),
        t!("flatpak_manage_page_title").to_string(),
        &null_toggle_button,
    );
    window_adw_view_switcher_sidebar_box.append(&flatpak_manage_page_toggle_button);

    app.connect_command_line(clone!(
        #[strong]
        apt_manage_page_toggle_button,
        #[strong]
        flatpak_manage_page_toggle_button,
        #[strong]
        window,
        #[strong]
        flatpak_flatref_install_button,
        #[strong]
        flatpak_entry_signal_action,
        move |_, cmdline| {
            // Create Vec from cmdline
            let mut gtk_application_args = Vec::new();
            for arg in cmdline.arguments() {
                match arg.to_str() {
                    Some(a) => gtk_application_args.push(a.to_string()),
                    None => {}
                }
            }

            // Check for cmd lines
            if !(gtk_application_args.contains(&"--hidden".to_string()))
                && !(gtk_application_args.contains(&"--flatpak-installer".to_string()))
                && !window.is_visible()
            {
                window.present();
            }

            if gtk_application_args.contains(&"--software-properties".to_string()) {
                apt_manage_page_toggle_button.set_active(true);
                apt_manage_page_toggle_button.emit_clicked();
            }
            if gtk_application_args.contains(&"--flatpak-settings".to_string()) {
                flatpak_manage_page_toggle_button.set_active(true);
                flatpak_manage_page_toggle_button.emit_clicked();
            }
            if gtk_application_args.contains(&"--flatpak-installer".to_string()) {
                flatpak_flatref_install_button.emit_clicked();
                let index = ((gtk_application_args
                    .iter()
                    .position(|r| r == "--flatpak-installer")
                    .unwrap() as i32)
                    + 1) as usize;
                if index > gtk_application_args.len() - 1 {
                    flatpak_entry_signal_action.activate(Some(&glib::Variant::from("")));
                } else {
                    flatpak_entry_signal_action
                        .activate(Some(&glib::Variant::from(&gtk_application_args[index])));
                }
            }

            0
        }
    ));

    // Refresh button

    refresh_button.connect_clicked(clone!(
        #[weak]
        apt_retry_signal_action,
        #[weak]
        flatpak_retry_signal_action,
        #[weak]
        window_adw_stack,
        #[strong]
        flatpak_ran_once,
        move |_| {
            match window_adw_stack.visible_child_name().unwrap().as_str() {
                "main_update_page" => {
                    *flatpak_ran_once.borrow_mut() = false;
                    apt_retry_signal_action.activate(None);
                }
                "apt_update_page" => apt_retry_signal_action.activate(None),
                "apt_manage_page" => apt_retry_signal_action.activate(None),
                "flatpak_update_page" => flatpak_retry_signal_action.activate(None),
                "flatpak_manage_page" => flatpak_retry_signal_action.activate(None),
                _ => {}
            }
        }
    ));

    let tray_service_context = MainContext::default();
    // The main loop executes the asynchronous block
    tray_service_context.spawn_local(clone!(
        #[strong]
        window,
        async move {
            while let Ok(state) = tray_service_receiver.recv().await {
                match state.as_str() {
                    "open" => {
                        if !window.is_visible() {
                            window.present();
                        }
                    }
                    _ => todo!(),
                }
            }
        }
    ));

    let constant_loop_context = MainContext::default();
    // The main loop executes the asynchronous block
    constant_loop_context.spawn_local(clone!(
        #[weak]
        window_banner,
        #[strong]
        update_sys_tray,
        #[strong]
        apt_retry_signal_action,
        #[strong]
        flatpak_retry_signal_action,
        async move {
            while let Ok(message) = constant_loop_receiver.recv().await {
                let banner_text = t!("banner_text_no_internet").to_string();
                match message {
                    ConstantLoopMessage::InternetStatus(state) => {
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
                    ConstantLoopMessage::RefreshRequest => {
                        update_sys_tray
                            .activate(Some(&glib::Variant::array_from_fixed_array(&[-1, -1])));
                        apt_retry_signal_action.activate(None);
                        flatpak_retry_signal_action.activate(None);
                    }
                }
            }
        }
    ));
}

fn add_content_button(
    window_adw_stack: &gtk::Stack,
    active: bool,
    name: String,
    title: String,
    null_toggle_button: &gtk::ToggleButton,
) -> gtk::ToggleButton {
    let toggle_button = gtk::ToggleButton::builder()
        .group(null_toggle_button)
        .label(&title)
        .active(active)
        .margin_top(5)
        .margin_bottom(5)
        .margin_start(10)
        .margin_end(10)
        .valign(gtk::Align::Start)
        .build();
    toggle_button.add_css_class("flat");
    toggle_button.connect_clicked(clone!(
        #[weak]
        window_adw_stack,
        move |toggle_button| {
            if toggle_button.is_active() {
                window_adw_stack.set_visible_child_name(&name);
            }
        }
    ));
    toggle_button
}
