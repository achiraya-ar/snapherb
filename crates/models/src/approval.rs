use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ApprovalSettings {
    pub work_unit: String,
    pub requester_position: String,
    pub receiver_name: String,
    pub receiver_position: String,
    pub orderer_name: String,
    pub orderer_position: String,
    pub payer_name: String,
    pub payer_position: String,
    pub supervisor_name: String,
}

impl Default for ApprovalSettings {
    fn default() -> Self {
        Self {
            work_unit: "กลุ่มงานเภสัชกรรม".to_string(),
            requester_position: "เจ้าหน้าที่".to_string(),
            receiver_name: String::new(),
            receiver_position: String::new(),
            orderer_name: "นายกิตติคุณ เขียวขำ".to_string(),
            orderer_position: "หัวหน้าหน่วยพัสดุ".to_string(),
            payer_name: "นางสาวศศิธร เสนา".to_string(),
            payer_position: "เจ้าพนักงานเภสัชกรรมชำนาญงาน".to_string(),
            supervisor_name: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_fill_the_official_form_people() {
        let approvals = ApprovalSettings::default();
        assert_eq!(approvals.work_unit, "กลุ่มงานเภสัชกรรม");
        assert_eq!(approvals.orderer_position, "หัวหน้าหน่วยพัสดุ");
        assert_eq!(approvals.payer_position, "เจ้าพนักงานเภสัชกรรมชำนาญงาน");
    }

    #[test]
    fn settings_round_trip_and_missing_fields_default() {
        let approvals = ApprovalSettings {
            receiver_name: "ผู้รับทดสอบ".into(),
            ..ApprovalSettings::default()
        };
        let json = serde_json::to_string(&approvals).expect("serialize");
        let back: ApprovalSettings = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, approvals);

        let partial: ApprovalSettings =
            serde_json::from_str(r#"{"workUnit":"งานทดสอบ"}"#).expect("partial");
        assert_eq!(partial.work_unit, "งานทดสอบ");
        assert_eq!(
            partial.orderer_position,
            ApprovalSettings::default().orderer_position
        );
    }
}
