use std::path::Path;

use serde::{Deserialize, Serialize};
pub use snap_herb_models::AppSettings;

pub const SETTINGS_FILE_NAME: &str = "snapherb.settings.json";

const SETTINGS_VERSION: u32 = 1;

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("settings file I/O failed: {0}")]
    Io(String),
    #[error("settings file is corrupt: {0}")]
    Corrupt(String),
}

#[derive(Debug, Serialize, Deserialize)]
struct SettingsFile {
    version: u32,
    #[serde(flatten)]
    settings: AppSettings,
}

pub async fn load(path: &Path) -> Result<AppSettings, SettingsError> {
    if !path.exists() {
        return Ok(AppSettings::default());
    }
    let raw = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| SettingsError::Io(e.to_string()))?;
    let file: SettingsFile =
        serde_json::from_str(&raw).map_err(|e| SettingsError::Corrupt(e.to_string()))?;
    if file.version != SETTINGS_VERSION {
        return Err(SettingsError::Corrupt(format!(
            "unsupported settings version {}",
            file.version
        )));
    }
    Ok(file.settings)
}

pub async fn save(path: &Path, settings: &AppSettings) -> Result<(), SettingsError> {
    let file = SettingsFile {
        version: SETTINGS_VERSION,
        settings: settings.clone(),
    };
    let json =
        serde_json::to_string_pretty(&file).map_err(|e| SettingsError::Corrupt(e.to_string()))?;
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| SettingsError::Io(e.to_string()))?;
    }
    tokio::fs::write(path, json)
        .await
        .map_err(|e| SettingsError::Io(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use snap_herb_models::DoctorRef;
    use tempfile::tempdir;

    #[tokio::test]
    async fn missing_file_returns_defaults() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join(SETTINGS_FILE_NAME);
        let settings = load(&path).await.expect("load");
        assert_eq!(settings, AppSettings::default());
    }

    #[tokio::test]
    async fn settings_round_trip_and_never_contain_credentials() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join(SETTINGS_FILE_NAME);
        let mut settings = AppSettings::default();
        settings.doctors.push(DoctorRef::new("D01", "สมชาย"));
        save(&path, &settings).await.expect("save");
        let loaded = load(&path).await.expect("load");
        assert_eq!(loaded, settings);
        let raw = tokio::fs::read_to_string(&path).await.expect("read");
        assert!(!raw.contains("password"));
    }

    #[tokio::test]
    async fn corrupt_file_is_rejected() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join(SETTINGS_FILE_NAME);
        tokio::fs::write(&path, "{ not json").await.expect("write");
        assert!(matches!(load(&path).await, Err(SettingsError::Corrupt(_))));
    }
}
