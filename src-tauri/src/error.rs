use serde::Serialize;
use snap_herb_core::RepositoryError;
use snap_herb_hosxp_connector::Error as ConnectorError;
use snap_herb_hosxp_connector::config::ConfigError;

use crate::requisitions::RequisitionsError;
use crate::settings::SettingsError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CommandErrorKind {
    NotConfigured,
    Connection,
    Guard,
    Query,
    Validation,
    Auth,
    Storage,
    Locked,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub kind: CommandErrorKind,
    pub message: String,
}

impl CommandError {
    pub fn new(kind: CommandErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn not_configured() -> Self {
        Self::new(CommandErrorKind::NotConfigured, "ยังไม่ได้ตั้งค่าการเชื่อมต่อ HOSxP")
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(
            CommandErrorKind::Validation,
            format!("ข้อมูลไม่ถูกต้อง: {}", message.into()),
        )
    }

    pub fn unauthorized() -> Self {
        Self::new(CommandErrorKind::Auth, "กรุณาเข้าสู่ระบบก่อนใช้งาน")
    }

    pub fn auth(message: impl Into<String>) -> Self {
        Self::new(CommandErrorKind::Auth, message)
    }

    pub fn locked(message: impl Into<String>) -> Self {
        Self::new(CommandErrorKind::Locked, message)
    }

    pub fn storage(message: impl Into<String>) -> Self {
        dev_log(&format!("storage failure: {}", message.into()));
        Self::new(CommandErrorKind::Storage, "บันทึกหรืออ่านข้อมูลไม่สำเร็จ")
    }
}

impl From<RepositoryError> for CommandError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::NotConfigured => Self::not_configured(),
            RepositoryError::Connection => Self::new(
                CommandErrorKind::Connection,
                "เชื่อมต่อฐานข้อมูล HOSxP ไม่สำเร็จ กรุณาตรวจสอบการตั้งค่าการเชื่อมต่อและเครือข่าย",
            ),
            RepositoryError::Query(detail) => {
                dev_log(&format!("query failed: {detail}"));
                Self::new(
                    CommandErrorKind::Query,
                    "ดึงข้อมูลจาก HOSxP ไม่สำเร็จ กรุณาลองใหม่อีกครั้ง",
                )
            }
            RepositoryError::Guard => {
                dev_log("read-only guard rejected a statement");
                Self::new(
                    CommandErrorKind::Guard,
                    "คำสั่งถูกปฏิเสธโดยระบบป้องกันการเขียนข้อมูล",
                )
            }
        }
    }
}

impl From<ConnectorError> for CommandError {
    fn from(err: ConnectorError) -> Self {
        match err {
            ConnectorError::Config(inner) => inner.into(),
            ConnectorError::Connect(inner) => {
                dev_log(&format!("connect failed: {inner}"));
                Self::new(
                    CommandErrorKind::Connection,
                    "เชื่อมต่อฐานข้อมูล HOSxP ไม่สำเร็จ กรุณาตรวจสอบการตั้งค่าการเชื่อมต่อและเครือข่าย",
                )
            }
            ConnectorError::Database(inner) => {
                dev_log(&format!("database error: {inner}"));
                Self::new(
                    CommandErrorKind::Query,
                    "ดึงข้อมูลจาก HOSxP ไม่สำเร็จ กรุณาลองใหม่อีกครั้ง",
                )
            }
            ConnectorError::Guard => {
                dev_log("read-only guard rejected a statement");
                Self::new(
                    CommandErrorKind::Guard,
                    "คำสั่งถูกปฏิเสธโดยระบบป้องกันการเขียนข้อมูล",
                )
            }
        }
    }
}

impl From<ConfigError> for CommandError {
    fn from(err: ConfigError) -> Self {
        Self::storage(err.to_string())
    }
}

impl From<SettingsError> for CommandError {
    fn from(err: SettingsError) -> Self {
        Self::storage(err.to_string())
    }
}

impl From<RequisitionsError> for CommandError {
    fn from(err: RequisitionsError) -> Self {
        Self::storage(err.to_string())
    }
}

pub(crate) fn dev_log(message: &str) {
    if cfg!(debug_assertions) {
        eprintln!("[snapherb] {message}");
    } else {
        let _ = message;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repository_errors_map_to_thai_messages() {
        let mapped = CommandError::from(RepositoryError::Connection);
        assert_eq!(mapped.kind, CommandErrorKind::Connection);
        assert!(mapped.message.contains("เชื่อมต่อ"));
        let mapped = CommandError::from(RepositoryError::Query("boom".into()));
        assert_eq!(mapped.kind, CommandErrorKind::Query);
        assert!(mapped.message.contains("ดึงข้อมูล"));
    }

    #[test]
    fn locked_errors_have_their_own_kind() {
        let mapped = CommandError::locked("ช่วงวันที่ถูกล็อก");
        assert_eq!(mapped.kind, CommandErrorKind::Locked);
    }
}
