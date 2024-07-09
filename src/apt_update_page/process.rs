use rust_apt::new_cache;
use rust_apt::cache::Upgrade;

#[derive(Debug)]
struct AptChangesInfo {
    package_count: u64,
    total_download_size: u64,
    total_installed_size: u64
}

impl AptChangesInfo {
    fn add_package(&mut self) {
        self.package_count += 1;
    }

    fn increase_total_download_size_by(&mut self, value: u64) {
        self.total_download_size += value;
    }

    fn increase_total_installed_size_by(&mut self, value: u64) {
        self.total_installed_size += value;
    }
}

pub fn apt_process_update(excluded_updates_vec: &Vec<String>) {

    // Emulate Apt Full Upgrade to get transaction info
    let mut apt_changes_struct = AptChangesInfo {
        package_count: 0,
        total_download_size: 0,
        total_installed_size: 0
    };

    let apt_cache = new_cache!().unwrap();

    apt_cache.upgrade(Upgrade::FullUpgrade).unwrap();

    for change in apt_cache.get_changes(false) {
        if !excluded_updates_vec.iter().any(|e| change.name().contains(e)) {
            apt_changes_struct.add_package();
            apt_changes_struct.increase_total_download_size_by(change.candidate().unwrap().size());
            apt_changes_struct.increase_total_installed_size_by(change.candidate().unwrap().installed_size());
        }
    }

    dbg!(apt_changes_struct);
}
