use chrono::{DateTime, Utc, NaiveDateTime};

pub fn is_current(now: &DateTime<Utc>) -> bool {
    return (Utc::now() - now.clone()).num_seconds() < 10;
}

pub fn from_timestamp(ts: i64) -> DateTime<Utc> {
    DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(ts, 0), Utc)
}

pub fn default_time() -> DateTime<Utc> {
    from_timestamp(0)
}

