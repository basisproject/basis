use chrono::{DateTime, Utc};
use exonum::{
    blockchain::{ExecutionError, ExecutionResult, Transaction, TransactionContext},
    crypto::{PublicKey, SecretKey},
    messages::{Message, RawTransaction, Signed},
};
use exonum_merkledb::Fork;
use crate::block::{
    SERVICE_ID,
    schema::Schema,
    models::proto,
    models::company::{Permission as CompanyPermission, Role as CompanyRole},
    models::access::Permission,
    transactions::{company, access},
};
use crate::util;
use super::CommonError;

#[derive(Debug, Fail)]
#[repr(u8)]
pub enum TransactionError {
    #[fail(display = "Invalid ID")]
    InvalidID = 0,

    #[fail(display = "Product not found")]
    ProductNotFound = 1,

    #[fail(display = "Company not found")]
    CompanyNotFound = 2,
}
define_exec_error!(TransactionError);

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::product::TxCreate")]
pub struct TxCreate {
    pub id: String,
    pub company_id: String,
    pub name: String,
    pub meta: String,
    pub active: bool,
    pub created: DateTime<Utc>,
}

impl TxCreate {
    #[allow(dead_code)]
    pub fn sign(pk: &PublicKey, sk: &SecretKey, id: &str, company_id: &str, name: &str, meta: &str, active: bool, created: &DateTime<Utc>) -> Signed<RawTransaction> {
        Message::sign_transaction(Self {id: id.to_owned(), company_id: company_id.to_owned(), name: name.to_owned(), meta: meta.to_owned(), active, created: created.clone() }, SERVICE_ID, *pk, sk)
    }
}

impl Transaction for TxCreate {
    fn execute(&self, context: TransactionContext) -> ExecutionResult {
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::product::TxDelete")]
pub struct TxDelete {
    pub id: String,
    pub deleted: DateTime<Utc>,
}

impl TxDelete {
    #[allow(dead_code)]
    pub fn sign(pk: &PublicKey, sk: &SecretKey, id: &str, deleted: &DateTime<Utc>) -> Signed<RawTransaction> {
        Message::sign_transaction(Self {id: id.to_owned(), deleted: deleted.clone() }, SERVICE_ID, *pk, sk)
    }
}

impl Transaction for TxDelete {
    fn execute(&self, context: TransactionContext) -> ExecutionResult {
        Ok(())
    }
}


