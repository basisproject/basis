use chrono::{DateTime, Utc};
use exonum::{
    crypto::{self, Hash, PublicKey},
    storage::{Fork, MapIndex, ProofListIndex, ProofMapIndex, Snapshot},
};
use crate::block::models::access::Role;
use crate::block::models::{
    user::User,
    company::{Company, CompanyType, Role as CompanyRole},
    company_member::CompanyMember,
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
        vec![self.users().merkle_root()]
    }

    // -------------------------------------------------------------------------
    // Users
    // -------------------------------------------------------------------------
    pub fn users(&self) -> ProofMapIndex<&T, Hash, User> {
        ProofMapIndex::new("factor.users.table", &self.view)
    }

    pub fn users_idx_pubkey(&self) -> MapIndex<&T, PublicKey, String> {
        MapIndex::new("factor.users.idx_pubkey", &self.view)
    }

    pub fn users_idx_email(&self) -> MapIndex<&T, String, String> {
        MapIndex::new("factor.users.idx_email", &self.view)
    }

    pub fn users_history(&self, id: &str) -> ProofListIndex<&T, Hash> {
        ProofListIndex::new_in_family("factor.users.history", &crypto::hash(id.as_bytes()), &self.view)
    }

    pub fn get_user(&self, id: &str) -> Option<User> {
        self.users().get(&crypto::hash(id.as_bytes()))
    }

    pub fn get_user_by_pubkey(&self, pubkey: &PublicKey) -> Option<User> {
        if let Some(id) = self.users_idx_pubkey().get(pubkey) {
            self.users().get(&crypto::hash(id.as_bytes()))
        } else {
            None
        }
    }

    pub fn get_user_by_email(&self, email: &str) -> Option<User> {
        if let Some(id) = self.users_idx_email().get(email) {
            self.users().get(&crypto::hash(id.as_bytes()))
        } else {
            None
        }
    }

    // -------------------------------------------------------------------------
    // Companies
    // -------------------------------------------------------------------------
    pub fn companies(&self) -> ProofMapIndex<&T, Hash, Company> {
        ProofMapIndex::new("factor.companies.table", &self.view)
    }

    pub fn companies_history(&self, id: &str) -> ProofListIndex<&T, Hash> {
        ProofListIndex::new_in_family("factor.companies.history", &crypto::hash(id.as_bytes()), &self.view)
    }

    pub fn get_company(&self, id: &str) -> Option<Company> {
        self.companies().get(&crypto::hash(id.as_bytes()))
    }

    // -------------------------------------------------------------------------
    // Company members
    // -------------------------------------------------------------------------
    pub fn companies_members(&self, company_id: &str) -> ProofMapIndex<&T, Hash, CompanyMember> {
        ProofMapIndex::new_in_family("factor.companies_members.table", &crypto::hash(company_id.as_bytes()), &self.view)
    }

    pub fn companies_members_history(&self, company_id: &str) -> ProofListIndex<&T, Hash> {
        ProofListIndex::new_in_family("factor.companies_members.history", &crypto::hash(company_id.as_bytes()), &self.view)
    }

    pub fn get_company_member(&self, company_id: &str, user_id: &str) -> Option<CompanyMember> {
        self.companies_members(company_id).get(&crypto::hash(user_id.as_bytes()))
    }
}

impl<'a> Schema<&'a mut Fork> {
    // -------------------------------------------------------------------------
    // Users
    // -------------------------------------------------------------------------
    pub fn users_mut(&mut self) -> ProofMapIndex<&mut Fork, Hash, User> {
        ProofMapIndex::new("factor.users.table", &mut self.view)
    }

    pub fn users_idx_pubkey_mut(&mut self) -> MapIndex<&mut Fork, PublicKey, String> {
        MapIndex::new("factor.users.idx_pubkey", &mut self.view)
    }

    pub fn users_idx_email_mut(&mut self) -> MapIndex<&mut Fork, String, String> {
        MapIndex::new("factor.users.idx_email", &mut self.view)
    }

    pub fn users_history_mut(&mut self, id: &str) -> ProofListIndex<&mut Fork, Hash> {
        ProofListIndex::new_in_family("factor.users.history", &crypto::hash(id.as_bytes()), &mut self.view)
    }

    pub fn users_create(&mut self, id: &str, pubkey: &PublicKey, roles: &Vec<Role>, email: &str, name: &str, meta: &str, created: &DateTime<Utc>, transaction: &Hash) {
        let user = {
            let mut history = self.users_history_mut(id);
            history.push(*transaction);
            let history_hash = history.merkle_root();
            User::new(id, pubkey, roles, email, name, meta, created, created, history.len(), &history_hash)
        };
        self.users_mut().put(&crypto::hash(id.as_bytes()), user);
        self.users_idx_pubkey_mut().put(pubkey, id.to_owned());
        self.users_idx_email_mut().put(&email.to_owned(), id.to_owned());
    }

