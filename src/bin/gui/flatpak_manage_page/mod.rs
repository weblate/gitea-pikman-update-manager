use crate::apt_package_row::AptPackageRow;
use add_dialog::add_dialog_fn;
use adw::gio::SimpleAction;
use adw::prelude::*;
use apt_deb822_tools::Deb822Repository;
use gtk::glib::{property::PropertyGet, clone, BoxedAnyObject};
use gtk::*;
use std::cell::Ref;
use std::ops::Deref;
use pika_unixsocket_tools::pika_unixsocket_tools::*;
use rust_apt::cache::*;
use rust_apt::new_cache;
use rust_apt::records::RecordField;
use std::cell::RefCell;
use std::process::Command;
use std::rc::Rc;
use std::thread;
use tokio::runtime::Runtime;
use crate::flatpak_ref_row::FlatpakRefRow;
use adw::prelude::*;
use gtk::glib::*;
use gtk::*;
use libflatpak::prelude::*;
use libflatpak::InstalledRef;

mod add_dialog;
mod install_ref_dialog;

enum FlatpakRemote {
    System(libflatpak::Remote),
    User(libflatpak::Remote)
}

pub fn flatpak_manage_page(
    window: adw::ApplicationWindow,
    flatpak_retry_signal_action: &SimpleAction,
) -> gtk::Box {
    let retry_signal_action = gio::SimpleAction::new("flatpak_manage_action", None);
    let cancellable_no = libflatpak::gio::Cancellable::NONE;

    let main_box = Box::builder()
        .hexpand(true)
        .vexpand(true)
        .orientation(Orientation::Vertical)
        .build();

    //

    let flatpak_remotes_label0 = gtk::Label::builder()
        .label(t!("flatpak_remotes_label"))
        .halign(gtk::Align::Start)
        .valign(gtk::Align::Start)
        .hexpand(true)
        .margin_top(15)
        .margin_start(15)
        .margin_end(15)
        .margin_bottom(5)
        .build();
    flatpak_remotes_label0.add_css_class("heading");

    let flatpak_remotes_label1 = gtk::Label::builder()
        .label(t!("flatpak_remotes_label1_label"))
        .halign(gtk::Align::Start)
        .valign(gtk::Align::Start)
        .hexpand(true)
        .margin_start(15)
        .margin_end(15)
        .build();

    let flatpak_remotes_selection_model_rc: Rc<RefCell<gtk::SingleSelection>> = Rc::new(RefCell::default());

    let flatpak_remotes_selection_model_rc_clone0 = Rc::clone(&flatpak_remotes_selection_model_rc);

    let flatpak_remotes_columnview_bin = adw::Bin::new();

    let flatpak_remotes_columnview_bin_clone0 = flatpak_remotes_columnview_bin.clone();
    
    retry_signal_action.connect_activate(clone!(
        #[weak]
        flatpak_remotes_columnview_bin_clone0,
        #[strong]
        cancellable_no,
        move |_, _| {
        
            let flatpak_system_installation =
            libflatpak::Installation::new_system(cancellable_no).unwrap();
            let flatpak_user_installation =
            libflatpak::Installation::new_user(cancellable_no).unwrap();
        
            let system_remotes = match libflatpak::Installation::list_remotes(&flatpak_system_installation, cancellable_no) {
                Ok(t) => t,
                Err(_) => Vec::new()
            };
        
            let user_remotes = match libflatpak::Installation::list_remotes(&flatpak_user_installation, cancellable_no) {
                Ok(t) => t,
                Err(_) => Vec::new()
            };

        let flatpak_remotes_list_store = gio::ListStore::new::<BoxedAnyObject>();

        for remote in system_remotes {
            flatpak_remotes_list_store.append(&BoxedAnyObject::new(FlatpakRemote::System(remote)));
        };

        for remote in user_remotes {
            flatpak_remotes_list_store.append(&BoxedAnyObject::new(FlatpakRemote::User(remote)));
        };

        let flatpak_remotes_selection_model = SingleSelection::new(Some(flatpak_remotes_list_store));

        (*flatpak_remotes_selection_model_rc_clone0.borrow_mut() = flatpak_remotes_selection_model.clone());

        let flatpak_remotes_columnview = ColumnView::builder()
            .vexpand(true)
            .model(&flatpak_remotes_selection_model)
            .build();

        //

        let flatpak_remotes_columnview_factory0 = gtk::SignalListItemFactory::new();
        
        flatpak_remotes_columnview_factory0.connect_setup(move |_factory, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let row = Label::builder()
                .halign(Align::Start)
                .build();
            item.set_child(Some(&row));
        });

        flatpak_remotes_columnview_factory0.connect_bind(move |_factory, item| {
            let item: &ListItem = item.downcast_ref::<gtk::ListItem>().unwrap();
            let child = item.child().and_downcast::<Label>().unwrap();
            let entry: BoxedAnyObject = item.item().and_downcast::<BoxedAnyObject>().unwrap();
            let entry_borrow = entry.borrow::<FlatpakRemote>();
            let remote_name = match entry_borrow.deref() {
                FlatpakRemote::System(remote) => {
                    remote.name().unwrap_or_default()
                }
                FlatpakRemote::User(remote) => {
                    remote.name().unwrap_or_default()
                }
            };
            child.set_label(&remote_name);
        });
        
        let flatpak_remotes_columnview_col1 = gtk::ColumnViewColumn::builder()
            .title(t!("flatpak_remotes_columnview_col0_title"))
            .factory(&flatpak_remotes_columnview_factory0)
            .build();

        //

        let flatpak_remotes_columnview_factory1 = gtk::SignalListItemFactory::new();
        
        flatpak_remotes_columnview_factory1.connect_setup(move |_factory, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let row = Label::builder()
                .halign(Align::Start)
                .build();
            item.set_child(Some(&row));
        });

        flatpak_remotes_columnview_factory1.connect_bind(move |_factory, item| {
            let item: &ListItem = item.downcast_ref::<gtk::ListItem>().unwrap();
            let child = item.child().and_downcast::<Label>().unwrap();
            let entry: BoxedAnyObject = item.item().and_downcast::<BoxedAnyObject>().unwrap();
            let entry_borrow = entry.borrow::<FlatpakRemote>();
            let remote_title = match entry_borrow.deref() {
                FlatpakRemote::System(remote) => {
                    remote.title().unwrap_or_default()
                }
                FlatpakRemote::User(remote) => {
                    remote.title().unwrap_or_default()
                }
            };
            child.set_label(&remote_title);
        });
        
        let flatpak_remotes_columnview_col0 = gtk::ColumnViewColumn::builder()
            .title(t!("flatpak_remotes_columnview_col1_title"))
            .factory(&flatpak_remotes_columnview_factory1)
            .build();

        //

        let flatpak_remotes_columnview_factory2 = gtk::SignalListItemFactory::new();
        
        flatpak_remotes_columnview_factory2.connect_setup(move |_factory, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let row = Label::builder()
                .halign(Align::Start)
                .build();
            item.set_child(Some(&row));
        });

        flatpak_remotes_columnview_factory2.connect_bind(move |_factory, item| {
            let item: &ListItem = item.downcast_ref::<gtk::ListItem>().unwrap();
            let child = item.child().and_downcast::<Label>().unwrap();
            let entry: BoxedAnyObject = item.item().and_downcast::<BoxedAnyObject>().unwrap();
            let entry_borrow = entry.borrow::<FlatpakRemote>();
            let remote_url = match entry_borrow.deref() {
                FlatpakRemote::System(remote) => {
                    remote.url().unwrap_or_default()
                }
                FlatpakRemote::User(remote) => {
                    remote.url().unwrap_or_default()
                }
            };
            child.set_label(&remote_url);
        });
        
        let flatpak_remotes_columnview_col2 = gtk::ColumnViewColumn::builder()
            .title(t!("flatpak_remotes_columnview_col2_title"))
            .factory(&flatpak_remotes_columnview_factory2)
            .expand(true)
            .build();

        //

        let flatpak_remotes_columnview_factory3 = gtk::SignalListItemFactory::new();
        
        flatpak_remotes_columnview_factory3.connect_setup(move |_factory, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let row = Label::builder()
                .halign(Align::Start)
                .build();
            item.set_child(Some(&row));
        });

        flatpak_remotes_columnview_factory3.connect_bind(move |_factory, item| {
            let item: &ListItem = item.downcast_ref::<gtk::ListItem>().unwrap();
            let child = item.child().and_downcast::<Label>().unwrap();
            let entry: BoxedAnyObject = item.item().and_downcast::<BoxedAnyObject>().unwrap();
            let entry_borrow = entry.borrow::<FlatpakRemote>();
            match entry_borrow.deref() {
                FlatpakRemote::System(remote) => {
                    child.set_label(&t!("flatpak_remotes_columnview_system").to_string());
                }
                FlatpakRemote::User(remote) => {
                    child.set_label(&t!("flatpak_remotes_columnview_user").to_string());
                }
            };
        });
        
        let flatpak_remotes_columnview_col3 = gtk::ColumnViewColumn::builder()
            .title(t!("flatpak_remotes_columnview_col3_title"))
            .factory(&flatpak_remotes_columnview_factory3)
            .build();

        //
        flatpak_remotes_columnview.append_column(&flatpak_remotes_columnview_col0);
        flatpak_remotes_columnview.append_column(&flatpak_remotes_columnview_col1);
        flatpak_remotes_columnview.append_column(&flatpak_remotes_columnview_col2);
        flatpak_remotes_columnview.append_column(&flatpak_remotes_columnview_col3);
        flatpak_remotes_columnview_bin_clone0.set_child(Some(&flatpak_remotes_columnview));
    }));

    retry_signal_action.activate(None);

    let flatpak_remotes_box = Box::builder()
        .orientation(Orientation::Vertical)
        .margin_bottom(3)
        .margin_top(3)
        .margin_end(3)
        .margin_start(3)
        .build();

    let flatpak_remotes_viewport = ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .has_frame(true)
        .margin_bottom(15)
        .margin_top(15)
        .margin_end(15)
        .margin_start(15)
        .child(&flatpak_remotes_box)
        .height_request(390)
        .build();
    flatpak_remotes_viewport.add_css_class("round-all-scroll");

    //

    let flatpak_remotes_edit_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .homogeneous(true)
        .build();
    flatpak_remotes_edit_box.add_css_class("linked");

    let flatpak_remote_add_button = Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text(t!("flatpak_remote_add_button_tooltip_text"))
        //.halign(Align::End)
        .valign(Align::End)
        .build();

    let flatpak_remote_remove_button = Button::builder()
        .icon_name("edit-delete-symbolic")
        .tooltip_text(t!("flatpak_remote_remove_button_tooltip_text"))
        //.halign(Align::End)
        .valign(Align::End)
        .build();

    flatpak_remote_add_button.connect_clicked(clone!(
        #[strong]
        window,
        #[strong]
        retry_signal_action,
        #[strong]
        flatpak_retry_signal_action,
            move
            |_|
            {
                add_dialog::add_dialog_fn(
                    window.clone(),
                    &retry_signal_action,
                    &flatpak_retry_signal_action,
                );
            }
        )
    );

    flatpak_remote_remove_button.connect_clicked(clone!(
        #[strong]
        window,
        #[strong]
        flatpak_remotes_selection_model_rc,
        #[strong]
        retry_signal_action,
        #[strong]
        flatpak_retry_signal_action,
        #[strong]
        cancellable_no,
            move
            |_|
            {
                {
                    let (mut installation, mut remote_name, mut cmd_installation): (libflatpak::Installation, libflatpak::glib::GString, String);
                    {
                        let flatpak_remotes_selection_model = flatpak_remotes_selection_model_rc.borrow();
                        let selection = flatpak_remotes_selection_model.selected_item().unwrap();
                        let item  = selection.downcast_ref::<BoxedAnyObject>().unwrap();
                        let flatpak_remote: Ref<FlatpakRemote> = item.borrow();
                        (installation, remote_name, cmd_installation) = match flatpak_remote.deref() {
                            FlatpakRemote::System(remote) => {
                                (libflatpak::Installation::new_system(cancellable_no).unwrap(), remote.name().unwrap_or_default(), "--system".to_string())
                            }
                            FlatpakRemote::User(remote) => {
                                (libflatpak::Installation::new_user(cancellable_no).unwrap(), remote.name().unwrap_or_default(), "--user".to_string())
                            }
                        };
                    }
                    match libflatpak::Installation::remove_remote(&installation, &remote_name, cancellable_no) {
                                Ok(_) => {
                                    retry_signal_action.activate(None);
                                    flatpak_retry_signal_action.activate(None);
                                }
                                Err(e) => {
                                    match e.matches(libflatpak::Error::RemoteUsed) {
                                        true => {
                                                    let flatpak_remote_add_error_dialog = adw::MessageDialog::builder()
                                                        .heading(t!("flatpak_remote_add_error_dialog_heading"))
                                                        .body(e.to_string() + "\n" + &t!("flatpak_remote_add_error_dialog_used_error_body").to_string())
                                                        .build();
                                                    flatpak_remote_add_error_dialog.add_response(
                                                        "flatpak_remote_add_error_dialog_used_no",
                                                        &t!("flatpak_remote_add_error_used_no_label").to_string(),
                                                        );
                                                    flatpak_remote_add_error_dialog.add_response(
                                                        "flatpak_remote_add_error_dialog_used_yes",
                                                        &t!("flatpak_remote_add_error_used_yes_label").to_string(),
                                                    );
                                                    flatpak_remote_add_error_dialog.set_response_appearance(
                                                        "flatpak_remote_add_error_dialog_used_yes",
                                                        adw::ResponseAppearance::Destructive,
                                                    );
                                                    let retry_signal_action_clone0 = retry_signal_action.clone();
                                                    let flatpak_retry_signal_action_clone0 = flatpak_retry_signal_action.clone();
                                                    flatpak_remote_add_error_dialog.clone()
                                                        .choose(None::<&gio::Cancellable>, move |choice| {
                                                            match choice.as_str() {
                                                                "flatpak_remote_add_error_dialog_used_yes" => {
                                                                    match duct::cmd!("flatpak", "remote-delete",  "--force", "--noninteractive", &cmd_installation, &remote_name).run() {
                                                                    Ok(_) => {
                                                                        retry_signal_action_clone0.activate(None);
                                                                        flatpak_retry_signal_action_clone0.activate(None);
                                                                    }
                                                                    Err(e) => {
                                                                        let flatpak_remote_add_error_dialog = adw::MessageDialog::builder()
                                                                            .heading(t!("flatpak_remote_add_error_dialog_heading"))
                                                                            .body(e.to_string())
                                                                            .build();
                                                                        flatpak_remote_add_error_dialog.add_response(
                                                                            "flatpak_remote_add_error_dialog_ok",
                                                                            &t!("flatpak_remote_add_error_dialog_ok_label").to_string(),
                                                                            );
                                                                        flatpak_remote_add_error_dialog.present();
                                                                    }
                                                                    }
                                                                }
                                                                _ => {}
                                                        }
                                                    }
                                                );
                                        }
                                        false => {
                                                let flatpak_remote_add_error_dialog = adw::MessageDialog::builder()
                                                    .heading(t!("flatpak_remote_add_error_dialog_heading"))
                                                    .body(e.to_string())
                                                    .build();
                                                flatpak_remote_add_error_dialog.add_response(
                                                    "flatpak_remote_add_error_dialog_ok",
                                                    &t!("flatpak_remote_add_error_dialog_ok_label").to_string(),
                                                    );
                                                flatpak_remote_add_error_dialog.present();
                                        }
                                    }
                                }
                        }
                }
            }
        )
    );

    //

    flatpak_remotes_edit_box.append(&flatpak_remote_add_button);
    flatpak_remotes_edit_box.append(&flatpak_remote_remove_button);

    flatpak_remotes_box.append(&flatpak_remotes_columnview_bin);
    flatpak_remotes_box.append(&flatpak_remotes_edit_box);

    //
    main_box.append(&flatpak_remotes_label0);
    main_box.append(&flatpak_remotes_label1);
    main_box.append(&flatpak_remotes_viewport);

    main_box
}