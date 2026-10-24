use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::approval::ApprovalSettings;
use crate::period::RequisitionPeriod;
use crate::thai::format_qty;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageRow {
    pub icode: String,
    pub qty: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportRow {
    pub icode: String,
    pub name: String,
    #[serde(default)]
    pub strength_text: String,
    pub usage_unit: String,
    pub pack_unit: String,
    pub units_per_pack: f64,
    pub max_packs: f64,
    pub usage: f64,
    pub requested: f64,
    #[serde(default)]
    pub added: bool,
    #[serde(default)]
    pub reason: String,
}

impl ReportRow {
    pub fn has_adjustment(&self) -> bool {
        self.added || !self.reason.trim().is_empty()
    }

    pub fn pack_label(&self) -> String {
        format!(
            "1 {} = {} {}",
            self.pack_unit,
            format_qty(self.units_per_pack),
            self.usage_unit
        )
    }

    pub fn usage_label(&self) -> String {
        format!("{} {}", format_qty(self.usage), self.usage_unit)
    }

    pub fn requested_label(&self) -> String {
        format!("{} {}", format_qty(self.requested), self.pack_unit)
    }

    pub fn max_label(&self) -> String {
        format!("{} {}", format_qty(self.max_packs), self.pack_unit)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ReportMeta {
    pub hospital_name: String,
    pub document_title: String,
    pub department_name: String,
}

impl Default for ReportMeta {
    fn default() -> Self {
        Self {
            hospital_name: "โรงพยาบาลสระโบสถ์".to_string(),
            document_title: "ใบเบิกยาสมุนไพร".to_string(),
            department_name: "กลุ่มงานเภสัชกรรม".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequisitionRecord {
    pub id: String,
    pub document_no: String,
    pub period: RequisitionPeriod,
    pub rows: Vec<ReportRow>,
    pub report: ReportMeta,
    #[serde(default)]
    pub approvals: ApprovalSettings,
    pub requested_by: String,
    pub printed_at: NaiveDateTime,
}

impl RequisitionRecord {
    pub fn total_requested(&self) -> f64 {
        self.rows.iter().map(|row| row.requested).sum()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeriodContext {
    pub conflicts: Vec<RequisitionRecord>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::period::RequisitionPeriod;

    fn row() -> ReportRow {
        ReportRow {
            icode: "1001".into(),
            name: "ขมิ้นชัน".into(),
            strength_text: "500 mg".into(),
            usage_unit: "เม็ด".into(),
            pack_unit: "กล่อง".into(),
            units_per_pack: 100.0,
            max_packs: 30.0,
            usage: 130.0,
            requested: 1.0,
            added: false,
            reason: String::new(),
        }
    }

    #[test]
    fn report_row_labels_are_print_ready() {
        let row = row();
        assert_eq!(row.pack_label(), "1 กล่อง = 100 เม็ด");
        assert_eq!(row.usage_label(), "130 เม็ด");
        assert_eq!(row.requested_label(), "1 กล่อง");
    }

    #[test]
    fn adjustments_require_a_reason_or_manual_add() {
        let mut row = row();
        assert!(!row.has_adjustment());
        row.reason = "สต็อกไม่พอ".into();
        assert!(row.has_adjustment());
        row.reason = "  ".into();
        assert!(!row.has_adjustment());
        row.added = true;
        assert!(row.has_adjustment());
    }

    #[test]
    fn report_meta_defaults_to_the_herbal_form() {
        let meta = ReportMeta::default();
        assert_eq!(meta.document_title, "ใบเบิกยาสมุนไพร");
        assert_eq!(meta.department_name, "กลุ่มงานเภสัชกรรม");
    }

    #[test]
    fn record_totals_requested_packs() {
        let record = RequisitionRecord {
            id: "req-1".into(),
            document_no: "SH-25691001-001".into(),
            period: RequisitionPeriod::new(
                chrono::NaiveDate::from_ymd_opt(2026, 10, 1).expect("date"),
                chrono::NaiveDate::from_ymd_opt(2026, 10, 31).expect("date"),
            )
            .expect("period"),
            rows: vec![row()],
            report: ReportMeta::default(),
            approvals: ApprovalSettings::default(),
            requested_by: "Pharmacist".into(),
            printed_at: chrono::NaiveDate::from_ymd_opt(2026, 10, 31)
                .and_then(|date| date.and_hms_opt(9, 0, 0))
                .expect("datetime"),
        };
        assert_eq!(record.total_requested(), 1.0);
    }
}
