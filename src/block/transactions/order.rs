use chrono::{DateTime, Utc};
use exonum::{
    blockchain::{ExecutionError, ExecutionResult, Transaction, TransactionContext},
};
use models::{
    proto,
    company::{Permission as CompanyPermission},
    access::Permission,
    order::{ProductEntry, ProcessStatus, CostCategory},
};
use crate::block::{
    schema::Schema,
    transactions::{company, access, costs},
};
use util;
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

    #[fail(display = "Cannot update a canceled order")]
    OrderCanceled = 3,

    #[fail(display = "Company not found")]
    CompanyNotFound = 4,

    #[fail(display = "Product not found")]
    ProductNotFound = 5,

    #[fail(display = "Product is missing costs")]
    CostsNotFound = 6,
}
define_exec_error!(TransactionError);

deftransaction! {
    #[exonum(pb = "proto::order::TxCreate")]
    pub struct TxCreate {
        pub id: String,
        pub company_id_from: String,
        pub company_id_to: String,
        pub cost_category: CostCategory,
        pub products: Vec<ProductEntry>,
        pub created: DateTime<Utc>,
    }
}

impl Transaction for TxCreate {
    fn execute(&self, context: TransactionContext) -> ExecutionResult {
        let pubkey = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        access::check(&mut schema, pubkey, Permission::OrderCreate)?;
        company::check(&mut schema, &self.company_id_from, pubkey, CompanyPermission::OrderCreate)?;

        match schema.get_company(&self.company_id_to) {
            Some(x) => {
                if !x.is_active() {
                    Err(TransactionError::CompanyNotFound)?;
                }
            }
            None => Err(TransactionError::CompanyNotFound)?,
        }

        let mut products = self.products.clone();
        for product in &mut products {
            match schema.get_product_with_costs_tagged(&product.product_id) {
                (Some(prod), Some(costs), tag) => {
                    if !prod.is_active() {
                        Err(TransactionError::ProductNotFound)?;
                    }
                    product.costs = costs;
                    product.resource = tag.is_some();
                }
                (Some(_), None, _) => {
                    Err(TransactionError::CostsNotFound)?;
                }
                _ => {
                    Err(TransactionError::ProductNotFound)?;
                }
            }
        }

        if schema.get_order(&self.id).is_some() {
            Err(TransactionError::IDExists)?;
        }
        if !util::time::is_current(&self.created) {
            match access::check(&mut schema, pubkey, Permission::TimeTravel) {
                Ok(_) => {}
                Err(_) => {
                    Err(CommonError::InvalidTime)?;
                }
            }
        }
        schema.orders_create(&self.id, &self.company_id_from, &self.company_id_to, &self.cost_category, &products, &self.created, &hash);
        costs::calculate_product_costs(&mut schema, &self.company_id_from)?;
        costs::calculate_product_costs(&mut schema, &self.company_id_to)?;
        Ok(())
    }
}

deftransaction!{
    #[exonum(pb = "proto::order::TxUpdateStatus")]
    pub struct TxUpdateStatus {
        pub id: String,
        pub process_status: ProcessStatus,
        pub updated: DateTime<Utc>,
    }
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
        if order.process_status == ProcessStatus::Canceled {
            Err(TransactionError::OrderCanceled)?;
        }

        access::check(&mut schema, pubkey, Permission::OrderUpdate)?;
        company::check(&mut schema, &order.company_id_to, pubkey, CompanyPermission::OrderUpdateProcessStatus)?;

        if !util::time::is_current(&self.updated) {
            match access::check(&mut schema, pubkey, Permission::TimeTravel) {
                Ok(_) => {}
                Err(_) => {
                    Err(CommonError::InvalidTime)?;
                }
            }
        }
        let company_id_from = order.company_id_from.clone();
        let company_id_to = order.company_id_to.clone();
        schema.orders_update_status(order, &self.process_status, &self.updated, &hash);
        costs::calculate_product_costs(&mut schema, &company_id_from)?;
        costs::calculate_product_costs(&mut schema, &company_id_to)?;
        Ok(())
    }
}

deftransaction! {
    #[exonum(pb = "proto::order::TxUpdateCostCategory")]
    pub struct TxUpdateCostCategory {
        pub id: String,
        pub cost_category: CostCategory,
        pub updated: DateTime<Utc>,
    }
}

impl Transaction for TxUpdateCostCategory {
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
        company::check(&mut schema, &order.company_id_from, pubkey, CompanyPermission::OrderUpdateCostCategory)?;

        if !util::time::is_current(&self.updated) {
            Err(CommonError::InvalidTime)?;
        }

        let company_id_from = order.company_id_from.clone();
        let company_id_to = order.company_id_to.clone();
        schema.orders_update_cost_category(order, &self.cost_category, &self.updated, &hash);
        costs::calculate_product_costs(&mut schema, &company_id_from)?;
        costs::calculate_product_costs(&mut schema, &company_id_to)?;
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use chrono::{DateTime, Utc, Duration};
    use models;
    use util;
    use crate::block::{transactions, schema::Schema};
    use crate::test::{self, gen_uuid};

