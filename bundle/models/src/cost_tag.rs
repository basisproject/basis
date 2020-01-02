use exonum::crypto::Hash;
use chrono::{DateTime, Utc};
use crate::proto;
use util;

#[derive(Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::cost_tag::CostTag", serde_pb_convert)]
pub struct CostTag {
    pub id: String,
    pub company_id: String,
    pub name: String,
    pub active: bool,
    pub meta: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub deleted: DateTime<Utc>,
    pub history_len: u64,
    pub history_hash: Hash,
}

impl CostTag {
    pub fn new(id: &str, company_id: &str, name: &str, active: bool, meta: &str, created: &DateTime<Utc>, updated: &DateTime<Utc>, deleted: Option<&DateTime<Utc>>, history_len: u64, history_hash: &Hash) -> Self {
        CostTag {
            id: id.to_owned(),
            company_id: company_id.to_owned(),
            name: name.to_owned(),
            active,
            meta: meta.to_owned(),
            created: created.clone(),
            updated: updated.clone(),
            deleted: deleted.unwrap_or(&util::time::default_time()).clone(),
            history_len,
            history_hash: history_hash.clone(),
        }
    }

    pub fn update(&self, name: Option<&str>, active: Option<bool>, meta: Option<&str>, updated: &DateTime<Utc>, history_hash: &Hash) -> Self {
        Self::new(
            &self.id,
            &self.company_id,
            name.unwrap_or(&self.name),
            active.unwrap_or(self.active),
            meta.unwrap_or(&self.meta),
            &self.created,
            updated,
            Some(&self.deleted),
            self.history_len + 1,
            history_hash
        )
    }

    pub fn delete(&self, deleted: &DateTime<Utc>, history_hash: &Hash) -> Self {
        Self::new(
            &self.id,
            &self.company_id,
            &self.name,
            self.active,
            &self.meta,
            &self.created,
            &self.updated,
            Some(deleted),
            self.history_len + 1,
            history_hash
        )
    }

    pub fn is_active(&self) -> bool {
        self.active && !self.is_deleted()
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted != util::time::default_time()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use util;

    fn make_date() -> DateTime<Utc> {
        chrono::offset::Utc::now()
    }

    fn make_hash() -> Hash {
        Hash::new([1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4])
    }

    fn make_cost_tag() -> CostTag {
        let date = make_date();
        CostTag::new(
            "4266954b-c5c0-43e4-a740-9e36c726451d",
            "b9eb0cc2-5b37-4fd1-83fd-8597625aee95",
            "Labor costs lol",
            true,
            r#"{"description":"VENEZUELA"}"#,
            &date,
            &date,
            None,
            0,
            &make_hash(),
        )
    }

    #[test]
    fn updates() {
        let tag = make_cost_tag();
        util::sleep(100);
        let date2 = make_date();
        let hash2 = Hash::new([1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4]);
        let tag2 = tag.clone().update(
            Some("ongoing labor expenses"),
            Some(false),
            Some(r#"{"description":"NORTH KOREA CHECKMATE LEFTISTS"}"#),
            &date2, 
            &hash2
        );
        assert_eq!(tag2.company_id, tag.company_id);
        assert_eq!(tag.name, "Labor costs lol");
        assert_eq!(tag2.name, "ongoing labor expenses");
        assert_eq!(tag.meta, r#"{"description":"VENEZUELA"}"#);
        assert_eq!(tag2.meta, r#"{"description":"NORTH KOREA CHECKMATE LEFTISTS"}"#);
        assert_eq!(tag.created, tag2.created);
        assert!(tag.updated != tag2.updated);
        assert_eq!(tag2.updated, date2);
        assert_eq!(tag2.history_len, tag.history_len + 1);
        assert_eq!(tag2.history_hash, hash2);
        assert!(!tag2.is_active());
    }

    #[test]
    fn deletes() {
        let tag = make_cost_tag();
        assert_eq!(tag.deleted, util::time::default_time());
        assert!(!tag.is_deleted());
        let date2 = make_date();
        let hash2 = Hash::new([56, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4]);
        let tag2 = tag.delete(&date2, &hash2);
        assert_eq!(tag2.deleted, date2);
        assert!(tag2.deleted != util::time::default_time());
        assert!(tag2.is_deleted());
        assert!(!tag2.is_active());
    }
}

