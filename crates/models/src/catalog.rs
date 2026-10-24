use serde::{Deserialize, Serialize};

use crate::thai::format_qty;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrugItem {
    pub icode: String,
    pub name: String,
    #[serde(default)]
    pub strength: String,
    #[serde(default)]
    pub units: String,
}

impl DrugItem {
    pub fn strength_label(&self) -> String {
        match (self.strength.trim(), self.units.trim()) {
            ("", "") => String::new(),
            (strength, "") => strength.to_string(),
            ("", units) => units.to_string(),
            (strength, units) => format!("{strength} {units}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HerbDrug {
    pub icode: String,
    pub name: String,
    #[serde(default)]
    pub strength_text: String,
    #[serde(default = "default_usage_unit")]
    pub usage_unit: String,
    #[serde(default = "default_pack_unit")]
    pub pack_unit: String,
    #[serde(default = "default_units_per_pack")]
    pub units_per_pack: f64,
    #[serde(default = "default_max_packs")]
    pub max_packs: f64,
    #[serde(default)]
    pub sort_order: i32,
}

fn default_usage_unit() -> String {
    "หน่วย".to_string()
}

fn default_pack_unit() -> String {
    "กล่อง".to_string()
}

fn default_units_per_pack() -> f64 {
    1.0
}

fn default_max_packs() -> f64 {
    1.0
}

impl HerbDrug {
    pub fn pack_label(&self) -> String {
        format!(
            "1 {} = {} {}",
            self.pack_unit,
            format_qty(self.units_per_pack),
            self.usage_unit
        )
    }

    pub fn max_label(&self) -> String {
        format!("{} {}", format_qty(self.max_packs), self.pack_unit)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct HerbCatalog {
    pub drugs: Vec<HerbDrug>,
}

impl HerbCatalog {
    pub fn is_empty(&self) -> bool {
        self.drugs.is_empty()
    }

    pub fn icodes(&self) -> Vec<String> {
        self.drugs.iter().map(|drug| drug.icode.clone()).collect()
    }

    pub fn drugs_in_order(&self) -> Vec<&HerbDrug> {
        let mut drugs: Vec<&HerbDrug> = self.drugs.iter().collect();
        drugs.sort_by_key(|drug| drug.sort_order);
        drugs
    }

    pub fn find(&self, icode: &str) -> Option<&HerbDrug> {
        self.drugs.iter().find(|drug| drug.icode == icode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn herb() -> HerbDrug {
        HerbDrug {
            icode: "1001".into(),
            name: "ขมิ้นชัน".into(),
            strength_text: "500 mg".into(),
            usage_unit: "เม็ด".into(),
            pack_unit: "กล่อง".into(),
            units_per_pack: 100.0,
            max_packs: 30.0,
            sort_order: 0,
        }
    }

    #[test]
    fn drug_item_strength_label_combines_values() {
        let item = DrugItem {
            icode: "1001".into(),
            name: "ขมิ้นชัน".into(),
            strength: "500".into(),
            units: "mg".into(),
        };
        assert_eq!(item.strength_label(), "500 mg");
    }

    #[test]
    fn herb_labels_describe_the_pack_and_maximum() {
        let herb = herb();
        assert_eq!(herb.pack_label(), "1 กล่อง = 100 เม็ด");
        assert_eq!(herb.max_label(), "30 กล่อง");
    }

    #[test]
    fn catalog_orders_and_finds_drugs() {
        let mut catalog = HerbCatalog::default();
        let mut first = herb();
        first.sort_order = 1;
        let mut second = herb();
        second.icode = "1002".into();
        second.sort_order = 0;
        catalog.drugs = vec![first, second];
        assert_eq!(catalog.drugs_in_order()[0].icode, "1002");
        assert!(catalog.find("1001").is_some());
        assert_eq!(catalog.icodes().len(), 2);
    }
}
