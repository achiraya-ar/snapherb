use std::collections::HashMap;

use snap_herb_models::{HerbCatalog, HerbDrug, ReportRow, UsageRow};

use crate::error::RequisitionError;

pub fn requested_for(usage: f64, units_per_pack: f64, max_packs: f64) -> f64 {
    if !usage.is_finite() || usage <= 0.0 || !units_per_pack.is_finite() || units_per_pack <= 0.0 {
        return 0.0;
    }
    let packs = (usage / units_per_pack).round();
    let ceiling = if max_packs.is_finite() {
        max_packs.floor().max(0.0)
    } else {
        0.0
    };
    packs.min(ceiling).max(0.0)
}

pub fn build_report(catalog: &HerbCatalog, usage: &[UsageRow]) -> Vec<ReportRow> {
    let usage_by_icode: HashMap<&str, f64> = usage
        .iter()
        .map(|row| (row.icode.as_str(), row.qty))
        .collect();

    catalog
        .drugs_in_order()
        .into_iter()
        .filter_map(|drug| {
            let used = usage_by_icode
                .get(drug.icode.as_str())
                .copied()
                .unwrap_or(0.0);
            if used <= 0.0 {
                return None;
            }
            let requested = requested_for(used, drug.units_per_pack, drug.max_packs);
            if requested <= 0.0 {
                return None;
            }
            Some(row_from_drug(drug, used, requested))
        })
        .collect()
}

pub fn add_configured_drug(
    catalog: &HerbCatalog,
    rows: &mut Vec<ReportRow>,
    icode: &str,
    requested: f64,
    reason: &str,
) -> Result<(), RequisitionError> {
    let reason = reason.trim();
    if reason.is_empty() {
        return Err(RequisitionError::MissingReason);
    }
    if rows.iter().any(|row| row.icode == icode) {
        return Err(RequisitionError::DrugAlreadyInReport(icode.to_string()));
    }
    let Some(drug) = catalog.find(icode) else {
        return Err(RequisitionError::DrugNotConfigured(icode.to_string()));
    };

    let requested = normalized_requested(requested)?;
    let requested = requested.min(drug.max_packs.floor().max(0.0));
    let row = ReportRow {
        icode: drug.icode.clone(),
        name: drug.name.clone(),
        strength_text: drug.strength_text.clone(),
        usage_unit: drug.usage_unit.clone(),
        pack_unit: drug.pack_unit.clone(),
        units_per_pack: drug.units_per_pack,
        max_packs: drug.max_packs,
        usage: 0.0,
        requested,
        added: true,
        reason: reason.to_string(),
    };
    insert_in_catalog_order(catalog, rows, row);
    Ok(())
}

pub fn adjust_requested(
    rows: &mut [ReportRow],
    icode: &str,
    requested: f64,
    reason: &str,
) -> Result<(), RequisitionError> {
    let reason = reason.trim();
    if reason.is_empty() {
        return Err(RequisitionError::MissingReason);
    }
    let Some(row) = rows.iter_mut().find(|row| row.icode == icode) else {
        return Err(RequisitionError::RowNotFound(icode.to_string()));
    };
    let requested = normalized_requested(requested)?;
    row.requested = requested.min(row.max_packs.floor().max(0.0));
    row.reason = reason.to_string();
    Ok(())
}

fn row_from_drug(drug: &HerbDrug, usage: f64, requested: f64) -> ReportRow {
    ReportRow {
        icode: drug.icode.clone(),
        name: drug.name.clone(),
        strength_text: drug.strength_text.clone(),
        usage_unit: drug.usage_unit.clone(),
        pack_unit: drug.pack_unit.clone(),
        units_per_pack: drug.units_per_pack,
        max_packs: drug.max_packs,
        usage,
        requested,
        added: false,
        reason: String::new(),
    }
}

fn normalized_requested(requested: f64) -> Result<f64, RequisitionError> {
    if !requested.is_finite() || requested < 0.0 {
        return Err(RequisitionError::InvalidRequested);
    }
    Ok(requested.round())
}

fn insert_in_catalog_order(catalog: &HerbCatalog, rows: &mut Vec<ReportRow>, row: ReportRow) {
    let position = catalog
        .drugs_in_order()
        .iter()
        .position(|drug| drug.icode == row.icode)
        .unwrap_or(usize::MAX);
    let order_of = |icode: &str| {
        catalog
            .drugs_in_order()
            .iter()
            .position(|drug| drug.icode == icode)
            .unwrap_or(usize::MAX)
    };
    let insert_at = rows
        .iter()
        .position(|existing| order_of(&existing.icode) > position)
        .unwrap_or(rows.len());
    rows.insert(insert_at, row);
}

#[cfg(test)]
mod tests {
    use super::*;
    use snap_herb_models::{DrugItem, HerbCatalog};

