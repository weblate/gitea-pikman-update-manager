use rust_apt::new_cache;
use tokio::net::{UnixStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use rust_apt::progress::{AcquireProgress, DynAcquireProgress};
use rust_apt::raw::{AcqTextStatus, ItemDesc, PkgAcquire};
use tokio::runtime::Runtime;
use pika_unixsocket_tools::*;

pub struct AptUpdateProgressSocket<'a> {
    pulse_interval: usize,
    max: usize,
    progress: f32,
    percent_socket_path: &'a str,
    status_socket_path: &'a str,
}

impl<'a> AptUpdateProgressSocket<'a> {
    /// Returns a new default progress instance.
    pub fn new(percent_socket_path: &'a str, status_socket_path: &'a str) -> Self {
        let mut progress = Self {
            pulse_interval: 0,
            max: 0,
            progress: 0.0,
            percent_socket_path: percent_socket_path,
            status_socket_path: status_socket_path
        };
        progress
    }
}

impl<'a> DynAcquireProgress for AptUpdateProgressSocket<'a> {
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
        let message = format!("Up-to-date: {} {}", item.description(), item.short_desc());
        println!("{}", message);
        Runtime::new().unwrap().block_on(send_progress_status(message, self.status_socket_path));
    }

    /// Called when an Item has started to download
    ///
    /// Prints out the short description and the expected size.
    fn fetch(&mut self, item: &ItemDesc) {
        let message = format!("Fetching: {} {}", item.description(), item.short_desc());
        println!("{}", message);
        Runtime::new().unwrap().block_on(send_progress_status(message, self.status_socket_path));
    }

    /// Called when an item is successfully and completely fetched.
    ///
    /// We don't print anything here to remain consistent with apt.
    fn done(&mut self, item: &ItemDesc) {
        let message = format!("Downloading: {} {}", item.description(), item.short_desc());
        println!("{}", message);
        Runtime::new().unwrap().block_on(send_progress_status(message, self.status_socket_path));
    }

    /// Called when progress has started.
    ///
    /// Start does not pass information into the method.
    ///
    /// We do not print anything here to remain consistent with apt.
    fn start(&mut self) {}

    /// Called when progress has finished.
    ///
    /// Stop does not pass information into the method.
    ///
    /// prints out the bytes downloaded and the overall average line speed.
    fn stop(&mut self, status: &AcqTextStatus) {}

    /// Called when an Item fails to download.
    ///
    /// Print out the ErrorText for the Item.
    fn fail(&mut self, item: &ItemDesc) {
        let message = format!("Download Failed!: {} {}", item.description(), item.short_desc());
        eprintln!("{}", message);
        Runtime::new().unwrap().block_on(send_progress_status(message, self.status_socket_path));
    }

    /// Called periodically to provide the overall progress information
    ///
    /// Draws the current progress.
    /// Each line has an overall percent meter and a per active item status
    /// meter along with an overall bandwidth and ETA indicator.
    fn pulse(&mut self, status: &AcqTextStatus, owner: &PkgAcquire) {
        let progress_percent: f32 = (status.current_bytes() as f32 * 100.0) / status.total_bytes() as f32;
        Runtime::new().unwrap().block_on(send_progress_percent(progress_percent, self.percent_socket_path));
    }
}

fn main() {
    let update_cache = new_cache!().unwrap();
    let percent_socket_path = "/tmp/pika_apt_update_percent.sock";
    let status_socket_path = "/tmp/pika_apt_update_status.sock";
    match update_cache.update(&mut AcquireProgress::new(AptUpdateProgressSocket::new(percent_socket_path, status_socket_path))) {
        Ok(_) => {
            Runtime::new().unwrap().block_on(send_successful_to_socket(percent_socket_path));
            Runtime::new().unwrap().block_on(send_successful_to_socket(status_socket_path));
        }
        Err(e) => {
            Runtime::new().unwrap().block_on(send_failed_to_socket(percent_socket_path));
            Runtime::new().unwrap().block_on(send_failed_to_socket(status_socket_path));
            panic!("{}", e.to_string())
        }
    };
}

async fn send_progress_percent(progress_f32: f32, socket_path: &str) {
    // Connect to the Unix socket
    let mut stream = UnixStream::connect(socket_path).await.expect("Could not connect to server");

    let message = progress_f32.to_string();
    // Send the message to the server
    stream.write_all(message.as_bytes()).await.expect("Failed to write to stream");
}

async fn send_progress_status(message: String, socket_path: &str) {
    // Connect to the Unix socket
    let mut stream = UnixStream::connect(socket_path).await.expect("Could not connect to server");

    // Send the message to the server
    stream.write_all(message.as_bytes()).await.expect("Failed to write to stream");
}