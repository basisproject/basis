use chrono::{DateTime, Utc};
use exonum::{
    blockchain::{ExecutionError, ExecutionResult, Transaction, TransactionContext},
};
use crate::block::{
    schema::Schema,
    models::proto,
    models::company::{Permission as CompanyPermission},
    models::access::Permission,
    models::order::{ProductEntry, ProcessStatus, ShippingEntry},
    transactions::{company, access},
};
use crate::util;
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

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::order::TxUpdateStatus")]
pub struct TxUpdateStatus {
    pub id: String,
    pub process_status: ProcessStatus,
    pub updated: DateTime<Utc>,
}

impl Transaction for TxUpdateStatus {
    fn execute(&self, context: TransactionContext) -> ExecutionResult {
        let pubkey = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        let ord = schema.get_order(&self.id);
        if ord.is_none() {
            Err(TransactionError::OrderNotFound)?;
        }
        let order = ord.unwrap();

        access::check(&mut schema, pubkey, Permission::OrderUpdate)?;
        company::check(&mut schema, &order.company_id_to, pubkey, CompanyPermission::OrderUpdateProcessStatus)?;

        if !util::time::is_current(&self.updated) {
            Err(CommonError::InvalidTime)?;
        }
        schema.orders_update_status(order, &self.process_status, &self.updated, &hash);
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::order::TxSetShipping")]
pub struct TxSetShipping {
    pub id: String,
    pub shipping: ShippingEntry,
    pub updated: DateTime<Utc>,
}

impl Transaction for TxSetShipping {
    fn execute(&self, context: TransactionContext) -> ExecutionResult {
        let pubkey = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        let ord = schema.get_order(&self.id);
        if ord.is_none() {
            Err(TransactionError::OrderNotFound)?;
        }
        let order = ord.unwrap();

        access::check(&mut schema, pubkey, Permission::OrderUpdate)?;
        company::check(&mut schema, &order.company_id_from, pubkey, CompanyPermission::OrderUpdateShipping)
            .or_else(|_| company::check(&mut schema, &order.company_id_to, pubkey, CompanyPermission::OrderUpdateShipping))?;

        if !util::time::is_current(&self.updated) {
            Err(CommonError::InvalidTime)?;
        }
        schema.orders_set_shipping(order, &self.shipping, &self.updated, &hash);
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::order::TxSetShippingPickup")]
pub struct TxSetShippingPickup {
    pub id: String,
    pub pickup: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

impl Transaction for TxSetShippingPickup {
    fn execute(&self, context: TransactionContext) -> ExecutionResult {
        let pubkey = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        let ord = schema.get_order(&self.id);
        if ord.is_none() {
            Err(TransactionError::OrderNotFound)?;
        }
        let order = ord.unwrap();

        access::check(&mut schema, pubkey, Permission::OrderUpdate)?;
        company::check(&mut schema, &order.shipping.company_id, pubkey, CompanyPermission::OrderUpdateShippingDates)?;

        if !util::time::is_current(&self.updated) {
            Err(CommonError::InvalidTime)?;
        }
        schema.orders_set_shipping_pickup(order, &self.pickup, &self.updated, &hash);
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::order::TxSetShippingDelivered")]
pub struct TxSetShippingDelivered {
    pub id: String,
    pub delivered: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

impl Transaction for TxSetShippingDelivered {
    fn execute(&self, context: TransactionContext) -> ExecutionResult {
        let pubkey = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        let ord = schema.get_order(&self.id);
        if ord.is_none() {
            Err(TransactionError::OrderNotFound)?;
        }
        let order = ord.unwrap();

        access::check(&mut schema, pubkey, Permission::OrderUpdate)?;
        company::check(&mut schema, &order.shipping.company_id, pubkey, CompanyPermission::OrderUpdateShippingDates)?;

        if !util::time::is_current(&self.updated) {
            Err(CommonError::InvalidTime)?;
        }
        schema.orders_set_shipping_delivered(order, &self.delivered, &self.updated, &hash);
        Ok(())
    }
}

