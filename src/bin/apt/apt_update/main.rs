use pika_unixsocket_tools::apt_update_progress_socket::AptUpdateProgressSocket;
use pika_unixsocket_tools::pika_unixsocket_tools::*;
use rust_apt::new_cache;
use rust_apt::progress::AcquireProgress;
use std::env;
use tokio::runtime::Runtime;

// Init translations for current crate.
#[macro_use]
extern crate rust_i18n;
i18n!("locales", fallback = "en_US");

fn main() {
    let args: Vec<String> = env::args().collect();
    rust_i18n::set_locale(&args[1]);
    let instance = single_instance::SingleInstance::new(
        "com.github.pikaos-linux.pikmanupdatemanager.update.manager",
    )
    .unwrap();

    let hit_strfmt_trans_str = t!("apt_update_str_hit").to_string();
    let fetch_strfmt_trans_str = t!("apt_update_str_fetch").to_string();
    let done_strfmt_trans_str = t!("apt_update_str_done").to_string();
    let fail_strfmt_trans_str = t!("apt_update_str_fail").to_string();

    let update_cache = new_cache!().unwrap();
    let percent_socket_path = "/tmp/pika_apt_update_percent.sock";
    let status_socket_path = "/tmp/pika_apt_update_status.sock";
    let speed_socket_path = "/tmp/pika_apt_update_speed.sock";
    match update_cache.update(&mut AcquireProgress::new(AptUpdateProgressSocket::new(
        percent_socket_path,
        status_socket_path,
        speed_socket_path,
        &hit_strfmt_trans_str,
        &fetch_strfmt_trans_str,
        &done_strfmt_trans_str,
        &fail_strfmt_trans_str,
    ))) {
        Ok(_) => {}
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

    assert!(!instance.is_single());
}
