use chrono::{DateTime, Utc};
use exonum::{
    crypto::{self, Hash, PublicKey},
};
use exonum_merkledb::{
    IndexAccess,
    ObjectHash,
    MapIndex,
    ProofListIndex,
    ProofMapIndex,
};
use crate::block::models::access::Role;
use crate::block::models::{
    user::User,
    company::{Company, CompanyType, Role as CompanyRole},
    company_member::CompanyMember,
};

#[derive(Debug)]
pub struct Schema<T> {
    access: T,
}

impl<T> AsMut<T> for Schema<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.access
    }
}

impl<T> Schema<T>
    where T: IndexAccess
{
    pub fn new(access: T) -> Self {
        Schema { access }
    }

    // TODO: ???
    // obviously this should be more general than just users?
    pub fn state_hash(&self) -> Vec<Hash> {
        vec![self.users().object_hash()]
    }

    // -------------------------------------------------------------------------
    // Users
    // -------------------------------------------------------------------------
    pub fn users(&self) -> ProofMapIndex<T, Hash, User> {
        ProofMapIndex::new("basis.users.table", self.access.clone())
    }

    pub fn users_idx_pubkey(&self) -> MapIndex<T, PublicKey, String> {
        MapIndex::new("basis.users.idx_pubkey", self.access.clone())
    }

    pub fn users_idx_email(&self) -> MapIndex<T, String, String> {
        MapIndex::new("basis.users.idx_email", self.access.clone())
    }

    pub fn users_history(&self, id: &str) -> ProofListIndex<T, Hash> {
        ProofListIndex::new_in_family("basis.users.history", &crypto::hash(id.as_bytes()), self.access.clone())
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

    pub fn users_create(&mut self, id: &str, pubkey: &PublicKey, roles: &Vec<Role>, email: &str, name: &str, meta: &str, created: &DateTime<Utc>, transaction: &Hash) {
        let user = {
            let mut history = self.users_history(id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            User::new(id, pubkey, roles, email, name, meta, created, created, history.len(), &history_hash)
        };
        self.users().put(&crypto::hash(id.as_bytes()), user);
        self.users_idx_pubkey().put(pubkey, id.to_owned());
        self.users_idx_email().put(&email.to_owned(), id.to_owned());
    }

    pub fn users_update(&mut self, user: User, id: &str, email: Option<&str>, name: Option<&str>, meta: Option<&str>, updated: &DateTime<Utc>, transaction: &Hash) {
        let old_email = user.email.clone();
        let user = {
            let mut history = self.users_history(id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            user.update(email, name, meta, updated, &history_hash)
        };
        let new_email = user.email.clone();
        self.users().put(&crypto::hash(id.as_bytes()), user);
        if email.is_some() && email != Some(old_email.as_str()) {
            self.users_idx_email().remove(&old_email);
            self.users_idx_email().put(&new_email, id.to_owned());
        }
    }

    pub fn users_set_pubkey(&mut self, user: User, id: &str, pubkey: &PublicKey, updated: &DateTime<Utc>, transaction: &Hash) {
        let pubkey_old = user.pubkey.clone();
        let user = {
            let mut history = self.users_history(id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            user.set_pubkey(pubkey, updated, &history_hash)
        };
        self.users().put(&crypto::hash(id.as_bytes()), user);
        self.users_idx_pubkey().remove(&pubkey_old);
        self.users_idx_pubkey().put(pubkey, id.to_owned());
    }

    pub fn users_set_roles(&mut self, user: User, id: &str, roles: &Vec<Role>, updated: &DateTime<Utc>, transaction: &Hash) {
        let user = {
            let mut history = self.users_history(id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            user.set_roles(roles, updated, &history_hash)
        };
        self.users().put(&crypto::hash(id.as_bytes()), user);
    }

    pub fn users_delete(&mut self, user: User, id: &str) {
        self.users().remove(&crypto::hash(id.as_bytes()));
        self.users_idx_pubkey().remove(&user.pubkey);
        self.users_idx_email().remove(&user.email);
        self.users_history(id).clear();
    }

    // -------------------------------------------------------------------------
    // Companies
    // -------------------------------------------------------------------------
    pub fn companies(&self) -> ProofMapIndex<T, Hash, Company> {
        ProofMapIndex::new("basis.companies.table", self.access.clone())
    }

    pub fn companies_history(&self, id: &str) -> ProofListIndex<T, Hash> {
        ProofListIndex::new_in_family("basis.companies.history", &crypto::hash(id.as_bytes()), self.access.clone())
    }

    pub fn get_company(&self, id: &str) -> Option<Company> {
        self.companies().get(&crypto::hash(id.as_bytes()))
    }

    pub fn companies_create(&mut self, id: &str, ty: CompanyType, region_id: Option<&str>, email: &str, name: &str, created: &DateTime<Utc>, transaction: &Hash) {
        let company = {
            let mut history = self.companies_history(id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            Company::new(id, ty, region_id, email, name, created, created, history.len(), &history_hash)
        };
        self.companies().put(&crypto::hash(id.as_bytes()), company.clone());
    }

    pub fn companies_update(&mut self, company: Company, id: &str, email: Option<&str>, name: Option<&str>, updated: &DateTime<Utc>, transaction: &Hash) {
        let company = {
            let mut history = self.companies_history(id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            company.update(email, name, updated, &history_hash)
        };
        self.companies().put(&crypto::hash(id.as_bytes()), company);
    }

    pub fn companies_set_type(&mut self, company: Company, id: &str, ty: CompanyType, updated: &DateTime<Utc>, transaction: &Hash) {
        let company = {
            let mut history = self.companies_history(id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            company.set_type(ty, updated, &history_hash)
        };
        self.companies().put(&crypto::hash(id.as_bytes()), company);
    }

    pub fn companies_delete(&mut self, id: &str) {
        self.companies().remove(&crypto::hash(id.as_bytes()));
        self.companies_members(id).clear();
        self.companies_history(id).clear();
    }

    // -------------------------------------------------------------------------
    // Company members
    // -------------------------------------------------------------------------
    pub fn companies_members(&self, company_id: &str) -> ProofMapIndex<T, Hash, CompanyMember> {
        ProofMapIndex::new_in_family("basis.companies_members.table", &crypto::hash(company_id.as_bytes()), self.access.clone())
    }

    pub fn companies_members_history(&self, company_id: &str, user_id: &str) -> ProofListIndex<T, Hash> {
        ProofListIndex::new_in_family("basis.companies_members.history", &crypto::hash(format!("{}:{}", company_id, user_id).as_bytes()), self.access.clone())
    }

    pub fn get_company_member(&self, company_id: &str, user_id: &str) -> Option<CompanyMember> {
        self.companies_members(company_id).get(&crypto::hash(user_id.as_bytes()))
    }

    pub fn companies_members_create(&mut self, company_id: &str, user_id: &str, roles: &Vec<CompanyRole>, created: &DateTime<Utc>, transaction: &Hash) {
        let member = {
            let mut history = self.companies_members_history(company_id, user_id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            CompanyMember::new(user_id, roles, created, created, history.len(), &history_hash)
        };
        self.companies_members(company_id).put(&crypto::hash(user_id.as_bytes()), member);
    }

    pub fn companies_members_set_roles(&mut self, company_id: &str, member: CompanyMember, roles: &Vec<CompanyRole>, updated: &DateTime<Utc>, transaction: &Hash) {
        let member = {
            let mut history = self.companies_history(company_id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            member.set_roles(roles, updated, &history_hash)
        };
        self.companies_members(company_id).put(&crypto::hash(member.user_id.as_bytes()), member);
    }

    pub fn companies_members_delete(&mut self, company_id: &str, user_id: &str) {
        self.companies_members(company_id).remove(&crypto::hash(user_id.as_bytes()));
        self.companies_members_history(company_id, user_id).clear();
    }
}

