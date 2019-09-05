// Workaround for `failure` see https://github.com/rust-lang-nursery/failure/issues/223 and
// ECR-1771 for the details.
#![allow(bare_trait_objects)]

use exonum::blockchain::{ExecutionError};

macro_rules! define_exec_error {
    ($item:ty) => {
        impl From<$item> for ExecutionError {
            fn from(value: $item) -> ExecutionError {
                let description = format!("{}", value);
                ExecutionError::with_description(value as u8, description)
            }
        }
    }
}

#[derive(Debug, Fail)]
#[repr(u8)]
pub enum CommonError {
    #[fail(display = "Bad time given (either too far in the future or past)")]
    InvalidTime = 0,

    #[fail(display = "Insufficient privileges")]
    InsufficientPrivileges = 1,

    #[fail(display = "User not found")]
    UserNotFound = 2,
}
define_exec_error!(CommonError);

pub mod access;
pub mod user;
pub mod company;
pub mod company_member;
pub mod product;
pub mod order;

#[derive(Serialize, Deserialize, Clone, Debug, TransactionSet)]
pub enum TransactionGroup {
    UserCreate(user::TxCreate),
    UserUpdate(user::TxUpdate),
    UserSetPubkey(user::TxSetPubkey),
    UserSetRoles(user::TxSetRoles),
    UserDelete(user::TxDelete),

    CompanyCreatePrivate(company::TxCreatePrivate),
    CompanyUpdate(company::TxUpdate),
    CompanySetType(company::TxSetType),
    CompanyDelete(company::TxDelete),

    CompanyMemberCreate(company_member::TxCreate),
    CompanyMemberSetRoles(company_member::TxSetRoles),
    CompanyMemberDelete(company_member::TxDelete),

    ProductCreate(product::TxCreate),
    ProductUpdate(product::TxUpdate),
    ProductSetOption(product::TxSetOption),
    ProductRemoveOption(product::TxRemoveOption),
    ProductSetVariant(product::TxSetVariant),
    ProductUpdateVariant(product::TxUpdateVariant),
    ProductRemoveVariant(product::TxRemoveVariant),
    ProductDelete(product::TxDelete),

    OrderCreate(order::TxCreate),
    OrderUpdateStatus(order::TxUpdateStatus),
    OrderSetShipping(order::TxSetShipping),
}


