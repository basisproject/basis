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
    transactions::{company, access},
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

        match schema.get_company_member(&self.company_id, &self.user_id) {
            Some(_) => {}
            None => Err(TransactionError::UserNotFound)?,
        }

        match schema.get_user_by_pubkey(&pubkey) {
            Some(user) => {
                if user.id != self.user_id {
                    company::check(&mut schema, &self.company_id, pubkey, CompanyPermission::LaborSetClock)?;
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
            Err(CommonError::InvalidTime)?;
        }

        schema.labor_create(&self.id, &self.company_id, &self.user_id, &self.created, &hash);
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
            company::check(&mut schema, &labor.company_id, pubkey, CompanyPermission::LaborSetClock)?;
        }

        let start = if self.start == util::time::default_time() { None } else { Some(&self.start) };
        let end = if self.end == util::time::default_time() { None } else { Some(&self.end) };

        if !util::time::is_current(&self.updated) {
            Err(CommonError::InvalidTime)?;
        }
        schema.labor_set_time(labor, start, end, &self.updated, &hash);
        Ok(())
    }
}

