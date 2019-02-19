// Workaround for `failure` see https://github.com/rust-lang-nursery/failure/issues/223 and
// ECR-1771 for the details.
#![allow(bare_trait_objects)]

use ::config::ConfigError;

#[derive(Debug, Fail)]
pub enum CError {
    #[fail(display = "Configuration error")]
    ConfigError(#[fail(cause)] ConfigError),

    #[fail(display = "Invalid account type")]
    InvalidAccountType,

    #[fail(display = "Invalid company type")]
    InvalidCompanyType,
}

pub type CResult<T> = Result<T, CError>;


macro_rules! make_err_converter {
    ($field:path, $errtype:ty) => {
        impl From<$errtype> for CError {
            fn from(err: $errtype) -> CError {
                if cfg!(feature = "panic-on-error") {
                    panic!("{:?}", err);
                } else {
                    $field(err)
                }
            }
        }
    }
}

make_err_converter!(CError::ConfigError, ConfigError);

