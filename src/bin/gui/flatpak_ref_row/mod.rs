mod imp;

use crate::flatpak_update_page::FlatpakRefStruct;
use glib::Object;
use gtk::glib;

glib::wrapper! {
    pub struct FlatpakRefRow(ObjectSubclass<imp::FlatpakRefRow>)
        @extends adw::ExpanderRow, gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl FlatpakRefRow {
    pub fn new(flatref: &FlatpakRefStruct) -> Self {
        let flatref = flatref.clone();
        Object::builder()
            .property("flatref-name", flatref.name)
            .property("flatref-arch", flatref.arch)
            .property("flatref-ref-name", flatref.ref_name)
            .property("flatref-summary", flatref.summary)
            .property("flatref-remote-name", flatref.remote_name)
            .property(
                "flatref-installed-size-installed",
                flatref.installed_size_installed,
            )
            .property(
                "flatref-installed-size-remote",
                flatref.installed_size_remote,
            )
            .property("flatref-download-size", flatref.download_size)
            .property("flatref-ref-format", flatref.ref_format)
            .property("flatref-is-system", flatref.is_system)
            .build()
    }
}
// ANCHOR_END: mod

impl Default for FlatpakRefRow {
    fn default() -> Self {
        Self::new(&FlatpakRefStruct {
            ref_name: "??".to_owned(),
            name: "??".to_owned(),
            arch: "??".to_owned(),
            summary: "??".to_owned(),
            remote_name: "??".to_owned(),
            installed_size_installed: 0,
            installed_size_remote: 0,
            download_size: 0,
            ref_format: "??".to_owned(),
            is_system: false,
            is_last: false,
        })
    }
}
