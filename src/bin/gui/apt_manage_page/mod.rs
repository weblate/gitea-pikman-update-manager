use crate::apt_package_row::AptPackageRow;
use adw::gio::SimpleAction;
use adw::prelude::*;
use apt_deb822_tools::Deb822Repository;
use gtk::glib::*;
use gtk::*;
use std::cell::Ref;
use pika_unixsocket_tools::pika_unixsocket_tools::*;
use property::PropertyGet;
use rust_apt::cache::*;
use rust_apt::new_cache;
use rust_apt::records::RecordField;
use std::cell::RefCell;
use std::process::Command;
use std::rc::Rc;
use std::thread;
use tokio::runtime::Runtime;

pub fn apt_manage_page(
    window: adw::ApplicationWindow,
    retry_signal_action: &SimpleAction,
) -> gtk::Box {

    let deb822_sources = Deb822Repository::get_deb822_sources().unwrap();

    let system_source = deb822_sources.iter().filter(|x| {
        match &x.repolib_id {
            Some(t) => {
                if t == "system" {
                    true
                } else {
                    false
                }
            }
            None => false
        }
    }).next().unwrap();

    let main_box = Box::builder()
        .hexpand(true)
        .vexpand(true)
        .orientation(Orientation::Vertical)
        .build();

    //

    let system_mirror_label0 = gtk::Label::builder()
        .label(t!("system_mirror_label0_label"))
        .halign(gtk::Align::Start)
        .valign(gtk::Align::Start)
        .hexpand(true)
        .margin_top(15)
        .margin_start(15)
        .margin_end(15)
        .margin_bottom(5)
        .build();
    system_mirror_label0.add_css_class("heading");

    let system_mirror_label1 = gtk::Label::builder()
        .label(t!("system_mirror_label1_label"))
        .halign(gtk::Align::Start)
        .valign(gtk::Align::Start)
        .hexpand(true)
        .margin_start(15)
        .margin_end(15)
        .build();

    let system_mirror_entry = gtk::Entry::builder()
        .placeholder_text(system_source.repolib_default_mirror.as_deref().unwrap())
        .text(system_source.uris.as_deref().unwrap())
        .valign(gtk::Align::Start)
        .margin_top(5)
        .margin_bottom(5)
        .margin_start(15)
        .margin_end(15)
        .build();

    //

    let unofficial_sources_label0 = gtk::Label::builder()
        .label(t!("unofficial_sources_label"))
        .halign(gtk::Align::Start)
        .valign(gtk::Align::Start)
        .hexpand(true)
        .margin_top(15)
        .margin_start(15)
        .margin_end(15)
        .margin_bottom(5)
        .build();
    unofficial_sources_label0.add_css_class("heading");

    let unofficial_sources_label1 = gtk::Label::builder()
        .label(t!("unofficial_sources_label1_label"))
        .halign(gtk::Align::Start)
        .valign(gtk::Align::Start)
        .hexpand(true)
        .margin_start(15)
        .margin_end(15)
        .build();

    let unofficial_sources_list_store = gio::ListStore::new::<BoxedAnyObject>();

    struct Row2 {
        col1: String,
        col2: String,
    }

    (0..10000).for_each(|i| {
        unofficial_sources_list_store.append(&BoxedAnyObject::new(Row2 {
            col1: format!("col1 {i}"),
            col2: format!("col2 {i}"),
        }))
    });

    let unofficial_sources_selection_model = SingleSelection::new(Some(unofficial_sources_list_store));

    /*let unofficial_sources_item_factory = SignalListItemFactory::new();

    unofficial_sources_item_factory.connect_setup(|_item_factory, list_item| {
        let label = gtk::Label::new(Some("DDD"));

        let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();

        list_item.set_child(Some(&label));
    });

    unofficial_sources_item_factory.connect_bind(|_item_factory, list_item| {
        let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();

        let list_item_item = list_item.item().unwrap().to_value().get::<String>().unwrap();
        let list_item_label = list_item.child().unwrap().downcast::<gtk::Label>().unwrap();

        list_item_label.set_label(&list_item_item);
    });
    */

    let unofficial_sources_columnview = ColumnView::builder()
        .margin_bottom(3)
        .margin_top(3)
        .margin_end(3)
        .margin_start(3)
        .vexpand(true)
        .model(&unofficial_sources_selection_model)
        .build();

        let col1factory = gtk::SignalListItemFactory::new();
    let col2factory = gtk::SignalListItemFactory::new();
    col1factory.connect_setup(move |_factory, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let row = Label::default();
        item.set_child(Some(&row));
    });

    col1factory.connect_bind(move |_factory, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let child = item.child().and_downcast::<Label>().unwrap();
        let entry = item.item().and_downcast::<BoxedAnyObject>().unwrap();
        let r: Ref<Row2> = entry.borrow();
        child.set_label(&r.col1.to_string());
    });
    col2factory.connect_setup(move |_factory, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let row = Label::default();
        item.set_child(Some(&row));
    });

    col2factory.connect_bind(move |_factory, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let child = item.child().and_downcast::<Label>().unwrap();
        let entry = item.item().and_downcast::<BoxedAnyObject>().unwrap();
        let r: Ref<Row2> = entry.borrow();
        child.set_label(&r.col2.to_string());
    });
    let col1: ColumnViewColumn = gtk::ColumnViewColumn::new(Some("Column 1"), Some(col1factory));
    let col2 = gtk::ColumnViewColumn::new(Some("Column 2"), Some(col2factory));
    col1.set_expand(true);
    unofficial_sources_selection_model.connect_selected_item_notify(|selection| {
        let selection = selection.selected_item().unwrap();
        let entry  = selection.downcast_ref::<BoxedAnyObject>().unwrap();
        let r: Ref<Row2> = entry.borrow();
        println!("{}", r.col2.to_string())
    });
    unofficial_sources_columnview.append_column(&col1);
    unofficial_sources_columnview.append_column(&col2);

    let unofficial_sources_boxedlist = ListBox::builder()
        .selection_mode(SelectionMode::None)
        .margin_bottom(3)
        .margin_top(3)
        .margin_end(3)
        .margin_start(3)
        .build();

    let unofficial_sources_viewport = ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .has_frame(true)
        .margin_bottom(15)
        .margin_top(15)
        .margin_end(15)
        .margin_start(15)
        .child(&unofficial_sources_columnview)
        .height_request(390)
        .build();
    unofficial_sources_viewport.add_css_class("round-all-scroll");

    //

    main_box.append(&system_mirror_label0);
    main_box.append(&system_mirror_label1);
    main_box.append(&system_mirror_entry);
    //
    main_box.append(&unofficial_sources_label0);
    main_box.append(&unofficial_sources_label1);
    main_box.append(&unofficial_sources_viewport);

    main_box
}