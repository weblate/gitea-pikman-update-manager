use rust_apt::new_cache;
use rust_apt::progress::{AcquireProgress, InstallProgress};
use tokio::runtime::Runtime;
use pika_unixsocket_tools::apt_update_progress_socket::AptUpdateProgressSocket;
use pika_unixsocket_tools::apt_install_progress_socket::AptInstallProgressSocket;
use pika_unixsocket_tools::pika_unixsocket_tools::*;

fn main() {
    let cache = new_cache!().unwrap();
    let percent_socket_path = "/tmp/pika_apt_upgrade_percent.sock";
    let status_socket_path = "/tmp/pika_apt_upgrade_status.sock";

    let pkg = cache.get("neovim").unwrap();
    let mut acquire_progress = AcquireProgress::new(AptUpdateProgressSocket::new(
        percent_socket_path,
        status_socket_path,
    ));
    let mut install_progress = InstallProgress::new(AptInstallProgressSocket::new(
        percent_socket_path,
        status_socket_path,
    ));

    pkg.mark_install(true, true);
    pkg.protect();
    cache.resolve(true).unwrap();

    match cache.get_archives(&mut acquire_progress) {
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

    match cache.do_install(&mut install_progress) {
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