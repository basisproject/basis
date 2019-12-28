use chrono::{DateTime, Utc};
use exonum::{
    blockchain::{ExecutionError, ExecutionResult, Transaction, TransactionContext},
};
use models::{
    proto,
    company::{Permission as CompanyPermission},
    access::Permission,
    product::{Unit, Dimensions, Input, Effort},
};
use crate::block::{
    schema::Schema,
    transactions::{company, access},
};
use util::{self, protobuf::empty_opt};
use super::CommonError;

#[derive(Debug, Fail)]
#[repr(u8)]
pub enum TransactionError {
    #[fail(display = "Invalid ID")]
    InvalidID = 0,

    #[fail(display = "Product not found")]
    ProductNotFound = 1,

    #[fail(display = "Company not found")]
    CompanyNotFound = 2,

    #[fail(display = "Product already deleted")]
    AlreadyDeleted = 4,
}
define_exec_error!(TransactionError);

deftransaction! {
    #[exonum(pb = "proto::product::TxCreate")]
    pub struct TxCreate {
        pub id: String,
        pub company_id: String,
        pub name: String,
        pub unit: Unit,
        pub mass_mg: f64,
        pub dimensions: Dimensions,
        pub inputs: Vec<Input>,
        pub effort: Effort,
        pub active: bool,
        pub meta: String,
        pub created: DateTime<Utc>,
    }
}

impl Transaction for TxCreate {
    fn execute(&self, context: TransactionContext) -> ExecutionResult {
        let pubkey = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        access::check(&mut schema, pubkey, Permission::ProductCreate)?;
        company::check(&mut schema, &self.company_id, pubkey, CompanyPermission::ProductCreate)?;

        if schema.get_product(&self.id).is_some() {
            Err(CommonError::IDExists)?;
        }
        if !util::time::is_current(&self.created) {
            Err(CommonError::InvalidTime)?;
        }
        schema.products_create(&self.id, &self.company_id, &self.name, &self.unit, self.mass_mg, &self.dimensions, &self.inputs, &self.effort, self.active, &self.meta, &self.created, &hash);
        Ok(())
    }
}

deftransaction! {
    #[exonum(pb = "proto::product::TxUpdate")]
    pub struct TxUpdate {
        pub id: String,
        pub name: String,
        pub unit: Unit,
        pub mass_mg: f64,
        pub dimensions: Dimensions,
        pub inputs: Vec<Input>,
        pub effort: Effort,
        pub active: bool,
        pub meta: String,
        pub updated: DateTime<Utc>,
    }
}

impl Transaction for TxUpdate {
    fn execute(&self, context: TransactionContext) -> ExecutionResult {
        let pubkey = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        let prod = schema.get_product(&self.id);
        if prod.is_none() {
            Err(TransactionError::ProductNotFound)?;
        }

        let product = prod.unwrap();
        access::check(&mut schema, pubkey, Permission::ProductUpdate)?;
        company::check(&mut schema, &product.company_id, pubkey, CompanyPermission::ProductUpdate)?;

        let name = empty_opt(&self.name).map(|x| x.as_str());
        let unit = empty_opt(&self.unit);
        let mass_mg = empty_opt(&self.mass_mg).map(|x| x.clone());
        let dimensions = empty_opt(&self.dimensions);
        let inputs = empty_opt(&self.inputs);
        let effort = empty_opt(&self.effort);
        let active = Some(self.active);
        let meta = empty_opt(&self.name).map(|x| x.as_str());

        if !util::time::is_current(&self.updated) {
            Err(CommonError::InvalidTime)?;
        }
        schema.products_update(product, name, unit, mass_mg, dimensions, inputs, effort, active, meta, &self.updated, &hash);
        Ok(())
    }
}

