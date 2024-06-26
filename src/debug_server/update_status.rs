use tokio::net::{UnixListener, UnixStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::task;
use std::path::Path;
use std::fs;

// Entry point of the server binary
#[tokio::main]
async fn main() {
    // Path to the Unix socket file
    let pika_apt_update_socket_path = "/tmp/pika_apt_update_status.sock";

    // Remove the socket file if it already exists
    if Path::new(pika_apt_update_socket_path).exists() {
        fs::remove_file(pika_apt_update_socket_path).expect("Could not remove existing socket file");
    }

    // Bind the Unix listener to the socket path
    let pika_apt_update_listener = UnixListener::bind(pika_apt_update_socket_path).expect("Could not bind");

    println!("Server listening on {}", pika_apt_update_socket_path);

    // Loop to accept incoming connections
    loop {
        // Accept an incoming connection
        match pika_apt_update_listener.accept().await {
            Ok((stream, _)) => {
                // Handle the connection in a separate task
                task::spawn(handle_client(stream));
            }
            Err(e) => {
                // Print error message if a connection fails
                eprintln!("pika_apt_update: Connection failed: {}", e);
            }
        }
    }
}

// Function to handle a single client connection
async fn handle_client(mut stream: UnixStream) {
    // Buffer to store incoming data
    let mut buffer = [0; 1024];

    // Read data from the stream
    match stream.read(&mut buffer).await {
        Ok(size) => {
            // Print the received message
            println!("pika_apt_update: Received: {}", String::from_utf8_lossy(&buffer[..size]));
        }
        Err(e) => {
            // Print error message if reading fails
            eprintln!("Failed to read from stream: {}", e);
        }
    }
}
