use std::path::PathBuf;

use bourbon_core::{SettingsFile, default_omarchy_colors_path};

pub(crate) struct Locations {
    pub(crate) settings: SettingsFile,
    pub(crate) omarchy_colors: PathBuf,
}

impl Locations {
    pub(crate) fn of_the_user() -> Self {
        Self {
            settings: SettingsFile::default_location(),
            omarchy_colors: default_omarchy_colors_path(),
        }
    }
}
