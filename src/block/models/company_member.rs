use exonum::crypto::Hash;
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
}

