use chrono::{DateTime, Utc};
use exonum::{
    blockchain::{ExecutionError, ExecutionResult, Transaction, TransactionContext},
};
use exonum_merkledb::IndexAccess;
use crate::block::{
    schema::Schema,
    models::proto,
    models::company::{Permission as CompanyPermission, Role as CompanyRole},
    models::access::Permission,
    transactions::{company, access},
};
use crate::util;
use super::CommonError;

/// Tells us if the given user is the only owner of a company object
pub fn is_only_owner<T>(schema: &mut Schema<T>, company_id: &str, user_id: &str) -> bool
    where T: IndexAccess
{
    let owners = schema.companies_members(company_id)
        .values()
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

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::company_member::TxCreate")]
pub struct TxCreate {
    pub company_id: String,
    pub user_id: String,
    pub roles: Vec<CompanyRole>,
    pub memo: String,
    pub created: DateTime<Utc>,
}

impl Transaction for TxCreate {
    fn execute(&self, context: TransactionContext) -> ExecutionResult {
        let pubkey = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        if schema.get_company(&self.company_id).is_none() {
            Err(TransactionError::CompanyNotFound)?;
        }

        access::check(&mut schema, pubkey, Permission::CompanyUpdateMembers)?;
        company::check(&mut schema, &self.company_id, pubkey, CompanyPermission::MemberCreate)?;

        if schema.get_user_by_pubkey(pubkey).is_none() {
            Err(CommonError::UserNotFound)?;
        }
        if schema.get_user(&self.user_id).is_none() {
            Err(CommonError::UserNotFound)?;
        }

        if schema.get_company_member(&self.company_id, &self.user_id).is_some() {
            Err(TransactionError::MemberExists)?
        } else if !util::time::is_current(&self.created) {
            Err(CommonError::InvalidTime)?
        } else {
            schema.companies_members_create(&self.company_id, &self.user_id, &self.roles, &self.created, &hash);
            Ok(())
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::company_member::TxSetRoles")]
pub struct TxSetRoles {
    pub company_id: String,
    pub user_id: String,
    pub roles: Vec<CompanyRole>,
    pub memo: String,
    pub updated: DateTime<Utc>,
}

impl Transaction for TxSetRoles {
    fn execute(&self, context: TransactionContext) -> ExecutionResult {
        let pubkey = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        if schema.get_company(&self.company_id).is_none() {
            Err(TransactionError::CompanyNotFound)?;
        }

        access::check(&mut schema, pubkey, Permission::CompanyUpdateMembers)?;
        company::check(&mut schema, &self.company_id, pubkey, CompanyPermission::MemberSetRoles)?;

        if is_only_owner(&mut schema, &self.company_id, &self.user_id) {
            if !&self.roles.contains(&CompanyRole::Owner) {
                Err(TransactionError::MustHaveOwner)?;
            }
        }

        if schema.get_user_by_pubkey(pubkey).is_none() {
            Err(CommonError::UserNotFound)?;
        }
        if schema.get_user(&self.user_id).is_none() {
            Err(CommonError::UserNotFound)?;
        }

        let member = match schema.get_company_member(&self.company_id, &self.user_id) {
            Some(x) => x,
            None => Err(TransactionError::MemberNotFound)?,
        };
        if !util::time::is_current(&self.updated) {
            Err(CommonError::InvalidTime)?
        } else {
            schema.companies_members_set_roles(&self.company_id, member, &self.roles, &self.updated, &hash);
            Ok(())
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::company_member::TxDelete")]
pub struct TxDelete {
    pub company_id: String,
    pub user_id: String,
    pub memo: String,
    pub deleted: DateTime<Utc>,
}

impl Transaction for TxDelete {
    fn execute(&self, context: TransactionContext) -> ExecutionResult {
        let pubkey = &context.author();

        let mut schema = Schema::new(context.fork());

        if schema.get_company(&self.company_id).is_none() {
            Err(TransactionError::CompanyNotFound)?;
        }

        access::check(&mut schema, pubkey, Permission::CompanyUpdateMembers)?;
        company::check(&mut schema, &self.company_id, pubkey, CompanyPermission::MemberDelete)?;

        if is_only_owner(&mut schema, &self.company_id, &self.user_id) {
            Err(TransactionError::MustHaveOwner)?;
        }

        if schema.get_user_by_pubkey(pubkey).is_none() {
            Err(CommonError::UserNotFound)?;
        }
        if schema.get_user(&self.user_id).is_none() {
            Err(CommonError::UserNotFound)?;
        }

        if schema.get_company_member(&self.company_id, &self.user_id).is_none() {
            Err(TransactionError::MemberNotFound)?;
        } else if !util::time::is_current(&self.deleted) {
            Err(CommonError::InvalidTime)?;
        }
        schema.companies_members_delete(&self.company_id, &self.user_id);
        Ok(())
    }
}

