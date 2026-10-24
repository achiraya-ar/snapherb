use async_trait::async_trait;
use snap_herb_models::{DoctorRef, DrugItem, OfficerAccount, RequisitionPeriod, UsageRow};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RepositoryError {
    #[error("HOSxP connection is not configured")]
    NotConfigured,
    #[error("cannot reach the HOSxP database")]
    Connection,
    #[error("query failed: {0}")]
    Query(String),
    #[error("read-only guard rejected a statement")]
    Guard,
}

#[async_trait]
pub trait HosxRepository: Send + Sync {
    async fn ping(&self) -> Result<(), RepositoryError>;

    async fn fetch_officer(
        &self,
        login_name: &str,
    ) -> Result<Option<OfficerAccount>, RepositoryError>;

    async fn search_drugs(&self, term: &str) -> Result<Vec<DrugItem>, RepositoryError>;

    async fn search_doctors(&self, term: &str) -> Result<Vec<DoctorRef>, RepositoryError>;

    async fn fetch_usage(
        &self,
        icodes: &[String],
        doctor_codes: &[String],
        period: &RequisitionPeriod,
    ) -> Result<Vec<UsageRow>, RepositoryError>;
}
