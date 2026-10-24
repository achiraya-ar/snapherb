use snap_herb_models::{RequisitionPeriod, RequisitionRecord};

pub fn existing_conflicts(
    records: &[RequisitionRecord],
    period: RequisitionPeriod,
) -> Vec<RequisitionRecord> {
    let mut conflicts: Vec<RequisitionRecord> = records
        .iter()
        .filter(|record| record.period.overlaps(&period))
        .cloned()
        .collect();
    conflicts.sort_by_key(|record| record.printed_at);
    conflicts
}

#[cfg(test)]
mod tests {
    use super::*;
    use snap_herb_models::{ReportMeta, ReportRow};

    fn date(y: i32, m: u32, d: u32) -> chrono::NaiveDate {
        chrono::NaiveDate::from_ymd_opt(y, m, d).expect("date")
    }

    fn record(id: &str, start: chrono::NaiveDate, end: chrono::NaiveDate) -> RequisitionRecord {
        RequisitionRecord {
            id: id.into(),
            document_no: format!("SH-{id}"),
            period: RequisitionPeriod::new(start, end).expect("period"),
            rows: vec![ReportRow {
                icode: "1001".into(),
                name: "ขมิ้นชัน".into(),
                strength_text: String::new(),
                usage_unit: "เม็ด".into(),
                pack_unit: "กล่อง".into(),
                units_per_pack: 100.0,
                max_packs: 30.0,
                usage: 130.0,
                requested: 1.0,
                added: false,
                reason: String::new(),
            }],
            report: ReportMeta::default(),
            approvals: snap_herb_models::ApprovalSettings::default(),
            requested_by: "Pharmacist".into(),
            printed_at: date(2026, 10, 31).and_hms_opt(9, 0, 0).expect("datetime"),
        }
    }

    #[test]
    fn existing_conflicts_require_overlap() {
        let period = RequisitionPeriod::new(date(2026, 10, 1), date(2026, 10, 31)).expect("period");
        let records = vec![
            record("a", date(2026, 9, 1), date(2026, 9, 30)),
            record("b", date(2026, 10, 31), date(2026, 11, 5)),
        ];
        let conflicts = existing_conflicts(&records, period);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].id, "b");
    }

    #[test]
    fn existing_conflicts_include_partial_overlap() {
        let period = RequisitionPeriod::new(date(2026, 10, 1), date(2026, 10, 31)).expect("period");
        let records = vec![record("a", date(2026, 10, 20), date(2026, 11, 2))];
        assert_eq!(existing_conflicts(&records, period).len(), 1);
    }
}
