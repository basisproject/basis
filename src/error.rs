// Workaround for `failure` see https://github.com/rust-lang-nursery/failure/issues/223 and
// ECR-1771 for the details.
#![allow(bare_trait_objects)]

use ::config::ConfigError;

#[derive(Debug, Fail)]
pub enum BError {
    #[fail(display = "Configuration error")]
    ConfigError(#[fail(cause)] ConfigError),

    #[fail(display = "Invalid role")]
    InvalidRole,
}

pub type CResult<T> = Result<T, BError>;


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

