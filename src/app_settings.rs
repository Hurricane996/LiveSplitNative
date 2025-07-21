use std::{fs::File, io, path::PathBuf};

use livesplit_core::HotkeyConfig;
use serde::{Deserialize, Serialize};
use std::io::BufReader;
use thiserror::Error;

#[derive(Default, Serialize, Deserialize)]
pub struct Settings {
    pub hkc: HotkeyConfig,
    pub splits_path: Option<PathBuf>,
    pub layout_path: Option<PathBuf>,
}

#[derive(Debug, Error)]
pub enum SettingsSaveError {
    #[error("Platform does not support config directory")]
    UnsupportedConfigDirectory,
    #[error("Could not open settings file  - got error {0}")]
    FailedToOpenSettingsFile(#[from] io::Error),
    #[error("Could not write settings  - got error {0}")]
    SerializationFailure(#[from] serde_json::Error),
}

impl Settings {
    pub fn load() -> Option<Self> {
        let mut settings_file_path = dirs::config_dir()?;

        settings_file_path.push("livesplit.json");

        serde_json::from_reader(BufReader::new(File::open(settings_file_path).ok()?)).ok()
    }

    pub fn save(&self) -> Result<(), SettingsSaveError> {
        let mut settings_file_path =
            dirs::config_dir().ok_or(SettingsSaveError::UnsupportedConfigDirectory)?;

        settings_file_path.push("livesplit.json");

        let writer = File::create(settings_file_path)?;

        serde_json::to_writer(writer, self)?;

        Ok(())
    }
}
