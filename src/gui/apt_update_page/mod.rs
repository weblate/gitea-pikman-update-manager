use std::{fs, thread};
use std::path::Path;
use rust_apt::*;
use rust_apt::cache::*;
use rust_apt::new_cache;
use std::process::Command;
use gtk::glib::prelude::*;
use gtk::glib::{clone, MainContext};
use gtk::prelude::*;
use adw::prelude::*;
use gtk::*;
use adw::*;
use gtk::Orientation::Vertical;
use tokio::net::{UnixListener, UnixStream};
use tokio::io::AsyncReadExt;
use tokio::runtime::Runtime;
use tokio::task;
use pika_unixsocket_tools::*;

pub fn apt_update_page(window: &adw::ApplicationWindow) -> gtk::Box {
    let (update_percent_sender, update_percent_receiver) = async_channel::unbounded::<String>();
    let update_percent_sender = update_percent_sender.clone();
    let (update_status_sender, update_status_receiver) = async_channel::unbounded::<String>();
    let update_status_sender = update_status_sender.clone();
    let (get_upgradable_sender, get_upgradable_receiver) = async_channel::unbounded::<String>();
    let get_upgradable_sender = get_upgradable_sender.clone();

    thread::spawn(move || {
        Runtime::new().unwrap().block_on(update_percent_socket_server(update_percent_sender));
    });

    thread::spawn(move || {
        Runtime::new().unwrap().block_on(update_status_socket_server(update_status_sender));
    });

    thread::spawn(move || {
        Runtime::new().unwrap().block_on(get_upgradable_socket_server(get_upgradable_sender));
    });

    Command::new("pkexec")
        .args(["/home/ward/RustroverProjects/project-leoali/target/debug/apt_update"])
        .spawn();

    let apt_update_dialog_child_box = gtk::Box::builder()
        .orientation(Vertical)
        .build();

    let apt_update_dialog_progress_bar = gtk::ProgressBar::builder()
        .show_text(true)
        .hexpand(true)
        .build();

    apt_update_dialog_child_box.append(&gtk::Spinner::builder()
        .hexpand(true)
        .valign(Align::Start)
        .halign(Align::Center)
        .spinning(true)
        .height_request(128)
        .width_request(128)
        .build());
    apt_update_dialog_child_box.append(&apt_update_dialog_progress_bar);

    let apt_update_dialog = adw::MessageDialog::builder()
        .transient_for(window)
        .extra_child(&apt_update_dialog_child_box)
        .heading(t!("apt_update_dialog_heading"))
        .hide_on_close(true)
        .width_request(500)
        .build();

    let update_percent_server_context = MainContext::default();
    // The main loop executes the asynchronous block
    update_percent_server_context.spawn_local(clone!(@weak apt_update_dialog_progress_bar, @weak apt_update_dialog   => async move {
        while let Ok(state) = update_percent_receiver.recv().await {
            match state.as_ref() {
                "FN_OVERRIDE_SUCCESSFUL" => {
                    apt_update_dialog.close()
                }
                _ => {
                    apt_update_dialog_progress_bar.set_fraction(state.parse::<f64>().unwrap()/100.0)
                }
            }
        }
        }));

    let update_status_server_context = MainContext::default();
    // The main loop executes the asynchronous block
    update_status_server_context.spawn_local(clone!(@weak apt_update_dialog => async move {
        while let Ok(state) = update_status_receiver.recv().await {
            match state.as_ref() {
                "FN_OVERRIDE_SUCCESSFUL" => {}
                _ => apt_update_dialog.set_body(&state)
            }
        }
        }));

    let get_upgradable_server_context = MainContext::default();
    // The main loop executes the asynchronous block
    get_upgradable_server_context.spawn_local(clone!(@weak window => async move {
        while let Ok(state) = get_upgradable_receiver.recv().await {
            println!("{}", state)
        }
        }));


    apt_update_dialog.present();
    gtk::Box::new(Vertical, 0)
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