use chrono::{DateTime, Utc};
use validator::Validate;
use exonum::{
    blockchain::{ExecutionError, ExecutionResult, Transaction, TransactionContext},
};
use models::{
    proto,
    company::{Permission as CompanyPermission},
    access::Permission,
    cost_tag::CostTagEntry,
};
use crate::block::{
    schema::Schema,
    transactions::{company, access, costs, cost_tag},
};
use util::{self, protobuf::empty_opt};
use super::CommonError;

#[derive(Debug, Fail)]
#[repr(u8)]
pub enum TransactionError {
    #[fail(display = "Labor record not found")]
    LaborNotFound = 0,

    #[fail(display = "Company not found")]
    CompanyNotFound = 2,

    #[fail(display = "User not found")]
    UserNotFound = 3,
}
define_exec_error!(TransactionError);

deftransaction! {
    #[exonum(pb = "proto::labor::TxCreate")]
    pub struct TxCreate {
        #[validate(custom = "super::validate_uuid")]
        pub id: String,
        #[validate(custom = "super::validate_uuid")]
        pub company_id: String,
        #[validate(custom = "super::validate_uuid")]
        pub user_id: String,
        pub cost_tags: Vec<CostTagEntry>,
        #[validate(custom = "super::validate_date")]
        pub created: DateTime<Utc>,
    }
}

