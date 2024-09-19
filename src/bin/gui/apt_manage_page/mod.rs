use crate::apt_package_row::AptPackageRow;
use adw::gio::SimpleAction;
use adw::prelude::*;
use apt_deb822_tools::Deb822Repository;
use gtk::glib::*;
use gtk::*;
use std::cell::Ref;
use std::ops::Deref;
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

    let mut unofficial_deb822_sources = deb822_sources.clone();

    let system_source = deb822_sources.iter().filter(|x| {
        match &x.repolib_id {
            Some(t) => {
                t == "system"
            }
            None => false
        }
    }).next().unwrap();
    
    unofficial_deb822_sources.retain(|x| {
        match &x.repolib_id {
            Some(t) => {
                !(t == "system")
            }
            None => true
        }
    });

    let legacy_apt_repos = apt_legacy_tools::LegacyAptSource::get_legacy_sources();

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

    enum AptSourceConfig {
        Legacy(apt_legacy_tools::LegacyAptSource),
        DEB822(apt_deb822_tools::Deb822Repository)
    }

    for deb822_source in unofficial_deb822_sources {
        unofficial_sources_list_store.append(&BoxedAnyObject::new(AptSourceConfig::DEB822(deb822_source)));
    };

    match legacy_apt_repos {
        Ok(vec) => {
            for legacy_repo in vec {
                unofficial_sources_list_store.append(&BoxedAnyObject::new(AptSourceConfig::Legacy(legacy_repo)));
            };
        }
        Err(_) => {}
    }

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

    //

    let unofficial_sources_columnview_factory0 = gtk::SignalListItemFactory::new();
    
    unofficial_sources_columnview_factory0.connect_setup(move |_factory, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let row = Label::builder()
            .halign(Align::Start)
            .build();
        item.set_child(Some(&row));
    });

    unofficial_sources_columnview_factory0.connect_bind(move |_factory, item| {
        let item: &ListItem = item.downcast_ref::<gtk::ListItem>().unwrap();
        let child = item.child().and_downcast::<Label>().unwrap();
        let entry: BoxedAnyObject = item.item().and_downcast::<BoxedAnyObject>().unwrap();
        let entry_borrow = entry.borrow::<AptSourceConfig>();
        let repo_name = match entry_borrow.deref() {
            AptSourceConfig::DEB822(src) => {
                match &src.repolib_name {
                    Some(name) => name,
                    None => match(&src.uris, &src.suites, &src.components) {
                        (Some(uris),Some(suites),Some(components)) => {
                            &format!("{} {} {}", uris, suites, components)
                        }
                        (_,_,_) => {
                            &t!("apt_source_parse_error").to_string()
                        }
                    }
                    
                    
                }
            }
            AptSourceConfig::Legacy(src) => {
                &format!("{} {} {} {}",
                    if src.is_source {
                        "(Legacy Src)"
                    } else {
                        "(Legacy)"
                    },
                    &src.url,
                    &src.suite,
                    &src.components
                )
            }
        };
        child.set_label(&repo_name);
    });
    
    let unofficial_sources_columnview_col0 = gtk::ColumnViewColumn::builder()
        .title(t!("unofficial_sources_columnview_col0_title"))
        .factory(&unofficial_sources_columnview_factory0)
        .expand(true)
        .build();

    //

    let unofficial_sources_columnview_factory1 = gtk::SignalListItemFactory::new();
    
    unofficial_sources_columnview_factory1.connect_setup(move |_factory, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let row = Label::builder()
            .halign(Align::Start)
            .build();
        item.set_child(Some(&row));
    });

    unofficial_sources_columnview_factory1.connect_bind(move |_factory, item| {
        let item: &ListItem = item.downcast_ref::<gtk::ListItem>().unwrap();
        let child = item.child().and_downcast::<Label>().unwrap();
        let entry: BoxedAnyObject = item.item().and_downcast::<BoxedAnyObject>().unwrap();
        let entry_borrow = entry.borrow::<AptSourceConfig>();
        let repo_enabled = match entry_borrow.deref() {
            AptSourceConfig::DEB822(src) => {
                match &src.enabled {
                    Some(t) => match t.to_lowercase().as_str() {
                        "yes" => true,
                        "true" => true,
                        "no" => false,
                        "false" => false,
                        _ => true,
                    }
                    None => true,
                }
            }
            AptSourceConfig::Legacy(src) => {
                src.enabled
            }
        };
        if repo_enabled {
            child.set_label(&t!("apt_repo_enabled"));
        } else {
            child.set_label(&t!("apt_repo_disabled"));
        }
    });
    
    let unofficial_sources_columnview_col1 = gtk::ColumnViewColumn::builder()
        .title(t!("unofficial_sources_columnview_col1_title"))
        .factory(&unofficial_sources_columnview_factory1)
        .build();

    //

    unofficial_sources_selection_model.connect_selected_item_notify(|selection| {
        //let selection = selection.selected_item().unwrap();
        //let entry  = selection.downcast_ref::<BoxedAnyObject>().unwrap();
        //let r: Ref<AptSourceConfig> = entry.borrow();
        //println!("{}", r.col2.to_string())
    });
    unofficial_sources_columnview.append_column(&unofficial_sources_columnview_col0);
    unofficial_sources_columnview.append_column(&unofficial_sources_columnview_col1);

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