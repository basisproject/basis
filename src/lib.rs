#[macro_use] extern crate log;

mod error;
#[macro_use]
mod util;
mod config;

use crate::error::CResult;

pub fn init(default_config: &str, local_config: &str) -> CResult<()> {
    config::init(default_config, local_config)?;
    // set up the logger now that we have our config and data folder set up
    match util::logger::setup_logger() {
        Ok(_) => {}
        Err(e) => {
            println!("conductor::init() -- problem setting up logging: {}", e);
            return Err(e);
        }
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inits() {
        init("./config/config.default.yaml", "./config/config.yaml").unwrap();
    }
}