impl Transaction for TxCreate {
    fn execute(&self, context: TransactionContext) -> ExecutionResult {
        validate_transaction!(self);
        let pubkey = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        access::check(&mut schema, pubkey, Permission::CompanyClockIn)?;

        let member = match schema.get_company_member_by_company_id_user_id(&self.company_id, &self.user_id) {
            Some(m) => m,
            None => Err(TransactionError::UserNotFound)?,
        };

        let can_edit_cost_tags = company::check(&mut schema, &member.company_id, pubkey, CompanyPermission::LaborTagCost).is_ok();
        let cost_tags = if can_edit_cost_tags && self.cost_tags.len() > 0 {
            &self.cost_tags
        } else {
            &member.default_cost_tags
        };
        let cost_tags = cost_tag::validate_cost_tags(&mut schema, &self.company_id, &cost_tags);

        match schema.get_user_by_pubkey(&pubkey) {
            Some(user) => {
                if user.id != self.user_id {
                    company::check(&mut schema, &self.company_id, pubkey, CompanyPermission::LaborSetClock)
                        .or_else(|_| {
                            access::check(&mut schema, pubkey, Permission::CompanyAdminClock)
                        })?;
                }
            }
            None => {
                Err(TransactionError::UserNotFound)?;
            }
        }

        if let Some(_) = schema.get_labor(&self.id) {
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

        schema.labor_create(&self.id, &self.company_id, &self.user_id, &member.occupation, &cost_tags, &self.created, &hash);
        Ok(())
    }
}

deftransaction! {
    #[exonum(pb = "proto::labor::TxUpdate")]
    pub struct TxUpdate {
        #[validate(custom = "super::validate_uuid")]
        pub id: String,
        pub cost_tags: Vec<CostTagEntry>,
        pub start: DateTime<Utc>,
        pub end: DateTime<Utc>,
        #[validate(custom = "super::validate_date")]
        pub updated: DateTime<Utc>,
    }
}

impl Transaction for TxUpdate {
    fn execute(&self, context: TransactionContext) -> ExecutionResult {
        let pubkey = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        let labor = schema.get_labor(&self.id)
            .ok_or_else(|| TransactionError::LaborNotFound)?;

        access::check(&mut schema, pubkey, Permission::CompanyClockIn)?;
        let can_edit_cost_tags = company::check(&mut schema, &labor.company_id, pubkey, CompanyPermission::LaborTagCost).is_ok();

        let user_id = match schema.get_user_by_pubkey(&pubkey) {
            Some(user) => user.id.clone(),
            None => {
                Err(TransactionError::UserNotFound)?
            }
        };

        if user_id != labor.user_id {
            company::check(&mut schema, &labor.company_id, pubkey, CompanyPermission::LaborSetClock)
                .or_else(|_| {
                    access::check(&mut schema, pubkey, Permission::CompanyAdminClock)
                })?;
        }

        let start = if self.start == util::time::default_time() { None } else { Some(&self.start) };
        let end = if self.end == util::time::default_time() { None } else { Some(&self.end) };
        let cost_tags = if can_edit_cost_tags {
            empty_opt(&self.cost_tags)
                .map(|cost_tags| cost_tag::validate_cost_tags(&mut schema, &labor.company_id, cost_tags))
        } else {
            None
        };

        if !util::time::is_current(&self.updated) {
            Err(CommonError::InvalidTime)?;
        }
        let has_end = end.is_some();
        let company_id = labor.company_id.clone();
        schema.labor_update(labor, cost_tags.as_ref(), start, end, &self.updated, &hash);
        if has_end {
            costs::calculate_product_costs(&mut schema, &company_id)?;
        }
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use util;
    use crate::block::{transactions, schema::Schema};
    use crate::test::{self, gen_uuid};
    use models::{
        costs::Costs,
        company,
    };

    #[test]
    fn rotating_indexes_work_properly() {
        let mut testkit = test::init_testkit();
        let uid = gen_uuid();
        let (tx_user, root_pub, root_sec) = test::tx_superuser(&uid);
        testkit.create_block_with_transactions(txvec![tx_user]);

        let co1_id = gen_uuid();
        let co1_founder_id = gen_uuid();
        let ctag1_op_id = gen_uuid();
        let tx_co1 = transactions::company::TxCreatePrivate::sign(
            &co1_id,
            &String::from("company1@basis.org"),
            &String::from("Widget Builders Inc"),
            &vec![company::TxCreatePrivateCostTag::new(&ctag1_op_id, "operating", "")],
            &company::TxCreatePrivateFounder::new(&co1_founder_id, "Master widget builder", &vec![CostTagEntry::new(&ctag1_op_id, 1)]),
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_co1]);

        let labor1_id = gen_uuid();
        let labor2_id = gen_uuid();
        let labor3_id = gen_uuid();
        let labor1_date: DateTime<Utc> = "2018-01-01T00:00:00Z".parse().unwrap();
        let labor2_date: DateTime<Utc> = "2018-07-01T00:00:00Z".parse().unwrap();
        let labor3_date: DateTime<Utc> = "2019-03-01T00:00:00Z".parse().unwrap();
        let tx_labor1 = transactions::labor::TxCreate::sign(
            &labor1_id,
            &co1_id,
            &uid,
            &vec![],
            &labor1_date,
            &root_pub,
            &root_sec
        );
        let tx_labor2 = transactions::labor::TxCreate::sign(
            &labor2_id,
            &co1_id,
            &uid,
            &vec![],
            &labor2_date,
            &root_pub,
            &root_sec
        );
        let tx_labor3 = transactions::labor::TxCreate::sign(
            &labor3_id,
            &co1_id,
            &uid,
            &vec![],
            &labor3_date,
            &root_pub,
            &root_sec
        );
        // NOTE: !! we do NOT save labor2/labor3 !!
        testkit.create_block_with_transactions(txvec![tx_labor1]);

        let snapshot = testkit.snapshot();
        let schema = Schema::new(&snapshot);
        let idx = schema.labor_idx_company_id_rolling(&co1_id);
        // rolling index contains even unfinalized items. tallies are in the
        // aggregates so this should def be 2, because the third labor record
        // above rotates out the first
        assert_eq!(idx.keys().filter(|x| !x.starts_with("_")).count(), 1);
        let tally_map = schema.costs_aggregate(&co1_id).get("labor.v1").expect("labor.v1 cost map doesn't exist");
        match tally_map.map_ref().get("operating") {
            Some(_) => panic!("labor.v1 tally map does not contain `hours` key"),
            None => {},
        }

        testkit.create_block_with_transactions(txvec![tx_labor2]);
        let snapshot = testkit.snapshot();
        let schema = Schema::new(&snapshot);
        let idx = schema.labor_idx_company_id_rolling(&co1_id);
        // rolling index contains even unfinalized items. tallies are in the
        // aggregates so this should def be 2, because the third labor record
        // above rotates out the first
        assert_eq!(idx.keys().filter(|x| !x.starts_with("_")).count(), 2);
        let tally_map = schema.costs_aggregate(&co1_id).get("labor.v1").expect("labor.v1 cost map doesn't exist");
        match tally_map.map_ref().get("hours") {
            Some(_) => panic!("labor.v1 tally map does not contain `hours` key"),
            None => {},
        }

        let labor1_enddate: DateTime<Utc> = "2018-01-01T04:00:00Z".parse().unwrap();
        let labor2_enddate: DateTime<Utc> = "2018-07-01T08:00:00Z".parse().unwrap();
        let labor3_enddate: DateTime<Utc> = "2019-03-01T06:00:00Z".parse().unwrap();

        let tx_labor1 = transactions::labor::TxUpdate::sign(
            &labor1_id,
            &vec![],
            &util::time::default_time(),
            &labor1_enddate,
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_labor1]);

