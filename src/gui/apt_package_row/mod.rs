mod imp;

use glib::Object;
use gtk::glib;
use crate::apt_update_page::AptPackageSocket;

glib::wrapper! {
    pub struct AptPackageRow(ObjectSubclass<imp::AptPackageRow>)
        @extends adw::ActionRow, gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl AptPackageRow {
    pub fn new(package: AptPackageSocket) -> Self {
        Object::builder()
            .property("package-name", package.name)
            .property("package-arch", package.arch)
            .property("package-installed-version", package.installed_version)
            .property("package-candidate-version", package.candidate_version)
            .build()
    }
}
// ANCHOR_END: mod

impl Default for AptPackageRow {
    fn default() -> Self {
        Self::new(AptPackageSocket{
            name: "name".to_string(),
            arch: "arch".to_string(),
            installed_version: "0.0".to_string(),
            candidate_version: "0.0".to_string()
        })
    }
}