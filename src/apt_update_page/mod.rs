use std::{fs, thread};
use std::path::Path;
use rust_apt::*;
use rust_apt::cache::*;
use rust_apt::new_cache;
use rust_apt::records::RecordField;
use std::process::Command;
use gtk::glib::*;
use adw::prelude::*;
use gtk::*;
use tokio::net::{UnixListener, UnixStream};
use tokio::io::AsyncReadExt;
use tokio::runtime::Runtime;
use tokio::task;
use pika_unixsocket_tools::*;
use crate::apt_package_row::AptPackageRow;

pub struct AptPackageSocket {
    pub name: String,
    pub arch: String,
    pub installed_version: String,
    pub candidate_version: String,
    pub description: String,
    pub source_uri: String,
    pub maintainer: String,
    pub size: u64,
    pub installed_size: u64
}

pub fn apt_update_page(window: adw::ApplicationWindow) -> gtk::Box {
    let (update_percent_sender, update_percent_receiver) = async_channel::unbounded::<String>();
    let update_percent_sender = update_percent_sender.clone();
    let (update_status_sender, update_status_receiver) = async_channel::unbounded::<String>();
    let update_status_sender = update_status_sender.clone();
    let (get_upgradable_sender, get_upgradable_receiver) = async_channel::unbounded();
    let get_upgradable_sender = get_upgradable_sender.clone();

    thread::spawn(move || {
        Runtime::new().unwrap().block_on(update_percent_socket_server(update_percent_sender));
    });

    thread::spawn(move || {
        Runtime::new().unwrap().block_on(update_status_socket_server(update_status_sender));
    });

    Command::new("pkexec")
        .args(["/home/ward/RustroverProjects/pika-idk-manager/target/debug/apt_update"])
        .spawn();

    let main_box = gtk::Box::builder()
        .hexpand(true)
        .vexpand(true)
        .orientation(Orientation::Vertical)
        .build();

    let searchbar = gtk::SearchEntry::builder()
        .search_delay(500)
        .margin_bottom(15)
        .margin_start(15)
        .margin_end(30)
        .margin_start(30)
        .build();
    searchbar.add_css_class("rounded-all-25");

    let packages_boxedlist = gtk::ListBox::builder()
        .selection_mode(SelectionMode::None)
        .margin_bottom(15)
        .margin_start(15)
        .margin_end(15)
        .margin_start(15)
        .build();
    packages_boxedlist.add_css_class("boxed-list");
    let rows_size_group = gtk::SizeGroup::new(SizeGroupMode::Both);

    let packages_viewport = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vexpand(true)
        .hexpand(true)
        .margin_bottom(15)
        .margin_start(15)
        .margin_end(15)
        .margin_start(15)
        .height_request(390)
        .child(&packages_boxedlist)
        .build();

    let apt_update_dialog_child_box = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .build();

    let apt_update_dialog_progress_bar = gtk::ProgressBar::builder()
        .show_text(true)
        .hexpand(true)
        .build();

    let apt_update_dialog_spinner = gtk::Spinner::builder()
        .hexpand(true)
        .valign(Align::Start)
        .halign(Align::Center)
        .spinning(true)
        .height_request(128)
        .width_request(128)
        .build();

    apt_update_dialog_child_box.append(&apt_update_dialog_spinner);
    apt_update_dialog_child_box.append(&apt_update_dialog_progress_bar);

    let apt_update_dialog = adw::MessageDialog::builder()
        .transient_for(&window)
        .extra_child(&apt_update_dialog_child_box)
        .heading(t!("apt_update_dialog_heading"))
        .hide_on_close(true)
        .width_request(500)
        .build();

    let update_percent_server_context = MainContext::default();
    // The main loop executes the asynchronous block
    update_percent_server_context.spawn_local(clone!(@weak apt_update_dialog_progress_bar, @weak apt_update_dialog, @strong get_upgradable_sender => async move {
        while let Ok(state) = update_percent_receiver.recv().await {
            match state.as_ref() {
                "FN_OVERRIDE_SUCCESSFUL" => {
                    let get_upgradable_sender = get_upgradable_sender.clone();
                    thread::spawn( move || {
                        // Create upgradable list cache
                        let upgradable_cache = new_cache!().unwrap();

                        // Create pack sort from upgradable_cache
                        let upgradable_sort = PackageSort::default().upgradable().names();

                        for pkg in upgradable_cache.packages(&upgradable_sort) {
                            let candidate_version_pkg = pkg.candidate().unwrap();
                            let package_struct = AptPackageSocket {
                                name: pkg.name().to_string(),
                                arch: pkg.arch().to_string(),
                                installed_version: pkg.installed().unwrap().version().to_string(),
                                candidate_version: candidate_version_pkg.version().to_string(),
                                description: match candidate_version_pkg.description() {
                                    Some(s) => s,
                                    _ => t!("apt_pkg_property_unknown").to_string()
                                },
                                source_uri: candidate_version_pkg.uris().collect::<Vec<String>>().join("\n"),
                                maintainer: match candidate_version_pkg.get_record(RecordField::Maintainer) {
                                    Some(s) => s,
                                    _ => t!("apt_pkg_property_unknown").to_string()
                                },
                                size: candidate_version_pkg.size(),
                                installed_size: candidate_version_pkg.installed_size()
                            };
                            get_upgradable_sender.send_blocking(package_struct).unwrap()
                        }
                    });
                    apt_update_dialog.close();
                }
                _ => {
                    apt_update_dialog_progress_bar.set_fraction(state.parse::<f64>().unwrap()/100.0)
                }
            }
        }
        }));

    let update_status_server_context = MainContext::default();
    // The main loop executes the asynchronous block
    update_status_server_context.spawn_local(clone!(@weak apt_update_dialog, @weak apt_update_dialog_spinner => async move {
        while let Ok(state) = update_status_receiver.recv().await {
            match state.as_ref() {
                "FN_OVERRIDE_SUCCESSFUL" => {}
                "FN_OVERRIDE_FAILED" => {
                    apt_update_dialog_spinner.set_spinning(false);
                    apt_update_dialog.set_body(&t!("apt_update_dialog_status_failed").to_string())
                }
                _ => apt_update_dialog.set_body(&state)
            }
        }
        }));

    let get_upgradable_server_context = MainContext::default();
    // The main loop executes the asynchronous block
    get_upgradable_server_context.spawn_local(clone!(@weak packages_boxedlist => async move {
        while let Ok(state) = get_upgradable_receiver.recv().await {
            packages_boxedlist.append(&AptPackageRow::new(AptPackageSocket {
                name: state.name,
                arch: state.arch,
                installed_version: state.installed_version,
                candidate_version: state.candidate_version,
                description: state.description,
                source_uri: state.source_uri,
                maintainer: state.maintainer,
                size: state.size,
                installed_size: state.installed_size
            }));
        }
        }));


    main_box.append(&searchbar);
    main_box.append(&packages_viewport);

    apt_update_dialog.present();
    main_box
}


