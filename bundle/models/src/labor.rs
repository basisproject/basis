use exonum::crypto::Hash;
use crate::proto;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::labor::Labor", serde_pb_convert)]
pub struct Labor {
    pub id: String,
    pub company_id: String,
    pub user_id: String,
    pub created: DateTime<Utc>,
    pub completed: DateTime<Utc>,
    pub history_len: u64,
    pub history_hash: Hash,
}

impl Labor {
    pub fn new(id: &str, company_id: &str, user_id: &str, created: &DateTime<Utc>, completed: Option<&DateTime<Utc>>, history_len: u64, history_hash: &Hash) -> Self {
        Self {
            id: id.to_owned(),
            company_id: company_id.to_owned(),
            user_id: user_id.to_owned(),
            created: created.clone(),
            completed: completed.unwrap_or(&util::time::default_time()).clone(),
            history_len,
            history_hash: history_hash.clone(),
        }
    }

    pub fn clock_out(&self, completed: &DateTime<Utc>, history_hash: &Hash) -> Self {
        Self::new(
            &self.id,
            &self.company_id,
            &self.user_id,
            &self.created,
            Some(completed),
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
            &date,
            None,
            0,
            &make_hash()
        )
    }

    #[test]
    fn clocks_out() {
        let labor = make_labor();
        util::sleep(100);
        let date2 = util::time::now();
        let hash2 = Hash::new([1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 233, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4]);
        let labor2 = labor.clock_out(&date2, &hash2);
        assert_eq!(labor.id, labor2.id);
        assert_eq!(labor.company_id, labor2.company_id);
        assert_eq!(labor.user_id, labor2.user_id);
        assert_eq!(labor.created, labor2.created);
        assert_eq!(labor.completed, util::time::default_time());
        assert_eq!(labor2.completed, date2);
        assert!(labor.completed != labor2.completed);
        assert_eq!(labor.history_len, 0);
        assert_eq!(labor2.history_len, 1);
        assert_eq!(labor2.history_hash, hash2);
    }
}

