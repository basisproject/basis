// Workaround for `failure` see https://github.com/rust-lang-nursery/failure/issues/223 and
// ECR-1771 for the details.
#![allow(bare_trait_objects)]

use validator::ValidationError;
use exonum::blockchain::{ExecutionError};
use chrono::{DateTime, Utc};

lazy_static! {
    static ref REGEX_UUID: regex::Regex = regex::Regex::new(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$").unwrap();
}

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
    #[fail(display = "Bad time given (is blank, too far in the future, or too far in the past)")]
    InvalidTime = 0,

    #[fail(display = "Insufficient privileges")]
    InsufficientPrivileges = 1,

    #[fail(display = "User not found")]
    UserNotFound = 2,

    #[fail(display = "Cannot calculate costs")]
    CostError = 3,

    #[fail(display = "ID already exists")]
    IDExists = 5,

    #[fail(display = "Invalid ID")]
    InvalidID = 6,

    #[fail(display = "Invalid email")]
    InvalidEmail = 7,

    #[fail(display = "Invalid enum val")]
    InvalidEnum = 8,

    #[fail(display = "Validation error")]
    ValidationError = 9,
}
define_exec_error!(CommonError);

#[macro_export]
macro_rules! deftransaction {
    (
        $( #[$met:meta] )*
        pub struct $name:ident {
            $(
                $( #[$fmet:meta] )*
                pub $field:ident: $ty:ty, 
            )*
        }
    ) => {
        #[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert, Validate)]
        $( #[$met] )*
        pub struct $name {
            $(
                $( #[$fmet] )*
                pub $field: $ty, 
            )*
        }

        impl $name {
            #[allow(dead_code)]
            pub fn sign( $( $field: &$ty, )* pk: &exonum::crypto::PublicKey, sk: &exonum::crypto::SecretKey) -> exonum::messages::Signed<exonum::messages::RawTransaction> {
                exonum::messages::Message::sign_transaction(
                    Self {
                        $( $field: $field.clone(), )*
                    },
                    crate::block::SERVICE_ID,
                    *pk,
                    sk,
                )
            }
        }
    };
}

#[macro_export]
macro_rules! validate_transaction {
    ( $obj:expr ) => {
        match $obj.validate() {
            Ok(_) => {}
            Err(e) => {
                warn!("validate_transaction() -- {}", e);
                for (_field, errs) in e.field_errors() {
                    for err in errs {
                        let errcode: String = err.code.clone().into();
                        match errcode.as_str() {
                            "uuid" => Err(CommonError::InvalidID),
                            "email" => Err(CommonError::InvalidEmail),
                            "date" => Err(CommonError::InvalidTime),
                            "enum" => Err(CommonError::InvalidEnum),
                            _ => Err(CommonError::ValidationError),
                        }?;
                        break;
                    }
                    break;
                }
            }
        }
    }
}

pub fn validate_uuid(uuid: &str) -> Result<(), ValidationError> {
    if !REGEX_UUID.is_match(uuid) {
        return Err(ValidationError::new("uuid"));
    }
    Ok(())
}

pub fn validate_enum<T>(enumval: &T) -> Result<(), ValidationError>
    where T: Default + PartialEq
{
    if enumval == &Default::default() {
        return Err(ValidationError::new("enum"));
    }
    Ok(())
}

pub fn validate_date(date: &DateTime<Utc>) -> Result<(), ValidationError> {
    if date == &util::time::default_time() {
        return Err(ValidationError::new("date"));
    }
    Ok(())
}

pub mod access;
pub mod user;
pub mod company;
pub mod company_member;
pub mod costs;
pub mod labor;
pub mod product;
pub mod resource_tag;
pub mod order;
pub mod cost_tag;

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
    CompanyMemberUpdate(company_member::TxUpdate),
    CompanyMemberDelete(company_member::TxDelete),

    LaborCreate(labor::TxCreate),
    LaborSetTime(labor::TxUpdate),
    LaborSetWage(labor::TxSetWage),

    ProductCreate(product::TxCreate),
    ProductUpdate(product::TxUpdate),
    ProductDelete(product::TxDelete),

    ResourceTagCreate(resource_tag::TxCreate),
    ResourceTagDelete(resource_tag::TxDelete),

    OrderCreate(order::TxCreate),
    OrderUpdateStatus(order::TxUpdateStatus),
    OrderUpdateCostTags(order::TxUpdateCostTags),

    CostTagCreate(cost_tag::TxCreate),
    CostTagUpdate(cost_tag::TxUpdate),
    CostTagDelete(cost_tag::TxDelete),
}

