use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike};

pub const THAI_MONTHS: [&str; 12] = [
    "มกราคม",
    "กุมภาพันธ์",
    "มีนาคม",
    "เมษายน",
    "พฤษภาคม",
    "มิถุนายน",
    "กรกฎาคม",
    "สิงหาคม",
    "กันยายน",
    "ตุลาคม",
    "พฤศจิกายน",
    "ธันวาคม",
];

pub const THAI_MONTHS_ABBR: [&str; 12] = [
    "ม.ค.",
    "ก.พ.",
    "มี.ค.",
    "เม.ย.",
    "พ.ค.",
    "มิ.ย.",
    "ก.ค.",
    "ส.ค.",
    "ก.ย.",
    "ต.ค.",
    "พ.ย.",
    "ธ.ค.",
];

pub const THAI_DAYS: [&str; 7] = ["อาทิตย์", "จันทร์", "อังคาร", "พุธ", "พฤหัสบดี", "ศุกร์", "เสาร์"];

pub const THAI_DAYS_ABBR: [&str; 7] = ["อา.", "จ.", "อ.", "พ.", "พฤ.", "ศ.", "ส."];

pub fn be_year(ce_year: i32) -> i32 {
    ce_year + 543
}

pub fn thai_month_full(date: NaiveDate) -> &'static str {
    THAI_MONTHS
        .get(date.month0() as usize)
        .copied()
        .unwrap_or("")
}

pub fn thai_month_abbr(date: NaiveDate) -> &'static str {
    THAI_MONTHS_ABBR
        .get(date.month0() as usize)
        .copied()
        .unwrap_or("")
}

pub fn thai_weekday_full(date: NaiveDate) -> &'static str {
    THAI_DAYS
        .get(Datelike::weekday(&date).num_days_from_sunday() as usize)
        .copied()
        .unwrap_or("")
}

pub fn thai_weekday_abbr(date: NaiveDate) -> &'static str {
    THAI_DAYS_ABBR
        .get(Datelike::weekday(&date).num_days_from_sunday() as usize)
        .copied()
        .unwrap_or("")
}

pub fn format_thai_date(date: NaiveDate) -> String {
    format!(
        "{} {} {}",
        date.day(),
        thai_month_full(date),
        be_year(date.year())
    )
}

pub fn format_thai_date_full(date: NaiveDate) -> String {
    format!(
        "วัน{}ที่ {} {} {}",
        thai_weekday_full(date),
        date.day(),
        thai_month_full(date),
        be_year(date.year())
    )
}

pub fn format_thai_date_abbr(date: NaiveDate) -> String {
    format!(
        "{} {} {} {}",
        thai_weekday_abbr(date),
        date.day(),
        thai_month_abbr(date),
        be_year(date.year())
    )
}

pub fn format_time_hm(time: NaiveTime) -> String {
    format!("{:02}:{:02}", time.hour(), time.minute())
}

pub fn format_thai_datetime_full(datetime: NaiveDateTime) -> String {
    format!(
        "{} เวลา {} น.",
        format_thai_date_full(datetime.date()),
        format_time_hm(datetime.time())
    )
}

pub fn format_thai_datetime_abbr(datetime: NaiveDateTime) -> String {
    format!(
        "{} {}",
        format_thai_date_abbr(datetime.date()),
        format_time_hm(datetime.time())
    )
}

pub fn format_thai_datetime_short(datetime: NaiveDateTime) -> String {
    format!(
        "{:02}/{:02}/{} {}",
        datetime.day(),
        datetime.month(),
        be_year(datetime.year()),
        format_time_hm(datetime.time())
    )
}

pub fn format_qty(qty: f64) -> String {
    let rounded = (qty * 100.0).round() / 100.0;
    if rounded.fract() == 0.0 {
        format!("{}", rounded as i64)
    } else {
        format!("{rounded}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("valid test date")
    }

    #[test]
    fn be_year_adds_543() {
        assert_eq!(be_year(2026), 2569);
    }

    #[test]
    fn dates_use_thai_months_and_buddhist_years() {
        let d = date(2026, 10, 18);
        assert_eq!(format_thai_date(d), "18 ตุลาคม 2569");
        assert_eq!(format_thai_date_full(d), "วันอาทิตย์ที่ 18 ตุลาคม 2569");
        assert_eq!(format_thai_date_abbr(d), "อา. 18 ต.ค. 2569");
    }

    #[test]
    fn datetimes_are_formatted_for_the_sheet() {
        let dt = date(2026, 10, 18)
            .and_hms_opt(16, 0, 0)
            .expect("valid test datetime");
        assert_eq!(
            format_thai_datetime_full(dt),
            "วันอาทิตย์ที่ 18 ตุลาคม 2569 เวลา 16:00 น."
        );
        assert_eq!(format_thai_datetime_abbr(dt), "อา. 18 ต.ค. 2569 16:00");
        assert_eq!(format_thai_datetime_short(dt), "18/10/2569 16:00");
    }

    #[test]
    fn quantities_trim_trailing_zeros() {
        assert_eq!(format_qty(2.0), "2");
        assert_eq!(format_qty(2.5), "2.5");
        assert_eq!(format_qty(0.1 + 0.2), "0.3");
    }
}
