use chrono::{DateTime, Utc};
use crate::block::models::{
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
}

impl CompanyMember {
    pub fn new(user_id: &str, roles: &Vec<Role>, created: &DateTime<Utc>, updated: &DateTime<Utc>) -> Self {
        Self {
            user_id: user_id.to_owned(),
            roles: roles.clone(),
            created: created.clone(),
            updated: updated.clone(),
        }
    }

    pub fn set_roles(self, roles: &Vec<Role>, updated: &DateTime<Utc>) -> Self {
        Self::new(
            &self.user_id,
            roles,
            &self.created,
            updated,
        )
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::util;

    fn make_date() -> DateTime<Utc> {
        chrono::offset::Utc::now()
    }

    // uhhhhhhhhuhuhuhuhuh...
    fn make_member() -> CompanyMember {
        let date = make_date();
        CompanyMember::new(
            "9fd8cdc6-04a8-4a35-9cd8-9dc6073a2d10",
            &vec![Role::Admin],
            &date,
            &date
        )
    }

    #[test]
    fn set_roles() {
        let member = make_member();
        util::sleep(100);
        let date2 = make_date();
        let roles2 = vec![Role::Purchaser, Role::ProductAdmin];
        let member2 = member.clone().set_roles(&roles2, &date2);
        assert_eq!(member.user_id, member2.user_id);
        assert!(member.roles != member2.roles);
        assert_eq!(member2.roles, roles2);
        assert_eq!(member.created, member2.created);
        assert!(member.updated != member2.updated);
        assert_eq!(member2.updated, date2);
    }
}

