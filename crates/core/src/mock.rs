use std::collections::HashMap;

use async_trait::async_trait;
use snap_herb_models::{DoctorRef, DrugItem, OfficerAccount, RequisitionPeriod, UsageRow};

use crate::repository::{HosxRepository, RepositoryError};

#[derive(Debug, Clone, Default)]
pub struct MockRepository {
    pub drugs: Vec<DrugItem>,
    pub doctors: Vec<DoctorRef>,
    pub usage: HashMap<String, f64>,
    pub officers: HashMap<String, OfficerAccount>,
    pub fail: Option<RepositoryError>,
}

impl MockRepository {
    fn check_fail(&self) -> Result<(), RepositoryError> {
        match &self.fail {
            Some(error) => Err(error.clone()),
            None => Ok(()),
        }
    }
}

#[async_trait]
impl HosxRepository for MockRepository {
    async fn ping(&self) -> Result<(), RepositoryError> {
        self.check_fail()
    }

    async fn fetch_officer(
        &self,
        login_name: &str,
    ) -> Result<Option<OfficerAccount>, RepositoryError> {
        self.check_fail()?;
        Ok(self.officers.get(login_name).cloned())
    }

    async fn search_drugs(&self, term: &str) -> Result<Vec<DrugItem>, RepositoryError> {
        self.check_fail()?;
        Ok(self
            .drugs
            .iter()
            .filter(|drug| drug.name.contains(term) || drug.icode.contains(term))
            .cloned()
            .collect())
    }

    async fn search_doctors(&self, term: &str) -> Result<Vec<DoctorRef>, RepositoryError> {
        self.check_fail()?;
        Ok(self
            .doctors
            .iter()
            .filter(|doctor| doctor.name.contains(term) || doctor.code.contains(term))
            .cloned()
            .collect())
    }

    async fn fetch_usage(
        &self,
        icodes: &[String],
        _doctor_codes: &[String],
        _period: &RequisitionPeriod,
    ) -> Result<Vec<UsageRow>, RepositoryError> {
        self.check_fail()?;
        Ok(icodes
            .iter()
            .filter_map(|icode| {
                self.usage.get(icode).map(|qty| UsageRow {
                    icode: icode.clone(),
                    qty: *qty,
                })
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn period() -> RequisitionPeriod {
        RequisitionPeriod::new(
            chrono::NaiveDate::from_ymd_opt(2026, 10, 1).expect("date"),
            chrono::NaiveDate::from_ymd_opt(2026, 10, 31).expect("date"),
        )
        .expect("period")
    }

    #[test]
    fn mock_matches_drugs_doctors_and_usage() {
        let mut usage = HashMap::new();
        usage.insert("1001".to_string(), 130.0);
        let repo = MockRepository {
            drugs: vec![DrugItem {
                icode: "1001".into(),
                name: "ขมิ้นชัน".into(),
                strength: "500".into(),
                units: "mg".into(),
            }],
            doctors: vec![DoctorRef::new("D01", "สมชาย")],
            usage,
            officers: HashMap::new(),
            fail: None,
        };
        let runtime = tokio::runtime::Runtime::new().expect("runtime");
        runtime.block_on(async {
            assert_eq!(repo.search_drugs("ขมิ้น").await.expect("drugs").len(), 1);
            assert_eq!(repo.search_doctors("D01").await.expect("doctors").len(), 1);
            let rows = repo
                .fetch_usage(&["1001".to_string()], &["D01".to_string()], &period())
                .await
                .expect("usage");
            assert_eq!(rows[0].qty, 130.0);
        });
    }
}
