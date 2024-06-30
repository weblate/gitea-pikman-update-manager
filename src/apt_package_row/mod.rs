mod imp;

use crate::apt_update_page::AptPackageSocket;
use glib::Object;
use gtk::glib;

glib::wrapper! {
    pub struct AptPackageRow(ObjectSubclass<imp::AptPackageRow>)
        @extends adw::ExpanderRow, gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl AptPackageRow {
    pub fn new(package: AptPackageSocket) -> Self {
        Object::builder()
            .property("package-name", package.name)
            .property("package-arch", package.arch)
            .property("package-installed-version", package.installed_version)
            .property("package-candidate-version", package.candidate_version)
            .property("package-description", package.description)
            .property("package-source-uri", package.source_uri)
            .property("package-maintainer", package.maintainer)
            .property("package-size", package.size)
            .property("package-installed-size", package.installed_size)
            .build()
    }
}
// ANCHOR_END: mod

impl Default for AptPackageRow {
    fn default() -> Self {
        Self::new(AptPackageSocket {
            name: "name".to_string(),
            arch: "arch".to_string(),
            installed_version: "0.0".to_string(),
            candidate_version: "0.0".to_string(),
            description: "??".to_string(),
            source_uri: "??".to_string(),
            maintainer: "??".to_string(),
            size: 0,
            installed_size: 0,
        })
    }
}
