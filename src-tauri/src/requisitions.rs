use std::path::Path;

use chrono::{Datelike, NaiveDateTime};
use serde::{Deserialize, Serialize};
use snap_herb_models::{
    ApprovalSettings, ReportMeta, ReportRow, RequisitionPeriod, RequisitionRecord, be_year,
};

pub const REQUISITIONS_FILE_NAME: &str = "snapherb.requisitions.json";

const REQUISITIONS_VERSION: u32 = 1;

#[derive(Debug, thiserror::Error)]
pub enum RequisitionsError {
    #[error("requisition store I/O failed: {0}")]
    Io(String),
    #[error("requisition store is corrupt: {0}")]
    Corrupt(String),
}

#[derive(Debug, Serialize, Deserialize)]
struct RequisitionFile {
    version: u32,
    #[serde(default)]
    records: Vec<RequisitionRecord>,
}

pub async fn load(path: &Path) -> Result<Vec<RequisitionRecord>, RequisitionsError> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| RequisitionsError::Io(e.to_string()))?;
    let file: RequisitionFile =
        serde_json::from_str(&raw).map_err(|e| RequisitionsError::Corrupt(e.to_string()))?;
    if file.version != REQUISITIONS_VERSION {
        return Err(RequisitionsError::Corrupt(format!(
            "unsupported requisition store version {}",
            file.version
        )));
    }
    Ok(file.records)
}

pub async fn save(path: &Path, records: &[RequisitionRecord]) -> Result<(), RequisitionsError> {
    let file = RequisitionFile {
        version: REQUISITIONS_VERSION,
        records: records.to_vec(),
    };
    let json = serde_json::to_string_pretty(&file)
        .map_err(|e| RequisitionsError::Corrupt(e.to_string()))?;
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| RequisitionsError::Io(e.to_string()))?;
    }
    tokio::fs::write(path, json)
        .await
        .map_err(|e| RequisitionsError::Io(e.to_string()))
}

pub fn build_record(
    existing: &[RequisitionRecord],
    period: RequisitionPeriod,
    rows: Vec<ReportRow>,
    report: ReportMeta,
    approvals: ApprovalSettings,
    requested_by: String,
    printed_at: NaiveDateTime,
) -> RequisitionRecord {
    let date = printed_at.date();
    let prefix = format!(
        "SH-{:04}{:02}{:02}-",
        be_year(date.year()),
        date.month(),
        date.day()
    );
    let mut sequence = existing
        .iter()
        .filter(|record| record.document_no.starts_with(&prefix))
        .count()
        + 1;
    let mut document_no = format!("{prefix}{sequence:03}");
    while existing
        .iter()
        .any(|record| record.document_no == document_no)
    {
        sequence += 1;
        document_no = format!("{prefix}{sequence:03}");
    }
    let id = format!(
        "req-{}-{sequence:03}",
        printed_at.and_utc().timestamp_millis()
    );
    RequisitionRecord {
        id,
        document_no,
        period,
        rows,
        report,
        approvals,
        requested_by,
        printed_at,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snap_herb_models::ReportMeta;
    use tempfile::tempdir;

    fn date(y: i32, m: u32, d: u32) -> chrono::NaiveDate {
        chrono::NaiveDate::from_ymd_opt(y, m, d).expect("date")
    }

    fn period() -> RequisitionPeriod {
        RequisitionPeriod::new(date(2026, 10, 1), date(2026, 10, 31)).expect("period")
    }

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

    fn datetime() -> NaiveDateTime {
        date(2026, 10, 31).and_hms_opt(9, 0, 0).expect("datetime")
    }

    #[test]
    fn build_record_numbers_documents_per_print_day() {
        let first = build_record(
            &[],
            period(),
            vec![row()],
            ReportMeta::default(),
            ApprovalSettings::default(),
            "Pharmacist".into(),
            datetime(),
        );
        assert_eq!(first.document_no, "SH-25691031-001");
        let second = build_record(
            std::slice::from_ref(&first),
            period(),
            vec![row()],
            ReportMeta::default(),
            ApprovalSettings::default(),
            "Pharmacist".into(),
            datetime(),
        );
        assert_eq!(second.document_no, "SH-25691031-002");
    }

    #[tokio::test]
    async fn store_round_trips_and_missing_file_is_empty() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join(REQUISITIONS_FILE_NAME);
        assert!(load(&path).await.expect("empty").is_empty());
        let record = build_record(
            &[],
            period(),
            vec![row()],
            ReportMeta::default(),
            ApprovalSettings::default(),
            "Pharmacist".into(),
            datetime(),
        );
        save(&path, std::slice::from_ref(&record))
            .await
            .expect("save");
        let loaded = load(&path).await.expect("load");
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].document_no, record.document_no);
    }
}
