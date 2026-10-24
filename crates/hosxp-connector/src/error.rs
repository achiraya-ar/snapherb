#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("connection settings error: {0}")]
    Config(#[from] crate::config::ConfigError),
    #[error("failed to connect to the HOSxP database: {0}")]
    Connect(sqlx::Error),
    #[error("database query failed: {0}")]
    Database(#[from] sqlx::Error),
    #[error("read-only guard rejected a statement")]
    Guard,
}

impl From<Error> for snap_herb_core::RepositoryError {
    fn from(err: Error) -> Self {
        match err {
            Error::Config(inner) => Self::Query(inner.to_string()),
            Error::Connect(_) => Self::Connection,
            Error::Database(inner) => Self::Query(inner.to_string()),
            Error::Guard => Self::Guard,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConfigError;
    use snap_herb_core::RepositoryError;

    #[test]
    fn maps_connect_to_connection() {
        assert_eq!(
            RepositoryError::from(Error::Connect(sqlx::Error::PoolTimedOut)),
            RepositoryError::Connection
        );
    }

    #[test]
    fn maps_database_to_query_preserving_detail() {
        let mapped = RepositoryError::from(Error::Database(sqlx::Error::RowNotFound));
        match mapped {
            RepositoryError::Query(detail) => assert!(detail.contains("no rows returned")),
            other => panic!("expected Query variant, got {other:?}"),
        }
    }

    #[test]
    fn maps_guard_and_config_to_typed_variants() {
        assert_eq!(RepositoryError::from(Error::Guard), RepositoryError::Guard);
        let mapped =
            RepositoryError::from(Error::Config(ConfigError::Keyring("unavailable".into())));
        assert!(matches!(mapped, RepositoryError::Query(_)));
    }
}
