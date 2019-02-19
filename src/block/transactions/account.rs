use exonum::{
    blockchain::{ExecutionError, ExecutionResult, Transaction, TransactionContext},
    crypto::{PublicKey, SecretKey},
    messages::{Message, RawTransaction, Signed},
};
use crate::block::{
    SERVICE_ID,
    schema::Schema,
    models::proto,
    models::account::AccountType,
};

#[derive(Debug, Fail)]
#[repr(u8)]
pub enum TransactionError {
    #[fail(display = "Invalid account type")]
    InvalidAccountType = 0,

    #[fail(display = "Account already exists")]
    AccountAlreadyExists = 1,

    #[fail(display = "Sender doesn't exist")]
    SenderNotFound = 2,

    #[fail(display = "Receiver doesn't exist")]
    ReceiverNotFound = 3,

    #[fail(display = "Account not found")]
    AccountNotFound = 4,

    #[fail(display = "Insufficient currency amount")]
    InsufficientCurrencyAmount = 5,

    #[fail(display = "Sender is same as receiver")]
    SenderSameAsReceiver = 6,
}

define_exec_error!(TransactionError);

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::account::Create")]
pub struct Create {
    pub name: String,
    pub account_type: AccountType,
}

impl Create {
    pub fn sign(pk: &PublicKey, sk: &SecretKey, account_type: AccountType, name: &str) -> Signed<RawTransaction> {
        Message::sign_transaction(Self { name: name.to_owned(), account_type }, SERVICE_ID, *pk, sk)
    }
}

impl Transaction for Create {
    fn execute(&self, mut context: TransactionContext) -> ExecutionResult {
        let pub_key = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        if schema.account(pub_key).is_none() {
            let name = &self.name;
            let account_type = self.account_type.clone();
            schema.account_create(pub_key, account_type, name, &hash);
            Ok(())
        } else {
            Err(TransactionError::AccountAlreadyExists)?
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::account::Update")]
pub struct Update {
    pub name: String,
}

impl Update {
    pub fn sign(pk: &PublicKey, sk: &SecretKey, name: &str) -> Signed<RawTransaction> {
        Message::sign_transaction(Self { name: name.to_owned() }, SERVICE_ID, *pk, sk)
    }
}

impl Transaction for Update {
    fn execute(&self, mut context: TransactionContext) -> ExecutionResult {
        let pub_key = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        if let Some(account) = schema.account(pub_key) {
            let name = &self.name;
            schema.account_update(account, name, &hash);
            Ok(())
        } else {
            Err(TransactionError::AccountNotFound)?
        }
    }
}

#[derive(Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::account::Transfer", serde_pb_convert)]
pub struct Transfer {
    pub to: PublicKey,
    pub amount: u64,
    pub seed: u64,
}

impl Transfer {
    pub fn sign(pk: &PublicKey, sk: &SecretKey, &to: &PublicKey, amount: u64, seed: u64) -> Signed<RawTransaction> {
        Message::sign_transaction(Self { to, amount, seed }, SERVICE_ID, *pk, sk)
    }
}

impl Transaction for Transfer {
    fn execute(&self, mut context: TransactionContext) -> ExecutionResult {
        let from = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        let to = &self.to;
        let amount = self.amount;

        if from == to {
            Err(TransactionError::SenderSameAsReceiver)?
        }

        let sender = schema.account(from).ok_or(TransactionError::SenderNotFound)?;
        let receiver = schema.account(to).ok_or(TransactionError::ReceiverNotFound)?;
        if sender.balance < amount {
            Err(TransactionError::InsufficientCurrencyAmount)?
        }
        schema.account_adjust_balance(sender, amount, true, &hash);
        schema.account_adjust_balance(receiver, amount, false, &hash);

        Ok(())
    }
}

#[derive(Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::account::Issue", serde_pb_convert)]
pub struct Issue {
    pub amount: u64,
    pub seed: u64,
}

impl Issue {
    pub fn sign(pk: &PublicKey, sk: &SecretKey, amount: u64, seed: u64) ->  Signed<RawTransaction> {
        Message::sign_transaction(Self { amount, seed }, SERVICE_ID, *pk, sk)
    }
}

impl Transaction for Issue {
    fn execute(&self, mut context: TransactionContext) -> ExecutionResult {
        let pub_key = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        if let Some(account) = schema.account(pub_key) {
            let amount = self.amount;
            schema.account_adjust_balance(account, amount, false, &hash);
            Ok(())
        } else {
            Err(TransactionError::ReceiverNotFound)?
        }
    }
}

