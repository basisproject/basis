//! This is a collection of utilities for our tests.

use uuid;
use exonum_testkit::{TestKit, TestKitBuilder};
use exonum::{
    crypto::{PublicKey, SecretKey},
    messages::{Signed, RawTransaction},
};
use hex::FromHex;
use crate::{
    config,
    block::{transactions, Service},
};

/// create a random new uuid
pub fn gen_uuid() -> String {
    format!("{}", uuid::Uuid::new_v4())
}

/// init our testkit and also make sure the config is primed
pub fn init_testkit() -> TestKit {
    config::init("./config/config.default.yaml", "./config/config.yaml").unwrap();
    TestKitBuilder::validator()
        .with_service(Service)
        .create()
}

/// create a transaction that creates a superuser that we can use for setting up
/// our test environments
pub fn tx_superuser(uid: &str) -> (Signed<RawTransaction>, PublicKey, SecretKey) {
    let root_pub = PublicKey::from_hex(config::get::<String>("tests.bootstrap_user.pub").unwrap_or(String::from(""))).unwrap();
    let root_sec = SecretKey::from_hex(config::get::<String>("tests.bootstrap_user.sec").unwrap_or(String::from(""))).unwrap();
    let txuser = transactions::user::TxCreate::sign(
        &uid.to_owned(),
        &root_pub,
        &vec![models::access::Role::SuperAdmin, models::access::Role::TimeTraveller],
        &String::from("frothy@gibbertarian.com"),
        &String::from("FREEDOM OR FAIR TRADE SOY TENDIES #PICKASIDE"),
        &String::from("{}"),
        &util::time::now(),
        &root_pub,
        &root_sec
    );
    (txuser, root_pub, root_sec)
}

