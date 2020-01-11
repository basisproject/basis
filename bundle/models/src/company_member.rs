use exonum::crypto::Hash;
use chrono::{DateTime, Utc};
use crate::{
    proto,
    company::Role,
    cost_tag::CostTagEntry,
};

#[derive(Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::company_member::CompanyMember", serde_pb_convert)]
pub struct CompanyMember {
    pub id: String,
    pub company_id: String,
    pub user_id: String,
    pub roles: Vec<Role>,
    pub occupation: String,
    pub default_cost_tags: Vec<CostTagEntry>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub history_len: u64,
    pub history_hash: Hash,
}

impl CompanyMember {
    pub fn new(id: &str, company_id: &str, user_id: &str, roles: &Vec<Role>, occupation: &str, default_cost_tags: &Vec<CostTagEntry>, created: &DateTime<Utc>, updated: &DateTime<Utc>, history_len: u64, &history_hash: &Hash) -> Self {
        Self {
            id: id.to_owned(),
            company_id: company_id.to_owned(),
            user_id: user_id.to_owned(),
            roles: roles.clone(),
            occupation: occupation.to_owned(),
            default_cost_tags: default_cost_tags.clone(),
            created: created.clone(),
            updated: updated.clone(),
            history_len,
            history_hash,
        }
    }

    pub fn update(self, roles: Option<&Vec<Role>>, occupation: Option<&str>, default_cost_tags: Option<&Vec<CostTagEntry>>, updated: &DateTime<Utc>, history_hash: &Hash) -> Self {
        Self::new(
            &self.id,
            &self.company_id,
            &self.user_id,
            roles.unwrap_or(&self.roles),
            occupation.unwrap_or(&self.occupation),
            default_cost_tags.unwrap_or(&self.default_cost_tags),
            &self.created,
            updated,
            self.history_len + 1,
            history_hash,
        )
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

    // uhhhhhhhhuhuhuhuhuh...
    fn make_member() -> CompanyMember {
        let date = make_date();
        CompanyMember::new(
            "360a4ce7-ffc4-4741-9673-e8cc0d489944",
            "880051b5-caba-4325-ac8c-385be90689fe",
            "9fd8cdc6-04a8-4a35-9cd8-9dc6073a2d10",
            &vec![Role::Admin],
            "Expert Baiter",
            &vec![CostTagEntry::new("labor costs", 10)],
            &date,
            &date,
            0,
            &make_hash()
        )
    }

    #[test]
    fn updates() {
        let member = make_member();
        util::sleep(100);
        let date2 = make_date();
        let hash2 = Hash::new([1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 233, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4]);
        let roles2 = vec![Role::Purchaser, Role::ProductAdmin];
        let member2 = member.clone().update(Some(&roles2), None, None, &date2, &hash2);
        assert_eq!(member.id, member2.id);
        assert_eq!(member.company_id, member2.company_id);
        assert_eq!(member.user_id, member2.user_id);
        assert!(member.roles != member2.roles);
        assert_eq!(member2.roles, roles2);
        assert_eq!(member.occupation, member2.occupation);
        assert_eq!(member.default_cost_tags, member2.default_cost_tags);
        assert_eq!(member.created, member2.created);
        assert!(member.updated != member2.updated);
        assert_eq!(member2.updated, date2);
        assert_eq!(member.history_len, member2.history_len - 1);
        assert!(member.history_hash != member2.history_hash);
        assert_eq!(member2.history_hash, hash2);

        util::sleep(100);
        let date3 = make_date();
        let hash3 = Hash::new([1, 37, 6, 4, 1, 37, 6, 4, 1, 37, 6, 133, 1, 37, 6, 4, 1, 37, 6, 4, 1, 37, 6, 4, 1, 37, 6, 4, 1, 37, 6, 4]);
        let default_cost_tags3 = vec![CostTagEntry::new("Tooth cleaning", 3)];
        let member3 = member2.clone().update(None, Some("Master Baiter"), Some(&default_cost_tags3), &date3, &hash3);
        assert_eq!(member2.id, member3.id);
        assert_eq!(member2.company_id, member3.company_id);
        assert_eq!(member2.user_id, member3.user_id);
        assert_eq!(member2.roles, member3.roles);
        assert_eq!(member3.roles, roles2);
        assert!(member2.occupation != member3.occupation);
        assert_eq!(member2.default_cost_tags[0].id, "labor costs");
        assert_eq!(member3.default_cost_tags[0].id, "Tooth cleaning");
        assert_eq!(member2.created, member3.created);
        assert!(member2.updated != member3.updated);
        assert_eq!(member3.updated, date3);
        assert_eq!(member2.history_len, member3.history_len - 1);
        assert!(member2.history_hash != member3.history_hash);
        assert_eq!(member3.history_hash, hash3);
    }
}

