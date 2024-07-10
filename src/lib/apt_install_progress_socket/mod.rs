use crate::pika_unixsocket_tools::*;
use rust_apt::progress::{DynAcquireProgress, DynInstallProgress};
use rust_apt::raw::{AcqTextStatus, ItemDesc, PkgAcquire};
use std::process::exit;
use tokio::io::{AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::runtime::Runtime;

pub struct AptInstallProgressSocket<'a> {
    pkgname: String,
    steps_done: u64,
    total_steps: u64,
    action: String,
    percent_socket_path: &'a str,
    status_socket_path: &'a str,
}

impl<'a> AptInstallProgressSocket<'a> {
    /// Returns a new default progress instance.
    pub fn new(percent_socket_path: &'a str, status_socket_path: &'a str) -> Self {
        let mut progress = Self {
            pkgname: String::new(),
            steps_done: 0,
            total_steps: 0,
            action: String::new(),
            percent_socket_path: percent_socket_path,
            status_socket_path: status_socket_path,
        };
        progress
    }
}

impl<'a> DynInstallProgress for AptInstallProgressSocket<'a> {
    fn status_changed(
        &mut self,
        pkgname: String,
        steps_done: u64,
        total_steps: u64,
        action: String
    ) {
        let progress_percent: f32 =
            (steps_done as f32 * 100.0) / total_steps as f32;
        Runtime::new().unwrap().block_on(send_progress_percent(
            progress_percent,
            self.percent_socket_path,
        ));
        Runtime::new().unwrap().block_on(send_progress_status(
            &action,
            self.status_socket_path,
        ));
    }

    fn error(
        &mut self,
        pkgname: String,
        steps_done: u64,
        total_steps: u64,
        error: String
    ) {
        let message = format!(
            "dpkg failure on {}: {}",
            pkgname,
            error
        );
        eprintln!("{}", &message);
        Runtime::new()
            .unwrap()
            .block_on(send_progress_status(&message, self.status_socket_path));
        Runtime::new()
            .unwrap()
            .block_on(send_failed_to_socket(self.percent_socket_path));
        Runtime::new()
            .unwrap()
            .block_on(send_failed_to_socket(self.status_socket_path));
        exit(53)
    }
}

async fn send_progress_percent(progress_f32: f32, socket_path: &str) {
    // Connect to the Unix socket
    let mut stream = UnixStream::connect(socket_path)
        .await
        .expect("Could not connect to server");

    let message = progress_f32.to_string();
    // Send the message to the server
    stream
        .write_all(message.as_bytes())
        .await
        .expect("Failed to write to stream");
}

async fn send_progress_status(message: &str, socket_path: &str) {
    // Connect to the Unix socket
    let mut stream = UnixStream::connect(socket_path)
        .await
        .expect("Could not connect to server");

    // Send the message to the server
    stream
        .write_all(message.as_bytes())
        .await
        .expect("Failed to write to stream");
}
