use std::default::Default;
use exonum::crypto::Hash;
use chrono::{DateTime, Utc};
use crate::{
    costs::Costs,
    proto,
};

proto_enum! {
    enum ProcessStatus {
        Unknown = 0,
		New = 1,
		Accepted = 2,
		Processing = 3,
        Completed = 4,
        Proxying = 5,
        Proxied = 6,
		Canceled = 7,
    };
    proto::order::Order_ProcessStatus
}

proto_enum! {
    enum CostCategory {
        UnknownCategory = 0,
        Inventory = 1,
        Operating = 2,
    };
    proto::order::Order_CostCategory
}

#[derive(Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::order::Order_ProductEntry", serde_pb_convert)]
pub struct ProductEntry {
    pub product_id: String,
    pub quantity: f64,
    pub costs: Costs,
}

impl ProductEntry {
    pub fn new(product_id: &str, quantity: f64, costs: &Costs) -> Self {
        Self {
            product_id: product_id.to_owned(),
            quantity,
            costs: costs.clone(),
        }
    }
}

#[derive(Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::order::Order", serde_pb_convert)]
pub struct Order {
    pub id: String,
    pub company_id_from: String,
    pub company_id_to: String,
    pub cost_category: CostCategory,
    pub products: Vec<ProductEntry>,
    pub process_status: ProcessStatus,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub history_len: u64,
    pub history_hash: Hash,
}

impl Order {
    pub fn new(id: &str, company_id_from: &str, company_id_to: &str, cost_category: &CostCategory, products: &Vec<ProductEntry>, process_status: &ProcessStatus, created: &DateTime<Utc>, updated: &DateTime<Utc>, history_len: u64, &history_hash: &Hash) -> Self {
        Self {
            id: id.to_owned(),
            company_id_from: company_id_from.to_owned(),
            company_id_to: company_id_to.to_owned(),
            cost_category: cost_category.clone(),
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
            &self.cost_category,
            &self.products,
            process_status,
            &self.created,
            updated,
            self.history_len + 1,
            history_hash
        )
    }

    pub fn update_cost_category(&self, cost_category: &CostCategory, updated: &DateTime<Utc>, history_hash: &Hash) -> Self {
        Self::new(
            &self.id,
            &self.company_id_from,
            &self.company_id_to,
            cost_category,
            &self.products,
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
    use util;
    use crate::{
        costs::Costs,
    };

    fn make_date() -> DateTime<Utc> {
        chrono::offset::Utc::now()
    }

    fn make_hash() -> Hash {
        Hash::new([1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4])
    }

    fn make_order() -> Order {
        let now = make_date();
        let mut cists1 = Costs::new();
        let mut cists2 = Costs::new();
        cists1.track("1234", 6969.0);
        cists2.track("5678", 1212.0);
        let products = vec![
            ProductEntry::new("ea682431-d0d0-48c5-9166-be5b76a35d62", 183.0, &cists1),
            ProductEntry::new("0aabf72f-0cbf-4363-a39d-502be618060d", 1.0, &cists2),
        ];
        Order::new(
            "a3c7a63d-e4de-49e3-889d-78853a2169e6",
            "87dc6845-6617-467a-88a3-5aff66ec87a0",
            "20bdec28-e49d-4fc2-be7d-d39eda4ba9f4",
            &CostCategory::Operating,
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
        assert_eq!(order.cost_category, order2.cost_category);
        assert_eq!(order.products.len(), order2.products.len());
        assert_eq!(order.process_status, ProcessStatus::New);
        assert_eq!(order2.process_status, ProcessStatus::Accepted);
        assert_eq!(order.created, order2.created);
        assert!(order.updated != order2.updated);
        assert_eq!(order2.updated, date2);
    }

    #[test]
    fn changes_category() {
        let order = make_order();
        util::sleep(100);
        let date2 = make_date();
        let hash2 = Hash::new([1, 28, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4]);
        let order2 = order.clone().update_cost_category(&CostCategory::Inventory, &date2, &hash2);
        assert_eq!(order.id, order2.id);
        assert_eq!(order.company_id_from, order2.company_id_from);
        assert_eq!(order.company_id_to, order2.company_id_to);
        assert_eq!(order.cost_category, CostCategory::Operating);
        assert_eq!(order2.cost_category, CostCategory::Inventory);
        assert_eq!(order.products.len(), order2.products.len());
        assert_eq!(order.process_status, order2.process_status);
        assert_eq!(order.created, order2.created);
        assert!(order.updated != order2.updated);
        assert_eq!(order2.updated, date2);
    }
}

