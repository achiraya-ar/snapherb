use std::sync::Arc;

use serde::Serialize;
use snap_herb_hosxp_connector::MySqlPool;
use snap_herb_hosxp_connector::config::CONFIG_FILE_NAME;
use snap_herb_models::{RequisitionRecord, SessionUser};
use tokio::sync::Mutex;

use crate::requisitions::REQUISITIONS_FILE_NAME;
use crate::settings::{AppSettings, SETTINGS_FILE_NAME};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ConnectionHealth {
    Unconfigured,
    Connected,
    Disconnected,
}

#[derive(Clone)]
pub struct AppState {
    pub config_dir: std::path::PathBuf,
    pub pool: Arc<Mutex<Option<MySqlPool>>>,
    pub health: Arc<Mutex<ConnectionHealth>>,
    pub settings: Arc<Mutex<AppSettings>>,
    pub session: Arc<Mutex<Option<SessionUser>>>,
    pub requisitions: Arc<Mutex<Vec<RequisitionRecord>>>,
}

impl AppState {
    pub fn new(
        config_dir: std::path::PathBuf,
        settings: AppSettings,
        requisitions: Vec<RequisitionRecord>,
    ) -> Self {
        Self {
            config_dir,
            pool: Arc::new(Mutex::new(None)),
            health: Arc::new(Mutex::new(ConnectionHealth::Unconfigured)),
            settings: Arc::new(Mutex::new(settings)),
            session: Arc::new(Mutex::new(None)),
            requisitions: Arc::new(Mutex::new(requisitions)),
        }
    }

    pub fn config_path(&self) -> std::path::PathBuf {
        self.config_dir.join(CONFIG_FILE_NAME)
    }

    pub fn settings_path(&self) -> std::path::PathBuf {
        self.config_dir.join(SETTINGS_FILE_NAME)
    }

    pub fn requisitions_path(&self) -> std::path::PathBuf {
        self.config_dir.join(REQUISITIONS_FILE_NAME)
    }
}
