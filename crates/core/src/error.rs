#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RequisitionError {
    #[error("drug {0} is not configured in the catalog")]
    DrugNotConfigured(String),
    #[error("drug {0} is already in the requisition")]
    DrugAlreadyInReport(String),
    #[error("drug {0} was not found in the requisition")]
    RowNotFound(String),
    #[error("a reason is required for every manual change")]
    MissingReason,
    #[error("the requested quantity must be a non-negative whole number")]
    InvalidRequested,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CatalogOpsError {
    #[error("name must not be empty")]
    EmptyName,
    #[error("code must not be empty")]
    EmptyCode,
    #[error("unit names must not be empty")]
    EmptyUnit,
    #[error("units per pack must be greater than zero")]
    InvalidUnitsPerPack,
    #[error("maximum packs must be a non-negative whole number")]
    InvalidMaxPacks,
    #[error("drug {0} is already in the catalog")]
    DuplicateDrug(String),
    #[error("doctor {0} is already selected")]
    DuplicateDoctor(String),
    #[error("drug {0} was not found in the catalog")]
    DrugNotFound(String),
    #[error("doctor {0} was not found in the selection")]
    DoctorNotFound(String),
}
