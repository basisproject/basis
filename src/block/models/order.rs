use std::default::Default;
use exonum::crypto::Hash;
use chrono::{DateTime, Utc};
use crate::block::models::proto;
use crate::util;

proto_enum! {
    enum ProcessStatus {
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

#[derive(Clone, Debug, PartialEq, ProtobufConvert)]
#[exonum(pb = "proto::order::Order_ShippingEntry", serde_pb_convert)]
pub struct ShippingEntry {
    pub company_id: String,
    pub address_from: String,
    pub address_to: String,
    pub pickup: DateTime<Utc>,
    pub delivered: DateTime<Utc>,
}

impl ShippingEntry {
    pub fn new(company_id: &str, address_from: &str, address_to: &str, pickup: &DateTime<Utc>, delivered: &DateTime<Utc>) -> Self {
        ShippingEntry {
            company_id: company_id.to_owned(),
            address_from: address_from.to_owned(),
            address_to: address_to.to_owned(),
            pickup: pickup.clone(),
            delivered: delivered.clone(),
        }
    }
}

impl Default for ShippingEntry {
    fn default() -> Self {
        ShippingEntry::new("", "", "", &util::time::default_time(), &util::time::default_time())
    }
}

#[derive(Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::order::Order", serde_pb_convert)]
pub struct Order {
    pub id: String,
    pub company_id_from: String,
    pub company_id_to: String,
    pub products: Vec<ProductEntry>,
    pub shipping: ShippingEntry,
    pub process_status: ProcessStatus,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub history_len: u64,
    pub history_hash: Hash,
}

impl Order {
    pub fn new(id: &str, company_id_from: &str, company_id_to: &str, products: &Vec<ProductEntry>, shipping: &ShippingEntry, process_status: &ProcessStatus, created: &DateTime<Utc>, updated: &DateTime<Utc>, history_len: u64, &history_hash: &Hash) -> Self {
        Self {
            id: id.to_owned(),
            company_id_from: company_id_from.to_owned(),
            company_id_to: company_id_to.to_owned(),
            products: products.clone(),
            shipping: shipping.clone(),
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
            &self.shipping,
            process_status,
            &self.created,
            updated,
            self.history_len + 1,
            history_hash
        )
    }

    pub fn set_shipping(&self, shipping: &ShippingEntry, updated: &DateTime<Utc>, history_hash: &Hash) -> Self {
        Self::new(
            &self.id,
            &self.company_id_from,
            &self.company_id_to,
            &self.products,
            shipping,
            &self.process_status,
            &self.created,
            updated,
            self.history_len + 1,
            history_hash
        )
    }

    pub fn set_shipping_pickup(&self, pickup: &DateTime<Utc>, updated: &DateTime<Utc>, history_hash: &Hash) -> Self {
        let mut shipping = self.shipping.clone();
        shipping.pickup = pickup.clone();

        Self::new(
            &self.id,
            &self.company_id_from,
            &self.company_id_to,
            &self.products,
            &shipping,
            &self.process_status,
            &self.created,
            updated,
            self.history_len + 1,
            history_hash
        )
    }

    pub fn set_shipping_delivered(&self, delivered: &DateTime<Utc>, updated: &DateTime<Utc>, history_hash: &Hash) -> Self {
        let mut shipping = self.shipping.clone();
        shipping.delivered = delivered.clone();

        Self::new(
            &self.id,
            &self.company_id_from,
            &self.company_id_to,
            &self.products,
            &shipping,
            &self.process_status,
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

    fn make_order() -> Order {
        let now = make_date();
        let products = vec![
            ProductEntry::new("ea682431-d0d0-48c5-9166-be5b76a35d62", "a179a5ec-cee2-48ab-99a6-dcb8a3b7cc2e", 183),
            ProductEntry::new("0aabf72f-0cbf-4363-a39d-502be618060d", "d7f3e0eb-4f67-45d5-83ce-1adadb443acb", 1),
        ];
        let shipping = Default::default();
        Order::new(
            "a3c7a63d-e4de-49e3-889d-78853a2169e6",
            "87dc6845-6617-467a-88a3-5aff66ec87a0",
            "20bdec28-e49d-4fc2-be7d-d39eda4ba9f4",
            &products,
            &shipping,
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
        assert_eq!(order.shipping, order2.shipping);
        assert_eq!(order.process_status, ProcessStatus::New);
        assert_eq!(order2.process_status, ProcessStatus::Accepted);
        assert_eq!(order.created, order2.created);
        assert!(order.updated != order2.updated);
        assert_eq!(order2.updated, date2);
    }

    #[test]
    fn sets_shipping() {
        let order = make_order();
        util::sleep(100);
        let date2 = make_date();
        let hash2 = Hash::new([1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4]);
        let shipping = ShippingEntry::new(
            "fc98d311-141e-48b4-8b09-7ee54af9e892",
            "Günther's Fine Gifts, 11169 Hammerschmidt lane, DankeschönFräulein, DE, 12269",
            "2-for-1 Deals Retirement Community, 1457 Fading Willow Lane, Gray Mare, AL, 99999",
            &util::time::default_time(),
            &util::time::default_time()
        );
        let order2 = order.set_shipping(&shipping, &date2, &hash2);
        assert_eq!(order.id, order2.id);
        assert_eq!(order.company_id_from, order2.company_id_from);
        assert_eq!(order.company_id_to, order2.company_id_to);
        assert_eq!(order.products.len(), order2.products.len());
        assert_eq!(order.shipping, Default::default());
        assert_eq!(order2.shipping, shipping);
        assert_eq!(order2.shipping.pickup, util::time::default_time());
        assert_eq!(order2.shipping.delivered, util::time::default_time());
        assert!(shipping != Default::default());
        assert_eq!(order.created, order2.created);
        assert!(order.updated != order2.updated);
        assert_eq!(order2.updated, date2);
        assert_eq!(order2.shipping.address_from, String::from("Günther's Fine Gifts, 11169 Hammerschmidt lane, DankeschönFräulein, DE, 12269"));
        assert_eq!(order.shipping.address_from, String::from(""));
        assert_eq!(order2.history_len, 1);
        assert_eq!(order2.history_len - 1, order.history_len);

        util::sleep(100);
        let date3 = make_date();
        util::sleep(100);
        let pickup = make_date();
        let hash3 = Hash::new([1, 27, 6, 4, 99, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4]);
        let order3 = order2.set_shipping_pickup(&pickup, &date3, &hash3);
        assert_eq!(order2.id, order3.id);
        assert_eq!(order2.company_id_from, order3.company_id_from);
        assert_eq!(order2.company_id_to, order3.company_id_to);
        assert_eq!(order2.products.len(), order3.products.len());
        assert_eq!(order3.shipping.pickup, pickup);
        assert_eq!(order3.shipping.delivered, util::time::default_time());
        assert_eq!(order2.created, order3.created);
        assert!(order2.updated != order3.updated);
        assert_eq!(order3.updated, date3);
        assert_eq!(order3.shipping.address_from, String::from("Günther's Fine Gifts, 11169 Hammerschmidt lane, DankeschönFräulein, DE, 12269"));
        assert_eq!(order3.history_len, 2);
        assert_eq!(order3.history_len - 1, order2.history_len);

        util::sleep(100);
        let date4 = make_date();
        util::sleep(100);
        let delivered = make_date();
        let hash4 = Hash::new([1, 27, 6, 4, 99, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4]);
        let order4 = order3.set_shipping_delivered(&delivered, &date4, &hash4);
        assert_eq!(order3.id, order4.id);
        assert_eq!(order3.company_id_from, order4.company_id_from);
        assert_eq!(order3.company_id_to, order4.company_id_to);
        assert_eq!(order3.products.len(), order4.products.len());
        assert_eq!(order4.shipping.pickup, order3.shipping.pickup);
        assert_eq!(order4.shipping.delivered, delivered);
        assert!(order3.shipping.delivered != order4.shipping.delivered);
        assert_eq!(order3.created, order4.created);
        assert!(order3.updated != order4.updated);
        assert_eq!(order4.updated, date4);
        assert_eq!(order4.shipping.address_to, String::from("2-for-1 Deals Retirement Community, 1457 Fading Willow Lane, Gray Mare, AL, 99999"));
        assert_eq!(order4.history_len, 3);
        assert_eq!(order4.history_len - 1, order3.history_len);
    }
}

