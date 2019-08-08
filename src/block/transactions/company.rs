use chrono::{DateTime, Utc};
use exonum::{
    blockchain::{ExecutionError, ExecutionResult, Transaction, TransactionContext},
    crypto::{PublicKey, SecretKey},
    messages::{Message, RawTransaction, Signed},
    storage::Fork,
};
use crate::block::{
    SERVICE_ID,
    schema::Schema,
    models::proto,
    models::company::{CompanyType, Permission as CompanyPermission, Role as CompanyRole},
    models::access::Permission,
    transactions::access,
};
use crate::util::{self, protobuf::empty_opt};
use super::CommonError;

pub fn check(schema: &mut Schema<&mut Fork>, company_id: &str, pubkey: &PublicKey, permission: CompanyPermission) -> Result<(), CommonError> {
    let user = match schema.get_user_by_pubkey(pubkey) {
        Some(x) => x,
        None => Err(CommonError::UserNotFound)?,
    };
    let member = match schema.get_company_member(company_id, &user.id) {
        Some(x) => x,
        None => Err(CommonError::InsufficientPrivileges)?,
    };
    for role in &member.roles {
        if role.can(&permission) {
            return Ok(())
        }
    }
    Err(CommonError::InsufficientPrivileges)
}

#[derive(Debug, Fail)]
#[repr(u8)]
pub enum TransactionError {
    #[fail(display = "Invalid ID")]
    InvalidID = 0,

    #[fail(display = "Invalid email")]
    InvalidEmail = 2,

    #[fail(display = "ID already exists")]
    IDExists = 3,

    #[fail(display = "Email already exists")]
    EmailExists = 5,

    #[fail(display = "Company not found")]
    CompanyNotFound = 6,
}
define_exec_error!(TransactionError);

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::company::TxCreatePrivate")]
pub struct TxCreatePrivate {
    pub id: String,
    pub email: String,
    pub name: String,
    pub created: DateTime<Utc>,
}

impl TxCreatePrivate {
    #[allow(dead_code)]
    pub fn sign(pk: &PublicKey, sk: &SecretKey, id: &str, email: &str, name: &str, created: &DateTime<Utc>) -> Signed<RawTransaction> {
        Message::sign_transaction(Self {id: id.to_owned(), email: email.to_owned(), name: name.to_owned(), created: created.clone() }, SERVICE_ID, *pk, sk)
    }
}

impl Transaction for TxCreatePrivate {
    fn execute(&self, mut context: TransactionContext) -> ExecutionResult {
        let pubkey = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        access::check(&mut schema, pubkey, Permission::CompanyCreatePrivate)?;
        let user = match schema.get_user_by_pubkey(pubkey) {
            Some(x) => x,
            None => Err(CommonError::UserNotFound)?,
        };

        if schema.get_company(&self.id).is_some() {
            Err(TransactionError::IDExists)?
        } else if !util::time::is_current(&self.created) {
            Err(CommonError::InvalidTime)?
        } else if !self.email.contains("@") {
            Err(TransactionError::InvalidEmail)?
        } else {
            schema.companies_create(&self.id, CompanyType::Private, None, &self.email, &self.name, &self.created, &hash);
            schema.companies_members_create(&self.id, &user.id, &vec![CompanyRole::Owner], &self.created, &hash);
            Ok(())
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::company::TxUpdate")]
pub struct TxUpdate {
    pub id: String,
    pub email: String,
    pub name: String,
    pub updated: DateTime<Utc>,
}

impl TxUpdate {
    #[allow(dead_code)]
    pub fn sign(pk: &PublicKey, sk: &SecretKey, id: &str, email: &str, name: &str, updated: &DateTime<Utc>) -> Signed<RawTransaction> {
        Message::sign_transaction(Self {id: id.to_owned(), email: email.to_owned(), name: name.to_owned(), updated: updated.clone() }, SERVICE_ID, *pk, sk)
    }
}

impl Transaction for TxUpdate {
    fn execute(&self, mut context: TransactionContext) -> ExecutionResult {
        let pubkey = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        match access::check(&mut schema, pubkey, Permission::CompanyAdminUpdate) {
            Ok(_) => {}
            Err(_) => {
                check(&mut schema, &self.id, pubkey, CompanyPermission::CompanyUpdate)?;
            }
        }

        // because protobuffers are kind of stupid and don't have null
        let email = empty_opt(&self.email).map(|x| x.as_str());
        let name = empty_opt(&self.name).map(|x| x.as_str());

        let company = match schema.get_company(self.id.as_str()) {
            Some(x) => x,
            None => Err(TransactionError::CompanyNotFound)?,
        };
        if let Some(email) = email.as_ref() {
            if !email.contains("@") {
                Err(TransactionError::InvalidEmail)?
            }
        }
        if !util::time::is_current(&self.updated) {
            Err(CommonError::InvalidTime)?
        }

        schema.companies_update(company, &self.id, email, name, &self.updated, &hash);
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::company::TxSetType")]
pub struct TxSetType {
    pub id: String,
    pub ty: CompanyType,
    pub updated: DateTime<Utc>,
}

impl TxSetType {
    #[allow(dead_code)]
    pub fn sign(pk: &PublicKey, sk: &SecretKey, id: &str, ty: CompanyType, updated: &DateTime<Utc>) -> Signed<RawTransaction> {
        Message::sign_transaction(Self {id: id.to_owned(), ty: ty, updated: updated.clone() }, SERVICE_ID, *pk, sk)
    }
}

impl Transaction for TxSetType {
    fn execute(&self, mut context: TransactionContext) -> ExecutionResult {
        let pubkey = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        let company = match schema.get_company(self.id.as_str()) {
            Some(x) => x,
            None => Err(TransactionError::CompanyNotFound)?,
        };

        access::check(&mut schema, pubkey, Permission::CompanySetType)?;

        if !util::time::is_current(&self.updated) {
            Err(CommonError::InvalidTime)?
        }

        schema.companies_set_type(company, &self.id, self.ty, &self.updated, &hash);
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::company::TxDelete")]
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

        match schema.get_company(self.id.as_str()) {
            Some(_) => (),
            None => Err(TransactionError::CompanyNotFound)?,
        }

        match access::check(&mut schema, pubkey, Permission::CompanyAdminDelete) {
            Ok(_) => {}
            Err(_) => {
                check(&mut schema, &self.id, pubkey, CompanyPermission::CompanyDelete)?;
            }
        }

        if !util::time::is_current(&self.deleted) {
            Err(CommonError::InvalidTime)?
        }

        schema.companies_delete(&self.id);
        Ok(())
    }
}