async fn update_percent_socket_server(buffer_sender: async_channel::Sender<String>) {
    // Path to the Unix socket file
    let socket_path = "/tmp/pika_apt_update_percent.sock";

    // Remove the socket file if it already exists
    if Path::new(socket_path).exists() {
        fs::remove_file(socket_path).expect("Could not remove existing socket file");
    }

    // Bind the Unix listener to the socket path
    let listener = UnixListener::bind(socket_path).expect("Could not bind");

    println!("Server listening on {}", socket_path);

    // Loop to accept incoming connections
    loop {
        // Accept an incoming connection
        match listener.accept().await {
            Ok((stream, _)) => {
                // Handle the connection in a separate task
                task::spawn(handle_client(stream, buffer_sender.clone()));
            }
            Err(e) => {
                // Print error message if a connection fails
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}

async fn update_status_socket_server(buffer_sender: async_channel::Sender<String>) {
    // Path to the Unix socket file
    let socket_path = "/tmp/pika_apt_update_status.sock";

    // Remove the socket file if it already exists
    if Path::new(socket_path).exists() {
        fs::remove_file(socket_path).expect("Could not remove existing socket file");
    }

    // Bind the Unix listener to the socket path
    let listener = UnixListener::bind(socket_path).expect("Could not bind");

    println!("Server listening on {}", socket_path);

    // Loop to accept incoming connections
    loop {
        // Accept an incoming connection
        match listener.accept().await {
            Ok((stream, _)) => {
                // Handle the connection in a separate task
                task::spawn(handle_client(stream, buffer_sender.clone()));
            }
            Err(e) => {
                // Print error message if a connection fails
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}
async fn get_upgradable_socket_server(buffer_sender: async_channel::Sender<String>) {
    // Path to the Unix socket file
    let socket_path = "/tmp/pika_apt_get_upgradable.sock";

    // Remove the socket file if it already exists
    if Path::new(socket_path).exists() {
        fs::remove_file(socket_path).expect("Could not remove existing socket file");
    }

    // Bind the Unix listener to the socket path
    let listener = UnixListener::bind(socket_path).expect("Could not bind");

    println!("Server listening on {}", socket_path);

    // Loop to accept incoming connections
    loop {
        // Accept an incoming connection
        match listener.accept().await {
            Ok((stream, _)) => {
                // Handle the connection in a separate task
                task::spawn(handle_client(stream, buffer_sender.clone()));
            }
            Err(e) => {
                // Print error message if a connection fails
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}