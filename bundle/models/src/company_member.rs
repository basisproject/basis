use exonum::crypto::Hash;
use chrono::{DateTime, Utc};
use crate::{
    proto,
    company::Role,
};

#[derive(Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::company_member::CompanyMember", serde_pb_convert)]
pub struct CompanyMember {
    pub user_id: String,
    pub roles: Vec<Role>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub history_len: u64,
    pub history_hash: Hash,
}

impl CompanyMember {
    pub fn new(user_id: &str, roles: &Vec<Role>, created: &DateTime<Utc>, updated: &DateTime<Utc>, history_len: u64, &history_hash: &Hash) -> Self {
        Self {
            user_id: user_id.to_owned(),
            roles: roles.clone(),
            created: created.clone(),
            updated: updated.clone(),
            history_len,
            history_hash,
        }
    }

    pub fn set_roles(self, roles: &Vec<Role>, updated: &DateTime<Utc>, history_hash: &Hash) -> Self {
        Self::new(
            &self.user_id,
            roles,
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
            "9fd8cdc6-04a8-4a35-9cd8-9dc6073a2d10",
            &vec![Role::Admin],
            &date,
            &date,
            0,
            &make_hash()
        )
    }

    #[test]
    fn set_roles() {
        let member = make_member();
        util::sleep(100);
        let date2 = make_date();
        let hash2 = Hash::new([1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 233, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4]);
        let roles2 = vec![Role::Purchaser, Role::ProductAdmin];
        let member2 = member.clone().set_roles(&roles2, &date2, &hash2);
        assert_eq!(member.user_id, member2.user_id);
        assert!(member.roles != member2.roles);
        assert_eq!(member2.roles, roles2);
        assert_eq!(member.created, member2.created);
        assert!(member.updated != member2.updated);
        assert_eq!(member2.updated, date2);
        assert_eq!(member.history_len, member2.history_len - 1);
        assert!(member.history_hash != member2.history_hash);
        assert_eq!(member2.history_hash, hash2);
    }
}

