use chrono::{DateTime, Utc};
use hex::FromHex;
use exonum::{
    blockchain::{ExecutionError, ExecutionResult, Transaction, TransactionContext},
    crypto::{PublicKey, SecretKey},
    messages::{Message, RawTransaction, Signed},
};
use crate::block::{
    SERVICE_ID,
    schema::Schema,
    models::proto,
    models::access::{Permission, Role},
    transactions::access,
};
use crate::util::{self, protobuf::empty_opt};
use crate::config;
use super::CommonError;

#[derive(Debug, Fail)]
#[repr(u8)]
pub enum TransactionError {
    #[fail(display = "Invalid ID")]
    InvalidID = 0,

    #[fail(display = "Invalid pubkey")]
    InvalidPubkey = 1,

    #[fail(display = "Invalid email")]
    InvalidEmail = 2,

    #[fail(display = "ID already exists")]
    IDExists = 3,

    #[fail(display = "Pubkey already exists")]
    PubkeyExists = 4,

    #[fail(display = "Email already exists")]
    EmailExists = 5,

    #[fail(display = "User not found")]
    UserNotFound = 6,
}
define_exec_error!(TransactionError);

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::user::TxCreate")]
pub struct TxCreate {
    pub id: String,
    pub pubkey: PublicKey,
    pub roles: Vec<Role>,
    pub email: String,
    pub name: String,
    pub meta: String,
    pub created: DateTime<Utc>,
}

impl TxCreate {
    #[allow(dead_code)]
    pub fn sign(pk: &PublicKey, sk: &SecretKey, id: &str, &pubkey: &PublicKey, roles: &Vec<Role>, email: &str, name: &str, meta: &str, created: &DateTime<Utc>) -> Signed<RawTransaction> {
        Message::sign_transaction(Self {id: id.to_owned(), pubkey, roles: roles.clone(), email: email.to_owned(), name: name.to_owned(), meta: meta.to_owned(), created: created.clone() }, SERVICE_ID, *pk, sk)
    }
}

