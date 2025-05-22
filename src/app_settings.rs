use std::{fs::File, path::PathBuf};

use livesplit_core::HotkeyConfig;
use serde::{Deserialize, Serialize};
use std::io::BufReader;

#[derive(Default, Serialize, Deserialize)]
pub struct Settings {
    pub hkc: HotkeyConfig,
    pub splits_path: Option<PathBuf>,
    pub layout_path: Option<PathBuf>,
}

impl Settings {
    pub fn load() -> Option<Self> {
        let mut settings_file_path = dirs::config_dir()?;

        settings_file_path.push("livesplit.json");

        serde_json::from_reader(BufReader::new(File::open(settings_file_path).ok()?)).ok()
    }

    pub fn save(&self) {
        let mut settings_file_path = dirs::config_dir().unwrap();

        settings_file_path.push("livesplit.json");

        let writer = File::create(settings_file_path).unwrap();

        serde_json::to_writer(writer, self).ok();
    }
}
