use rust_apt::new_cache;
use rust_apt::cache::*;
use tokio::net::{UnixStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use serde::{Serialize};
#[derive(Serialize)]
struct AptPackageSocket {
    name: String,
    arch: String,
    installed_version: String,
    candidate_version: String
}
#[tokio::main]
async fn main() {
    // Create upgradable list cache
    let upgradable_cache = new_cache!().unwrap();

    // Create pack sort from upgradable_cache
    let upgradable_sort = PackageSort::default().upgradable().names();

    for pkg in upgradable_cache.packages(&upgradable_sort) {
            let package_struct = AptPackageSocket {
            name: pkg.name().to_string(),
            arch: pkg.arch().to_string(),
            installed_version: pkg.installed().unwrap().version().to_string(),
            candidate_version: pkg.candidate().unwrap().version().to_string()
        };

        // Path to the Unix socket file
        let socket_path = "/tmp/pika_apt_get_upgradable.sock";

        // Connect to the Unix socket
        let mut stream = UnixStream::connect(socket_path).await.expect("Could not connect to server");

        let message = serde_json::to_string(&package_struct).unwrap();
        // Send the message to the server
        stream.write_all(message.as_bytes()).await.expect("Failed to write to stream");

        // Buffer to store the server's response
        let mut buffer = [0; 2024];

        // Read the response from the server
        match stream.read(&mut buffer).await {
            Ok(size) => {
                // Print the received response
                //println!("Response from Server on GTK4: {}", String::from_utf8_lossy(&buffer[..size]));
            }
            Err(e) => {
                // Print error message if reading fails
                //eprintln!("Failed to read Server on GTK4 with Error: {}", e);
            }
        }
    }
}