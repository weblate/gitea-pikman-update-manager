use rust_apt::new_cache;
use rust_apt::cache::*;
use tokio::net::{UnixListener, UnixStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use rust_apt::raw::progress::{AcquireProgress, AptAcquireProgress};
use serde::{Serialize, Deserialize};
#[derive(Serialize)]
struct AptSendablePackage {
    name: String,
    arch: String,
    installed_version: String,
    candidate_version: String
}
#[tokio::main]
async fn main() {
    let update_cache = new_cache!().unwrap();
    let mut update_progress: Box<dyn AcquireProgress> = Box::new(AptAcquireProgress::new());
    if let Err(error) = update_cache.update(&mut update_progress) {
        for msg in error.what().split(';') {
            if msg.starts_with("E:") {
                println!("Error: {}", &msg[2..]);
            }
            if msg.starts_with("W:") {
                println!("Warning: {}", &msg[2..]);
            }
        }
    }

    // Create upgradable list cache
    let upgradable_cache = new_cache!().unwrap();

    // Create pack sort from upgradable_cache
    let upgradable_sort = PackageSort::default().upgradable().names();

    for pkg in upgradable_cache.packages(&upgradable_sort).unwrap().into_iter() {
        let package_struct = AptSendablePackage {
            name: pkg.name().to_string(),
            arch: pkg.arch().to_string(),
            installed_version: pkg.installed().unwrap().version().to_string(),
            candidate_version: pkg.candidate().unwrap().version().to_string()
        };

        // Path to the Unix socket file
        let socket_path = "/tmp/rust-ipc.sock";

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
                println!("Response from Server on GTK4: {}", String::from_utf8_lossy(&buffer[..size]));
            }
            Err(e) => {
                // Print error message if reading fails
                eprintln!("Failed to read Server on GTK4 with Error: {}", e);
            }
        }
    }

    let cache = new_cache!().unwrap();
    let pkg = cache.get("neovim").unwrap();
    let mut progress = AptAcquireProgress::new_box();

    progress.status

    pkg.mark_install(true, true);
    pkg.protect();
    cache.resolve(true).unwrap();

    cache.get_archives(&mut progress).unwrap();

    progress.pulse()
}