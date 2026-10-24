use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq)]
pub struct OfficerAccount {
    pub officer_id: i64,
    pub login_name: String,
    pub password_md5: String,
    pub active: String,
    pub auth_type_id: i64,
    pub doctor_code: Option<String>,
    pub display_name: String,
}

impl fmt::Debug for OfficerAccount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OfficerAccount")
            .field("officer_id", &self.officer_id)
            .field("login_name", &self.login_name)
            .field("password_md5", &"<redacted>")
            .field("active", &self.active)
            .field("auth_type_id", &self.auth_type_id)
            .field("doctor_code", &self.doctor_code)
            .field("display_name", &self.display_name)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionUser {
    pub officer_id: i64,
    pub login_name: String,
    pub display_name: String,
    pub doctor_code: Option<String>,
    pub role: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionSummary {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn officer_debug_redacts_the_password_hash() {
        let account = OfficerAccount {
            officer_id: 1,
            login_name: "pharm".into(),
            password_md5: "5f4dcc3b5aa765d61d8327deb882cf99".into(),
            active: "Y".into(),
            auth_type_id: 1,
            doctor_code: None,
            display_name: "Pharmacist".into(),
        };
        let debug = format!("{account:?}");
        assert!(debug.contains("<redacted>"));
        assert!(!debug.contains("5f4dcc3b5aa765d61d8327deb882cf99"));
    }
}
