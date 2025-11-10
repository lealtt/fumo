use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDate, NaiveTime, TimeZone, Utc};

#[derive(Clone, Copy, Debug)]
pub struct ResetTime {
    pub hour: u32,
    pub minute: u32,
    pub timezone_offset_secs: i32,
}

impl ResetTime {
    pub fn new(hour: u32, minute: u32, timezone_offset_secs: i32) -> Self {
        Self {
            hour,
            minute,
            timezone_offset_secs,
        }
    }

    pub fn brt(hour: u32, minute: u32) -> Self {
        Self::new(hour, minute, -3 * 60 * 60)
    }

    pub fn timezone_offset(&self) -> FixedOffset {
        FixedOffset::east_opt(self.timezone_offset_secs).expect("valid timezone offset")
    }

    pub fn as_naive_time(&self) -> NaiveTime {
        NaiveTime::from_hms_opt(self.hour, self.minute, 0).expect("valid reset time")
    }
}

impl Default for ResetTime {
    fn default() -> Self {
        Self::brt(21, 0) // 21:00 BRT
    }
}

/// Reset period
#[derive(Clone, Copy, Debug)]
pub enum ResetPeriod {
    Daily,
    Weekly,
    Monthly,
}

/// Returns a Discord timestamp in short date/time format (f)
pub fn describe_absolute(target: DateTime<Utc>) -> String {
    let timestamp = target.timestamp();
    format!("<t:{timestamp}:f>")
}

/// Returns a Discord timestamp in relative time format (R)
pub fn describe_relative(target: DateTime<Utc>) -> String {
    let timestamp = target.timestamp();
    format!("<t:{timestamp}:R>")
}

/// Attempts to parse an RFC3339 timestamp and format it relatively for Discord embeds
pub fn describe_relative_from_str(value: &str) -> Option<String> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|dt| describe_relative(dt.with_timezone(&Utc)))
}

/// Calculates the next reset from a reference date
pub fn next_reset_from(
    reference: DateTime<Utc>,
    period: ResetPeriod,
    reset_config: &ResetTime,
) -> DateTime<Utc> {
    let offset = reset_config.timezone_offset();
    let reset_time = reset_config.as_naive_time();
    let local = reference.with_timezone(&offset);
    let date = local.date_naive();

    let target_date = match period {
        ResetPeriod::Daily => {
            if local.time() < reset_time {
                date
            } else {
                date + Duration::days(1)
            }
        }
        ResetPeriod::Weekly => date + Duration::days(7),
        ResetPeriod::Monthly => add_one_month(date),
    };

    let target_naive = target_date.and_time(reset_time);

    offset
        .from_local_datetime(&target_naive)
        .single()
        .expect("unique local reset time")
        .with_timezone(&Utc)
}

/// Adds one month to a date, handling day overflow
pub fn add_one_month(date: NaiveDate) -> NaiveDate {
    let mut year = date.year();
    let mut month = date.month();

    if month == 12 {
        year += 1;
        month = 1;
    } else {
        month += 1;
    }

    let last_day = days_in_month(year, month);
    let day = date.day().min(last_day);

    NaiveDate::from_ymd_opt(year, month, day).expect("valid next month date")
}

/// Returns the number of days in a month
pub fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

/// Checks if a year is a leap year
pub fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}
