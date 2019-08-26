use std::collections::HashMap;
use exonum::crypto::Hash;
use chrono::{DateTime, Utc};
use crate::block::models::proto;
use crate::util;

proto_enum! {
    enum OrderProcessStatus {
        Unknown = 0,
		New = 1,
		Accepted = 2,
		Processing = 3,
        Completed = 4,
		Shipped = 5,
		Delivered = 6,
		Cancelled = 7,
    };
    proto::order::Order_ProcessStatus
}

#[derive(Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::order::Order_ProductEntry", serde_pb_convert)]
struct ProductEntry {
    pub product_id: String,
    pub product_variant_id: String,
    pub quantity: u64,
}

#[derive(Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::order::Order_ShippingEntry", serde_pb_convert)]
struct OrderShippingEntry {
    pub company_id: String,
    pub address_from: String,
    pub address_to: String,
    pub pickup: DateTime<Utc>,
    pub delivery: DateTime<Utc>,
}

#[derive(Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::order::Order", serde_pb_convert)]
struct Order {
    pub id: String,
    pub company_id_from: String,
    pub company_id_to: String,
    pub products: Vec<ProductEntry>,
    pub shipping: OrderShippingEntry,
    pub process_status: OrderProcessStatus,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub history_len: u64,
    pub history_hash: Hash,
}

impl Order {
    pub fn new(id: &str, company_id_from: &str, company_id_to: &str, products: &Vec<ProductEntry>, shipping: &OrderShippingEntry, process_status: OrderProcessStatus, created: &DateTime<Utc>, updated: &DateTime<Utc>, history_len: u64, &history_hash: &Hash) -> Self {
        Self {
            id: id.to_owned(),
            company_id_from: company_id_from.to_owned(),
            company_id_to: company_id_to.to_owned(),
            products: products.clone(),
            shipping: shipping.clone(),
            process_status,
            created: created.clone(),
            updated: updated.clone(),
            history_len: history_len,
            history_hash: history_hash,
        }
    }

    pub fn update_status(&self, process_status: OrderProcessStatus, updated: &DateTime<Utc>, history_hash: &Hash) -> Self {
        Self::new(
            &self.id,
            &self.company_id_from,
            &self.company_id_to,
            &self.products,
            &self.shipping,
            process_status,
            &self.created,
            updated,
            self.history_len + 1,
            history_hash
        )
    }

    pub fn set_shipping(&self, shipping: &OrderShippingEntry, updated: &DateTime<Utc>, history_hash: &Hash) -> Self {
        Self::new(
            &self.id,
            &self.company_id_from,
            &self.company_id_to,
            &self.products,
            shipping,
            self.process_status.clone(),
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
    use crate::util;

    fn make_date() -> DateTime<Utc> {
        chrono::offset::Utc::now()
    }

    fn make_hash() -> Hash {
        Hash::new([1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4])
    }
}

