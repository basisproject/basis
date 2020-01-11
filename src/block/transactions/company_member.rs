use chrono::{DateTime, Utc};
use validator::Validate;
use exonum::{
    blockchain::{ExecutionError, ExecutionResult, Transaction, TransactionContext},
};
use exonum_merkledb::IndexAccess;
use models::{
    proto,
    company::{Permission as CompanyPermission, Role as CompanyRole},
    access::Permission,
    cost_tag::CostTagEntry,
};
use util::{
    self,
    protobuf::empty_opt,
};
use crate::block::{
    schema::Schema,
    transactions::{company, access, cost_tag},
};
use super::CommonError;

/// Tells us if the given user is the only owner of a company object
pub fn is_only_owner<T>(schema: &mut Schema<T>, company_id: &str, user_id: &str) -> bool
    where T: IndexAccess
{
    let owners = schema.companies_members_idx_company_id(company_id)
        .values()
        .map(|member_id| schema.get_company_member(&member_id))
        .filter(|m| m.is_some())
        .map(|m| m.unwrap())
        .filter(|m| m.roles.contains(&CompanyRole::Owner))
        .map(|m| m.user_id.to_owned())
        .collect::<Vec<_>>();
    owners.len() == 1 && owners.contains(&user_id.to_owned())
}

#[derive(Debug, Fail)]
#[repr(u8)]
pub enum TransactionError {
    #[fail(display = "Invalid ID")]
    InvalidID = 0,

    #[fail(display = "Company not found")]
    CompanyNotFound = 1,

    #[fail(display = "That user is already a member of the company")]
    MemberExists = 2,

    #[fail(display = "User not found")]
    MemberNotFound = 3,

    #[fail(display = "Company must have at least one owner")]
    MustHaveOwner = 4,
}
define_exec_error!(TransactionError);

deftransaction! {
    #[exonum(pb = "proto::company_member::TxCreate")]
    pub struct TxCreate {
        #[validate(custom = "super::validate_uuid")]
        pub id: String,
        #[validate(custom = "super::validate_uuid")]
        pub company_id: String,
        #[validate(custom = "super::validate_uuid")]
        pub user_id: String,
        pub roles: Vec<CompanyRole>,
        pub occupation: String,
        pub default_cost_tags: Vec<CostTagEntry>,
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

        if schema.get_company_member(&self.id).is_some() {
            Err(CommonError::IDExists)?;
        }

        if schema.get_company(&self.company_id).is_none() {
            Err(TransactionError::CompanyNotFound)?;
        }

        access::check(&mut schema, pubkey, Permission::CompanyUpdateMembers)?;
        company::check(&mut schema, &self.company_id, pubkey, CompanyPermission::MemberCreate)?;
        let default_cost_tags = match company::check(&mut schema, &self.company_id, pubkey, CompanyPermission::LaborTagCost) {
            Ok(_) => cost_tag::validate_cost_tags(&mut schema, &self.company_id, &self.default_cost_tags),
            Err(_) => vec![],
        };

        if schema.get_user(&self.user_id).is_none() {
            Err(CommonError::UserNotFound)?;
        }

        if schema.get_company_member_by_company_id_user_id(&self.company_id, &self.user_id).is_some() {
            Err(TransactionError::MemberExists)?
        } else if !util::time::is_current(&self.created) {
            Err(CommonError::InvalidTime)?
        } else {
            schema.companies_members_create(&self.id, &self.company_id, &self.user_id, &self.roles, &self.occupation, &default_cost_tags, &self.created, &hash);
            Ok(())
        }
    }
}

deftransaction! {
    #[exonum(pb = "proto::company_member::TxUpdate")]
    pub struct TxUpdate {
        #[validate(custom = "super::validate_uuid")]
        pub id: String,
        pub roles: Vec<CompanyRole>,
        pub occupation: String,
        pub default_cost_tags: Vec<CostTagEntry>,
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

        let member = match schema.get_company_member(&self.id) {
            Some(x) => x,
            None => Err(TransactionError::MemberNotFound)?,
        };

        if schema.get_company(&member.company_id).is_none() {
            Err(TransactionError::CompanyNotFound)?;
        }

        access::check(&mut schema, pubkey, Permission::CompanyUpdateMembers)?;
        company::check(&mut schema, &member.company_id, pubkey, CompanyPermission::MemberSetRoles)?;
        let can_edit_cost_tags = company::check(&mut schema, &member.company_id, pubkey, CompanyPermission::LaborTagCost).is_ok();

        if schema.get_user(&member.user_id).is_none() {
            Err(CommonError::UserNotFound)?;
        }

        if is_only_owner(&mut schema, &member.company_id, &member.user_id) {
            if !&self.roles.contains(&CompanyRole::Owner) {
                Err(TransactionError::MustHaveOwner)?;
            }
        }

        if !util::time::is_current(&self.updated) {
            Err(CommonError::InvalidTime)?
        } else {
            let roles = empty_opt(&self.roles);
            let occupation = empty_opt(&self.occupation).map(|x| x.as_str());
            let default_cost_tags = if can_edit_cost_tags {
                empty_opt(&self.default_cost_tags)
                    .map(|cost_tags| cost_tag::validate_cost_tags(&mut schema, &member.company_id, cost_tags))
            } else {
                None
            };
            schema.companies_members_update(member, roles, occupation, default_cost_tags.as_ref(), &self.updated, &hash);
            Ok(())
        }
    }
}

deftransaction! {
    #[exonum(pb = "proto::company_member::TxDelete")]
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

        let mut schema = Schema::new(context.fork());

        let member = match schema.get_company_member(&self.id) {
            Some(x) => x,
            None => Err(TransactionError::MemberNotFound)?,
        };

        if schema.get_company(&member.company_id).is_none() {
            Err(TransactionError::CompanyNotFound)?;
        }

        access::check(&mut schema, pubkey, Permission::CompanyUpdateMembers)?;
        company::check(&mut schema, &member.company_id, pubkey, CompanyPermission::MemberDelete)?;

        if is_only_owner(&mut schema, &member.company_id, &member.user_id) {
            Err(TransactionError::MustHaveOwner)?;
        }

        if !util::time::is_current(&self.deleted) {
            Err(CommonError::InvalidTime)?;
        }
        schema.companies_members_delete(member);
        Ok(())
    }
}