    #[test]
    fn rotating_indexes_work_properly() {
        let mut testkit = test::init_testkit();
        let uid = gen_uuid();
        let (tx_user, root_pub, root_sec) = test::tx_superuser(&uid);
        testkit.create_block_with_transactions(txvec![tx_user]);

        let co1_id = gen_uuid();
        let co2_id = gen_uuid();
        let tx_co1 = transactions::company::TxCreatePrivate::sign(
            &co1_id,
            &String::from("company1@basis.org"),
            &String::from("Widget Builders Inc"),
            &String::from("Widget builder"),
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        let tx_co2 = transactions::company::TxCreatePrivate::sign(
            &co2_id,
            &String::from("company2@basis.org"),
            &String::from("Widget Distributors Inc"),
            &String::from("Widget builder"),
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_co1, tx_co2]);

        let prod_id = gen_uuid();
        let tx_prod = transactions::product::TxCreate::sign(
            &prod_id,
            &co1_id,
            &String::from("Red widget"),
            &models::product::Unit::Millimeter,
            &3.0,
            &models::product::Dimensions::new(100.0, 100.0, 100.0),
            &Vec::new(),
            &models::product::Effort::new(&models::product::EffortTime::Minutes, 7),
            &true,
            &String::from("{}"),
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_prod]);

        let tag_id = gen_uuid();
        let tx_tag = transactions::resource_tag::TxCreate::sign(
            &tag_id,
            &prod_id,
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_tag]);

        let labor_id = gen_uuid();
        let tx_labor1 = transactions::labor::TxCreate::sign(
            &labor_id,
            &co1_id,
            &uid,
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_labor1]);

        let now = util::time::now();
        let then = now - Duration::hours(8);
        let tx_labor2 = transactions::labor::TxSetTime::sign(
            &labor_id,
            &then,
            &now,
            &now,
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_labor2]);

        let ord1_id = gen_uuid();
        let ord2_id = gen_uuid();
        let ord3_id = gen_uuid();
        let tx_ord1 = transactions::order::TxCreate::sign(
            &ord1_id,
            &co2_id,
            &co1_id,
            &models::order::CostCategory::Operating,
            &vec![models::order::ProductEntry::new(&prod_id, 2.0, &models::costs::Costs::new(), false)],
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        let tx_ord2 = transactions::order::TxCreate::sign(
            &ord2_id,
            &co2_id,
            &co1_id,
            &models::order::CostCategory::Operating,
            &vec![models::order::ProductEntry::new(&prod_id, 2.0, &models::costs::Costs::new(), false)],
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        let tx_ord3 = transactions::order::TxCreate::sign(
            &ord3_id,
            &co2_id,
            &co1_id,
            &models::order::CostCategory::Operating,
            &vec![models::order::ProductEntry::new(&prod_id, 2.0, &models::costs::Costs::new(), false)],
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_ord1, tx_ord2, tx_ord3]);

        let snapshot = testkit.snapshot();
        let num_orders = Schema::new(&snapshot).orders().keys().count();
        let idx_from = Schema::new(&snapshot).orders_idx_company_id_from_rolling(&co2_id);
        let idx_to = Schema::new(&snapshot).orders_idx_company_id_to_rolling(&co1_id);
        assert_eq!(num_orders, 3);
        assert_eq!(idx_from.keys().count(), 3);
        assert_eq!(idx_to.keys().count(), 3);

        // test for resource tagging
        let order = Schema::new(&snapshot).get_order(&ord1_id).unwrap();
        assert!(order.products[0].is_resource());

        let ord1_date: DateTime<Utc> = "2018-01-01T00:00:00Z".parse().unwrap();
        let ord2_date: DateTime<Utc> = "2018-07-01T00:00:00Z".parse().unwrap();
        let tx_ord1_stat = transactions::order::TxUpdateStatus::sign(
            &ord1_id,
            &models::order::ProcessStatus::Finalized,
            &ord1_date,
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_ord1_stat]);

        let snapshot = testkit.snapshot();
        let idx_from = Schema::new(&snapshot).orders_idx_company_id_from_rolling(&co2_id);
        let idx_to = Schema::new(&snapshot).orders_idx_company_id_to_rolling(&co1_id);
        assert_eq!(idx_from.keys().count(), 3);
        assert_eq!(idx_to.keys().count(), 3);

        let tx_ord2_stat = transactions::order::TxUpdateStatus::sign(
            &ord2_id,
            &models::order::ProcessStatus::Finalized,
            &ord2_date,
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_ord2_stat]);

        let snapshot = testkit.snapshot();
        let idx_from = Schema::new(&snapshot).orders_idx_company_id_from_rolling(&co2_id);
        let idx_to = Schema::new(&snapshot).orders_idx_company_id_to_rolling(&co1_id);
        assert_eq!(idx_from.keys().count(), 3);
        assert_eq!(idx_to.keys().count(), 3);
    }
}

