use exonum::crypto::Hash;
use exonum::proto::ProtobufConvert;
use crate::block::models::proto;
use crate::error::CError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum CompanyType {
    RegionOwned,
    WorkerOwned,
}

impl ProtobufConvert for CompanyType {
    type ProtoStruct = u32;

    fn to_pb(&self) -> Self::ProtoStruct {
        match *self {
            CompanyType::RegionOwned => 1,
            CompanyType::WorkerOwned => 2,
        }
    }

    fn from_pb(pb: Self::ProtoStruct) -> Result<Self, failure::Error> {
        match pb {
            1 => Ok(CompanyType::RegionOwned),
            2 => Ok(CompanyType::WorkerOwned),
            _ => Err(From::from(CError::InvalidCompanyType)),
        }
    }
}

#[derive(Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::company::Company", serde_pb_convert)]
pub struct Company {
    pub id: String,
    pub company_type: CompanyType,
    pub name: String,
    pub meta: String,
    pub active: bool,
    pub history_len: u64,
    pub history_hash: Hash,
}

impl Company {
    pub fn new(id: &str, company_type: CompanyType, name: &str, meta: &str, active: bool, history_len: u64, &history_hash: &Hash) -> Self {
        Self {
            id: id.to_owned(),
            company_type,
            name: name.to_owned(),
            meta: meta.to_owned(),
            active,
            history_len,
            history_hash,
        }
    }

    pub fn update(self, name: &str, meta: &str, history_hash: &Hash) -> Self {
        Self::new(
            self.id.as_str(),
            self.company_type.clone(),
            name,
            meta,
            self.active,
            self.history_len + 1,
            history_hash,
        )
    }

    pub fn close(self, history_hash: &Hash) -> Self {
        Self::new(
            self.id.as_str(),
            self.company_type.clone(),
            self.name.as_str(),
            self.meta.as_str(),
            false,
            self.history_len + 1,
            history_hash,
        )
    }
}


