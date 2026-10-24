use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::thai::format_thai_date;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequisitionPeriod {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PeriodError {
    #[error("the period start must not be after its end")]
    InvalidRange,
}

impl RequisitionPeriod {
    pub fn new(start_date: NaiveDate, end_date: NaiveDate) -> Result<Self, PeriodError> {
        if start_date <= end_date {
            Ok(Self {
                start_date,
                end_date,
            })
        } else {
            Err(PeriodError::InvalidRange)
        }
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        self.start_date <= other.end_date && other.start_date <= self.end_date
    }

    pub fn contains(&self, date: NaiveDate) -> bool {
        self.start_date <= date && date <= self.end_date
    }

    pub fn ends_before(&self, date: NaiveDate) -> bool {
        self.end_date < date
    }

    pub fn thai_range(&self) -> String {
        if self.start_date == self.end_date {
            format_thai_date(self.start_date)
        } else {
            format!(
                "{} ถึง {}",
                format_thai_date(self.start_date),
                format_thai_date(self.end_date)
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("valid test date")
    }

    fn period(start: NaiveDate, end: NaiveDate) -> RequisitionPeriod {
        RequisitionPeriod::new(start, end).expect("valid test period")
    }

    #[test]
    fn period_rejects_reversed_ranges() {
        assert!(RequisitionPeriod::new(date(2026, 10, 2), date(2026, 10, 1)).is_err());
    }

    #[test]
    fn overlaps_includes_touching_days() {
        let a = period(date(2026, 10, 1), date(2026, 10, 10));
        let b = period(date(2026, 10, 10), date(2026, 10, 20));
        let c = period(date(2026, 10, 11), date(2026, 10, 20));
        assert!(a.overlaps(&b));
        assert!(!a.overlaps(&c));
    }

    #[test]
    fn contains_is_inclusive() {
        let a = period(date(2026, 10, 1), date(2026, 10, 10));
        assert!(a.contains(date(2026, 10, 1)));
        assert!(a.contains(date(2026, 10, 10)));
        assert!(!a.contains(date(2026, 10, 11)));
    }

    #[test]
    fn ends_before_rejects_today_and_future() {
        let a = period(date(2026, 10, 1), date(2026, 10, 10));
        assert!(a.ends_before(date(2026, 10, 11)));
        assert!(!a.ends_before(date(2026, 10, 10)));
        assert!(!a.ends_before(date(2026, 10, 9)));
    }

    #[test]
    fn thai_range_uses_buddhist_years() {
        let a = period(date(2026, 10, 1), date(2026, 10, 31));
        assert_eq!(a.thai_range(), "1 ตุลาคม 2569 ถึง 31 ตุลาคม 2569");
        let single = period(date(2026, 10, 1), date(2026, 10, 1));
        assert_eq!(single.thai_range(), "1 ตุลาคม 2569");
    }
}
