use chrono::{DateTime, Utc};
use exonum::{
    blockchain::{ExecutionError, ExecutionResult, Transaction, TransactionContext},
};
use models::{
    proto,
    company::{Permission as CompanyPermission},
    access::Permission,
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
    #[fail(display = "Labor record not found")]
    LaborNotFound = 0,

    #[fail(display = "Company not found")]
    CompanyNotFound = 2,

    #[fail(display = "User not found")]
    UserNotFound = 3,

    #[fail(display = "ID already exists")]
    IDExists = 4,
}
define_exec_error!(TransactionError);

deftransaction! {
    #[exonum(pb = "proto::labor::TxCreate")]
    pub struct TxCreate {
        pub id: String,
        pub company_id: String,
        pub user_id: String,
        pub created: DateTime<Utc>,
    }
}

impl Transaction for TxCreate {
    fn execute(&self, context: TransactionContext) -> ExecutionResult {
        let pubkey = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        access::check(&mut schema, pubkey, Permission::CompanyClockIn)?;

        let member = match schema.get_company_member(&self.company_id, &self.user_id) {
            Some(m) => m,
            None => Err(TransactionError::UserNotFound)?,
        };

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

        schema.labor_create(&self.id, &self.company_id, &self.user_id, &member.occupation, &self.created, &hash);
        Ok(())
    }
}

deftransaction! {
    #[exonum(pb = "proto::labor::TxSetTime")]
    pub struct TxSetTime {
        pub id: String,
        pub start: DateTime<Utc>,
        pub end: DateTime<Utc>,
        pub updated: DateTime<Utc>,
    }
}

impl Transaction for TxSetTime {
    fn execute(&self, context: TransactionContext) -> ExecutionResult {
        let pubkey = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        let lab = schema.get_labor(&self.id);
        if lab.is_none() {
            Err(TransactionError::LaborNotFound)?;
        }
        let labor = lab.unwrap();

        access::check(&mut schema, pubkey, Permission::CompanyClockIn)?;

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

        if !util::time::is_current(&self.updated) {
            Err(CommonError::InvalidTime)?;
        }
        let has_end = end.is_some();
        let company_id = labor.company_id.clone();
        schema.labor_set_time(labor, start, end, &self.updated, &hash);
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

    #[test]
    fn rotating_indexes_work_properly() {
        let mut testkit = test::init_testkit();
        let uid = gen_uuid();
        let (tx_user, root_pub, root_sec) = test::tx_superuser(&uid);
        testkit.create_block_with_transactions(txvec![tx_user]);

        let co1_id = gen_uuid();
        let tx_co1 = transactions::company::TxCreatePrivate::sign(
            &co1_id,
            &String::from("company1@basis.org"),
            &String::from("Widget Builders Inc"),
            &String::from("Master widget builder"),
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
            &labor1_date,
            &root_pub,
            &root_sec
        );
        let tx_labor2 = transactions::labor::TxCreate::sign(
            &labor2_id,
            &co1_id,
            &uid,
            &labor2_date,
            &root_pub,
            &root_sec
        );
        let tx_labor3 = transactions::labor::TxCreate::sign(
            &labor3_id,
            &co1_id,
            &uid,
            &labor3_date,
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_labor1, tx_labor2, tx_labor3]);

        let snapshot = testkit.snapshot();
        let idx = Schema::new(&snapshot).labor_idx_company_id_rolling(&co1_id);
        assert_eq!(idx.keys().count(), 0);

        let labor1_date: DateTime<Utc> = "2018-01-01T04:00:00Z".parse().unwrap();
        let labor2_date: DateTime<Utc> = "2018-07-01T08:00:00Z".parse().unwrap();
        let labor3_date: DateTime<Utc> = "2019-03-01T06:00:00Z".parse().unwrap();
        let tx_labor1 = transactions::labor::TxSetTime::sign(
            &labor1_id,
            &util::time::default_time(),
            &labor1_date,
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_labor1]);

        let snapshot = testkit.snapshot();
        let idx = Schema::new(&snapshot).labor_idx_company_id_rolling(&co1_id);
        assert_eq!(idx.keys().count(), 1);

        let tx_labor2 = transactions::labor::TxSetTime::sign(
            &labor2_id,
            &util::time::default_time(),
            &labor2_date,
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_labor2]);

        let snapshot = testkit.snapshot();
        let idx = Schema::new(&snapshot).labor_idx_company_id_rolling(&co1_id);
        assert_eq!(idx.keys().count(), 2);

        let tx_labor3 = transactions::labor::TxSetTime::sign(
            &labor3_id,
            &util::time::default_time(),
            &labor3_date,
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_labor3]);

        let snapshot = testkit.snapshot();
        let idx = Schema::new(&snapshot).labor_idx_company_id_rolling(&co1_id);
        assert_eq!(idx.keys().count(), 2);
    }
}

