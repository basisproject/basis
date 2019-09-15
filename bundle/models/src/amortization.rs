use exonum::crypto::Hash;
use chrono::{DateTime, Utc};
use crate::{
    costs::Costs,
    proto,
};

#[derive(Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::amortization::Amortization", serde_pb_convert)]
pub struct Amortization {
    pub id: String,
    pub company_id: String,
    pub name: String,
    pub costs: Costs,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub meta: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub history_len: u64,
    pub history_hash: Hash,
}

impl Amortization {
    pub fn new(id: &str, company_id: &str, name: &str, costs: &Costs, start: &DateTime<Utc>, end: &DateTime<Utc>, meta: &str, created: &DateTime<Utc>, updated: &DateTime<Utc>, history_len: u64, history_hash: &Hash) -> Self {
        Self {
            id: id.to_owned(),
            company_id: company_id.to_owned(),
            name: name.to_owned(),
            costs: costs.clone(),
            start: start.clone(),
            end: end.clone(),
            meta: meta.to_owned(),
            created: created.clone(),
            updated: updated.clone(),
            history_len,
            history_hash: history_hash.clone(),
        }
    }
}

