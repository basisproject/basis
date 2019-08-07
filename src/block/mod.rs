pub mod models;
pub mod schema;
pub mod transactions;
pub mod api;

use exonum::{
    api::{ServiceApiBuilder},
    blockchain::{self, Transaction, TransactionSet, TransactionMessage, BlockProof},
    crypto::{Hash},
    helpers::fabric::{self, Context},
    messages::RawTransaction,
    storage::Snapshot,
    storage::proof_map_index::MapProof,
    storage::proof_list_index::ListProof,
};
pub use crate::block::schema::Schema;
use crate::block::transactions::TransactionGroup;

pub const SERVICE_ID: u16 = 128;
pub const SERVICE_NAME: &str = "factor";

#[derive(Debug, Fail)]
#[repr(u8)]
pub enum ApiError {
    #[fail(display = "Bad query")]
    BadQuery = 0,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectProof<T> {
    table: MapProof<Hash, Hash>,
    object: MapProof<Hash, T>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectHistory {
    pub proof: ListProof<Hash>,
    pub transactions: Vec<TransactionMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProofResult<T> {
    pub block_proof: Option<BlockProof>,
    pub item_proof: ObjectProof<T>,
    pub item_history: Option<ObjectHistory>,
    pub item: Option<T>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListResult<T> {
    pub items: Vec<T>,
}

#[derive(Default, Debug)]
pub struct Service;

impl blockchain::Service for Service {
    fn service_id(&self) -> u16 {
        SERVICE_ID
    }

    fn service_name(&self) -> &str {
        SERVICE_NAME
    }

    fn state_hash(&self, view: &dyn Snapshot) -> Vec<Hash> {
        let schema = Schema::new(view);
        schema.state_hash()
    }

    fn tx_from_raw(&self, raw: RawTransaction) -> Result<Box<dyn Transaction>, failure::Error> {
        TransactionGroup::tx_from_raw(raw).map(Into::into)
    }

    fn wire_api(&self, builder: &mut ServiceApiBuilder) {
        api::user::UserApi::wire(builder);
        api::company::CompanyApi::wire(builder);
        //api::company_member::CompanyMemberApi::wire(builder);
    }
}

#[derive(Debug)]
pub struct ServiceFactory;

impl fabric::ServiceFactory for ServiceFactory {
    fn service_name(&self) -> &str {
        SERVICE_NAME
    }

    fn make_service(&mut self, _: &Context) -> Box<dyn blockchain::Service> {
        Box::new(Service)
    }
}

