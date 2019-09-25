use exonum::crypto::Hash;
use crate::proto;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::labor::Labor", serde_pb_convert)]
pub struct Labor {
    pub id: String,
    pub company_id: String,
    pub user_id: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub history_len: u64,
    pub history_hash: Hash,
}

impl Labor {
    pub fn new(id: &str, company_id: &str, user_id: &str, start: Option<&DateTime<Utc>>, end: Option<&DateTime<Utc>>, created: &DateTime<Utc>, updated: &DateTime<Utc>, history_len: u64, history_hash: &Hash) -> Self {
        Self {
            id: id.to_owned(),
            company_id: company_id.to_owned(),
            user_id: user_id.to_owned(),
            start: start.unwrap_or(&util::time::default_time()).clone(),
            end: end.unwrap_or(&util::time::default_time()).clone(),
            created: created.clone(),
            updated: updated.clone(),
            history_len,
            history_hash: history_hash.clone(),
        }
    }

    pub fn set_time(&self, start: Option<&DateTime<Utc>>, end: Option<&DateTime<Utc>>, updated: &DateTime<Utc>, history_hash: &Hash) -> Self {
        Self::new(
            &self.id,
            &self.company_id,
            &self.user_id,
            Some(start.unwrap_or(&self.start)),
            Some(end.unwrap_or(&self.end)),
            &self.created,
            updated,
            self.history_len + 1,
            history_hash
        )
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use util;

    fn make_hash() -> Hash {
        Hash::new([1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4])
    }

    fn make_labor() -> Labor {
        let date = util::time::now();
        Labor::new(
            "9fd8cdc6-04a8-4a35-9cd8-9dc6073a2d10",
            "df874abc-5583-4740-9f4e-3236530bcc1e",
            "7de177ba-d589-4f7b-94e0-96d2b0752460",
            Some(&date),
            None,
            &date,
            &date,
            0,
            &make_hash()
        )
    }

    #[test]
    fn set_time() {
        let labor = make_labor();
        util::sleep(100);
        let date2 = util::time::now();
        let hash2 = Hash::new([1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 233, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4]);
        let labor2 = labor.set_time(Some(&date2), None, &date2, &hash2);
        assert_eq!(labor.id, labor2.id);
        assert_eq!(labor.company_id, labor2.company_id);
        assert_eq!(labor.user_id, labor2.user_id);
        assert!(labor.start != labor2.start);
        assert_eq!(labor2.start, date2);
        assert_eq!(labor.end, util::time::default_time());
        assert_eq!(labor2.end, util::time::default_time());
        assert_eq!(labor.created, labor2.created);
        assert_eq!(labor.history_len, 0);
        assert_eq!(labor2.history_len, 1);
        assert_eq!(labor2.history_hash, hash2);
        util::sleep(100);
        let date3 = util::time::now();
        let hash3 = Hash::new([1, 37, 6, 4, 1, 37, 6, 4, 1, 37, 6, 133, 1, 37, 6, 4, 1, 37, 6, 4, 1, 37, 6, 4, 1, 37, 6, 4, 1, 37, 6, 4]);
        let labor3 = labor2.set_time(None, Some(&date3), &date3, &hash3);
        assert_eq!(labor2.id, labor3.id);
        assert_eq!(labor2.company_id, labor3.company_id);
        assert_eq!(labor2.user_id, labor3.user_id);
        assert_eq!(labor2.start, labor3.start);
        assert_eq!(labor3.start, date2);
        assert_eq!(labor2.end, util::time::default_time());
        assert_eq!(labor3.end, date3);
        assert_eq!(labor2.created, labor3.created);
        assert_eq!(labor2.history_len, 1);
        assert_eq!(labor3.history_len, 2);
        assert_eq!(labor3.history_hash, hash3);
    }
}

