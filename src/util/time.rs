use chrono::{DateTime, Utc};

pub fn is_current(now: &DateTime<Utc>) -> bool {
    return (Utc::now() - now.clone()).num_seconds() < 10;
}

