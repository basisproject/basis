// Workaround for `failure` see https://github.com/rust-lang-nursery/failure/issues/223 and
// ECR-1771 for the details.
#![allow(bare_trait_objects)]

pub mod account;

use exonum::blockchain::ExecutionError;

const ERROR_SENDER_SAME_AS_RECEIVER: u8 = 0;

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
}

impl From<TransactionError> for ExecutionError {
    fn from(value: TransactionError) -> ExecutionError {
        let description = format!("{}", value);
        ExecutionError::with_description(value as u8, description)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, TransactionSet)]
pub enum TransactionGroup {
    AccountCreate(account::Create),
    AccountUpdate(account::Update),
    AccountTransfer(account::Transfer),
    AccountIssue(account::Issue),
}


