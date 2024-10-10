use crate::pika_unixsocket_tools::*;
use rust_apt::progress::DynAcquireProgress;
use rust_apt::raw::{AcqTextStatus, ItemDesc, PkgAcquire};
use std::process::exit;
use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;
use tokio::runtime::Runtime;

pub struct AptUpdateProgressSocket<'a> {
    last_pulse_bytes: u64,
    pulse_interval: usize,
    percent_socket_path: &'a str,
    status_socket_path: &'a str,
    speed_socket_path: &'a str,
    hit_strfmt_trans_str: &'a str,
    fetch_strfmt_trans_str: &'a str,
    done_strfmt_trans_str: &'a str,
    fail_strfmt_trans_str: &'a str,
}

impl<'a> AptUpdateProgressSocket<'a> {
    /// Returns a new default progress instance.
    pub fn new(
        percent_socket_path: &'a str,
        status_socket_path: &'a str,
        speed_socket_path: &'a str,
        hit_strfmt_trans_str: &'a str,
        fetch_strfmt_trans_str: &'a str,
        done_strfmt_trans_str: &'a str,
        fail_strfmt_trans_str: &'a str,
    ) -> Self {
        let progress = Self {
            last_pulse_bytes: 0,
            pulse_interval: 1000000,
            percent_socket_path: percent_socket_path,
            status_socket_path: status_socket_path,
            speed_socket_path: speed_socket_path,
            hit_strfmt_trans_str: hit_strfmt_trans_str,
            fetch_strfmt_trans_str: fetch_strfmt_trans_str,
            done_strfmt_trans_str: done_strfmt_trans_str,
            fail_strfmt_trans_str: fail_strfmt_trans_str,
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
    fn pulse_interval(&self) -> usize {
        self.pulse_interval
    }

    /// Called when an item is confirmed to be up-to-date.
    ///
    /// Prints out the short description and the expected size.
    fn hit(&mut self, item: &ItemDesc) {
        let message = &strfmt::strfmt(
            &self.hit_strfmt_trans_str,
            &std::collections::HashMap::from([
                ("DESC".to_string(), item.description()),
                ("SHORT_DESC".to_string(), item.short_desc()),
            ]),
        )
        .unwrap();
        println!("{}", message);
        Runtime::new()
            .unwrap()
            .block_on(send_progress_status(&message, self.status_socket_path));
    }

    /// Called when an Item has started to download
    ///
    /// Prints out the short description and the expected size.
    fn fetch(&mut self, item: &ItemDesc) {
        let message = &strfmt::strfmt(
            &self.fetch_strfmt_trans_str,
            &std::collections::HashMap::from([
                ("DESC".to_string(), item.description()),
                ("SHORT_DESC".to_string(), item.short_desc()),
            ]),
        )
        .unwrap();
        println!("{}", message);
        Runtime::new()
            .unwrap()
            .block_on(send_progress_status(&message, self.status_socket_path));
    }

    /// Called when an item is successfully and completely fetched.
    ///
    /// We don't print anything here to remain consistent with apt.
    fn done(&mut self, item: &ItemDesc) {
        let message = &strfmt::strfmt(
            &self.done_strfmt_trans_str,
            &std::collections::HashMap::from([
                ("DESC".to_string(), item.description()),
                ("SHORT_DESC".to_string(), item.short_desc()),
            ]),
        )
        .unwrap();
        println!("{}", message);
        Runtime::new()
            .unwrap()
            .block_on(send_progress_status(&message, self.status_socket_path));
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
    fn stop(&mut self, _status: &AcqTextStatus) {}

    /// Called when an Item fails to download.
    ///
    /// Print out the ErrorText for the Item.
    fn fail(&mut self, item: &ItemDesc) {
        let message = &strfmt::strfmt(
            &self.fail_strfmt_trans_str,
            &std::collections::HashMap::from([
                ("DESC".to_string(), item.description()),
                ("SHORT_DESC".to_string(), item.short_desc()),
            ]),
        )
        .unwrap();
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

    /// Called periodically to provide the overall progress information
    ///
    /// Draws the current progress.
    /// Each line has an overall percent meter and a per active item status
    /// meter along with an overall bandwidth and ETA indicator.
    fn pulse(&mut self, status: &AcqTextStatus, _owner: &PkgAcquire) {
        let progress_percent: f32 =
            (status.current_bytes() as f32 * 100.0) / status.total_bytes() as f32;
        let speed = if self.pulse_interval != 0 {
            (status.current_bytes() as f64 - self.last_pulse_bytes as f64) / (self.pulse_interval as f64 / 1000000.0)
        } else {
            status.current_bytes() as f64 - self.last_pulse_bytes as f64
        };
        self.last_pulse_bytes = status.current_bytes();
        Runtime::new().unwrap().block_on(send_progress_percent(
            progress_percent,
            self.percent_socket_path,
        ));
        Runtime::new().unwrap().block_on(send_progress_status(
            &(pretty_bytes::converter::convert(speed) + "ps").to_lowercase(),
            self.speed_socket_path,
        ));
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
