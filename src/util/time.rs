use chrono::{DateTime, Utc, NaiveDateTime};

pub fn is_current(now: &DateTime<Utc>) -> bool {
    return (Utc::now() - now.clone()).num_seconds() < 10;
}

pub fn default_time() -> DateTime<Utc> {
    DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc)
}

