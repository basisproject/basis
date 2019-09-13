use std::default::Default;
use exonum::crypto::Hash;
use chrono::{DateTime, Utc};
use crate::proto;

proto_enum! {
    enum ProcessStatus {
        Unknown = 0,
		New = 1,
		Accepted = 2,
		Processing = 3,
        Completed = 4,
        Proxied = 5,
		Canceled = 6,
    };
    proto::order::Order_ProcessStatus
}

#[derive(Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::order::Order_ProductEntry", serde_pb_convert)]
pub struct ProductEntry {
    pub product_id: String,
    pub product_variant_id: String,
    pub quantity: u64,
}

impl ProductEntry {
    pub fn new(product_id: &str, product_variant_id: &str, quantity: u64) -> Self {
        ProductEntry {
            product_id: product_id.to_owned(),
            product_variant_id: product_variant_id.to_owned(),
            quantity,
        }
    }
}

#[derive(Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::order::Order", serde_pb_convert)]
pub struct Order {
    pub id: String,
    pub company_id_from: String,
    pub company_id_to: String,
    pub products: Vec<ProductEntry>,
    pub process_status: ProcessStatus,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub history_len: u64,
    pub history_hash: Hash,
}

impl Order {
    pub fn new(id: &str, company_id_from: &str, company_id_to: &str, products: &Vec<ProductEntry>, process_status: &ProcessStatus, created: &DateTime<Utc>, updated: &DateTime<Utc>, history_len: u64, &history_hash: &Hash) -> Self {
        Self {
            id: id.to_owned(),
            company_id_from: company_id_from.to_owned(),
            company_id_to: company_id_to.to_owned(),
            products: products.clone(),
            process_status: process_status.clone(),
            created: created.clone(),
            updated: updated.clone(),
            history_len: history_len,
            history_hash: history_hash,
        }
    }

    pub fn update_status(&self, process_status: &ProcessStatus, updated: &DateTime<Utc>, history_hash: &Hash) -> Self {
        Self::new(
            &self.id,
            &self.company_id_from,
            &self.company_id_to,
            &self.products,
            process_status,
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
    use util;

    fn make_date() -> DateTime<Utc> {
        chrono::offset::Utc::now()
    }

    fn make_hash() -> Hash {
        Hash::new([1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4])
    }

    fn make_order() -> Order {
        let now = make_date();
        let products = vec![
            ProductEntry::new("ea682431-d0d0-48c5-9166-be5b76a35d62", "a179a5ec-cee2-48ab-99a6-dcb8a3b7cc2e", 183),
            ProductEntry::new("0aabf72f-0cbf-4363-a39d-502be618060d", "d7f3e0eb-4f67-45d5-83ce-1adadb443acb", 1),
        ];
        Order::new(
            "a3c7a63d-e4de-49e3-889d-78853a2169e6",
            "87dc6845-6617-467a-88a3-5aff66ec87a0",
            "20bdec28-e49d-4fc2-be7d-d39eda4ba9f4",
            &products,
            &ProcessStatus::New,
            &now,
            &now,
            0,
            &make_hash()
        )
    }

    #[test]
    fn changes_status() {
        let order = make_order();
        util::sleep(100);
        let date2 = make_date();
        let hash2 = Hash::new([1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4]);
        let order2 = order.clone().update_status(&ProcessStatus::Accepted, &date2, &hash2);
        assert_eq!(order.id, order2.id);
        assert_eq!(order.company_id_from, order2.company_id_from);
        assert_eq!(order.company_id_to, order2.company_id_to);
        assert_eq!(order.products.len(), order2.products.len());
        assert_eq!(order.process_status, ProcessStatus::New);
        assert_eq!(order2.process_status, ProcessStatus::Accepted);
        assert_eq!(order.created, order2.created);
        assert!(order.updated != order2.updated);
        assert_eq!(order2.updated, date2);
    }
}

