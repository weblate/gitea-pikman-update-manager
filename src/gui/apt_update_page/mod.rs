use rust_apt::*;
use rust_apt::cache::*;
use rust_apt::new_cache;
pub fn apt_update_page() {
    let cache = match new_cache!() {
        Ok(t) => t,
        Err(_) => panic!("APT CACHE FAIL")
    };
    let sort = PackageSort::default().upgradable().names();
}