use std::sync::RwLock;
use lazy_static::lazy_static;
use serde::de::DeserializeOwned;
use error::BResult;

lazy_static! {
	static ref CONFIG: RwLock<config::Config> = RwLock::new(config::Config::default());
}

/// Load the config from file/env
pub fn init(default_config: &str, local_config: &str) -> BResult<()> {
    let mut config_guard = lockw!(*CONFIG);
    config_guard
        .merge(config::File::with_name(default_config))?
        .merge(config::File::with_name(local_config))?
        .merge(config::Environment::with_prefix("basis").separator("__"))?;
    Ok(())
}

/// Get a config value by key
pub fn get<T: DeserializeOwned>(key: &str) -> BResult<T> {
    let config_guard = lockr!(*CONFIG);
    Ok(config_guard.get(key)?)
}