deftransaction! {
    #[exonum(pb = "proto::product::TxDelete")]
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

        if !util::time::is_current(&self.deleted) {
            Err(CommonError::InvalidTime)?;
        }

        let prod = schema.get_product(&self.id);
        if prod.is_none() {
            Err(TransactionError::ProductNotFound)?;
        }
        let product = prod.unwrap();

        access::check(&mut schema, pubkey, Permission::ProductDelete)?;
        company::check(&mut schema, &product.company_id, pubkey, CompanyPermission::ProductDelete)?;

        if product.is_deleted() {
            Err(TransactionError::AlreadyDeleted)?;
        }
        schema.products_delete(product, &self.deleted, &hash);
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use models;
    use util;
    use crate::block::{transactions, schema::Schema};
    use crate::test::{self, gen_uuid};

    #[test]
    fn active_product_index_works() {
        let mut testkit = test::init_testkit();
        let uid = gen_uuid();
        let (tx_user, root_pub, root_sec) = test::tx_superuser(&uid);
        testkit.create_block_with_transactions(txvec![tx_user]);

        let co_id = gen_uuid();
        let co_founder_id = gen_uuid();
        let tx_co = transactions::company::TxCreatePrivate::sign(
            &co_id,
            &String::from("company1@basis.org"),
            &String::from("Widget Builders Inc"),
            &co_founder_id,
            &String::from("Widgets, Builder of"),
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_co]);

        let prod1_id = gen_uuid();
        let tx_prod1 = transactions::product::TxCreate::sign(
            &prod1_id,
            &co_id,
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
        let prod2_id = gen_uuid();
        let tx_prod2 = transactions::product::TxCreate::sign(
            &prod2_id,
            &co_id,
            &String::from("Odd widget"),
            &models::product::Unit::Millimeter,
            &3.7,
            &models::product::Dimensions::new(140.0, 200.0, 1000.0),
            &Vec::new(),
            &models::product::Effort::new(&models::product::EffortTime::Minutes, 6),
            &true,
            &String::from("{}"),
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_prod1, tx_prod2]);

        let snapshot = testkit.snapshot();
        let active_products = Schema::new(&snapshot).get_active_products_for_company(&co_id);
        assert_eq!(active_products.len(), 2);

        let tx_update = transactions::product::TxUpdate::sign(
            &prod1_id,
            &Default::default(),
            &Default::default(),
            &Default::default(),
            &Default::default(),
            &Default::default(),
            &Default::default(),
            &false,
            &Default::default(),
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_update]);

        let snapshot = testkit.snapshot();
        let active_products = Schema::new(&snapshot).get_active_products_for_company(&co_id);
        assert_eq!(active_products.len(), 1);
        assert_eq!(active_products[0].id, prod2_id);

        let tx_update = transactions::product::TxUpdate::sign(
            &prod2_id,
            &Default::default(),
            &Default::default(),
            &Default::default(),
            &Default::default(),
            &Default::default(),
            &Default::default(),
            &false,
            &Default::default(),
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_update]);

        let snapshot = testkit.snapshot();
        let active_products = Schema::new(&snapshot).get_active_products_for_company(&co_id);
        assert_eq!(active_products.len(), 0);

        let tx_update1 = transactions::product::TxUpdate::sign(
            &prod1_id,
            &Default::default(),
            &Default::default(),
            &Default::default(),
            &Default::default(),
            &Default::default(),
            &Default::default(),
            &true,
            &Default::default(),
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        let tx_update2 = transactions::product::TxUpdate::sign(
            &prod2_id,
            &Default::default(),
            &Default::default(),
            &Default::default(),
            &Default::default(),
            &Default::default(),
            &Default::default(),
            &true,
            &Default::default(),
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_update1, tx_update2]);

        let snapshot = testkit.snapshot();
        let active_products = Schema::new(&snapshot).get_active_products_for_company(&co_id);
        assert_eq!(active_products.len(), 2);

        let tx_delete = transactions::product::TxDelete::sign(
            &prod2_id,
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_delete]);

        let snapshot = testkit.snapshot();
        let active_products = Schema::new(&snapshot).get_active_products_for_company(&co_id);
        assert_eq!(active_products.len(), 1);
        assert_eq!(active_products[0].id, prod1_id);

        let tx_delete = transactions::product::TxDelete::sign(
            &prod1_id,
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_delete]);

        let snapshot = testkit.snapshot();
        let active_products = Schema::new(&snapshot).get_active_products_for_company(&co_id);
        assert_eq!(active_products.len(), 0);
    }
}

