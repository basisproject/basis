use chrono::{DateTime, Utc};
use exonum::{
    crypto::{self, Hash, PublicKey},
};
use exonum_merkledb::{
    IndexAccess,
    ObjectHash,
    ListIndex,
    MapIndex,
    ProofListIndex,
    ProofMapIndex,
    KeySetIndex,
};
use util;
use models::{
    access::Role,
    user::User,
    company::{Company, CompanyType, Role as CompanyRole},
    company_member::CompanyMember,
    labor::Labor,
    product::{Product, Unit, Dimensions},
    resource_tag::ResourceTag,
    order::{Order, ProcessStatus, ProductEntry},
    costs::{Costs, CostsTallyMap},
    cost_tag::{CostTag, CostTagEntry, Costable},
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

/// Given a rotational index key, return a DateTime object
fn key_to_datetime(key: &str) -> DateTime<Utc> {
    match key.find(":") {
        Some(x) => {
            match &key[0..x].parse::<i64>() {
                Ok(ts) => util::time::from_timestamp(*ts),
                Err(_) => util::time::default_time(),
            }
        }
        None => util::time::default_time(),
    }
}

/// Given a rotational index key, return the id of the object
fn key_to_id(key: &str) -> String {
    match key.find(":") {
        Some(x) => String::from(&key[(x + 1)..]),
        None => key.to_owned()
    }
}

/// Given a time-rotated index, test if the given timestamp is obsolete (as in,
/// is older than the oldest record in the index, signifying that it has been
/// rotated out and no longer belongs in the index).
fn is_rotate_record_obsolete<T>(idx: &mut MapIndex<T, String, String>, timestamp: i64) -> bool
    where T: IndexAccess,
{
    match idx.keys().next() {
        Some(key) => {
            let key_date = key_to_datetime(&key);
            timestamp < key_date.timestamp()
        }
        None => false,
    }
}

fn index_and_rotate_mapindex<T, F>(idx: &mut MapIndex<T, String, String>, timestamp: i64, item_id: &str, cutoff: &DateTime<Utc>, mut op_cb: F)
    where T: IndexAccess,
          F: FnMut(String, bool),
{
    let key = format!("{}:{}", timestamp, item_id);

    op_cb(item_id.to_owned(), false);
    idx.put(&key, item_id.to_owned());
    let mut remove_keys = Vec::new();
    for k in idx.keys() {
        let date = key_to_datetime(&k);
        if date < *cutoff {
            remove_keys.push(k);
        } else {
            break;
        }
    }
    for k in &remove_keys {
        let item_id = key_to_id(k);
        op_cb(item_id, true);
        idx.remove(k);
    }
}

impl<T> Schema<T>
    where T: IndexAccess
{
    pub fn new(access: T) -> Self {
        Schema { access }
    }

    pub fn state_hash(&self) -> Vec<Hash> {
        vec![
            self.users().object_hash(),
            self.companies().object_hash(),
            self.companies_members().object_hash(),
            self.labor().object_hash(),
            self.products().object_hash(),
            self.resource_tags().object_hash(),
            self.orders().object_hash(),
        ]
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

    pub fn companies_create(&mut self, id: &str, ty: &CompanyType, region_id: Option<&str>, email: &str, name: &str, created: &DateTime<Utc>, transaction: &Hash) {
        let company = {
            let mut history = self.companies_history(id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            Company::new(id, ty, region_id, email, name, created, created, history.len(), &history_hash)
        };
        self.companies().put(&crypto::hash(id.as_bytes()), company.clone());
    }

    pub fn companies_update(&mut self, company: Company, email: Option<&str>, name: Option<&str>, updated: &DateTime<Utc>, transaction: &Hash) {
        let company = {
            let mut history = self.companies_history(&company.id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            company.update(email, name, updated, &history_hash)
        };
        self.companies().put(&crypto::hash(company.id.as_bytes()), company);
    }

    pub fn companies_set_type(&mut self, company: Company, ty: &CompanyType, updated: &DateTime<Utc>, transaction: &Hash) {
        let company = {
            let mut history = self.companies_history(&company.id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            company.set_type(ty, updated, &history_hash)
        };
        self.companies().put(&crypto::hash(company.id.as_bytes()), company);
    }

    pub fn companies_delete(&mut self, id: &str) {
        self.companies().remove(&crypto::hash(id.as_bytes()));
        self.companies_members_delete_by_company(id);
        self.companies_history(id).clear();
    }

    // -------------------------------------------------------------------------
    // Company members
    // -------------------------------------------------------------------------
    pub fn companies_members(&self) -> ProofMapIndex<T, Hash, CompanyMember> {
        ProofMapIndex::new("basis.companies_members.table", self.access.clone())
    }

    pub fn companies_members_history(&self, id: &str) -> ProofListIndex<T, Hash> {
        ProofListIndex::new_in_family("basis.companies_members.history", &crypto::hash(id.as_bytes()), self.access.clone())
    }

    pub fn companies_members_idx_company_id(&self, company_id: &str) -> MapIndex<T, String, String> {
        MapIndex::new_in_family("basis.companies_members.idx_company_id", &crypto::hash(company_id.as_bytes()), self.access.clone())
    }

    pub fn get_company_member(&self, id: &str) -> Option<CompanyMember> {
        self.companies_members().get(&crypto::hash(id.as_bytes()))
    }

    pub fn get_company_member_by_company_id_user_id(&self, company_id: &str, user_id: &str) -> Option<CompanyMember> {
        self.companies_members_idx_company_id(company_id).get(user_id)
            .and_then(|member_id| { self.get_company_member(&member_id) })
    }

    pub fn companies_members_create(&mut self, id: &str, company_id: &str, user_id: &str, roles: &Vec<CompanyRole>, occupation: &str, wage: f64, default_cost_tags: &Vec<CostTagEntry>, created: &DateTime<Utc>, transaction: &Hash) {
        let member = {
            let mut history = self.companies_members_history(id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            CompanyMember::new(id, company_id, user_id, roles, occupation, wage, default_cost_tags, created, created, history.len(), &history_hash)
        };
        self.companies_members().put(&crypto::hash(id.as_bytes()), member);
        self.companies_members_idx_company_id(company_id).put(&user_id.to_owned(), id.to_owned());
    }

    pub fn companies_members_update(&mut self, member: CompanyMember, roles: Option<&Vec<CompanyRole>>, occupation: Option<&str>, wage: Option<f64>, default_cost_tags: Option<&Vec<CostTagEntry>>, updated: &DateTime<Utc>, transaction: &Hash) {
        let member = {
            let mut history = self.companies_members_history(&member.id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            member.update(roles, occupation, wage, default_cost_tags, updated, &history_hash)
        };
        self.companies_members().put(&crypto::hash(member.id.as_bytes()), member);
    }

    pub fn companies_members_delete(&mut self, member: CompanyMember) {
        self.companies_members().remove(&crypto::hash(member.id.as_bytes()));
        self.companies_members_idx_company_id(&member.company_id).remove(&member.user_id);
        self.companies_members_history(&member.id).clear();
    }

    pub fn companies_members_delete_by_company(&mut self, company_id: &str) {
        // don't delete while iterating...
        let mut members = Vec::new();
        for (user_id, member_id) in self.companies_members_idx_company_id(company_id).iter() {
            members.push((user_id, member_id));
        }
        for (user_id, member_id) in members {
            let tmp_member = CompanyMember::new(&member_id, company_id, &user_id, &vec![], "tmp", 0.0, &vec![], &util::time::now(), &util::time::now(), 0, &Default::default());
            self.companies_members_delete(tmp_member);
        }
        self.companies_members_idx_company_id(company_id).clear();
    }

    // -------------------------------------------------------------------------
    // Labor
    // -------------------------------------------------------------------------
    pub fn labor(&self) -> ProofMapIndex<T, Hash, Labor> {
        ProofMapIndex::new("basis.labor.table", self.access.clone())
    }

    pub fn labor_history(&self, id: &str) -> ProofListIndex<T, Hash> {
        ProofListIndex::new_in_family("basis.labor.history", &crypto::hash(id.as_bytes()), self.access.clone())
    }

    pub fn labor_idx_company_id(&self, company_id: &str) -> ListIndex<T, String> {
        ListIndex::new_in_family("basis.labor.idx_company_id", &crypto::hash(company_id.as_bytes()), self.access.clone())
    }

    pub fn labor_idx_company_id_rolling(&self, company_id: &str) -> MapIndex<T, String, String> {
        MapIndex::new_in_family("basis.labor.idx_company_id_rolling", &crypto::hash(company_id.as_bytes()), self.access.clone())
    }

    pub fn get_labor(&self, id: &str) -> Option<Labor> {
        self.labor().get(&crypto::hash(id.as_bytes()))
    }

    pub fn get_labor_recent(&self, company_id: &str) -> Vec<Labor> {
        self.labor_idx_company_id_rolling(company_id)
            .values()
            .map(|x| self.get_labor(&x))
            .filter(|x| x.is_some())
            .map(|x| x.unwrap())
            .filter(|x| x.is_finalized())
            .collect::<Vec<_>>()
    }

    pub fn labor_create(&mut self, id: &str, company_id: &str, user_id: &str, occupation: &str, wage: f64, cost_tags: &Vec<CostTagEntry>, created: &DateTime<Utc>, transaction: &Hash) {
        let labor = {
            let mut history = self.labor_history(id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            Labor::new(id, company_id, user_id, occupation, wage, cost_tags, Some(created), None, created, created, history.len(), &history_hash)
        };
        self.labor().put(&crypto::hash(id.as_bytes()), labor.clone());
        self.labor_idx_company_id(company_id).push(id.to_owned());
        self.labor_update_rolling_index(&labor, None);
    }

    pub fn labor_update(&mut self, labor: Labor, cost_tags: Option<&Vec<CostTagEntry>>, start: Option<&DateTime<Utc>>, end: Option<&DateTime<Utc>>, updated: &DateTime<Utc>, transaction: &Hash) {
        let id = labor.id.clone();
        let labor_original = labor.clone();
        let labor = {
            let mut history = self.labor_history(&id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            labor.update(cost_tags, start, end, updated, &history_hash)
        };
        self.labor().put(&crypto::hash(id.as_bytes()), labor.clone());
        self.labor_update_rolling_index(&labor, Some(&labor_original));
    }

    pub fn labor_set_wage(&mut self, labor: Labor, wage: f64, updated: &DateTime<Utc>, transaction: &Hash) {
        let id = labor.id.clone();
        let labor_original = labor.clone();
        let labor = {
            let mut history = self.labor_history(&id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            labor.set_wage(wage, updated, &history_hash)
        };
        self.labor().put(&crypto::hash(id.as_bytes()), labor.clone());
        self.labor_update_rolling_index(&labor, Some(&labor_original));
    }

    fn labor_update_rolling_index(&self, labor: &Labor, original: Option<&Labor>) {
        let mut idx = self.labor_idx_company_id_rolling(&labor.company_id);
        if is_rotate_record_obsolete(&mut idx, labor.created.timestamp()) {
            return;
        }
        // one year cutoff, hardcoded for now
        let cutoff = util::time::from_timestamp(labor.created.timestamp() - (3600 * 24 * 365));
        let labor_tbl = self.labor();
        let mut cost_agg = self.costs_aggregate(&labor.company_id);
        let mut bucket_map_labor = match cost_agg.get("labor.v1") {
            Some(x) => x,
            None => CostsTallyMap::new(),
        };

        let mut op_cb_impl = |labor: Labor, is_remove: bool| {
            if !labor.is_finalized() {
                return;
            }
            info!("labor::rolling::agg::{} -- company {}", if is_remove { "remove" } else { "add" }, labor.company_id);
            let tagged_costs = labor.get_tagged_costs();
            if is_remove {
                bucket_map_labor.subtract_map(&tagged_costs);
            } else {
                bucket_map_labor.add_map(&tagged_costs);
            }
        };
        original.map(|x| op_cb_impl(x.clone(), true));
        let op_cb = |labor_id: String, is_remove: bool| {
            let labor = match labor_tbl.get(&crypto::hash(labor_id.as_bytes())) {
                Some(x) => x,
                None => return,
            };
            op_cb_impl(labor, is_remove);
        };
        index_and_rotate_mapindex(&mut idx, labor.created.timestamp(), &labor.id, &cutoff, op_cb);
        cost_agg.put(&String::from("labor.v1"), bucket_map_labor);
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

    pub fn products_idx_company_active(&self, company_id: &str) -> KeySetIndex<T, String> {
        KeySetIndex::new_in_family("basis.products.idx_company_active", &crypto::hash(company_id.as_bytes()), self.access.clone())
    }

    pub fn products_history(&self, id: &str) -> ProofListIndex<T, Hash> {
        ProofListIndex::new_in_family("basis.products.history", &crypto::hash(id.as_bytes()), self.access.clone())
    }

    pub fn get_product(&self, id: &str) -> Option<Product> {
        self.products().get(&crypto::hash(id.as_bytes()))
    }

    pub fn get_products_by_company_id(&self, company_id: &str) -> Vec<Product> {
        self.products_idx_company_id(company_id)
            .iter()
            .map(|x| self.get_product(&x))
            .filter(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect::<Vec<_>>()
    }

    pub fn get_active_products_for_company(&self, company_id: &str) -> Vec<Product> {
        self.products_idx_company_active(company_id)
            .iter()
            .map(|x| self.get_product(&x))
            .filter(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect::<Vec<_>>()
    }

    pub fn get_active_products_for_company_extended(&self, company_id: &str) -> Vec<(Product, Option<Costs>, Option<ResourceTag>)> {
        self.products_idx_company_active(company_id)
            .iter()
            .map(|x| self.get_product_with_costs_tagged(&x))
            .filter(|(x, ..)| x.is_some())
            .map(|(x, y, z)| (x.unwrap(), y, z))
            .collect::<Vec<_>>()
    }

    pub fn products_create(&mut self, id: &str, company_id: &str, name: &str, unit: &Unit, mass_mg: f64, dimensions: &Dimensions, cost_tags: &Vec<CostTagEntry>, active: bool, meta: &str, created: &DateTime<Utc>, transaction: &Hash) {
        let product = {
            let mut history = self.products_history(id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            Product::new(id, company_id, name, unit, mass_mg, dimensions, cost_tags, active, meta, created, created, None, history.len(), &history_hash)
        };
        let active = product.active;
        self.products().put(&crypto::hash(id.as_bytes()), product);
        self.products_idx_company_id(company_id).insert(id.to_owned());
        if active {
            self.products_idx_company_active(company_id).insert(id.to_owned());
        }
    }

    pub fn products_update(&mut self, product: Product, name: Option<&str>, unit: Option<&Unit>, mass_mg: Option<f64>, dimensions: Option<&Dimensions>, cost_tags: Option<&Vec<CostTagEntry>>, active: Option<bool>, meta: Option<&str>, updated: &DateTime<Utc>, transaction: &Hash) {
        let id = product.id.clone();
        let product = {
            let mut history = self.products_history(&id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            product.update(name, unit, mass_mg, dimensions, cost_tags, active, meta, updated, &history_hash)
        };
        let active = product.active;
        let company_id = product.company_id.clone();
        self.products().put(&crypto::hash(id.as_bytes()), product);
        if active {
            self.products_idx_company_active(&company_id).insert(id);
        } else {
            self.products_idx_company_active(&company_id).remove(&id);
        }
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
        self.products_idx_company_active(&company_id).remove(&id);
    }

    // -------------------------------------------------------------------------
    // Resource tags
    // -------------------------------------------------------------------------
    pub fn resource_tags(&self) -> ProofMapIndex<T, Hash, ResourceTag> {
        ProofMapIndex::new("basis.resource_tags.table", self.access.clone())
    }

    pub fn resource_tags_history(&self, id: &str) -> ProofListIndex<T, Hash> {
        ProofListIndex::new_in_family("basis.resource_tags.history", &crypto::hash(id.as_bytes()), self.access.clone())
    }

    pub fn resource_tags_idx_product_id(&self) -> MapIndex<T, String, String> {
        MapIndex::new("basis.resource_tags.idx_product_id", self.access.clone())
    }

    pub fn get_resource_tag(&self, id: &str) -> Option<ResourceTag> {
        self.resource_tags().get(&crypto::hash(id.as_bytes()))
    }

    pub fn get_resource_tag_by_product_id(&self, product_id: &str) -> Option<ResourceTag> {
        self.resource_tags_idx_product_id().get(product_id)
            .and_then(|tag_id| self.get_resource_tag(&tag_id))
    }

    pub fn resource_tags_create(&mut self, id: &str, product_id: &str, created: &DateTime<Utc>, transaction: &Hash) {
        let resource_tag = {
            let mut history = self.resource_tags_history(id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            ResourceTag::new(id, product_id, created, created, None, history.len(), &history_hash)
        };
        let product_id = resource_tag.product_id.clone();
        self.resource_tags().put(&crypto::hash(id.as_bytes()), resource_tag);
        self.resource_tags_idx_product_id().put(&product_id, id.to_owned());
    }

    pub fn resource_tags_delete(&mut self, resource_tag: ResourceTag, deleted: &DateTime<Utc>, transaction: &Hash) {
        let id = resource_tag.id.clone();
        let product_id = resource_tag.product_id.clone();
        let resource_tag = {
            let mut history = self.resource_tags_history(&id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            resource_tag.delete(deleted, &history_hash)
        };
        self.resource_tags().put(&crypto::hash(id.as_bytes()), resource_tag);
        self.resource_tags_idx_product_id().remove(&product_id);
    }

    // -------------------------------------------------------------------------
    // Product costs
    // -------------------------------------------------------------------------
    pub fn product_costs(&self) -> MapIndex<T, String, Costs> {
        MapIndex::new("basis.product_costs.table", self.access.clone())
    }

    pub fn costs_aggregate(&self, company_id: &str) -> MapIndex<T, String, CostsTallyMap> {
        MapIndex::new_in_family("basis.costs_aggregate.table", &crypto::hash(company_id.as_bytes()), self.access.clone())
    }

    pub fn get_product_costs(&self, product_id: &str) -> Option<Costs> {
        self.product_costs().get(product_id)
    }

    pub fn get_product_with_costs_tagged(&self, product_id: &str) -> (Option<Product>, Option<Costs>, Option<ResourceTag>) {
        let product = self.get_product(product_id);
        let (costs, tag) = if product.is_some() {
            (self.get_product_costs(product_id), self.get_resource_tag_by_product_id(product_id))
        } else {
            (None, None)
        };
        (product, costs, tag)
    }

    pub fn product_costs_attach(&self, product_id: &str, costs: &Costs) {
        self.product_costs().put(&product_id.to_string(), costs.clone());
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

    pub fn orders_idx_company_id_from(&self, company_id: &str) -> ListIndex<T, String> {
        ListIndex::new_in_family("basis.orders.idx_company_id_from", &crypto::hash(company_id.as_bytes()), self.access.clone())
    }

    pub fn orders_idx_company_id_to(&self, company_id: &str) -> ListIndex<T, String> {
        ListIndex::new_in_family("basis.orders.idx_company_id_to", &crypto::hash(company_id.as_bytes()), self.access.clone())
    }

    pub fn orders_idx_company_id_from_rolling(&self, company_id: &str) -> MapIndex<T, String, String> {
        MapIndex::new_in_family("basis.orders.idx_company_id_from_rolling", &crypto::hash(company_id.as_bytes()), self.access.clone())
    }

    pub fn orders_idx_company_id_to_rolling(&self, company_id: &str) -> MapIndex<T, String, String> {
        MapIndex::new_in_family("basis.orders.idx_company_id_to_rolling", &crypto::hash(company_id.as_bytes()), self.access.clone())
    }

    pub fn get_order(&self, id: &str) -> Option<Order> {
        self.orders().get(&crypto::hash(id.as_bytes()))
    }

    pub fn get_orders_incoming_recent(&self, company_id: &str) -> Vec<Order> {
        self.orders_idx_company_id_to_rolling(company_id)
            .values()
            .map(|x| self.get_order(&x))
            .filter(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect::<Vec<_>>()
    }

    pub fn get_orders_outgoing_recent(&self, company_id: &str) -> Vec<Order> {
        self.orders_idx_company_id_from_rolling(company_id)
            .values()
            .map(|x| self.get_order(&x))
            .filter(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect::<Vec<_>>()
    }

    pub fn orders_create(&self, id: &str, company_id_from: &str, company_id_to: &str, cost_tags: &Vec<CostTagEntry>, products: &Vec<ProductEntry>, created: &DateTime<Utc>, transaction: &Hash) {
        let order = {
            let mut history = self.orders_history(id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            Order::new(id, company_id_from, company_id_to, cost_tags, products, &ProcessStatus::New, &created, &created, history.len(), &history_hash)
        };
        let id = order.id.clone();
        self.orders().put(&crypto::hash(id.as_bytes()), order.clone());
        self.orders_idx_company_id_from(company_id_from).push(id.clone());
        self.orders_idx_company_id_to(company_id_to).push(id.clone());
        self.orders_update_rolling_index(&order, None);
    }

    pub fn orders_update_status(&self, order: Order, process_status: &ProcessStatus, updated: &DateTime<Utc>, transaction: &Hash) {
        let id = order.id.clone();
        let order_original = order.clone();
        let order = {
            let mut history = self.orders_history(&id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            order.update_status(process_status, updated, &history_hash)
        };
        self.orders().put(&crypto::hash(id.as_bytes()), order.clone());
        self.orders_update_rolling_index(&order, Some(&order_original));
    }

    pub fn orders_update_cost_tags(&self, order: Order, cost_tags: &Vec<CostTagEntry>, updated: &DateTime<Utc>, transaction: &Hash) {
        let id = order.id.clone();
        let order_original = order.clone();
        let order = {
            let mut history = self.orders_history(&id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            order.update_cost_tags(cost_tags, updated, &history_hash)
        };
        self.orders().put(&crypto::hash(id.as_bytes()), order.clone());
        self.orders_update_rolling_index(&order, Some(&order_original));
    }

    fn orders_update_rolling_index(&self, order: &Order, original: Option<&Order>) {
        let mut idx_from = self.orders_idx_company_id_from_rolling(&order.company_id_from);
        let mut idx_to = self.orders_idx_company_id_to_rolling(&order.company_id_to);
        if is_rotate_record_obsolete(&mut idx_from, order.created.timestamp()) {
            return;
        }
        // one year cutoff, hardcoded for now
        let cutoff = util::time::from_timestamp(order.created.timestamp() - (3600 * 24 * 365));
        let order_tbl = self.orders();

        // company from (the company making the order) is going to track this
        // order as costs
        let mut cost_agg = self.costs_aggregate(&order.company_id_from);
        let mut bucket_map_costs = match cost_agg.get("costs.v1") {
            Some(x) => x,
            None => CostsTallyMap::new(),
        };
        let mut op_cb_impl = |order: Order, is_remove: bool| {
            if order.process_status != ProcessStatus::Finalized {
                return;
            }
            info!("orders::rolling::agg::{} -- company {}", if is_remove { "remove" } else { "add" }, order.company_id_from);
            let tagged_costs = order.get_tagged_costs();
            if is_remove {
                bucket_map_costs.subtract_map(&tagged_costs);
            } else {
                bucket_map_costs.add_map(&tagged_costs);
            }
        };
        original.map(|x| op_cb_impl(x.clone(), true));
        let op_cb = |order_id: String, is_remove: bool| {
            let order = match order_tbl.get(&crypto::hash(order_id.as_bytes())) {
                Some(x) => x,
                None => return,
            };
            op_cb_impl(order, is_remove)
        };
        index_and_rotate_mapindex(&mut idx_from, order.created.timestamp(), &order.id, &cutoff, op_cb);
        cost_agg.put(&String::from("costs.v1"), bucket_map_costs);

        // company to (the receiver) is going to track this order as product
        // output(s)
        let mut cost_agg = self.costs_aggregate(&order.company_id_to);
        let mut bucket_map_outputs = match cost_agg.get("product_outputs.v1") {
            Some(x) => x,
            None => CostsTallyMap::new(),
        };
        // one year cutoff, hardcoded for now
        let cutoff = util::time::from_timestamp(order.created.timestamp() - (3600 * 24 * 365));
        let mut op_cb_impl = |order: Order, is_remove: bool| {
            if order.process_status != ProcessStatus::Finalized {
                return;
            }
            info!("orders::rolling::agg::{} -- company {}", if is_remove { "remove" } else { "add" }, order.company_id_from);
            let mut outputs = Costs::new();
            for entry in &order.products {
                outputs.track(&entry.product_id, entry.quantity);
            }
            if is_remove {
                bucket_map_outputs.subtract("outputs", &outputs);
            } else {
                bucket_map_outputs.add("outputs", &outputs);
            }
        };
        match original {
            Some(x) => op_cb_impl(x.clone(), true),
            None => {}
        }
        let op_cb = |order_id: String, is_remove: bool| {
            let order = match order_tbl.get(&crypto::hash(order_id.as_bytes())) {
                Some(x) => x,
                None => return,
            };
            op_cb_impl(order, is_remove)
        };
        index_and_rotate_mapindex(&mut idx_to, order.created.timestamp(), &order.id, &cutoff, op_cb);
        cost_agg.put(&String::from("product_outputs.v1"), bucket_map_outputs);
    }

    // -------------------------------------------------------------------------
    // Cost tags
    // -------------------------------------------------------------------------
    pub fn cost_tags(&self) -> ProofMapIndex<T, Hash, CostTag> {
        ProofMapIndex::new("basis.cost_tags.table", self.access.clone())
    }

    pub fn cost_tags_history(&self, id: &str) -> ProofListIndex<T, Hash> {
        ProofListIndex::new_in_family("basis.cost_tags.history", &crypto::hash(id.as_bytes()), self.access.clone())
    }

    pub fn cost_tags_idx_company_id(&self, company_id: &str) -> KeySetIndex<T, String> {
        KeySetIndex::new_in_family("basis.cost_tags.idx_company_id", &crypto::hash(company_id.as_bytes()), self.access.clone())
    }

    pub fn get_cost_tag(&self, id: &str) -> Option<CostTag> {
        self.cost_tags().get(&crypto::hash(id.as_bytes()))
    }

    pub fn get_cost_tags_by_company_id(&self, company_id: &str) -> Vec<CostTag> {
        self.cost_tags_idx_company_id(company_id)
            .iter()
            .map(|id| self.get_cost_tag(&id))
            .filter(|ct| ct.is_some())
            .map(|ct| ct.unwrap())
            .collect::<Vec<_>>()
    }

    pub fn cost_tags_create(&mut self, id: &str, company_id: &str, name: &str, active: bool, meta: &str, created: &DateTime<Utc>, transaction: &Hash) {
        let cost_tag = {
            let mut history = self.cost_tags_history(id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            CostTag::new(id, company_id, name, active, meta, created, created, None, history.len(), &history_hash)
        };
        let company_id = cost_tag.company_id.clone();
        self.cost_tags().put(&crypto::hash(id.as_bytes()), cost_tag);
        self.cost_tags_idx_company_id(&company_id).insert(id.to_owned());
    }

    pub fn cost_tags_update(&mut self, cost_tag: CostTag, name: Option<&str>, active: Option<bool>, meta: Option<&str>, updated: &DateTime<Utc>, transaction: &Hash) {
        let id = cost_tag.id.clone();
        let cost_tag = {
            let mut history = self.cost_tags_history(&id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            cost_tag.update(name, active, meta, updated, &history_hash)
        };
        self.cost_tags().put(&crypto::hash(id.as_bytes()), cost_tag);
    }

    pub fn cost_tags_delete(&mut self, cost_tag: CostTag, deleted: &DateTime<Utc>, transaction: &Hash) {
        let id = cost_tag.id.clone();
        let company_id = cost_tag.company_id.clone();
        let cost_tag = {
            let mut history = self.cost_tags_history(&id);
            history.push(*transaction);
            let history_hash = history.object_hash();
            cost_tag.delete(deleted, &history_hash)
        };
        self.cost_tags().put(&crypto::hash(id.as_bytes()), cost_tag);
        self.cost_tags_idx_company_id(&company_id).remove(&id);
    }
}