    pub fn users_update(&mut self, user: User, id: &str, email: Option<&str>, name: Option<&str>, meta: Option<&str>, updated: &DateTime<Utc>, transaction: &Hash) {
        let old_email = user.email.clone();
        let user = {
            let mut history = self.users_history_mut(id);
            history.push(*transaction);
            let history_hash = history.merkle_root();
            user.update(email, name, meta, updated, &history_hash)
        };
        let new_email = user.email.clone();
        self.users_mut().put(&crypto::hash(id.as_bytes()), user);
        if email.is_some() && email != Some(old_email.as_str()) {
            self.users_idx_email_mut().remove(&old_email);
            self.users_idx_email_mut().put(&new_email, id.to_owned());
        }
    }

    pub fn users_set_pubkey(&mut self, user: User, id: &str, pubkey: &PublicKey, updated: &DateTime<Utc>, transaction: &Hash) {
        let pubkey_old = user.pubkey.clone();
        let user = {
            let mut history = self.users_history_mut(id);
            history.push(*transaction);
            let history_hash = history.merkle_root();
            user.set_pubkey(pubkey, updated, &history_hash)
        };
        self.users_mut().put(&crypto::hash(id.as_bytes()), user);
        self.users_idx_pubkey_mut().remove(&pubkey_old);
        self.users_idx_pubkey_mut().put(pubkey, id.to_owned());
    }

    pub fn users_set_roles(&mut self, user: User, id: &str, roles: &Vec<Role>, updated: &DateTime<Utc>, transaction: &Hash) {
        let user = {
            let mut history = self.users_history_mut(id);
            history.push(*transaction);
            let history_hash = history.merkle_root();
            user.set_roles(roles, updated, &history_hash)
        };
        self.users_mut().put(&crypto::hash(id.as_bytes()), user);
    }

    pub fn users_delete(&mut self, user: User, id: &str, transaction: &Hash) {
        let mut history = self.users_history_mut(id);
        history.push(*transaction);
        self.users_mut().remove(&crypto::hash(id.as_bytes()));
        self.users_idx_pubkey_mut().remove(&user.pubkey);
        self.users_idx_email_mut().remove(&user.email);
    }

    // -------------------------------------------------------------------------
    // Companies
    // -------------------------------------------------------------------------
    pub fn companies_mut(&mut self) -> ProofMapIndex<&mut Fork, Hash, Company> {
        ProofMapIndex::new("factor.companies.table", &mut self.view)
    }

    pub fn companies_history_mut(&mut self, id: &str) -> ProofListIndex<&mut Fork, Hash> {
        ProofListIndex::new_in_family("factor.companies.history", &crypto::hash(id.as_bytes()), &mut self.view)
    }

    pub fn companies_create(&mut self, id: &str, ty: CompanyType, email: &str, name: &str, created: &DateTime<Utc>, transaction: &Hash) {
        let company = {
            let mut history = self.companies_history_mut(id);
            history.push(*transaction);
            let history_hash = history.merkle_root();
            Company::new(id, ty, email, name, created, created, history.len(), &history_hash)
        };
        self.companies_mut().put(&crypto::hash(id.as_bytes()), company);
    }

    pub fn companies_update(&mut self, company: Company, id: &str, email: Option<&str>, name: Option<&str>, updated: &DateTime<Utc>, transaction: &Hash) {
        let company = {
            let mut history = self.companies_history_mut(id);
            history.push(*transaction);
            let history_hash = history.merkle_root();
            company.update(email, name, updated, &history_hash)
        };
        self.companies_mut().put(&crypto::hash(id.as_bytes()), company);
    }

    pub fn companies_set_type(&mut self, company: Company, id: &str, ty: CompanyType, updated: &DateTime<Utc>, transaction: &Hash) {
        let company = {
            let mut history = self.companies_history_mut(id);
            history.push(*transaction);
            let history_hash = history.merkle_root();
            company.set_type(ty, updated, &history_hash)
        };
        self.companies_mut().put(&crypto::hash(id.as_bytes()), company);
    }

    pub fn companies_delete(&mut self, id: &str, transaction: &Hash) {
        let mut history = self.companies_history_mut(id);
        history.push(*transaction);
        self.companies_mut().remove(&crypto::hash(id.as_bytes()));
        self.companies_members_mut(id).clear();
    }

    // -------------------------------------------------------------------------
    // Company members
    // -------------------------------------------------------------------------
    pub fn companies_members_mut(&mut self, company_id: &str) -> ProofMapIndex<&mut Fork, Hash, CompanyMember> {
        ProofMapIndex::new_in_family("factor.companies_members.table", &crypto::hash(company_id.as_bytes()), &mut self.view)
    }

    pub fn companies_members_history_mut(&mut self, company_id: &str) -> ProofListIndex<&mut Fork, Hash> {
        ProofListIndex::new_in_family("factor.companies_members.history", &crypto::hash(company_id.as_bytes()), &mut self.view)
    }

    pub fn companies_members_create(&mut self, company_id: &str, user_id: &str, roles: &Vec<CompanyRole>, created: &DateTime<Utc>, transaction: &Hash) {
        let member = {
            let mut history = self.companies_members_history_mut(company_id);
            history.push(*transaction);
            let history_hash = history.merkle_root();
            CompanyMember::new(user_id, roles, created, created, history.len(), &history_hash)
        };
        self.companies_members_mut(company_id).put(&crypto::hash(user_id.as_bytes()), member);
    }
}

