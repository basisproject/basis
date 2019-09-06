use std::collections::HashMap;
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
    KeySetIndex,
};
use crate::block::models::access::Role;
use crate::block::models::{
    user::User,
    company::{Company, CompanyType, Role as CompanyRole},
    company_member::CompanyMember,
    product::{Product, ProductVariant},
    order::{Order, ProcessStatus, ProductEntry, ShippingEntry},
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

    // -------------------------------------------------------------------------
    // Products
    // -------------------------------------------------------------------------
    pub fn products(&self) -> ProofMapIndex<T, Hash, Product> {
        ProofMapIndex::new("basis.products.table", self.access.clone())
    }

    pub fn products_idx_company_id(&self, company_id: &str) -> KeySetIndex<T, String> {
        KeySetIndex::new_in_family("basis.products.idx_company_id", &crypto::hash(company_id.as_bytes()), self.access.clone())
    }

    pub fn products_history(&self, id: &str) -> ProofListIndex<T, Hash> {
        ProofListIndex::new_in_family("basis.products.history", &crypto::hash(id.as_bytes()), self.access.clone())
    }

    pub fn get_product(&self, id: &str) -> Option<Product> {
        self.products().get(&crypto::hash(id.as_bytes()))
    }

    pub fn get_products_for_company_id(&self, company_id: &str) -> Vec<Product> {
        self.products_idx_company_id(company_id)
            .iter()
            .map(|x| self.get_product(&x))
            .filter(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect::<Vec<_>>()
    }

    pub fn products_create(&mut self, id: &str, company_id: &str, name: &str, meta: &str, active: bool, created: &DateTime<Utc>, transaction: &Hash) {
        let product = {
            let mut history = self.products_history(id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            Product::new(id, company_id, name, &HashMap::new(), &HashMap::new(), meta, active, &created, &created, None, history.len(), &history_hash)
        };
        self.products().put(&crypto::hash(id.as_bytes()), product);
        self.products_idx_company_id(company_id).insert(id.to_owned());
    }

    pub fn products_update(&mut self, product: Product, name: Option<&str>, meta: Option<&str>, active: Option<bool>, updated: &DateTime<Utc>, transaction: &Hash) {
        let id = product.id.clone();
        let product = {
            let mut history = self.products_history(&id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            product.update(name, meta, active, updated, &history_hash)
        };
        self.products().put(&crypto::hash(id.as_bytes()), product);
    }

    pub fn products_set_option(&mut self, product: Product, name: &str, title: &str, updated: &DateTime<Utc>, transaction: &Hash) {
        let id = product.id.clone();
        let product = {
            let mut history = self.products_history(&id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            product.set_option(name, title, updated, &history_hash)
        };
        self.products().put(&crypto::hash(id.as_bytes()), product);
    }

    pub fn products_remove_option(&mut self, product: Product, name: &str, updated: &DateTime<Utc>, transaction: &Hash) {
        let id = product.id.clone();
        let product = {
            let mut history = self.products_history(&id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            product.remove_option(name, updated, &history_hash)
        };
        self.products().put(&crypto::hash(id.as_bytes()), product);
    }

    pub fn products_set_variant(&mut self, product: Product, variant: &ProductVariant, updated: &DateTime<Utc>, transaction: &Hash) {
        let id = product.id.clone();
        let product = {
            let mut history = self.products_history(&id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            product.set_variant(variant, updated, &history_hash)
        };
        self.products().put(&crypto::hash(id.as_bytes()), product);
    }

    pub fn products_update_variant(&mut self, product: Product, variant_id: &str, name: Option<&str>, active: Option<bool>, meta: Option<&str>, updated: &DateTime<Utc>, transaction: &Hash) {
        let id = product.id.clone();
        let product = {
            let mut history = self.products_history(&id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            product.update_variant(variant_id, name, active, meta, updated, &history_hash)
        };
        self.products().put(&crypto::hash(id.as_bytes()), product);
    }

    pub fn products_remove_variant(&mut self, product: Product, variant_id: &str, updated: &DateTime<Utc>, transaction: &Hash) {
        let id = product.id.clone();
        let product = {
            let mut history = self.products_history(&id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            product.remove_variant(variant_id, updated, &history_hash)
        };
        self.products().put(&crypto::hash(id.as_bytes()), product);
    }

    pub fn products_delete(&mut self, product: Product, deleted: &DateTime<Utc>, transaction: &Hash) {
        let id = product.id.clone();
        let company_id = product.company_id.clone();
        let product = {
            let mut history = self.products_history(&id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            product.delete(deleted, &history_hash)
        };
        self.products().put(&crypto::hash(product.id.as_bytes()), product);
        self.products_idx_company_id(&company_id).remove(&id);
    }

    // -------------------------------------------------------------------------
    // Orders
    // -------------------------------------------------------------------------
    pub fn orders(&self) -> ProofMapIndex<T, Hash, Order> {
        ProofMapIndex::new("basis.orders.table", self.access.clone())
    }

    pub fn orders_history(&self, id: &str) -> ProofListIndex<T, Hash> {
        ProofListIndex::new_in_family("basis.orders.history", &crypto::hash(id.as_bytes()), self.access.clone())
    }

    pub fn get_order(&self, id: &str) -> Option<Order> {
        self.orders().get(&crypto::hash(id.as_bytes()))
    }

    pub fn orders_create(&self, id: &str, company_id_from: &str, company_id_to: &str, products: &Vec<ProductEntry>, created: &DateTime<Utc>, transaction: &Hash) {
        let order = {
            let mut history = self.orders_history(id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            Order::new(id, company_id_from, company_id_to, products, &Default::default(), &ProcessStatus::New, &created, &created, history.len(), &history_hash)
        };
        self.orders().put(&crypto::hash(id.as_bytes()), order);
    }

    pub fn orders_update_status(&self, order: Order, process_status: &ProcessStatus, updated: &DateTime<Utc>, transaction: &Hash) {
        let id = order.id.clone();
        let order = {
            let mut history = self.orders_history(&id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            order.update_status(process_status, updated, &history_hash)
        };
        self.orders().put(&crypto::hash(id.as_bytes()), order);
    }

    pub fn orders_set_shipping(&self, order: Order, shipping: &ShippingEntry, updated: &DateTime<Utc>, transaction: &Hash) {
        let id = order.id.clone();
        let order = {
            let mut history = self.orders_history(&id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            order.set_shipping(shipping, updated, &history_hash)
        };
        self.orders().put(&crypto::hash(id.as_bytes()), order);
    }

    pub fn orders_set_shipping_pickup(&self, order: Order, pickup: &DateTime<Utc>, updated: &DateTime<Utc>, transaction: &Hash) {
        let id = order.id.clone();
        let order = {
            let mut history = self.orders_history(&id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            order.set_shipping_pickup(pickup, updated, &history_hash)
        };
        self.orders().put(&crypto::hash(id.as_bytes()), order);
    }

    pub fn orders_set_shipping_delivered(&self, order: Order, delivered: &DateTime<Utc>, updated: &DateTime<Utc>, transaction: &Hash) {
        let id = order.id.clone();
        let order = {
            let mut history = self.orders_history(&id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            order.set_shipping_delivered(delivered, updated, &history_hash)
        };
        self.orders().put(&crypto::hash(id.as_bytes()), order);
    }
}

