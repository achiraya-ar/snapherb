pub const PING_SQL: &str = "SELECT 1";

pub const SEARCH_DRUGS_SQL: &str = "\
SELECT icode, COALESCE(name, '') AS name, COALESCE(strength, '') AS strength, COALESCE(units, '') AS units
FROM drugitems
WHERE icode LIKE '1%'
  AND (icode LIKE ? OR name LIKE ?)
ORDER BY name
LIMIT 50";

pub const SEARCH_DOCTORS_SQL: &str = "\
SELECT COALESCE(code, '') AS code, COALESCE(name, '') AS name
FROM doctor
WHERE code LIKE ? OR name LIKE ?
ORDER BY name
LIMIT 50";

pub const FETCH_OFFICER_SQL: &str = "\
SELECT CAST(o.officer_id AS SIGNED) AS officer_id,
       o.officer_login_name AS login_name,
       COALESCE(o.officer_login_password_md5, '') AS password_md5,
       COALESCE(o.officer_active, 'N') AS active,
       CAST(COALESCE(o.officer_authentication_type_id, 1) AS SIGNED) AS auth_type_id,
       NULLIF(o.officer_doctor_code, '') AS doctor_code,
       COALESCE(
           NULLIF(TRIM(CONCAT(COALESCE(o.officer_pname, ''), COALESCE(o.officer_fname, ''), ' ', COALESCE(o.officer_lname, ''))), ''),
           NULLIF(TRIM(d.name), ''),
           ''
       ) AS display_name
FROM officer o
LEFT JOIN doctor d ON d.code = NULLIF(o.officer_doctor_code, '')
WHERE o.officer_login_name = ?
LIMIT 1";

const USAGE_HEAD: &str = "\
SELECT o.icode AS icode, CAST(SUM(o.qty) AS DOUBLE) AS qty
FROM opitemrece o
JOIN drugitems d ON d.icode = o.icode
WHERE o.icode IN (";

const USAGE_MIDDLE: &str = "\
)
  AND o.qty > 0
  AND o.sub_type = '1'
  AND IFNULL(o.item_type, '') <> 'A'
  AND o.rxdate >= ?
  AND o.rxdate <= ?
  AND o.doctor IN (";

const USAGE_TAIL: &str = "\
)
GROUP BY o.icode
ORDER BY o.icode";

pub fn usage_in_period_sql(icodes_len: usize, doctors_len: usize) -> String {
    let item_placeholders = std::iter::repeat_n("?", icodes_len)
        .collect::<Vec<_>>()
        .join(", ");
    let doctor_placeholders = std::iter::repeat_n("?", doctors_len)
        .collect::<Vec<_>>()
        .join(", ");
    format!("{USAGE_HEAD}{item_placeholders}{USAGE_MIDDLE}{doctor_placeholders}{USAGE_TAIL}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::readonly_guard::assert_read_only;

    #[test]
    fn every_constant_passes_the_read_only_guard() {
        for sql in [
            PING_SQL,
            SEARCH_DRUGS_SQL,
            SEARCH_DOCTORS_SQL,
            FETCH_OFFICER_SQL,
        ] {
            assert!(assert_read_only(sql).is_ok(), "guard must accept: {sql}");
        }
        for items in [1_usize, 2, 7] {
            for doctors in [1_usize, 2, 5] {
                let sql = usage_in_period_sql(items, doctors);
                assert!(assert_read_only(&sql).is_ok(), "guard must accept: {sql}");
            }
        }
    }

    #[test]
    fn usage_query_binds_items_dates_and_doctors() {
        let sql = usage_in_period_sql(3, 2);
        assert_eq!(sql.matches('?').count(), 3 + 2 + 2);
        assert!(sql.contains("IN (?, ?, ?)"));
        assert!(sql.contains("o.doctor IN (?, ?)"));
        assert!(sql.contains("o.rxdate >= ?"));
        assert!(sql.contains("o.rxdate <= ?"));
    }

    #[test]
    fn usage_query_uses_the_line_prescriber_and_drug_master() {
        let sql = usage_in_period_sql(2, 2);
        assert!(sql.contains("FROM opitemrece o"));
        assert!(sql.contains("JOIN drugitems d ON d.icode = o.icode"));
        assert!(sql.contains("o.doctor IN (?, ?)"));
        assert!(!sql.contains("rx_doctor"));
        assert!(sql.contains("o.qty > 0"));
        assert!(sql.contains("o.sub_type = '1'"));
        assert!(sql.contains("IFNULL(o.item_type, '') <> 'A'"));
        assert!(!sql.contains(';'));
    }

    #[test]
    fn doctor_search_query_uses_doctor_master() {
        assert!(SEARCH_DOCTORS_SQL.contains("FROM doctor"));
        assert!(SEARCH_DOCTORS_SQL.contains("code LIKE ?"));
        assert!(SEARCH_DOCTORS_SQL.contains("name LIKE ?"));
        assert!(SEARCH_DOCTORS_SQL.contains("LIMIT 50"));
        assert!(!SEARCH_DOCTORS_SQL.contains(';'));
    }

    #[test]
    fn officer_query_uses_only_safe_columns() {
        assert!(FETCH_OFFICER_SQL.contains("FROM officer o"));
        assert!(FETCH_OFFICER_SQL.contains("officer_login_password_md5"));
        assert!(!FETCH_OFFICER_SQL.contains("officer_login_password "));
        assert!(!FETCH_OFFICER_SQL.contains("opduser"));
        assert!(FETCH_OFFICER_SQL.contains("LIMIT 1"));
        assert!(!FETCH_OFFICER_SQL.contains(';'));
    }
}
