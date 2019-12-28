use chrono::{DateTime, Utc};
use exonum::{
    blockchain::{ExecutionError, ExecutionResult, Transaction, TransactionContext},
    crypto::{PublicKey},
};
use exonum_merkledb::IndexAccess;
use models::{
    proto,
    company::{CompanyType, Permission as CompanyPermission, Role as CompanyRole},
    access::Permission,
};
use crate::block::{
    schema::Schema,
    transactions::access,
};
use util::{self, protobuf::empty_opt};
use super::CommonError;

pub fn check<T>(schema: &mut Schema<T>, company_id: &str, pubkey: &PublicKey, permission: CompanyPermission) -> Result<(), CommonError>
    where T: IndexAccess
{
    let user = match schema.get_user_by_pubkey(pubkey) {
        Some(x) => x,
        None => Err(CommonError::UserNotFound)?,
    };
    let member = match schema.get_company_member_by_company_id_user_id(company_id, &user.id) {
        Some(x) => x,
        None => Err(CommonError::InsufficientPrivileges)?,
    };
    for role in &member.roles {
        if role.can(&permission) {
            return Ok(())
        }
    }
    Err(CommonError::InsufficientPrivileges)
}

#[derive(Debug, Fail)]
#[repr(u8)]
pub enum TransactionError {
    #[fail(display = "Invalid ID")]
    InvalidID = 0,

    #[fail(display = "Invalid email")]
    InvalidEmail = 2,

    #[fail(display = "Email already exists")]
    EmailExists = 5,

    #[fail(display = "Company not found")]
    CompanyNotFound = 6,
}
define_exec_error!(TransactionError);

deftransaction! {
    #[exonum(pb = "proto::company::TxCreatePrivate")]
    pub struct TxCreatePrivate {
        pub id: String,
        pub email: String,
        pub name: String,
        pub founder_member_id: String,
        pub founder_occupation: String,
        pub created: DateTime<Utc>,
    }
}

impl Transaction for TxCreatePrivate {
    fn execute(&self, context: TransactionContext) -> ExecutionResult {
        let pubkey = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        access::check(&mut schema, pubkey, Permission::CompanyCreatePrivate)?;
        let user = match schema.get_user_by_pubkey(pubkey) {
            Some(x) => x,
            None => Err(CommonError::UserNotFound)?,
        };

        if schema.get_company_member(&self.founder_member_id).is_some() {
            Err(CommonError::IDExists)?;
        }

        if schema.get_company(&self.id).is_some() {
            Err(CommonError::IDExists)?
        } else if !util::time::is_current(&self.created) {
            Err(CommonError::InvalidTime)?
        } else if !self.email.contains("@") {
            Err(TransactionError::InvalidEmail)?
        } else {
            schema.companies_create(&self.id, &CompanyType::Private, None, &self.email, &self.name, &self.created, &hash);
            schema.companies_members_create(&self.founder_member_id, &self.id, &user.id, &vec![CompanyRole::Owner], &self.founder_occupation, &self.created, &hash);
            Ok(())
        }
    }
}

deftransaction! {
    #[exonum(pb = "proto::company::TxUpdate")]
    pub struct TxUpdate {
        pub id: String,
        pub email: String,
        pub name: String,
        pub updated: DateTime<Utc>,
    }
}

impl Transaction for TxUpdate {
    fn execute(&self, context: TransactionContext) -> ExecutionResult {
        let pubkey = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        access::check(&mut schema, pubkey, Permission::CompanyAdminUpdate)
            .or_else(|_| {
                check(&mut schema, &self.id, pubkey, CompanyPermission::CompanyUpdate)
            })?;

        // because protobuffers are kind of stupid and don't have null
        let email = empty_opt(&self.email).map(|x| x.as_str());
        let name = empty_opt(&self.name).map(|x| x.as_str());

        let company = match schema.get_company(self.id.as_str()) {
            Some(x) => x,
            None => Err(TransactionError::CompanyNotFound)?,
        };
        if let Some(email) = email.as_ref() {
            if !email.contains("@") {
                Err(TransactionError::InvalidEmail)?
            }
        }
        if !util::time::is_current(&self.updated) {
            Err(CommonError::InvalidTime)?
        }

        schema.companies_update(company, email, name, &self.updated, &hash);
        Ok(())
    }
}

deftransaction! {
    #[exonum(pb = "proto::company::TxSetType")]
    pub struct TxSetType {
        pub id: String,
        pub ty: CompanyType,
        pub updated: DateTime<Utc>,
    }
}

impl Transaction for TxSetType {
    fn execute(&self, context: TransactionContext) -> ExecutionResult {
        let pubkey = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        let company = match schema.get_company(self.id.as_str()) {
            Some(x) => x,
            None => Err(TransactionError::CompanyNotFound)?,
        };

        access::check(&mut schema, pubkey, Permission::CompanySetType)?;

        if !util::time::is_current(&self.updated) {
            Err(CommonError::InvalidTime)?
        }

        schema.companies_set_type(company, &self.ty, &self.updated, &hash);
        Ok(())
    }
}

deftransaction! {
    #[exonum(pb = "proto::company::TxDelete")]
    pub struct TxDelete {
        pub id: String,
        pub memo: String,
        pub deleted: DateTime<Utc>,
    }
}

impl Transaction for TxDelete {
    fn execute(&self, context: TransactionContext) -> ExecutionResult {
        let pubkey = &context.author();

        let mut schema = Schema::new(context.fork());

        match schema.get_company(self.id.as_str()) {
            Some(_) => (),
            None => Err(TransactionError::CompanyNotFound)?,
        }


        access::check(&mut schema, pubkey, Permission::CompanyAdminDelete)
            .or_else(|_| {
                check(&mut schema, &self.id, pubkey, CompanyPermission::CompanyDelete)
            })?;

        if !util::time::is_current(&self.deleted) {
            Err(CommonError::InvalidTime)?
        }

        schema.companies_delete(&self.id);
        Ok(())
    }
}

