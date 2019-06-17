use exonum::crypto::{Hash, PublicKey};
use crate::block::models::proto;
use chrono::{DateTime, Utc};
use crate::block::transactions::access::Role;

#[derive(Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::user::User", serde_pb_convert)]
pub struct User {
    pub id: String,
    pub pubkey: PublicKey,
    pub roles: Vec<Role>,
    pub email: String,
    pub name: String,
    pub meta: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub history_len: u64,
    pub history_hash: Hash,
}

impl User {
    pub fn new(id: &str, &pubkey: &PublicKey, roles: &Vec<Role>, email: &str, name: &str, meta: &str, created: &DateTime<Utc>, updated: &DateTime<Utc>, history_len: u64, &history_hash: &Hash) -> Self {
        Self {
            id: id.to_owned(),
            pubkey,
            roles: roles.clone(),
            email: email.to_owned(),
            name: name.to_owned(),
            meta: meta.to_owned(),
            created: created.clone(),
            updated: updated.clone(),
            history_len,
            history_hash,
        }
    }

    pub fn update(self, email: Option<&str>, name: Option<&str>, meta: Option<&str>, updated: &DateTime<Utc>, history_hash: &Hash) -> Self {
        Self::new(
            &self.id,
            &self.pubkey,
            &self.roles,
            email.unwrap_or(&self.email),
            name.unwrap_or(&self.name),
            meta.unwrap_or(&self.meta),
            &self.created,
            updated,
            self.history_len + 1,
            history_hash
        )
    }

    pub fn set_pubkey(self, pubkey: &PublicKey, updated: &DateTime<Utc>, history_hash: &Hash) -> Self {
        Self::new(
            &self.id,
            pubkey,
            &self.roles,
            &self.email,
            &self.name,
            &self.meta,
            &self.created,
            updated,
            self.history_len + 1,
            history_hash
        )
    }

    pub fn set_roles(self, roles: &Vec<Role>, updated: &DateTime<Utc>, history_hash: &Hash) -> Self {
        Self::new(
            &self.id,
            &self.pubkey,
            roles,
            &self.email,
            &self.name,
            &self.meta,
            &self.created,
            updated,
            self.history_len + 1,
            history_hash
        )
    }
}

