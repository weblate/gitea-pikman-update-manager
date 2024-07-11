use pika_unixsocket_tools::apt_install_progress_socket::AptInstallProgressSocket;
use pika_unixsocket_tools::apt_update_progress_socket::AptUpdateProgressSocket;
use pika_unixsocket_tools::pika_unixsocket_tools::*;
use rust_apt::cache::Upgrade;
use rust_apt::new_cache;
use rust_apt::progress::{AcquireProgress, InstallProgress};
use tokio::runtime::Runtime;

fn main() {
    let percent_socket_path = "/tmp/pika_apt_upgrade_percent.sock";
    let status_socket_path = "/tmp/pika_apt_upgrade_status.sock";
    let json_file_path = "/tmp/pika-apt-exclusions.json";
    let mut excluded_updates_vec: Vec<String> = Vec::new();

    if std::path::Path::new(json_file_path).exists() {
        let data = std::fs::read_to_string(json_file_path).expect("Unable to read file");
        let json: serde_json::Value =
            serde_json::from_str(&data).expect("JSON was not well-formatted");

        if let serde_json::Value::Array(exclusions) = &json["exclusions"] {
            for exclusion in exclusions {
                match exclusion["package"].as_str() {
                    Some(v) => {
                        excluded_updates_vec.push(v.to_owned());
                    }
                    None => {}
                }
            }
        }
    }

    let apt_cache = new_cache!().unwrap();

    apt_cache.upgrade(Upgrade::FullUpgrade).unwrap();

    let apt_upgrade_cache = if excluded_updates_vec.is_empty() {
        apt_cache
    } else {
        let apt_upgrade_cache = new_cache!().unwrap();
        for change in apt_cache.get_changes(false) {
            if !excluded_updates_vec
                .iter()
                .any(|e| change.name().contains(e))
            {
                let pkg = apt_upgrade_cache.get(change.name()).unwrap();
                if change.marked_upgrade() || change.marked_install() || change.marked_downgrade() {
                    pkg.mark_install(true, false);
                } else if change.marked_delete() {
                    pkg.mark_delete(false);
                }
                pkg.protect();
            }
        }
        apt_upgrade_cache
    };

    apt_upgrade_cache.resolve(true).unwrap();

    let mut acquire_progress = AcquireProgress::new(AptUpdateProgressSocket::new(
        percent_socket_path,
        status_socket_path,
    ));
    let mut install_progress = InstallProgress::new(AptInstallProgressSocket::new(
        percent_socket_path,
        status_socket_path,
    ));

    apt_upgrade_cache.resolve(true).unwrap();

    match apt_upgrade_cache.get_archives(&mut acquire_progress) {
        Ok(_) => {}
        Err(e) => {
            panic!("{}", e.to_string())
        }
    };

    match apt_upgrade_cache.do_install(&mut install_progress) {
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
}
