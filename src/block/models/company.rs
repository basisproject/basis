use exonum::proto::ProtobufConvert;
use exonum::crypto::Hash;
use chrono::{DateTime, Utc};
use serde_json::{self, Value};
use crate::block::models::proto;
use crate::error::CError;

proto_enum! {
    enum CompanyType {
        Public = 1,
        Member = 2,
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
    MemberUpdate,
    MemberDelete,

    ProductCreate,
    ProductUpdate,
    ProductDelete,

    ProductOfferingCreate,
    ProductOfferingUpdate,
    ProductOfferingDelete,

    OrderCreate,
    OrderUpdateProcessStatus,
    OrderUpdatePaymentStatus,
    OrderCancel,
    Order,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Role {
    Owner,
    Admin,
    MemberAdmin,
    ProductAdmin,
    Buyer,
    Seller,
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
                    Permission::MemberUpdate,
                    Permission::MemberDelete,
                ]
            }
            Role::ProductAdmin => {
                vec![
                ]
            }
            Role::Buyer => {
                vec![
                ]
            }
            Role::Seller => {
                vec![
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
            .map_err(|_| From::from(CError::InvalidRole))
    }
}

#[derive(Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::company::Company", serde_pb_convert)]
pub struct Company {
    pub id: String,
    pub ty: CompanyType,
    pub email: String,
    pub name: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub history_len: u64,
    pub history_hash: Hash,
}

impl Company {
    pub fn new(id: &str, ty: CompanyType, email: &str, name: &str, created: &DateTime<Utc>, updated: &DateTime<Utc>, history_len: u64, &history_hash: &Hash) -> Self {
        Self {
            id: id.to_owned(),
            ty: ty,
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
            self.ty,
            email.unwrap_or(&self.email),
            name.unwrap_or(&self.name),
            &self.created,
            updated,
            self.history_len + 1,
            history_hash
        )
    }

    pub fn set_type(self, ty: CompanyType, updated: &DateTime<Utc>, history_hash: &Hash) -> Self { 
        Self::new(
            &self.id,
            ty,
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

    #[test]
    fn permissions_work() {
        let owner = Role::Owner;
        assert!(owner.can(&Permission::All));
        assert!(owner.can(&Permission::CompanyUpdate));
        assert!(owner.can(&Permission::CompanyDelete));
        assert!(owner.can(&Permission::MemberCreate));
        assert!(owner.can(&Permission::MemberUpdate));
        assert!(owner.can(&Permission::MemberDelete));
        assert!(owner.can(&Permission::ProductCreate));
        assert!(owner.can(&Permission::ProductUpdate));
        assert!(owner.can(&Permission::ProductDelete));
        assert!(owner.can(&Permission::ProductOfferingCreate));
        assert!(owner.can(&Permission::ProductOfferingUpdate));
        assert!(owner.can(&Permission::ProductOfferingDelete));
        assert!(owner.can(&Permission::OrderCreate));
        assert!(owner.can(&Permission::OrderUpdateProcessStatus));
        assert!(owner.can(&Permission::OrderUpdatePaymentStatus));
        assert!(owner.can(&Permission::OrderCancel));
        assert!(owner.can(&Permission::Order));

        let admin = Role::Admin;
        assert!(admin.can(&Permission::CompanyUpdate));
        assert!(!admin.can(&Permission::CompanyDelete));
        assert!(admin.can(&Permission::MemberCreate));
        assert!(admin.can(&Permission::MemberUpdate));
        assert!(admin.can(&Permission::MemberDelete));
        assert!(admin.can(&Permission::ProductCreate));
        assert!(admin.can(&Permission::ProductUpdate));
        assert!(admin.can(&Permission::ProductDelete));
        assert!(admin.can(&Permission::ProductOfferingCreate));
        assert!(admin.can(&Permission::ProductOfferingUpdate));
        assert!(admin.can(&Permission::ProductOfferingDelete));
        assert!(admin.can(&Permission::OrderCreate));
        assert!(admin.can(&Permission::OrderUpdateProcessStatus));
        assert!(admin.can(&Permission::OrderUpdatePaymentStatus));
        assert!(admin.can(&Permission::OrderCancel));
        assert!(admin.can(&Permission::Order));

        let member_admin = Role::MemberAdmin;
        assert!(!member_admin.can(&Permission::CompanyUpdate));
        assert!(!member_admin.can(&Permission::CompanyDelete));
        assert!(member_admin.can(&Permission::MemberCreate));
        assert!(member_admin.can(&Permission::MemberUpdate));
        assert!(member_admin.can(&Permission::MemberDelete));
        assert!(!member_admin.can(&Permission::ProductCreate));
        assert!(!member_admin.can(&Permission::ProductUpdate));
        assert!(!member_admin.can(&Permission::ProductDelete));
        assert!(!member_admin.can(&Permission::ProductOfferingCreate));
        assert!(!member_admin.can(&Permission::ProductOfferingUpdate));
        assert!(!member_admin.can(&Permission::ProductOfferingDelete));
        assert!(!member_admin.can(&Permission::OrderCreate));
        assert!(!member_admin.can(&Permission::OrderUpdateProcessStatus));
        assert!(!member_admin.can(&Permission::OrderUpdatePaymentStatus));
        assert!(!member_admin.can(&Permission::OrderCancel));
        assert!(!member_admin.can(&Permission::Order));
    }
}
