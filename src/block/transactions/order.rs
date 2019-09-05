use chrono::{DateTime, Utc};
use exonum::{
    blockchain::{ExecutionError, ExecutionResult, Transaction, TransactionContext},
    crypto::{PublicKey, SecretKey},
    messages::{Message, RawTransaction, Signed},
};
use crate::block::{
    SERVICE_ID,
    schema::Schema,
    models::proto,
    models::company::{Permission as CompanyPermission},
    models::access::Permission,
    models::order::ProductEntry,
    transactions::{company, access},
};
use crate::util::{self, protobuf::empty_opt};
use super::CommonError;

#[derive(Debug, Fail)]
#[repr(u8)]
pub enum TransactionError {
    #[fail(display = "Invalid ID")]
    InvalidID = 0,

    #[fail(display = "Order not found")]
    OrderNotFound = 1,

    #[fail(display = "ID already exists")]
    IDExists = 2,
}
define_exec_error!(TransactionError);

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::order::TxCreate")]
pub struct TxCreate {
    pub id: String,
    pub company_id_from: String,
    pub company_id_to: String,
    pub products: Vec<ProductEntry>,
    pub created: DateTime<Utc>,
}

impl Transaction for TxCreate {
    fn execute(&self, context: TransactionContext) -> ExecutionResult {
        let pubkey = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        access::check(&mut schema, pubkey, Permission::OrderCreate)?;
        company::check(&mut schema, &self.company_id_from, pubkey, CompanyPermission::OrderCreate)?;

        if schema.get_order(&self.id).is_some() {
            Err(TransactionError::IDExists)?;
        }
        if !util::time::is_current(&self.created) {
            Err(CommonError::InvalidTime)?;
        }
        schema.orders_create(&self.id, &self.company_id_from, &self.company_id_to, &self.products, &self.created, &hash);
        Ok(())
    }
}

