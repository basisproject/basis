use exonum::{
    crypto::{self, Hash, PublicKey},
    storage::{Fork, ProofListIndex, ProofMapIndex, Snapshot},
};
use crate::block::models::{
    account::{Account, AccountType},
    company::{Company, CompanyType},
};

#[derive(Debug)]
pub struct Schema<T> {
    view: T,
}

impl<T> AsMut<T> for Schema<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.view
    }
}

impl<T> Schema<T>
where
    T: AsRef<dyn Snapshot>,
{
    pub fn new(view: T) -> Self {
        Schema { view }
    }

    pub fn state_hash(&self) -> Vec<Hash> {
        vec![self.accounts().merkle_root()]
    }

    // -------------------------------------------------------------------------
    // Accounts
    // -------------------------------------------------------------------------
    pub fn accounts(&self) -> ProofMapIndex<&T, PublicKey, Account> {
        ProofMapIndex::new("conductor.accounts", &self.view)
    }

    pub fn account_history(&self, public_key: &PublicKey) -> ProofListIndex<&T, Hash> {
        ProofListIndex::new_in_family("conductor.accounts.history", public_key, &self.view)
    }

    pub fn account(&self, pub_key: &PublicKey) -> Option<Account> {
        self.accounts().get(pub_key)
    }

    // -------------------------------------------------------------------------
    // Companies
    // -------------------------------------------------------------------------
    pub fn companies(&self) -> ProofMapIndex<&T, Hash, Company> {
        ProofMapIndex::new("conductor.companies", &self.view)
    }

    pub fn company_history(&self, id: &str) -> ProofListIndex<&T, Hash> {
        ProofListIndex::new_in_family("conductor.companies.history", id, &self.view)
    }

    pub fn company(&self, id: &str) -> Option<Company> {
        self.companies().get(&crypto::hash(id.as_bytes()))
    }
}

impl<'a> Schema<&'a mut Fork> {
    // -------------------------------------------------------------------------
    // Accounts
    // -------------------------------------------------------------------------
    pub fn accounts_mut(&mut self) -> ProofMapIndex<&mut Fork, PublicKey, Account> {
        ProofMapIndex::new("conductor.accounts", &mut self.view)
    }

    pub fn account_history_mut(&mut self, public_key: &PublicKey) -> ProofListIndex<&mut Fork, Hash> {
        ProofListIndex::new_in_family("conductor.accounts.history", public_key, &mut self.view)
    }

    pub fn account_create(&mut self, key: &PublicKey, account_type: AccountType, name: &str, transaction: &Hash) {
        let account = {
            let mut history = self.account_history_mut(key);
            history.push(*transaction);
            let history_hash = history.merkle_root();
            Account::new(key, account_type, name, 0, history.len(), &history_hash)
        };
        self.accounts_mut().put(key, account);
    }

    pub fn account_update(&mut self, account: Account, name: &str, transaction: &Hash) {
        let account = {
            let mut history = self.account_history_mut(&account.pub_key);
            history.push(*transaction);
            let history_hash = history.merkle_root();
            account.update(name, &history_hash)
        };
        self.accounts_mut().put(&account.pub_key, account.clone());
    }

    pub fn account_adjust_balance(&mut self, account: Account, amount: u64, negative: bool, transaction: &Hash) {
        let account = {
            let mut history = self.account_history_mut(&account.pub_key);
            history.push(*transaction);
            let history_hash = history.merkle_root();
            let balance = account.balance;
            let new_balance = if negative { balance - amount } else { balance + amount };
            account.set_balance(new_balance, &history_hash)
        };
        self.accounts_mut().put(&account.pub_key, account.clone());
    }

    // -------------------------------------------------------------------------
    // Companies
    // -------------------------------------------------------------------------
    pub fn companies_mut(&mut self) -> ProofMapIndex<&mut Fork, Hash, Company> {
        ProofMapIndex::new("conductor.companies", &mut self.view)
    }

    pub fn company_history_mut(&mut self, id: &str) -> ProofListIndex<&mut Fork, Hash> {
        ProofListIndex::new_in_family("conductor.companies.history", id, &mut self.view)
    }

    pub fn company_create(&mut self, id: &str, company_type: CompanyType, name: &str, meta: &str, transaction: &Hash) {
        let company = {
            let mut history = self.company_history_mut(id);
            history.push(*transaction);
            let history_hash = history.merkle_root();
            Company::new(id, company_type, name, meta, true, history.len(), &history_hash)
        };
        self.companies_mut().put(&crypto::hash(id.as_bytes()), company);
    }

    pub fn company_update(&mut self, company: Company, name: &str, meta: &str, transaction: &Hash) {
        let company = {
            let mut history = self.company_history_mut(company.id.as_str());
            history.push(*transaction);
            let history_hash = history.merkle_root();
            company.update(name, meta, &history_hash)
        };
        self.companies_mut().put(&crypto::hash(company.id.as_bytes()), company);
    }

    pub fn company_close(&mut self, company: Company, transaction: &Hash) {
        let company = {
            let mut history = self.company_history_mut(company.id.as_str());
            history.push(*transaction);
            let history_hash = history.merkle_root();
            company.close(&history_hash)
        };
        self.companies_mut().put(&crypto::hash(company.id.as_bytes()), company);
    }
}

