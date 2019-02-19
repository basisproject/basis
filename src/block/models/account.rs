use exonum::crypto::{Hash, PublicKey};
use exonum::proto::ProtobufConvert;
use crate::block::models::proto;
use crate::error::CError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum AccountType {
    Region,
    Company,
    Person,
}

impl ProtobufConvert for AccountType {
    type ProtoStruct = u32;

    fn to_pb(&self) -> Self::ProtoStruct {
        match *self {
            AccountType::Region => 1,
            AccountType::Company => 2,
            AccountType::Person => 3,
        }
    }

    fn from_pb(pb: Self::ProtoStruct) -> Result<Self, failure::Error> {
        match pb {
            1 => Ok(AccountType::Region),
            2 => Ok(AccountType::Company),
            3 => Ok(AccountType::Person),
            _ => Err(From::from(CError::InvalidAccountType)),
        }
    }
}

#[derive(Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::account::Account", serde_pb_convert)]
pub struct Account {
    pub pub_key: PublicKey,
    pub account_type: AccountType,
    pub name: String,
    pub balance: u64,
    pub history_len: u64,
    pub history_hash: Hash,
}

impl Account {
    pub fn new(&pub_key: &PublicKey, account_type: AccountType, name: &str, balance: u64, history_len: u64, &history_hash: &Hash) -> Self {
        Self {
            pub_key,
            account_type,
            name: name.to_owned(),
            balance,
            history_len,
            history_hash,
        }
    }

    pub fn update(self, name: &str, history_hash: &Hash) -> Self {
        Self::new(
            &self.pub_key,
            self.account_type.clone(),
            name,
            self.balance,
            self.history_len + 1,
            history_hash,
        )
    }

    pub fn set_balance(self, balance: u64, history_hash: &Hash) -> Self {
        Self::new(
            &self.pub_key,
            self.account_type.clone(),
            &self.name,
            balance,
            self.history_len + 1,
            history_hash,
        )
    }
}

