use serde::{Deserialize, Serialize};

use crate::approval::ApprovalSettings;
use crate::catalog::HerbCatalog;
use crate::doctor::DoctorRef;
use crate::report::ReportMeta;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct AppSettings {
    pub doctors: Vec<DoctorRef>,
    pub catalog: HerbCatalog,
    pub report: ReportMeta,
    pub approvals: ApprovalSettings,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_default_is_empty_and_print_ready() {
        let settings = AppSettings::default();
        assert!(settings.doctors.is_empty());
        assert!(settings.catalog.is_empty());
        assert_eq!(settings.report, ReportMeta::default());
        assert_eq!(settings.approvals, ApprovalSettings::default());
    }

    #[test]
    fn settings_round_trip() {
        let mut settings = AppSettings::default();
        settings.doctors.push(DoctorRef::new("D01", "สมชาย"));
        let json = serde_json::to_string(&settings).expect("serialize");
        let back: AppSettings = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, settings);
    }
}
