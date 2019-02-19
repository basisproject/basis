use uuid::Uuid;
use exonum::{
    blockchain::{ExecutionError, ExecutionResult, Transaction, TransactionContext},
    crypto::{PublicKey, SecretKey},
    messages::{Message, RawTransaction, Signed},
};
use crate::block::{
    SERVICE_ID,
    schema::Schema,
    models::proto,
    models::company::CompanyType,
};

#[derive(Debug, Fail)]
#[repr(u8)]
pub enum TransactionError {
    #[fail(display = "Invalid company type")]
    InvalidCompanyType = 0,
}

define_exec_error!(TransactionError);

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::company::Create")]
pub struct Create {
    pub company_type: CompanyType,
    pub name: String,
    pub meta: String,
}

impl Create {
    pub fn sign(pk: &PublicKey, sk: &SecretKey, company_type: CompanyType, name: &str, meta: &str) -> Signed<RawTransaction> {
        Message::sign_transaction(Self { company_type, name: name.to_owned(), meta: meta.to_owned() }, SERVICE_ID, *pk, sk)
    }
}

impl Transaction for Create {
    fn execute(&self, mut context: TransactionContext) -> ExecutionResult {
        let pub_key = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());
        let mut company_id: String;
        loop {
            company_id = Uuid::new_v4().to_string();
            if schema.company(company_id.as_str()).is_none() {
                break;
            }
        }
        schema.company_create(company_id.as_str(), self.company_type.clone(), self.name.as_str(), self.meta.as_str(), &hash);
        Ok(())
    }
}

