use chrono::{DateTime, Utc};
use validator::Validate;
use exonum::{
    blockchain::{ExecutionError, ExecutionResult, Transaction, TransactionContext},
};
use exonum_merkledb::IndexAccess;
use models::{
    proto,
    company::{Permission as CompanyPermission},
    access::Permission,
    cost_tag::CostTagEntry,
};
use util::{
    self,
    protobuf::empty_opt,
};
use crate::block::{
    schema::Schema,
    transactions::{company, access},
};
use super::CommonError;

#[derive(Debug, Fail)]
#[repr(u8)]
pub enum TransactionError {
    #[fail(display = "Invalid ID")]
    InvalidID = 0,

    #[fail(display = "Company not found")]
    CompanyNotFound = 1,

    #[fail(display = "Cost tag not found")]
    CostTagNotFound = 2,

    #[fail(display = "Cost tag is already deleted")]
    AlreadyDeleted = 3,
}
define_exec_error!(TransactionError);

pub fn validate_cost_tags<T>(schema: &mut Schema<T>, company_id: &str, cost_tags: &Vec<CostTagEntry>) -> Vec<CostTagEntry>
    where T: IndexAccess
{
    cost_tags.clone().into_iter()
        .filter(|entry| {
            match schema.get_cost_tag(&entry.id) {
                Some(tag) => tag.company_id == company_id,
                None => false,
            }
        })
        .collect::<Vec<_>>()
}

deftransaction! {
    #[exonum(pb = "proto::cost_tag::TxCreate")]
    pub struct TxCreate {
        #[validate(custom = "super::validate_uuid")]
        pub id: String,
        #[validate(custom = "super::validate_uuid")]
        pub company_id: String,
        #[validate(length(min = 2))]
        pub name: String,
        pub active: bool,
        pub meta: String,
        pub memo: String,
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

        access::check(&mut schema, pubkey, Permission::CostTagCreate)?;
        company::check(&mut schema, &self.company_id, pubkey, CompanyPermission::CostTagCreate)?;

        match schema.get_company(&self.company_id) {
            Some(_) => {}
            None => Err(TransactionError::CompanyNotFound)?,
        }

        if let Some(_) = schema.get_cost_tag(&self.id) {
            Err(CommonError::IDExists)?;
        }

        if !util::time::is_current(&self.created) {
            Err(CommonError::InvalidTime)?;
        }

        schema.cost_tags_create(&self.id, &self.company_id, &self.name, self.active, &self.meta, &self.created, &hash);
        Ok(())
    }
}
deftransaction! {
    #[exonum(pb = "proto::cost_tag::TxUpdate")]
    pub struct TxUpdate {
        #[validate(custom = "super::validate_uuid")]
        pub id: String,
        #[validate(length(min = 2))]
        pub name: String,
        pub active: bool,
        pub meta: String,
        pub memo: String,
        #[validate(custom = "super::validate_date")]
        pub updated: DateTime<Utc>,
    }
}

impl Transaction for TxUpdate {
    fn execute(&self, context: TransactionContext) -> ExecutionResult {
        validate_transaction!(self);
        let pubkey = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        let cost_tag = schema.get_cost_tag(&self.id)
            .ok_or_else(|| TransactionError::CostTagNotFound)?;

        access::check(&mut schema, pubkey, Permission::CostTagUpdate)?;
        company::check(&mut schema, &cost_tag.company_id, pubkey, CompanyPermission::CostTagUpdate)?;

        if !util::time::is_current(&self.updated) {
            Err(CommonError::InvalidTime)?;
        }

        let name = empty_opt(&self.name).map(|x| x.as_str());
        let active = Some(self.active);
        let meta = empty_opt(&self.meta).map(|x| x.as_str());

        schema.cost_tags_update(cost_tag, name, active, meta, &self.updated, &hash);
        Ok(())
    }
}

deftransaction! {
    #[exonum(pb = "proto::cost_tag::TxDelete")]
    pub struct TxDelete {
        #[validate(custom = "super::validate_uuid")]
        pub id: String,
        pub memo: String,
        #[validate(custom = "super::validate_date")]
        pub deleted: DateTime<Utc>,
    }
}

impl Transaction for TxDelete {
    fn execute(&self, context: TransactionContext) -> ExecutionResult {
        validate_transaction!(self);
        let pubkey = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        let cost_tag = schema.get_cost_tag(&self.id)
            .ok_or_else(|| TransactionError::CostTagNotFound)?;

        access::check(&mut schema, pubkey, Permission::CostTagDelete)?;
        company::check(&mut schema, &cost_tag.company_id, pubkey, CompanyPermission::CostTagDelete)?;

        if cost_tag.is_deleted() {
            Err(TransactionError::AlreadyDeleted)?;
        }

        if !util::time::is_current(&self.deleted) {
            Err(CommonError::InvalidTime)?;
        }

        schema.cost_tags_delete(cost_tag, &self.deleted, &hash);
        Ok(())
    }
}

