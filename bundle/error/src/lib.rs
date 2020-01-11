// Workaround for `failure` see https://github.com/rust-lang-nursery/failure/issues/223 and
// ECR-1771 for the details.
#![allow(bare_trait_objects)]

#[macro_use] extern crate failure;

use config::ConfigError;

#[derive(Debug, Fail)]
pub enum BError {
    #[fail(display = "Configuration error")]
    ConfigError(#[fail(cause)] ConfigError),

    #[fail(display = "Invalid role")]
    InvalidRole,

    #[fail(display = "Missing product in costing data")]
    CostMissingProduct,

    #[fail(display = "Missing tag in costing data")]
    CostMissingTag,
}

pub type BResult<T> = Result<T, BError>;

macro_rules! make_err_converter {
    ($field:path, $errtype:ty) => {
        impl From<$errtype> for BError {
            fn from(err: $errtype) -> BError {
                if cfg!(feature = "panic-on-error") {
                    panic!("{:?}", err);
                } else {
                    $field(err)
                }
            }
        }
    }
}

make_err_converter!(BError::ConfigError, ConfigError);

