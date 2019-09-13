use exonum::proto::ProtobufConvert;
use exonum::crypto::Hash;
use chrono::{DateTime, Utc};
use serde_json::{self, Value};
use crate::proto;
use error::BError;

proto_enum! {
    enum CompanyType {
        Unknown = 0,
        Public = 1,
        Syndicate = 2,
        Private = 3,
    };
    proto::company::CompanyType
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Permission {
    All,
    AllBut(Vec<Permission>),

    CompanyUpdate,
    CompanyDelete,

    MemberCreate,
    MemberSetRoles,
    MemberDelete,

    ProductCreate,
    ProductUpdate,
    ProductDelete,

    OrderCreate,
    OrderUpdateProcessStatus,
    OrderUpdateShipping,
    OrderUpdateShippingDates,
    OrderCancel,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Role {
    Owner,
    Admin,
    MemberAdmin,
    ProductAdmin,
    Purchaser,
    Supplier,
}

impl Role {
    pub fn permissions(&self) -> Vec<Permission> {
        match *self {
            Role::Owner => {
                vec![Permission::All]
            }
            Role::Admin => {
                vec![
                    Permission::AllBut(vec![Permission::CompanyDelete]),
                ]
            }
            Role::MemberAdmin => {
                vec![
                    Permission::MemberCreate,
                    Permission::MemberSetRoles,
                    Permission::MemberDelete,
                ]
            }
            Role::ProductAdmin => {
                vec![
                    Permission::ProductCreate,
                    Permission::ProductUpdate,
                    Permission::ProductDelete,
                ]
            }
            Role::Purchaser => {
                vec![
                    Permission::OrderCreate,
                    Permission::OrderCancel,
                ]
            }
            Role::Supplier => {
                vec![
                    Permission::OrderUpdateProcessStatus,
                    Permission::OrderCancel,
                ]
            }
        }
    }

    pub fn can(&self, perm: &Permission) -> bool {
        for p in &self.permissions() {
            match p {
                Permission::All => {
                    return true;
                }
                Permission::AllBut(x) => {
                    if x.contains(perm) {
                        return false;
                    }
                    return true;
                }
                _ => {
                    if p == perm {
                        return true;
                    }
                }
            }
        }
        false
    }
}

impl ProtobufConvert for Role {
    type ProtoStruct = String;

    fn to_pb(&self) -> Self::ProtoStruct {
        match serde_json::to_value(self) {
            Ok(Value::String(x)) => x,
            _ => String::from("<invalid-role>"),
        }
    }

    fn from_pb(pb: Self::ProtoStruct) -> Result<Self, failure::Error> {
        serde_json::from_value::<Role>(Value::String(pb))
            .map_err(|_| From::from(BError::InvalidRole))
    }
}

#[derive(Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::company::Company", serde_pb_convert)]
pub struct Company {
    pub id: String,
    pub ty: CompanyType,
    pub region_id: String,  // should be an Option, but protobufs are stupid
    pub email: String,
    pub name: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub history_len: u64,
    pub history_hash: Hash,
}

impl Company {
    pub fn new(id: &str, ty: &CompanyType, region_id: Option<&str>, email: &str, name: &str, created: &DateTime<Utc>, updated: &DateTime<Utc>, history_len: u64, &history_hash: &Hash) -> Self {
        Self {
            id: id.to_owned(),
            ty: ty.clone(),
            region_id: region_id.map(|x| x.to_owned()).unwrap_or("".to_owned()),
            email: email.to_owned(),
            name: name.to_owned(),
            created: created.clone(),
            updated: updated.clone(),
            history_len,
            history_hash,
        }
    }

    pub fn update(self, email: Option<&str>, name: Option<&str>, updated: &DateTime<Utc>, history_hash: &Hash) -> Self {
        Self::new(
            &self.id,
            &self.ty,
            Some(&self.region_id),
            email.unwrap_or(&self.email),
            name.unwrap_or(&self.name),
            &self.created,
            updated,
            self.history_len + 1,
            history_hash
        )
    }

    pub fn set_type(self, ty: &CompanyType, updated: &DateTime<Utc>, history_hash: &Hash) -> Self { 
        Self::new(
            &self.id,
            ty,
            Some(&self.region_id),
            &self.email,
            &self.name,
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

    #[test]
    fn permissions_work() {
        let owner = Role::Owner;
        assert!(owner.can(&Permission::All));
        assert!(owner.can(&Permission::CompanyUpdate));
        assert!(owner.can(&Permission::CompanyDelete));
        assert!(owner.can(&Permission::MemberCreate));
        assert!(owner.can(&Permission::MemberSetRoles));
        assert!(owner.can(&Permission::MemberDelete));
        assert!(owner.can(&Permission::ProductCreate));
        assert!(owner.can(&Permission::ProductUpdate));
        assert!(owner.can(&Permission::ProductDelete));
        assert!(owner.can(&Permission::OrderCreate));
        assert!(owner.can(&Permission::OrderUpdateProcessStatus));
        assert!(owner.can(&Permission::OrderCancel));

        let admin = Role::Admin;
        assert!(admin.can(&Permission::CompanyUpdate));
        assert!(!admin.can(&Permission::CompanyDelete));
        assert!(admin.can(&Permission::MemberCreate));
        assert!(admin.can(&Permission::MemberSetRoles));
        assert!(admin.can(&Permission::MemberDelete));
        assert!(admin.can(&Permission::ProductCreate));
        assert!(admin.can(&Permission::ProductUpdate));
        assert!(admin.can(&Permission::ProductDelete));
        assert!(admin.can(&Permission::OrderCreate));
        assert!(admin.can(&Permission::OrderUpdateProcessStatus));
        assert!(admin.can(&Permission::OrderCancel));

        let member_admin = Role::MemberAdmin;
        assert!(!member_admin.can(&Permission::CompanyUpdate));
        assert!(!member_admin.can(&Permission::CompanyDelete));
        assert!(member_admin.can(&Permission::MemberCreate));
        assert!(member_admin.can(&Permission::MemberSetRoles));
        assert!(member_admin.can(&Permission::MemberDelete));
        assert!(!member_admin.can(&Permission::ProductCreate));
        assert!(!member_admin.can(&Permission::ProductUpdate));
        assert!(!member_admin.can(&Permission::ProductDelete));
        assert!(!member_admin.can(&Permission::OrderCreate));
        assert!(!member_admin.can(&Permission::OrderUpdateProcessStatus));
        assert!(!member_admin.can(&Permission::OrderCancel));
    }

    fn make_date() -> DateTime<Utc> {
        chrono::offset::Utc::now()
    }

    fn make_hash() -> Hash {
        Hash::new([1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4])
    }

    fn make_company() -> Company {
        let date = make_date();
        Company::new(
            "1dd0ec02-1c6d-4791-8ba5-eb9e16964c26",
            &CompanyType::Private,
            None,
            "homayun@friendless.com",
            "LEMONADE STANDS UNLIMITED",
            &date,
            &date,
            0,
            &make_hash()
        )
    }

    #[test]
    fn updates() {
        let company = make_company();
        util::sleep(100);
        let date2 = make_date();
        let hash2 = Hash::new([1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4]);
        let company2 = company.clone().update(None, None, &date2, &hash2);
        assert_eq!(company.id, company2.id);
        assert_eq!(company.ty, company2.ty);
        assert_eq!(company.region_id, company2.region_id);
        assert_eq!(company.email, company2.email);
        assert_eq!(company.name, company2.name);
        assert_eq!(company.created, company2.created);
        assert!(company.updated != company2.updated);
        assert_eq!(company2.updated, date2);
        assert_eq!(company.history_len, company2.history_len - 1);
        assert!(company.history_hash != company2.history_hash);
        assert_eq!(company2.history_hash, hash2);
        util::sleep(100);
        let date3 = make_date();
        let hash3 = Hash::new([1, 37, 6, 4, 1, 37, 6, 4, 1, 37, 6, 4, 1, 37, 6, 4, 1, 37, 6, 4, 1, 37, 6, 4, 1, 37, 6, 4, 1, 37, 6, 4]);
        let company3 = company2.clone().update(Some("spiffy@jiffy.com"), Some("I AM THE WALRUS"), &date3, &hash3);
        assert_eq!(company2.id, company3.id);
        assert_eq!(company2.ty, company3.ty);
        assert!(company2.email != company3.email);
        assert!(company2.name != company3.name);
        assert_eq!(company3.email, "spiffy@jiffy.com");
        assert_eq!(company3.name, "I AM THE WALRUS");
        assert_eq!(company2.created, company3.created);
        assert!(company2.updated != company3.updated);
        assert_eq!(company3.updated, date3);
        assert_eq!(company2.history_len, company3.history_len - 1);
        assert!(company2.history_hash != company3.history_hash);
        assert_eq!(company3.history_hash, hash3);
    }

    #[test]
    fn sets_type() {
        let company = make_company();
        util::sleep(100);
        let date2 = make_date();
        let hash2 = Hash::new([1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4]);
        let company2 = company.clone().set_type(&CompanyType::Syndicate, &date2, &hash2);
        assert_eq!(company.id, company2.id);
        assert!(company.ty != company2.ty);
        assert_eq!(company2.ty, CompanyType::Syndicate);
        assert_eq!(company.email, company2.email);
        assert_eq!(company.name, company2.name);
        assert_eq!(company.created, company2.created);
        assert!(company.updated != company2.updated);
        assert_eq!(company2.updated, date2);
        assert_eq!(company.history_len, company2.history_len - 1);
        assert!(company.history_hash != company2.history_hash);
        assert_eq!(company2.history_hash, hash2);
    }
}

