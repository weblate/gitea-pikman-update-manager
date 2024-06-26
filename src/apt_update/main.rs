use std::time::Duration;
use adw::glib::Sender;
use async_channel::Receiver;
use rust_apt::new_cache;
use rust_apt::cache::*;
use tokio::net::{UnixListener, UnixStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use rust_apt::progress::{AcquireProgress, DynAcquireProgress};
use rust_apt::raw::{AcqTextStatus, ItemDesc, ItemState, PkgAcquire};
use std::thread;
use tokio::runtime::Runtime;





use serde::{Serialize, Deserialize};
#[derive(Serialize)]
struct AptSendablePackage {
    name: String,
    arch: String,
    installed_version: String,
    candidate_version: String
}
pub struct AptUpdateProgressSocket {
    pulse_interval: usize,
    max: usize,
    progress: f32
}

impl<'a> AptUpdateProgressSocket {
    /// Returns a new default progress instance.
    pub fn new() -> Self {
        let mut progress = Self {
            pulse_interval: 0,
            max: 0,
            progress: 0.0,
        };
        progress
    }
}

impl<'a> DynAcquireProgress for AptUpdateProgressSocket {
    /// Used to send the pulse interval to the apt progress class.
    ///
    /// Pulse Interval is in microseconds.
    ///
    /// Example: 1 second = 1000000 microseconds.
    ///
    /// Apt default is 500000 microseconds or 0.5 seconds.
    ///
    /// The higher the number, the less frequent pulse updates will be.
    ///
    /// Pulse Interval set to 0 assumes the apt defaults.
    fn pulse_interval(&self) -> usize { self.pulse_interval }

    /// Called when an item is confirmed to be up-to-date.
    ///
    /// Prints out the short description and the expected size.
    fn hit(&mut self, item: &ItemDesc) {
        println!("Download: {}: {}", item.uri(), item.description());
    }

    /// Called when an Item has started to download
    ///
    /// Prints out the short description and the expected size.
    fn fetch(&mut self, item: &ItemDesc) {
        println!("Fetch: {}: {}", item.uri(), item.description());
    }

    /// Called when an item is successfully and completely fetched.
    ///
    /// We don't print anything here to remain consistent with apt.
    fn done(&mut self, _: &ItemDesc) {
        println!("APT Cache Update Complete!")
    }

    /// Called when progress has started.
    ///
    /// Start does not pass information into the method.
    ///
    /// We do not print anything here to remain consistent with apt.
    fn start(&mut self) {
        println!("Starting APT Cache Update.");
    }

    /// Called when progress has finished.
    ///
    /// Stop does not pass information into the method.
    ///
    /// prints out the bytes downloaded and the overall average line speed.
    fn stop(&mut self, status: &AcqTextStatus) {
        println!("APT Cache Update Stopped!")
    }

    /// Called when an Item fails to download.
    ///
    /// Print out the ErrorText for the Item.
    fn fail(&mut self, item: &ItemDesc) {
        println!("APT Cache Update Failed!")
    }

    /// Called periodically to provide the overall progress information
    ///
    /// Draws the current progress.
    /// Each line has an overall percent meter and a per active item status
    /// meter along with an overall bandwidth and ETA indicator.
    fn pulse(&mut self, status: &AcqTextStatus, owner: &PkgAcquire) {
        let progress_percent: f32 = (status.current_bytes() as f32 * 100.0) / status.total_bytes() as f32;
        Runtime::new().unwrap().block_on(send_update_progress(progress_percent));
    }
}

fn main() {
    let update_cache = new_cache!().unwrap();
    match update_cache.update(&mut AcquireProgress::new(AptUpdateProgressSocket::new())) {
        Ok(_) => {}
        Err(e) => panic!("{}", e.to_string())
    };
}

async fn send_update_progress(progress_f32: f32) {
    // Path to the Unix socket file
    let socket_path = "/tmp/pika_apt_update.sock";

    // Connect to the Unix socket
    let mut stream = UnixStream::connect(socket_path).await.expect("Could not connect to server");

    let message = progress_f32.to_string();
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

async fn get_upgradable_packages() {
    // Create upgradable list cache
    let upgradable_cache = new_cache!().unwrap();

    // Create pack sort from upgradable_cache
    let upgradable_sort = PackageSort::default().upgradable().names();

    for pkg in upgradable_cache.packages(&upgradable_sort) {
        let package_struct = AptSendablePackage {
            name: pkg.name().to_string(),
            arch: pkg.arch().to_string(),
            installed_version: pkg.installed().unwrap().version().to_string(),
            candidate_version: pkg.candidate().unwrap().version().to_string()
        };

        // Path to the Unix socket file
        let socket_path = "/tmp/pika_apt_update.sock";

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
}