use chrono::{DateTime, Utc};
use exonum::{
    crypto::{self, Hash, PublicKey},
    storage::{Fork, MapIndex, ProofListIndex, ProofMapIndex, Snapshot},
};
use crate::block::transactions::access::Role;
use crate::block::models::{
    user::{User},
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
}