        let snapshot = testkit.snapshot();
        let schema = Schema::new(&snapshot);
        let idx = schema.labor_idx_company_id_rolling(&co1_id);
        let tally_map = schema.costs_aggregate(&co1_id).get("labor.v1").expect("labor.v1 cost map doesn't exist");
        let tally = tally_map.map_ref().get(&ctag1_op_id).expect("operating key in labor costs map doesn't exist");
        assert_eq!(idx.keys().filter(|x| !x.starts_with("_")).count(), 2);
        assert_eq!(tally.len(), 1);
        assert_eq!(tally.total(), Costs::new_with_labor("Master widget builder", 4.0));

        let tx_labor2 = transactions::labor::TxUpdate::sign(
            &labor2_id,
            &vec![],
            &util::time::default_time(),
            &labor2_enddate,
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_labor2]);

        let snapshot = testkit.snapshot();
        let schema = Schema::new(&snapshot);
        let idx = schema.labor_idx_company_id_rolling(&co1_id);
        let tally_map = schema.costs_aggregate(&co1_id).get("labor.v1").expect("labor.v1 cost map doesn't exist");
        let tally = tally_map.map_ref().get(&ctag1_op_id).expect("hours key in labor costs map doesn't exist");
        assert_eq!(idx.keys().filter(|x| !x.starts_with("_")).count(), 2);
        assert_eq!(tally.len(), 2);
        assert_eq!(tally.total(), Costs::new_with_labor("Master widget builder", 4.0 + 8.0));

        testkit.create_block_with_transactions(txvec![tx_labor3]);
        let snapshot = testkit.snapshot();
        let schema = Schema::new(&snapshot);
        let idx = schema.labor_idx_company_id_rolling(&co1_id);
        let tally_map = schema.costs_aggregate(&co1_id).get("labor.v1").expect("labor.v1 cost map doesn't exist");
        let tally = tally_map.map_ref().get(&ctag1_op_id).expect("hours key in labor costs map doesn't exist");
        assert_eq!(idx.keys().filter(|x| !x.starts_with("_")).count(), 2);
        assert_eq!(tally.total(), Costs::new_with_labor("Master widget builder", (4.0 + 8.0) - 4.0));

        let tx_labor3 = transactions::labor::TxUpdate::sign(
            &labor3_id,
            &vec![],
            &util::time::default_time(),
            &labor3_enddate,
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_labor3]);

        let snapshot = testkit.snapshot();
        let schema = Schema::new(&snapshot);
        let idx = schema.labor_idx_company_id_rolling(&co1_id);
        let tally_map = schema.costs_aggregate(&co1_id).get("labor.v1").expect("labor.v1 cost map doesn't exist");
        let tally = tally_map.map_ref().get(&ctag1_op_id).expect("hours key in labor costs map doesn't exist");
        assert_eq!(idx.keys().filter(|x| !x.starts_with("_")).count(), 2);
        assert_eq!(tally.len(), 2);
        assert_eq!(tally.total(), Costs::new_with_labor("Master widget builder", (4.0 + 8.0 + 6.0) - 4.0));
    }
}

