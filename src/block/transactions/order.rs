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
    transactions::{company, access},
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
        schema.orders_create(&self.id, &self.company_id_from, &self.company_id_to, &self.cost_category, &self.products, &self.created, &hash);
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
        schema.orders_update_status(order, &self.process_status, &self.updated, &hash);
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
        if order.process_status == ProcessStatus::Canceled {
            Err(TransactionError::OrderCanceled)?;
        }

        access::check(&mut schema, pubkey, Permission::OrderUpdate)?;
        company::check(&mut schema, &order.company_id_to, pubkey, CompanyPermission::OrderUpdateProcessStatus)?;

        if !util::time::is_current(&self.updated) {
            Err(CommonError::InvalidTime)?;
        }
        schema.orders_update_cost_category(order, &self.cost_category, &self.updated, &hash);
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
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
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        let tx_co2 = transactions::company::TxCreatePrivate::sign(
            &co2_id,
            &String::from("company2@basis.org"),
            &String::from("Widget Distributors Inc"),
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
            &models::product::Effort::new(&models::product::EffortTime::Minutes, 6),
            &true,
            &String::from("{}"),
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_prod]);

        let ord1_id = gen_uuid();
        let ord2_id = gen_uuid();
        let ord3_id = gen_uuid();
        let tx_ord1 = transactions::order::TxCreate::sign(
            &ord1_id,
            &co2_id,
            &co1_id,
            &models::order::CostCategory::Operating,
            &vec![models::order::ProductEntry::new(&prod_id, 2.0, &models::costs::Costs::new())],
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        let tx_ord2 = transactions::order::TxCreate::sign(
            &ord2_id,
            &co2_id,
            &co1_id,
            &models::order::CostCategory::Operating,
            &vec![models::order::ProductEntry::new(&prod_id, 2.0, &models::costs::Costs::new())],
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        let tx_ord3 = transactions::order::TxCreate::sign(
            &ord3_id,
            &co2_id,
            &co1_id,
            &models::order::CostCategory::Operating,
            &vec![models::order::ProductEntry::new(&prod_id, 2.0, &models::costs::Costs::new())],
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_ord1, tx_ord2, tx_ord3]);

        let snapshot = testkit.snapshot();
        let idx_from = Schema::new(&snapshot).orders_idx_company_id_from_rolling(&co2_id);
        let idx_to = Schema::new(&snapshot).orders_idx_company_id_to_rolling(&co1_id);
        assert_eq!(idx_from.keys().count(), 0);
        assert_eq!(idx_to.keys().count(), 0);

        let ord1_date: DateTime<Utc> = "2018-01-01T00:00:00Z".parse().unwrap();
        let ord2_date: DateTime<Utc> = "2018-07-01T00:00:00Z".parse().unwrap();
        let ord3_date: DateTime<Utc> = "2019-03-01T00:00:00Z".parse().unwrap();
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
        assert_eq!(idx_from.keys().count(), 1);
        assert_eq!(idx_to.keys().count(), 1);

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
        assert_eq!(idx_from.keys().count(), 2);
        assert_eq!(idx_to.keys().count(), 2);

        // this third order pushes the first off the rotate list, so the counts
        // below will both be 2 instead of 3.
        let tx_ord3_stat = transactions::order::TxUpdateStatus::sign(
            &ord3_id,
            &models::order::ProcessStatus::Finalized,
            &ord3_date,
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_ord3_stat]);

        let snapshot = testkit.snapshot();
        let idx_from = Schema::new(&snapshot).orders_idx_company_id_from_rolling(&co2_id);
        let idx_to = Schema::new(&snapshot).orders_idx_company_id_to_rolling(&co1_id);
        assert_eq!(idx_from.keys().count(), 2);
        assert_eq!(idx_to.keys().count(), 2);
        // success!
    }
}