impl Transaction for TxCreate {
    fn execute(&self, mut context: TransactionContext) -> ExecutionResult {
        let pubkey = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        let bootstrap_key = PublicKey::from_hex(config::get::<String>("tests.bootstrap_user_key").unwrap_or(String::from("")));
        if bootstrap_key.is_err() || bootstrap_key.as_ref() != Ok(&pubkey) {
            access::check(&mut schema, pubkey, Permission::UserCreate)?;
        }

        if schema.get_user(self.id.as_str()).is_some() {
            Err(TransactionError::IDExists)?
        } else if schema.get_user_by_pubkey(&self.pubkey).is_some() {
            Err(TransactionError::PubkeyExists)?
        } else if schema.get_user_by_email(&self.email).is_some() {
            Err(TransactionError::EmailExists)?
        } else if !util::time::is_current(&self.created) {
            Err(CommonError::InvalidTime)?
        } else if !self.email.contains("@") {
            Err(TransactionError::InvalidEmail)?
        } else {
            schema.users_create(&self.id, &self.pubkey, &self.roles, &self.email, &self.name, &self.meta, &self.created, &hash);
            Ok(())
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::user::TxUpdate")]
pub struct TxUpdate {
    pub id: String,
    pub email: String,
    pub name: String,
    pub meta: String,
    pub updated: DateTime<Utc>,
}

impl TxUpdate {
    #[allow(dead_code)]
    pub fn sign(pk: &PublicKey, sk: &SecretKey, id: &str, email: &str, name: &str, meta: &str, updated: &DateTime<Utc>) -> Signed<RawTransaction> {
        Message::sign_transaction(Self {id: id.to_owned(), email: email.to_owned(), name: name.to_owned(), meta: meta.to_owned(), updated: updated.clone() }, SERVICE_ID, *pk, sk)
    }
}

impl Transaction for TxUpdate {
    fn execute(&self, mut context: TransactionContext) -> ExecutionResult {
        let pubkey = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        match access::check(&mut schema, pubkey, Permission::UserAdminUpdate) {
            Ok(_) => {}
            Err(_) => {
                match access::check(&mut schema, pubkey, Permission::UserUpdate) {
                    Ok(_) => {
                        match schema.get_user_by_pubkey(&pubkey) {
                            Some(user) => {
                                if user.id != self.id {
                                    Err(CommonError::InsufficientPrivileges)?
                                }
                            }
                            None => { 
                                Err(CommonError::InsufficientPrivileges)?
                            }
                        }
                    }
                    Err(e) => { Err(e)? }
                }
            }
        }

        // because protobuffers are kind of stupid and don't have null
        let email = empty_opt(&self.email).map(|x| x.as_str());
        let name = empty_opt(&self.name).map(|x| x.as_str());
        let meta = empty_opt(&self.meta).map(|x| x.as_str());

        let user = match schema.get_user(self.id.as_str()) {
            Some(x) => x,
            None => Err(TransactionError::UserNotFound)?,
        };
        if let Some(email) = email.as_ref() {
            if let Some(user) = schema.get_user_by_email(email) {
                if user.id != self.id {
                    Err(TransactionError::EmailExists)?
                }
            }
            if !email.contains("@") {
                Err(TransactionError::InvalidEmail)?
            }
        }
        if !util::time::is_current(&self.updated) {
            Err(CommonError::InvalidTime)?
        }

        schema.users_update(user, &self.id, email, name, meta, &self.updated, &hash);
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::user::TxSetPubkey")]
pub struct TxSetPubkey {
    pub id: String,
    pub pubkey: PublicKey,
    pub memo: String,
    pub updated: DateTime<Utc>,
}

impl TxSetPubkey {
    #[allow(dead_code)]
    pub fn sign(pk: &PublicKey, sk: &SecretKey, id: &str, &pubkey: &PublicKey, memo: &str, updated: &DateTime<Utc>) -> Signed<RawTransaction> {
        Message::sign_transaction(Self {id: id.to_owned(), pubkey, memo: memo.to_owned(), updated: updated.clone() }, SERVICE_ID, *pk, sk)
    }
}

impl Transaction for TxSetPubkey {
    fn execute(&self, mut context: TransactionContext) -> ExecutionResult {
        let pubkey = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        access::check(&mut schema, pubkey, Permission::UserAdminUpdate)?;

        let user = match schema.get_user(self.id.as_str()) {
            Some(x) => x,
            None => Err(TransactionError::UserNotFound)?,
        };
        match schema.get_user_by_pubkey(&self.pubkey) {
            Some(x) => {
                if x.id != self.id {
                    Err(TransactionError::PubkeyExists)?
                }
            }
            _ => {}
        }
        if !util::time::is_current(&self.updated) {
            Err(CommonError::InvalidTime)?
        }

        schema.users_set_pubkey(user, &self.id, &self.pubkey, &self.updated, &hash);
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::user::TxSetRoles")]
pub struct TxSetRoles {
    pub id: String,
    pub roles: Vec<Role>,
    pub memo: String,
    pub updated: DateTime<Utc>,
}

impl TxSetRoles {
    #[allow(dead_code)]
    pub fn sign(pk: &PublicKey, sk: &SecretKey, id: &str, roles: &Vec<Role>, memo: &str, updated: &DateTime<Utc>) -> Signed<RawTransaction> {
        Message::sign_transaction(Self {id: id.to_owned(), roles: roles.clone(), memo: memo.to_owned(), updated: updated.clone() }, SERVICE_ID, *pk, sk)
    }
}

impl Transaction for TxSetRoles {
    fn execute(&self, mut context: TransactionContext) -> ExecutionResult {
        let pubkey = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        access::check(&mut schema, pubkey, Permission::UserAdminUpdate)?;

        let user = match schema.get_user(self.id.as_str()) {
            Some(x) => x,
            None => Err(TransactionError::UserNotFound)?,
        };
        if !util::time::is_current(&self.updated) {
            Err(CommonError::InvalidTime)?
        }

        schema.users_set_roles(user, &self.id, &self.roles, &self.updated, &hash);
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::user::TxDelete")]
pub struct TxDelete {
    pub id: String,
    pub memo: String,
    pub deleted: DateTime<Utc>,
}

impl TxDelete {
    #[allow(dead_code)]
    pub fn sign(pk: &PublicKey, sk: &SecretKey, id: &str, memo: &str, deleted: &DateTime<Utc>) -> Signed<RawTransaction> {
        Message::sign_transaction(Self {id: id.to_owned(), memo: memo.to_owned(), deleted: deleted.clone() }, SERVICE_ID, *pk, sk)
    }
}

impl Transaction for TxDelete {
    fn execute(&self, mut context: TransactionContext) -> ExecutionResult {
        let pubkey = &context.author();

        let mut schema = Schema::new(context.fork());

        let user = match schema.get_user(self.id.as_str()) {
            Some(x) => x,
            None => Err(TransactionError::UserNotFound)?,
        };

        match access::check(&mut schema, pubkey, Permission::UserDelete) {
            Ok(_) => {}
            Err(_) => {
                if &user.pubkey != pubkey {
                    Err(CommonError::InsufficientPrivileges)?;
                }
            }
        }

        if !util::time::is_current(&self.deleted) {
            Err(CommonError::InvalidTime)?
        }

        schema.users_delete(user, &self.id);
        Ok(())
    }
}

