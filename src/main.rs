//#![feature(trace_macros)]
//trace_macros!(true);

#[macro_use] extern crate exonum_derive;
#[macro_use] extern crate failure;
#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate util;
#[cfg(test)] #[macro_use] extern crate exonum_testkit;

mod logger;
mod config;
mod block;
#[cfg(test)] mod test;

use error::BResult;
use exonum::helpers::fabric::NodeBuilder;
use exonum_configuration as configuration;

pub fn init(default_config: &str, local_config: &str) -> BResult<()> {
    config::init(default_config, local_config)?;
    // set up the logger now that we have our config and data folder set up
    match logger::setup_logger() {
        Ok(_) => {}
        Err(e) => {
            println!("basis::init() -- problem setting up logging: {}", e);
            //return Err(e);
        }
    };
    Ok(())
}

fn main() {
    init("./config/config.default.yaml", "./config/config.yaml").unwrap();
    exonum::crypto::init();
    if let Err(err) = exonum::helpers::init_logger() {
        drop(err);
        // honestly don't give a shit
        //warn!("basis::main() -- error initializing exonum logger: {}", err);
    }

    let node = NodeBuilder::new()
        .with_service(Box::new(configuration::ServiceFactory))
        .with_service(Box::new(block::ServiceFactory));
    node.run();
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    use exonum::crypto;
    use exonum_testkit::{TestKit, TestKitBuilder, txvec};
    use crate::block::{
        transactions::user,
        schema::Schema,
        Service,
    };

    /// Creates a testkit together with the API wrapper defined above.
    fn create_testkit() -> TestKit {
        TestKitBuilder::validator()
            .with_service(Service)
            .create()
    }

    fn do_init() {
        init("./config/config.default.yaml", "./config/config.yaml").unwrap();
    }

    #[test]
    fn account_functions() {
        do_init();
        let mut testkit = create_testkit();
        let (region_pkey, region_skey) = crypto::gen_keypair();
        let (larry_pkey, larry_skey) = crypto::gen_keypair();
        testkit.create_block_with_transactions(txvec![
            account::Create::sign(&region_pkey, &region_skey, AccountType::Region, "region::6970d03b-56d5-42db-8012-9bf4131add14::bank::general"),
            account::Issue::sign(&region_pkey, &region_skey, "Initial balance", 1000000 * 100, 1),
            account::Create::sign(&larry_pkey, &larry_skey, AccountType::Person, "Larry"),
            account::Transfer::sign(&region_pkey, &larry_skey, &larry_pkey, "Alright, Parker...shut up, thank you, Parker. Shut up.", 100 * 100, 2),
            account::Update::sign(&larry_pkey, &larry_skey, "Larry Weber"),
        ]);

        // ---------------------------------------------------------------------
        // let's pretend we don't exist...
        // ---------------------------------------------------------------------

        let (region_account, larry_account) = {
            let snapshot = testkit.snapshot();
            (
                Schema::new(&snapshot).account(&region_pkey).expect("No account persisted"),
                Schema::new(&snapshot).account(&larry_pkey).expect("No account persisted"),
            )
        };
        assert_eq!(&region_account.pub_key, &region_pkey);
        assert_eq!(region_account.name, "region::6970d03b-56d5-42db-8012-9bf4131add14::bank::general");
        assert_eq!(region_account.balance, 999900 * 100);
        assert_eq!(region_account.account_type, AccountType::Region);
        assert_eq!(&larry_account.pub_key, &larry_pkey);
        assert_eq!(larry_account.name, "Larry Weber");
        assert_eq!(larry_account.account_type, AccountType::Person);
        assert_eq!(larry_account.balance, 100 * 100);
    }
}
*/

