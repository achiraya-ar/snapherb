use snap_herb_models::{DoctorRef, DrugItem, HerbCatalog, HerbDrug};

use crate::error::CatalogOpsError;

pub fn add_doctor(
    doctors: &mut Vec<DoctorRef>,
    code: &str,
    name: &str,
) -> Result<(), CatalogOpsError> {
    let code = code.trim();
    let name = name.trim();
    if code.is_empty() {
        return Err(CatalogOpsError::EmptyCode);
    }
    if name.is_empty() {
        return Err(CatalogOpsError::EmptyName);
    }
    if doctors.iter().any(|doctor| doctor.code == code) {
        return Err(CatalogOpsError::DuplicateDoctor(code.to_string()));
    }
    doctors.push(DoctorRef::new(code, name));
    Ok(())
}

pub fn remove_doctor(doctors: &mut Vec<DoctorRef>, code: &str) -> Result<(), CatalogOpsError> {
    let before = doctors.len();
    doctors.retain(|doctor| doctor.code != code);
    if doctors.len() == before {
        return Err(CatalogOpsError::DoctorNotFound(code.to_string()));
    }
    Ok(())
}

pub fn add_herb(
    catalog: &mut HerbCatalog,
    item: &DrugItem,
    usage_unit: &str,
    pack_unit: &str,
    units_per_pack: f64,
    max_packs: f64,
) -> Result<(), CatalogOpsError> {
    let icode = item.icode.trim();
    let name = item.name.trim();
    if icode.is_empty() {
        return Err(CatalogOpsError::EmptyCode);
    }
    if name.is_empty() {
        return Err(CatalogOpsError::EmptyName);
    }
    if catalog.drugs.iter().any(|drug| drug.icode == icode) {
        return Err(CatalogOpsError::DuplicateDrug(icode.to_string()));
    }
    let usage_unit = usage_unit.trim();
    let pack_unit = pack_unit.trim();
    if usage_unit.is_empty() || pack_unit.is_empty() {
        return Err(CatalogOpsError::EmptyUnit);
    }
    validate_numbers(units_per_pack, max_packs)?;
    let sort_order = catalog
        .drugs
        .iter()
        .map(|drug| drug.sort_order)
        .max()
        .map_or(0, |max| max + 1);
    catalog.drugs.push(HerbDrug {
        icode: icode.to_string(),
        name: name.to_string(),
        strength_text: item.strength_label(),
        usage_unit: usage_unit.to_string(),
        pack_unit: pack_unit.to_string(),
        units_per_pack,
        max_packs,
        sort_order,
    });
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn update_herb(
    catalog: &mut HerbCatalog,
    icode: &str,
    name: &str,
    strength_text: &str,
    usage_unit: &str,
    pack_unit: &str,
    units_per_pack: f64,
    max_packs: f64,
) -> Result<(), CatalogOpsError> {
    let name = name.trim();
    let usage_unit = usage_unit.trim();
    let pack_unit = pack_unit.trim();
    if name.is_empty() {
        return Err(CatalogOpsError::EmptyName);
    }
    if usage_unit.is_empty() || pack_unit.is_empty() {
        return Err(CatalogOpsError::EmptyUnit);
    }
    validate_numbers(units_per_pack, max_packs)?;
    let drug = catalog
        .drugs
        .iter_mut()
        .find(|drug| drug.icode == icode)
        .ok_or_else(|| CatalogOpsError::DrugNotFound(icode.to_string()))?;
    drug.name = name.to_string();
    drug.strength_text = strength_text.trim().to_string();
    drug.usage_unit = usage_unit.to_string();
    drug.pack_unit = pack_unit.to_string();
    drug.units_per_pack = units_per_pack;
    drug.max_packs = max_packs;
    Ok(())
}

pub fn remove_herb(catalog: &mut HerbCatalog, icode: &str) -> Result<(), CatalogOpsError> {
    let before = catalog.drugs.len();
    catalog.drugs.retain(|drug| drug.icode != icode);
    if catalog.drugs.len() == before {
        return Err(CatalogOpsError::DrugNotFound(icode.to_string()));
    }
    Ok(())
}

pub fn move_herb(
    catalog: &mut HerbCatalog,
    icode: &str,
    delta: i32,
) -> Result<(), CatalogOpsError> {
    let mut order: Vec<usize> = (0..catalog.drugs.len()).collect();
    order.sort_by_key(|&index| catalog.drugs[index].sort_order);
    let position = order
        .iter()
        .position(|&index| catalog.drugs[index].icode == icode)
        .ok_or_else(|| CatalogOpsError::DrugNotFound(icode.to_string()))?;
    let target = position as i64 + i64::from(delta);
    if target < 0 || target >= order.len() as i64 {
        return Ok(());
    }
    let (current, neighbour) = (order[position], order[target as usize]);
    let current_order = catalog.drugs[current].sort_order;
    catalog.drugs[current].sort_order = catalog.drugs[neighbour].sort_order;
    catalog.drugs[neighbour].sort_order = current_order;
    Ok(())
}

fn validate_numbers(units_per_pack: f64, max_packs: f64) -> Result<(), CatalogOpsError> {
    if !units_per_pack.is_finite() || units_per_pack <= 0.0 {
        return Err(CatalogOpsError::InvalidUnitsPerPack);
    }
    if !max_packs.is_finite() || max_packs < 0.0 || max_packs.fract().abs() > 1e-9 {
        return Err(CatalogOpsError::InvalidMaxPacks);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(icode: &str, name: &str) -> DrugItem {
        DrugItem {
            icode: icode.into(),
            name: name.into(),
            strength: "500".into(),
            units: "mg".into(),
        }
    }

    #[test]
    fn doctors_are_unique_and_removable() {
        let mut doctors = Vec::new();
        add_doctor(&mut doctors, "D01", "สมชาย").expect("doctor added");
        assert_eq!(
            add_doctor(&mut doctors, "D01", "ซ้ำ"),
            Err(CatalogOpsError::DuplicateDoctor("D01".into()))
        );
        remove_doctor(&mut doctors, "D01").expect("doctor removed");
        assert!(doctors.is_empty());
    }

    #[test]
    fn add_herb_seeds_pack_fields_and_order() {
        let mut catalog = HerbCatalog::default();
        add_herb(
            &mut catalog,
            &item("1001", "ขมิ้นชัน"),
            "เม็ด",
            "กล่อง",
            100.0,
            30.0,
        )
        .expect("herb added");
        assert_eq!(catalog.drugs[0].strength_text, "500 mg");
        assert_eq!(catalog.drugs[0].pack_label(), "1 กล่อง = 100 เม็ด");
        assert_eq!(catalog.drugs[0].sort_order, 0);
    }

    #[test]
    fn add_herb_rejects_duplicates_and_invalid_pack_numbers() {
        let mut catalog = HerbCatalog::default();
        add_herb(
            &mut catalog,
            &item("1001", "ขมิ้นชัน"),
            "เม็ด",
            "กล่อง",
            100.0,
            30.0,
        )
        .expect("herb added");
        assert_eq!(
            add_herb(
                &mut catalog,
                &item("1001", "ซ้ำ"),
                "เม็ด",
                "กล่อง",
                100.0,
                30.0
            ),
            Err(CatalogOpsError::DuplicateDrug("1001".into()))
        );
        assert_eq!(
            add_herb(
                &mut catalog,
                &item("1002", "ฟ้าทะลายโจร"),
                "เม็ด",
                "กล่อง",
                0.0,
                30.0
            ),
            Err(CatalogOpsError::InvalidUnitsPerPack)
        );
    }

    #[test]
    fn update_herb_edits_all_fields() {
        let mut catalog = HerbCatalog::default();
        add_herb(
            &mut catalog,
            &item("1001", "ขมิ้นชัน"),
            "เม็ด",
            "กล่อง",
            100.0,
            30.0,
        )
        .expect("herb added");
        update_herb(
            &mut catalog,
            "1001",
            "ขมิ้นชันแคปซูล",
            "500 mg",
            "แคปซูล",
            "ขวด",
            50.0,
            10.0,
        )
        .expect("herb updated");
        let drug = &catalog.drugs[0];
        assert_eq!(drug.name, "ขมิ้นชันแคปซูล");
        assert_eq!(drug.usage_unit, "แคปซูล");
        assert_eq!(drug.units_per_pack, 50.0);
        assert_eq!(drug.max_packs, 10.0);
    }

    #[test]
    fn move_herb_swaps_neighbours() {
        let mut catalog = HerbCatalog::default();
        add_herb(&mut catalog, &item("1001", "a"), "เม็ด", "กล่อง", 10.0, 2.0).expect("a");
        add_herb(&mut catalog, &item("1002", "b"), "เม็ด", "กล่อง", 10.0, 2.0).expect("b");
        move_herb(&mut catalog, "1002", -1).expect("moved");
        assert_eq!(catalog.drugs_in_order()[0].icode, "1002");
    }
}