    fn catalog() -> HerbCatalog {
        let mut catalog = HerbCatalog::default();
        crate::catalog_ops::add_herb(
            &mut catalog,
            &DrugItem {
                icode: "1001".into(),
                name: "ขมิ้นชัน".into(),
                strength: "500".into(),
                units: "mg".into(),
            },
            "เม็ด",
            "กล่อง",
            100.0,
            30.0,
        )
        .expect("herb 1");
        crate::catalog_ops::add_herb(
            &mut catalog,
            &DrugItem {
                icode: "1002".into(),
                name: "ฟ้าทะลายโจร".into(),
                strength: "500".into(),
                units: "mg".into(),
            },
            "เม็ด",
            "กล่อง",
            100.0,
            30.0,
        )
        .expect("herb 2");
        catalog
    }

    fn usage(icode: &str, qty: f64) -> UsageRow {
        UsageRow {
            icode: icode.into(),
            qty,
        }
    }

    #[test]
    fn pack_rounding_uses_half_up() {
        assert_eq!(requested_for(500.0, 100.0, 30.0), 5.0);
        assert_eq!(requested_for(130.0, 100.0, 30.0), 1.0);
        assert_eq!(requested_for(149.0, 100.0, 30.0), 1.0);
        assert_eq!(requested_for(150.0, 100.0, 30.0), 2.0);
        assert_eq!(requested_for(50.0, 100.0, 30.0), 1.0);
    }

    #[test]
    fn pack_rounding_caps_at_maximum() {
        assert_eq!(requested_for(5000.0, 100.0, 30.0), 30.0);
        assert_eq!(requested_for(5000.0, 100.0, 0.0), 0.0);
        assert_eq!(requested_for(0.0, 100.0, 30.0), 0.0);
    }

    #[test]
    fn build_report_only_includes_used_drugs_in_catalog_order() {
        let catalog = catalog();
        let rows = build_report(&catalog, &[usage("1002", 150.0), usage("1001", 130.0)]);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].icode, "1001");
        assert_eq!(rows[0].requested, 1.0);
        assert_eq!(rows[1].icode, "1002");
        assert_eq!(rows[1].requested, 2.0);
    }

    #[test]
    fn build_report_ignores_unknown_and_unused_icodes() {
        let rows = build_report(&catalog(), &[usage("9999", 500.0)]);
        assert!(rows.is_empty());
    }

    #[test]
    fn build_report_hides_used_drugs_whose_requested_packs_round_to_zero() {
        let rows = build_report(&catalog(), &[usage("1001", 20.0)]);
        assert!(rows.is_empty());
    }

    #[test]
    fn add_drug_requires_reason_and_inserts_in_order() {
        let catalog = catalog();
        let mut rows = build_report(&catalog, &[usage("1002", 150.0)]);
        assert_eq!(
            add_configured_drug(&catalog, &mut rows, "1001", 2.0, " "),
            Err(RequisitionError::MissingReason)
        );
        add_configured_drug(&catalog, &mut rows, "1001", 2.0, "สำรอง").expect("added");
        assert_eq!(rows[0].icode, "1001");
        assert_eq!(rows[0].usage, 0.0);
        assert!(rows[0].added);
        assert_eq!(
            add_configured_drug(&catalog, &mut rows, "1001", 2.0, "ซ้ำ"),
            Err(RequisitionError::DrugAlreadyInReport("1001".into()))
        );
    }

    #[test]
    fn add_drug_caps_at_max_and_rounds_to_whole_packs() {
        let catalog = catalog();
        let mut rows = Vec::new();
        add_configured_drug(&catalog, &mut rows, "1001", 99.0, "สำรอง").expect("added");
        assert_eq!(rows[0].requested, 30.0);
        let mut rows = Vec::new();
        add_configured_drug(&catalog, &mut rows, "1001", 2.4, "สำรอง").expect("added");
        assert_eq!(rows[0].requested, 2.0);
    }

    #[test]
    fn adjust_requested_records_reason_and_caps() {
        let catalog = catalog();
        let mut rows = build_report(&catalog, &[usage("1001", 130.0)]);
        adjust_requested(&mut rows, "1001", 3.0, "ผู้ป่วยใช้มากกว่าปกติ").expect("adjusted");
        assert_eq!(rows[0].requested, 3.0);
        assert_eq!(rows[0].reason, "ผู้ป่วยใช้มากกว่าปกติ");
        adjust_requested(&mut rows, "1001", 100.0, "ปรับตามสต็อก").expect("adjusted");
        assert_eq!(rows[0].requested, 30.0);
        assert_eq!(
            adjust_requested(&mut rows, "9999", 1.0, "เหตุผล"),
            Err(RequisitionError::RowNotFound("9999".into()))
        );
        assert_eq!(
            adjust_requested(&mut rows, "1001", 1.0, " "),
            Err(RequisitionError::MissingReason)
        );
    }
}
