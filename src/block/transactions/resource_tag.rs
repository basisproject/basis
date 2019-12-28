use chrono::{DateTime, Utc};
use exonum::{
    blockchain::{ExecutionError, ExecutionResult, Transaction, TransactionContext},
};
use models::{
    proto,
    access::Permission,
};
use crate::block::{
    schema::Schema,
    transactions::access,
};
use util;
use super::CommonError;

#[derive(Debug, Fail)]
#[repr(u8)]
pub enum TransactionError {
    #[fail(display = "Resource tag not found")]
    ResourceTagNotFound = 0,

    #[fail(display = "Product not found")]
    ProductNotFound = 2,

    #[fail(display = "Resource tag is already deleted")]
    AlreadyDeleted = 4,
}
define_exec_error!(TransactionError);

deftransaction! {
    #[exonum(pb = "proto::resource_tag::TxCreate")]
    pub struct TxCreate {
        pub id: String,
        pub product_id: String,
        pub created: DateTime<Utc>,
    }
}

impl Transaction for TxCreate {
    fn execute(&self, context: TransactionContext) -> ExecutionResult {
        let pubkey = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        access::check(&mut schema, pubkey, Permission::ResourceTagCreate)?;

        match schema.get_product(&self.product_id) {
            Some(_) => {}
            None => Err(TransactionError::ProductNotFound)?,
        }

        if let Some(_) = schema.get_resource_tag(&self.id) {
            Err(CommonError::IDExists)?;
        }

        if !util::time::is_current(&self.created) {
            Err(CommonError::InvalidTime)?;
        }

        schema.resource_tags_create(&self.id, &self.product_id, &self.created, &hash);
        Ok(())
    }
}

deftransaction! {
    #[exonum(pb = "proto::resource_tag::TxDelete")]
    pub struct TxDelete {
        pub id: String,
        pub deleted: DateTime<Utc>,
    }
}

impl Transaction for TxDelete {
    fn execute(&self, context: TransactionContext) -> ExecutionResult {
        let pubkey = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        access::check(&mut schema, pubkey, Permission::ResourceTagDelete)?;

        let tag = schema.get_resource_tag(&self.id);
        if tag.is_none() {
            Err(TransactionError::ResourceTagNotFound)?;
        }

        let resource_tag = tag.unwrap();

        if resource_tag.is_deleted() {
            Err(TransactionError::AlreadyDeleted)?;
        }

        if !util::time::is_current(&self.deleted) {
            Err(CommonError::InvalidTime)?;
        }

        schema.resource_tags_delete(resource_tag, &self.deleted, &hash);
        Ok(())
    }
}

