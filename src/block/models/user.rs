use exonum::crypto::{Hash, PublicKey};
use crate::block::models::proto;
use chrono::{DateTime, Utc};
use crate::block::models::access::Role;

#[derive(Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::user::User", serde_pb_convert)]
pub struct User {
    pub id: String,
    pub pubkey: PublicKey,
    pub roles: Vec<Role>,
    pub email: String,
    pub name: String,
    pub meta: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub history_len: u64,
    pub history_hash: Hash,
}

impl User {
    pub fn new(id: &str, &pubkey: &PublicKey, roles: &Vec<Role>, email: &str, name: &str, meta: &str, created: &DateTime<Utc>, updated: &DateTime<Utc>, history_len: u64, &history_hash: &Hash) -> Self {
        Self {
            id: id.to_owned(),
            pubkey,
            roles: roles.clone(),
            email: email.to_owned(),
            name: name.to_owned(),
            meta: meta.to_owned(),
            created: created.clone(),
            updated: updated.clone(),
            history_len,
            history_hash,
        }
    }

    pub fn update(self, email: Option<&str>, name: Option<&str>, meta: Option<&str>, updated: &DateTime<Utc>, history_hash: &Hash) -> Self {
        Self::new(
            &self.id,
            &self.pubkey,
            &self.roles,
            email.unwrap_or(&self.email),
            name.unwrap_or(&self.name),
            meta.unwrap_or(&self.meta),
            &self.created,
            updated,
            self.history_len + 1,
            history_hash
        )
    }

    pub fn set_pubkey(self, pubkey: &PublicKey, updated: &DateTime<Utc>, history_hash: &Hash) -> Self {
        Self::new(
            &self.id,
            pubkey,
            &self.roles,
            &self.email,
            &self.name,
            &self.meta,
            &self.created,
            updated,
            self.history_len + 1,
            history_hash
        )
    }

    pub fn set_roles(self, roles: &Vec<Role>, updated: &DateTime<Utc>, history_hash: &Hash) -> Self {
        Self::new(
            &self.id,
            &self.pubkey,
            roles,
            &self.email,
            &self.name,
            &self.meta,
            &self.created,
            updated,
            self.history_len + 1,
            history_hash
        )
    }
}

pub mod tests {
    use super::*;
    use crate::util;

    fn make_date() -> DateTime<Utc> {
        chrono::offset::Utc::now()
    }

    fn make_hash() -> Hash {
        Hash::new([1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4])
    }

    fn make_user() -> User {
        let date = make_date();

        User::new(
            "0ca3a0d4-63f2-4e5d-8250-f4528506c0d9",
            &PublicKey::new([1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4]),
            &vec![Role::SuperAdmin],
            "socialism.is.when@govt.does.stuff",
            "Carl Mark",
            r#"{"hates":"freedom"}"#,
            &date,
            &date,
            0,
            &make_hash()
        )
    }

    #[test]
    fn updates() {
        let user = make_user();
        util::sleep(100);
        let date2 = make_date();
        let hash = Hash::new([1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4]);
        let user2 = user.clone().update(None, None, None, &date2, &hash);
        assert_eq!(user.id, user2.id);
        assert_eq!(user.pubkey, user2.pubkey);
        assert_eq!(user.roles[0], user2.roles[0]);
        assert_eq!(user.email, user2.email);
        assert_eq!(user.name, user2.name);
        assert_eq!(user.meta, user2.meta);
        assert_eq!(user.created, user2.created);
        assert!(user.updated != user2.updated);
        assert_eq!(user2.updated, date2);
        assert_eq!(user.history_len, user2.history_len - 1);
        assert!(user.history_hash != user2.history_hash);
        assert_eq!(user2.history_hash, hash);
        util::sleep(100);
        let user3 = user.clone().update(Some("taxes.are@socialsim.com"), Some("Henrietta Hamper"), Some(r#"{"best_friend":"literally stalin"}"#), &make_date(), &make_hash());
        assert_eq!(user.id, user3.id);
        assert_eq!(user.pubkey, user3.pubkey);
        assert_eq!(user.roles[0], user3.roles[0]);
        assert_eq!("taxes.are@socialsim.com", user3.email);
        assert_eq!("Henrietta Hamper", user3.name);
        assert_eq!(r#"{"best_friend":"literally stalin"}"#, user3.meta);
    }

    #[test]
    fn sets_pubkey() {
        let user = make_user();
        util::sleep(100);
        let pubkey = PublicKey::new([1, 2, 6, 4, 1, 2, 6, 4, 1, 2, 6, 4, 1, 2, 6, 4, 1, 2, 6, 4, 1, 2, 6, 4, 1, 2, 6, 4, 1, 2, 6, 4]);
        let hash = Hash::new([1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4]);
        let date2 = make_date();
        let user2 = user.clone().set_pubkey(&pubkey, &date2, &hash);
        assert_eq!(user.id, user2.id);
        assert!(user.pubkey != user2.pubkey);
        assert_eq!(user2.pubkey, pubkey);
        assert_eq!(user.created, user2.created);
        assert!(user.updated != user2.updated);
        assert_eq!(user2.updated, date2);
        assert_eq!(user.history_len, user2.history_len - 1);
        assert!(user.history_hash != user2.history_hash);
        assert_eq!(user2.history_hash, hash);
    }

    #[test]
    fn sets_roles() {
        let user = make_user();
        util::sleep(100);
        let date2 = make_date();
        let hash = Hash::new([69, 27, 6, 4, 69, 27, 6, 4, 69, 27, 6, 4, 69, 27, 6, 4, 69, 27, 6, 4, 69, 27, 6, 4, 69, 27, 6, 4, 69, 27, 6, 4]);
        let roles2 = vec![Role::User, Role::IdentityAdmin];
        let user2 = user.clone().set_roles(&roles2, &date2, &hash);
        assert_eq!(user.id, user2.id);
        assert!(user.roles != user2.roles);
        assert_eq!(user2.roles, roles2);
        assert_eq!(user.created, user2.created);
        assert!(user.updated != user2.updated);
        assert_eq!(user2.updated, date2);
        assert_eq!(user.history_len, user2.history_len - 1);
        assert!(user.history_hash != user2.history_hash);
        assert_eq!(user2.history_hash, hash);
    }
}

