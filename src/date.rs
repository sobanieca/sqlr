//! JavaScript `Date` semantics: dates are milliseconds since the Unix epoch.
//! Local time conversions use the system time zone (`TZ`) through libc.

const MS_PER_DAY: i64 = 86_400_000;
/// Largest time value a JavaScript `Date` can hold.
const MAX_TIME: f64 = 8.64e15;

/// Days since 1970-01-01 for a proleptic Gregorian date.
pub fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let month_index = (month + 9) % 12;
    let doy = (153 * month_index + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// (year, month 1-12, day) for days since 1970-01-01.
pub fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

/// Broken-down date and time.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DateTime {
    pub year: i64,
    pub month: i64,
    pub day: i64,
    pub hour: i64,
    pub minute: i64,
    pub second: i64,
    pub millisecond: i64,
}

impl DateTime {
    pub fn date(year: i64, month: i64, day: i64) -> Self {
        DateTime {
            year,
            month,
            day,
            ..Default::default()
        }
    }

    /// Interprets the date and time as UTC (`Date.UTC`).
    pub fn to_utc_ms(self) -> f64 {
        let days = days_from_civil(self.year, self.month, self.day);
        let ms = days * MS_PER_DAY
            + self.hour * 3_600_000
            + self.minute * 60_000
            + self.second * 1000
            + self.millisecond;
        time_clip(ms as f64)
    }

    /// Interprets the date and time in the local time zone
    /// (`new Date(year, month - 1, ...)`).
    pub fn to_local_ms(self) -> f64 {
        let utc = self.to_utc_ms();
        if utc.is_nan() {
            return utc;
        }
        let guess = utc - local_offset_ms(utc);
        let adjusted = utc - local_offset_ms(guess);
        time_clip(adjusted)
    }

    fn from_ms(ms: f64) -> Self {
        let ms = ms as i64;
        let days = ms.div_euclid(MS_PER_DAY);
        let rest = ms.rem_euclid(MS_PER_DAY);
        let (year, month, day) = civil_from_days(days);
        DateTime {
            year,
            month,
            day,
            hour: rest / 3_600_000,
            minute: rest / 60_000 % 60,
            second: rest / 1000 % 60,
            millisecond: rest % 1000,
        }
    }
}

fn time_clip(ms: f64) -> f64 {
    if !ms.is_finite() || ms.abs() > MAX_TIME {
        f64::NAN
    } else {
        ms
    }
}

/// `Date.prototype.toISOString`, `None` for invalid dates.
pub fn to_iso_string(ms: f64) -> Option<String> {
    if ms.is_nan() {
        return None;
    }
    let dt = DateTime::from_ms(ms);
    let year = if (0..=9999).contains(&dt.year) {
        format!("{:04}", dt.year)
    } else if dt.year < 0 {
        format!("-{:06}", -dt.year)
    } else {
        format!("+{:06}", dt.year)
    };
    Some(format!(
        "{year}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        dt.month, dt.day, dt.hour, dt.minute, dt.second, dt.millisecond
    ))
}

const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// `Date.prototype.toString`, e.g.
/// `Tue Jan 02 2024 00:00:00 GMT+0000 (Coordinated Universal Time)`.
pub fn to_js_string(ms: f64) -> String {
    if ms.is_nan() {
        return "Invalid Date".to_string();
    }
    let local = local_time(ms);
    let dt = DateTime::from_ms(ms + local.offset_ms);
    let days = days_from_civil(dt.year, dt.month, dt.day);
    let weekday = WEEKDAYS[(days + 4).rem_euclid(7) as usize];
    let year = if dt.year < 0 {
        format!("-{:06}", -dt.year)
    } else {
        format!("{:04}", dt.year)
    };
    let offset_minutes = (local.offset_ms / 60_000.0) as i64;
    let sign = if offset_minutes < 0 { '-' } else { '+' };
    let offset_minutes = offset_minutes.abs();
    format!(
        "{weekday} {} {:02} {year} {:02}:{:02}:{:02} GMT{sign}{:02}{:02} ({})",
        MONTHS[(dt.month - 1) as usize],
        dt.day,
        dt.hour,
        dt.minute,
        dt.second,
        offset_minutes / 60,
        offset_minutes % 60,
        time_zone_name(&local.zone)
    )
}

struct LocalTime {
    offset_ms: f64,
    zone: String,
}

fn local_offset_ms(ms: f64) -> f64 {
    local_time(ms).offset_ms
}

fn local_time(ms: f64) -> LocalTime {
    let seconds = (ms / 1000.0).floor() as libc::time_t;
    // SAFETY: `tm` is plain data and `localtime_r` only writes into it.
    unsafe {
        let mut tm: libc::tm = std::mem::zeroed();
        if libc::localtime_r(&seconds, &mut tm).is_null() {
            return LocalTime {
                offset_ms: 0.0,
                zone: "UTC".to_string(),
            };
        }
        let zone = if tm.tm_zone.is_null() {
            String::new()
        } else {
            std::ffi::CStr::from_ptr(tm.tm_zone)
                .to_string_lossy()
                .into_owned()
        };
        LocalTime {
            offset_ms: tm.tm_gmtoff as f64 * 1000.0,
            zone,
        }
    }
}

/// Long time zone names as printed by V8 for common abbreviations.
fn time_zone_name(abbreviation: &str) -> String {
    let name = match abbreviation {
        "UTC" | "GMT" | "UCT" | "Z" => "Coordinated Universal Time",
        "CET" => "Central European Standard Time",
        "CEST" => "Central European Summer Time",
        "EET" => "Eastern European Standard Time",
        "EEST" => "Eastern European Summer Time",
        "WET" => "Western European Standard Time",
        "WEST" => "Western European Summer Time",
        "BST" => "British Summer Time",
        "EST" => "Eastern Standard Time",
        "EDT" => "Eastern Daylight Time",
        "CST" => "Central Standard Time",
        "CDT" => "Central Daylight Time",
        "MST" => "Mountain Standard Time",
        "MDT" => "Mountain Daylight Time",
        "PST" => "Pacific Standard Time",
        "PDT" => "Pacific Daylight Time",
        "JST" => "Japan Standard Time",
        "IST" => "India Standard Time",
        "AEST" => "Australian Eastern Standard Time",
        "AEDT" => "Australian Eastern Daylight Time",
        other => other,
    };
    name.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_civil_dates() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(days_from_civil(2024, 1, 2), 19724);
        assert_eq!(civil_from_days(19724), (2024, 1, 2));
        assert_eq!(civil_from_days(-1), (1969, 12, 31));
        assert_eq!(days_from_civil(1, 1, 1), -719_162);
    }

    #[test]
    fn formats_iso_strings() {
        let ms = DateTime {
            year: 2024,
            month: 1,
            day: 2,
            hour: 3,
            minute: 4,
            second: 5,
            millisecond: 678,
        }
        .to_utc_ms();
        assert_eq!(to_iso_string(ms).unwrap(), "2024-01-02T03:04:05.678Z");
        assert_eq!(to_iso_string(-1.0).unwrap(), "1969-12-31T23:59:59.999Z");
        assert_eq!(
            to_iso_string(DateTime::date(10000, 1, 1).to_utc_ms()).unwrap(),
            "+010000-01-01T00:00:00.000Z"
        );
        assert_eq!(to_iso_string(f64::NAN), None);
    }
}
