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
            Err(CommonError::IDExists)?;
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

        // create our STINKIN COMPANIES
        let co1_id = gen_uuid();
        let co2_id = gen_uuid();
        let co3_id = gen_uuid();
        let co1_founder_id = gen_uuid();
        let co2_founder_id = gen_uuid();
        let co3_founder_id = gen_uuid();
        let tx_co1 = transactions::company::TxCreatePrivate::sign(
            &co1_id,
            &String::from("company1@basis.org"),
            &String::from("Widget Builders Inc"),
            &co1_founder_id,
            &String::from("Widget builder"),
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        let tx_co2 = transactions::company::TxCreatePrivate::sign(
            &co2_id,
            &String::from("company2@basis.org"),
            &String::from("Widget Distributors Inc"),
            &co2_founder_id,
            &String::from("Widget builder"),
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        let tx_co3 = transactions::company::TxCreatePrivate::sign(
            &co3_id,
            &String::from("company3@basis.org"),
            &String::from("Miner 49er"),
            &co3_founder_id,
            &String::from("Coal miner"),
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_co1, tx_co2, tx_co3]);

        // create our coal product
        let prod_coal_id = gen_uuid();
        let tx_prod = transactions::product::TxCreate::sign(
            &prod_coal_id,
            &co3_id,
            &String::from("Coal"),
            &models::product::Unit::Millimeter,
            &1.0,
            &models::product::Dimensions::new(100.0, 100.0, 100.0),
            &Vec::new(),
            &models::product::Effort::new(&models::product::EffortTime::Minutes, 3),
            &true,
            &String::from("{}"),
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_prod]);

        // tag coal as a resource
        let tag_id = gen_uuid();
        let tx_tag = transactions::resource_tag::TxCreate::sign(
            &tag_id,
            &prod_coal_id,
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_tag]);

        // make some WIDGETS
        let prod_widget_id = gen_uuid();
        let tx_prod = transactions::product::TxCreate::sign(
            &prod_widget_id,
            &co1_id,
            &String::from("Red widget"),
            &models::product::Unit::Millimeter,
            &3.0,
            &models::product::Dimensions::new(100.0, 100.0, 100.0),
            &vec![models::product::Input::new(&prod_coal_id, 6.0)],
            &models::product::Effort::new(&models::product::EffortTime::Minutes, 7),
            &true,
            &String::from("{}"),
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_prod]);

        // log some labor into the widget builder and miner
        let labor1_id = gen_uuid();
        let labor2_id = gen_uuid();
        let tx_labor1 = transactions::labor::TxCreate::sign(
            &labor1_id,
            &co1_id,
            &uid,
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        let tx_labor2 = transactions::labor::TxCreate::sign(
            &labor2_id,
            &co3_id,
            &uid,
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_labor1, tx_labor2]);

        let now = util::time::now();
        let then = now - Duration::hours(8);
        let tx_labor_fin1 = transactions::labor::TxSetTime::sign(
            &labor1_id,
            &then,
            &now,
            &now,
            &root_pub,
            &root_sec
        );
        let tx_labor_fin2 = transactions::labor::TxSetTime::sign(
            &labor2_id,
            &then,
            &now,
            &now,
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_labor_fin1, tx_labor_fin2]);

        // orders!
        let costs = models::costs::Costs::new();
        // widget builder orders coal
        let ord1_id = gen_uuid();
        let ord1_date: DateTime<Utc> = "2018-01-01T04:00:00Z".parse().unwrap();
        let tx_ord1 = transactions::order::TxCreate::sign(
            &ord1_id,
            &co1_id,
            &co3_id,
            &models::order::CostCategory::Operating,
            &vec![models::order::ProductEntry::new(&prod_coal_id, 100.0, &costs, false)],
            &ord1_date,
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_ord1]);
        // finalize
        let tx_ord1_stat = transactions::order::TxUpdateStatus::sign(
            &ord1_id,
            &models::order::ProcessStatus::Finalized,
            &ord1_date,
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_ord1_stat]);

        // widget builder orders coal (again, gets costs of coal into widgets)
        let ord1_1_id = gen_uuid();
        let ord1_1_date: DateTime<Utc> = "2018-01-02T04:00:00Z".parse().unwrap();
        let tx_ord1_1 = transactions::order::TxCreate::sign(
            &ord1_1_id,
            &co1_id,
            &co3_id,
            &models::order::CostCategory::Operating,
            &vec![models::order::ProductEntry::new(&prod_coal_id, 50.0, &costs, false)],
            &ord1_1_date,
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_ord1_1]);
        // finalize
        let tx_ord1_1_stat = transactions::order::TxUpdateStatus::sign(
            &ord1_1_id,
            &models::order::ProcessStatus::Finalized,
            &ord1_1_date,
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_ord1_1_stat]);

        // widget distributor orders widgets
        let ord2_id = gen_uuid();
        let ord3_id = gen_uuid();
        let ord2_date: DateTime<Utc> = "2018-01-03T04:00:00Z".parse().unwrap();
        let ord3_date: DateTime<Utc> = "2018-07-01T08:00:00Z".parse().unwrap();
        let costs = models::costs::Costs::new();
        let tx_ord2 = transactions::order::TxCreate::sign(
            &ord2_id,
            &co2_id,
            &co1_id,
            &models::order::CostCategory::Operating,
            &vec![models::order::ProductEntry::new(&prod_widget_id, 12.0, &costs, false)],
            &ord2_date,
            &root_pub,
            &root_sec
        );
        // opps need maor widgets lol thx
        let tx_ord3 = transactions::order::TxCreate::sign(
            &ord3_id,
            &co2_id,
            &co1_id,
            &models::order::CostCategory::Operating,
            &vec![models::order::ProductEntry::new(&prod_widget_id, 5.0, &costs, false)],
            &ord3_date,
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_ord2, tx_ord3]);

        let snapshot = testkit.snapshot();
        let schema = Schema::new(&snapshot);
        let num_orders = schema.orders().keys().count();
        let idx_from = schema.orders_idx_company_id_from_rolling(&co2_id);
        let idx_to = schema.orders_idx_company_id_to_rolling(&co1_id);
        assert_eq!(num_orders, 4);
        assert_eq!(idx_from.keys().filter(|x| !x.starts_with("_")).count(), 2);
        assert_eq!(idx_to.keys().filter(|x| !x.starts_with("_")).count(), 2);

        let costs_map = schema.costs_aggregate(&co2_id).get("costs.v1").expect("costs.v1 cost map doesn't exist");
        let costs_inputs_map = schema.costs_aggregate(&co2_id).get("costs_inputs.v1").expect("costs_inputs.v1 cost map doesn't exist");
        assert_eq!(costs_map.map_ref().is_empty(), true);
        assert_eq!(costs_inputs_map.map_ref().is_empty(), true);

        // finalize our widget orders, we should start seeing tracking now
        let tx_ord2_stat = transactions::order::TxUpdateStatus::sign(
            &ord2_id,
            &models::order::ProcessStatus::Finalized,
            &ord1_date,
            &root_pub,
            &root_sec
        );
        let tx_ord3_stat = transactions::order::TxUpdateStatus::sign(
            &ord3_id,
            &models::order::ProcessStatus::Finalized,
            &ord1_date,
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_ord2_stat, tx_ord3_stat]);

        let snapshot = testkit.snapshot();
        let schema = Schema::new(&snapshot);
        let num_orders = schema.orders().keys().count();
        let idx_from = schema.orders_idx_company_id_from_rolling(&co2_id);
        let idx_to = schema.orders_idx_company_id_to_rolling(&co1_id);
        let prod_costs = schema.get_product_costs(&prod_widget_id).expect("missing widget product costs");
        assert_eq!(num_orders, 4);
        assert_eq!(idx_from.keys().filter(|x| !x.starts_with("_")).count(), 2);
        assert_eq!(idx_to.keys().filter(|x| !x.starts_with("_")).count(), 2);
        assert_eq!(prod_costs.get(&prod_coal_id), 8.823529411764707);
        assert_eq!(prod_costs.get_labor("Widget builder"), 0.47058823529411764);
        assert_eq!(prod_costs.get_labor("Coal miner"), 0.23529411764705882);

        let costs_map = schema.costs_aggregate(&co2_id).get("costs.v1").expect("costs.v1 cost map doesn't exist");
        let op_costs_bucket = costs_map.map_ref().get("Operating").expect("costs.v1 cost map does not contain `Operating` costs");
        let op_costs = op_costs_bucket.total();
        assert_eq!(op_costs_bucket.len(), 2);
        assert_eq!(op_costs.get(&prod_coal_id), 62.5);
        assert!(op_costs.get_labor("Coal miner") - (1.0 + (2.0 / 3.0)) < 0.000000001);
        assert!(op_costs.get_labor("Widget builder") - (10.0 / 3.0) < 0.000000001);

        // distrib orders more widgets, should cycle out ord2
        let ord4_id = gen_uuid();
        let ord4_date: DateTime<Utc> = "2019-03-01T06:00:00Z".parse().unwrap();
        let tx_ord4 = transactions::order::TxCreate::sign(
            &ord4_id,
            &co2_id,
            &co1_id,
            &models::order::CostCategory::Operating,
            &vec![models::order::ProductEntry::new(&prod_widget_id, 3.0, &costs, false)],
            &ord4_date,
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_ord4]);

        let snapshot = testkit.snapshot();
        let schema = Schema::new(&snapshot);
        let costs_map = schema.costs_aggregate(&co2_id).get("costs.v1").expect("costs.v1 cost map doesn't exist");
        let op_costs_bucket = costs_map.map_ref().get("Operating").expect("costs.v1 cost map does not contain `Operating` costs");
        let op_costs = op_costs_bucket.total();
        assert_eq!(op_costs_bucket.len(), 1);
        assert_eq!(op_costs.get(&prod_coal_id), 62.5);
        assert!(op_costs.get_labor("Coal miner") - (1.0 + (2.0 / 3.0)) < 0.000000001);
        assert!(op_costs.get_labor("Widget builder") - (10.0 / 3.0) < 0.000000001);

        let tx_ord4_stat = transactions::order::TxUpdateStatus::sign(
            &ord4_id,
            &models::order::ProcessStatus::Finalized,
            &ord4_date,
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_ord4_stat]);
        // test for resource tagging
        let order = Schema::new(&snapshot).get_order(&ord1_id).unwrap();
        assert!(order.products[0].is_resource());

        let snapshot = testkit.snapshot();
        let schema = Schema::new(&snapshot);
        let idx_from = schema.orders_idx_company_id_from_rolling(&co2_id);
        let idx_to = schema.orders_idx_company_id_to_rolling(&co1_id);
        assert_eq!(idx_from.keys().filter(|x| !x.starts_with("_")).count(), 2);
        assert_eq!(idx_to.keys().filter(|x| !x.starts_with("_")).count(), 2);
        let costs_map = schema.costs_aggregate(&co2_id).get("costs.v1").expect("costs.v1 cost map doesn't exist");
        let op_costs_bucket = costs_map.map_ref().get("Operating").expect("costs.v1 cost map does not contain `Operating` costs");
        let op_costs = op_costs_bucket.total();
        assert_eq!(op_costs_bucket.len(), 2);
        assert_eq!(op_costs.get(&prod_coal_id), 88.97058823529412);
        assert!(op_costs.get_labor("Coal miner") - 2.3725490196078427 < 0.000000001);
        assert!(op_costs.get_labor("Widget builder") - 4.745098039215685 < 0.000000001);
    }
}

