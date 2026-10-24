# HOSxP integration notes

SnapHerb opens the hospital's HOSxP MySQL/MariaDB database **read-only**.
The real security boundary is the database account (`GRANT SELECT` only);
the app also sets a read-only session and rejects non-`SELECT` statements
before they reach the driver.

## Login (`officer`)

The login query reads one `officer` row by `officer_login_name`, joins
`doctor` for a display-name fallback, and verifies
`MD5(password) = officer_login_password_md5`. The plaintext password column
and `opduser` are never selected.

The MD5 assumption follows the hospital's existing HOSxP integration and
should be verified with one real test account before production use.

## Doctor search (`doctor`)

```sql
SELECT COALESCE(code, '') AS code, COALESCE(name, '') AS name
FROM doctor
WHERE code LIKE ? OR name LIKE ?
ORDER BY name
LIMIT 50
```

- The operator's search term is escaped (`\`, `%`, `_`) and bound as a
  `%...%` pattern.
- `doctor.code` is the value stored in `opitemrece.doctor` for each drug order.

## Herbal medicine search (`drugitems`)

```sql
SELECT icode, COALESCE(name, '') AS name,
       COALESCE(strength, '') AS strength,
       COALESCE(units, '') AS units
FROM drugitems
WHERE icode LIKE '1%'
  AND (icode LIKE ? OR name LIKE ?)
ORDER BY name
LIMIT 50
```

The picker shows the HOSxP `units` value as a suggestion for the usage unit.
The operator still chooses the dispensing unit, units per dispensing unit,
and the per-period maximum in SnapHerb settings.

## Usage in a date range (`opitemrece`)

The date range is inclusive: `rxdate` from the start date through the end
date. Usage is summed per configured `icode` and filtered to the selected
prescriber codes on each drug order line.

```sql
SELECT o.icode AS icode, CAST(SUM(o.qty) AS DOUBLE) AS qty
FROM opitemrece o
JOIN drugitems d ON d.icode = o.icode
WHERE o.icode IN (?, ...)
  AND o.qty > 0
  AND o.sub_type = '1'
  AND IFNULL(o.item_type, '') <> 'A'
  AND o.rxdate >= ?
  AND o.rxdate <= ?
  AND o.doctor IN (?, ...)
GROUP BY o.icode
ORDER BY o.icode
```

Notes:

- The HOSxP data dictionary defines `opitemrece.doctor` as the prescriber
  on the order line, so the filter is applied per drug line.
- The `drugitems` join and `sub_type = '1'` keep non-drug charge lines out.
- `item_type = 'A'` rows are automatic charges, not dispensed drugs.
- `qty > 0` ignores billing corrections.
- Usage is the recorded drug-order quantity in `opitemrece.qty`, not a
  dispense reconciliation from `opi_dispense*` (whose `vn + item_no` join is
  not unique and would fan out).
- The query counts every department's order lines by the selected doctors
  (`opitemrece` covers OPD and IPD). Add an `an` or `dep_code` filter only if
  the hospital's requisition policy requires one.
- The application never writes to HOSxP.

## Schema verification checklist

These items should be confirmed against the production HOSxP instance:

- [ ] `doctor.code` / `doctor.name` are the prescriber master columns.
- [ ] `opitemrece.doctor` stores the selected doctor's `doctor.code` on
      every herbal drug order line.
- [ ] `opitemrece.rxdate` is the order/dispensing date used for the
      requisition period.
- [ ] `opitemrece.qty`, `sub_type`, and `item_type` semantics match the
      filters above.
- [ ] `drugitems.units` is suitable as the default usage-unit suggestion.
- [ ] An index supports the date/icode/doctor aggregation (for example
      `opitemrece(icode, doctor, rxdate)`).
- [ ] `officer_login_password_md5` verification works with one real account.

If a field differs on the hospital instance, update
`crates/hosxp-connector/src/queries.rs` and its guard tests together.
