use pika_unixsocket_tools::apt_update_progress_socket::AptUpdateProgressSocket;
use pika_unixsocket_tools::pika_unixsocket_tools::*;
use rust_apt::new_cache;
use rust_apt::progress::{AcquireProgress, DynAcquireProgress};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::runtime::Runtime;

fn main() {
    let update_cache = new_cache!().unwrap();
    let percent_socket_path = "/tmp/pika_apt_update_percent.sock";
    let status_socket_path = "/tmp/pika_apt_update_status.sock";
    match update_cache.update(&mut AcquireProgress::new(AptUpdateProgressSocket::new(
        percent_socket_path,
        status_socket_path,
    ))) {
        Ok(_) => {
            Runtime::new()
                .unwrap()
                .block_on(send_successful_to_socket(percent_socket_path));
            Runtime::new()
                .unwrap()
                .block_on(send_successful_to_socket(status_socket_path));
        }
        Err(e) => {
            Runtime::new()
                .unwrap()
                .block_on(send_failed_to_socket(percent_socket_path));
            Runtime::new()
                .unwrap()
                .block_on(send_failed_to_socket(status_socket_path));
            panic!("{}", e.to_string())
        }
    };
}
