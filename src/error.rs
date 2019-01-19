use ::quick_error::quick_error;
use ::config::ConfigError;

quick_error! {
    #[derive(Debug)]
    /// Conductor's main error object.
    pub enum CError {
        ConfigError(err: ConfigError) {
            cause(err)
            description("config error")
            display("config error: {}", err)
        }
    }
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

