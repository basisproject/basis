// Workaround for `failure` see https://github.com/rust-lang-nursery/failure/issues/223 and
// ECR-1771 for the details.
#![allow(bare_trait_objects)]

macro_rules! define_exec_error {
    ($item:ty) => {
        impl From<$item> for ExecutionError {
            fn from(value: $item) -> ExecutionError {
                let description = format!("{}", value);
                ExecutionError::with_description(value as u8, description)
            }
        }
    }
}

pub mod account;
pub mod company;

#[derive(Serialize, Deserialize, Clone, Debug, TransactionSet)]
pub enum TransactionGroup {
    AccountCreate(account::Create),
    AccountUpdate(account::Update),
    AccountTransfer(account::Transfer),
    AccountIssue(account::Issue),

    CompanyCreate(company::Create),
}


