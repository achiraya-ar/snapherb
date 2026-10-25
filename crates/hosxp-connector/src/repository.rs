use async_trait::async_trait;
use snap_herb_core::{HosxRepository, RepositoryError};
use snap_herb_models::{DoctorRef, DrugItem, OfficerAccount, RequisitionPeriod, UsageRow};
use sqlx::Row;
use sqlx::mysql::MySqlPool;

use crate::error::Error;
use crate::queries::{
    FETCH_OFFICER_SQL, PING_SQL, SEARCH_DOCTORS_SQL, SEARCH_DRUGS_SQL, usage_in_period_sql,
};
use crate::readonly_guard::assert_read_only;

#[derive(Debug, Clone)]
pub struct ConnectorRepository {
    pool: MySqlPool,
}

impl ConnectorRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &MySqlPool {
        &self.pool
    }

    fn guard(sql: &str) -> Result<(), RepositoryError> {
        assert_read_only(sql).map_err(|_| Error::Guard.into())
    }
}

fn escape_like(term: &str) -> String {
    let mut escaped = String::with_capacity(term.len());
    for c in term.chars() {
        if matches!(c, '\\' | '%' | '_') {
            escaped.push('\\');
        }
        escaped.push(c);
    }
    escaped
}

#[async_trait]
impl HosxRepository for ConnectorRepository {
    async fn ping(&self) -> Result<(), RepositoryError> {
        Self::guard(PING_SQL)?;
        sqlx::query(PING_SQL)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|e| Error::Database(e).into())
    }

    async fn fetch_officer(
        &self,
        login_name: &str,
    ) -> Result<Option<OfficerAccount>, RepositoryError> {
        Self::guard(FETCH_OFFICER_SQL)?;
        let row = sqlx::query(FETCH_OFFICER_SQL)
            .bind(login_name)
            .fetch_optional(&self.pool)
            .await
            .map_err(Error::Database)?;
        row.map(|row| -> Result<OfficerAccount, Error> {
            Ok(OfficerAccount {
                officer_id: row.try_get("officer_id").map_err(Error::Database)?,
                login_name: row.try_get("login_name").map_err(Error::Database)?,
                password_md5: row.try_get("password_md5").map_err(Error::Database)?,
                active: row.try_get("active").map_err(Error::Database)?,
                auth_type_id: row.try_get("auth_type_id").map_err(Error::Database)?,
                doctor_code: row.try_get("doctor_code").map_err(Error::Database)?,
                display_name: row.try_get("display_name").map_err(Error::Database)?,
            })
        })
        .transpose()
        .map_err(Into::into)
    }

    async fn search_drugs(&self, term: &str) -> Result<Vec<DrugItem>, RepositoryError> {
        Self::guard(SEARCH_DRUGS_SQL)?;
        let pattern = format!("%{}%", escape_like(term.trim()));
        let rows = sqlx::query(SEARCH_DRUGS_SQL)
            .bind(&pattern)
            .bind(&pattern)
            .fetch_all(&self.pool)
            .await
            .map_err(Error::Database)?;
        rows.into_iter()
            .map(|row| {
                Ok(DrugItem {
                    icode: row.try_get("icode").map_err(Error::Database)?,
                    name: row.try_get("name").map_err(Error::Database)?,
                    strength: row.try_get("strength").map_err(Error::Database)?,
                    units: row.try_get("units").map_err(Error::Database)?,
                })
            })
            .collect::<Result<Vec<_>, Error>>()
            .map_err(Into::into)
    }

    async fn search_doctors(&self, term: &str) -> Result<Vec<DoctorRef>, RepositoryError> {
        Self::guard(SEARCH_DOCTORS_SQL)?;
        let pattern = format!("%{}%", escape_like(term.trim()));
        let rows = sqlx::query(SEARCH_DOCTORS_SQL)
            .bind(&pattern)
            .bind(&pattern)
            .fetch_all(&self.pool)
            .await
            .map_err(Error::Database)?;
        rows.into_iter()
            .map(|row| {
                Ok(DoctorRef {
                    code: row.try_get("code").map_err(Error::Database)?,
                    name: row.try_get("name").map_err(Error::Database)?,
                })
            })
            .collect::<Result<Vec<_>, Error>>()
            .map_err(Into::into)
    }

    async fn fetch_usage(
        &self,
        icodes: &[String],
        doctor_codes: &[String],
        period: &RequisitionPeriod,
    ) -> Result<Vec<UsageRow>, RepositoryError> {
        if icodes.is_empty() || doctor_codes.is_empty() {
            return Ok(Vec::new());
        }
        let sql = usage_in_period_sql(icodes.len(), doctor_codes.len());
        Self::guard(&sql)?;

        let mut query = sqlx::query(sqlx::AssertSqlSafe(sql));
        for icode in icodes {
            query = query.bind(icode);
        }
        query = query.bind(period.start_date).bind(period.end_date);
        for code in doctor_codes {
            query = query.bind(code);
        }

        let rows = query.fetch_all(&self.pool).await.map_err(Error::Database)?;
        rows.into_iter()
            .map(|row| {
                Ok(UsageRow {
                    icode: row.try_get("icode").map_err(Error::Database)?,
                    qty: row.try_get("qty").map_err(Error::Database)?,
                })
            })
            .collect::<Result<Vec<_>, Error>>()
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_like_escapes_wildcards_and_backslash() {
        assert_eq!(escape_like("para"), "para");
        assert_eq!(escape_like("100%"), "100\\%");
        assert_eq!(escape_like("a_b"), "a\\_b");
        assert_eq!(escape_like("c:\\x"), "c:\\\\x");
        assert_eq!(escape_like("สมุนไพร"), "สมุนไพร");
    }
}
