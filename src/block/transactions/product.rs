use chrono::{DateTime, Utc};
use exonum::{
    blockchain::{ExecutionError, ExecutionResult, Transaction, TransactionContext},
    crypto::{PublicKey, SecretKey},
    messages::{Message, RawTransaction, Signed},
};
use crate::block::{
    SERVICE_ID,
    schema::Schema,
    models::proto,
    models::company::{Permission as CompanyPermission},
    models::access::Permission,
    models::product::ProductVariant,
    transactions::{company, access},
};
use crate::util::{self, protobuf::empty_opt};
use super::CommonError;

#[derive(Debug, Fail)]
#[repr(u8)]
pub enum TransactionError {
    #[fail(display = "Invalid ID")]
    InvalidID = 0,

    #[fail(display = "Product not found")]
    ProductNotFound = 1,

    #[fail(display = "Company not found")]
    CompanyNotFound = 2,

    #[fail(display = "ID already exists")]
    IDExists = 3,

    #[fail(display = "Product already deleted")]
    AlreadyDeleted = 4,
}
define_exec_error!(TransactionError);

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::product::TxCreate")]
pub struct TxCreate {
    pub id: String,
    pub company_id: String,
    pub name: String,
    pub meta: String,
    pub active: bool,
    pub created: DateTime<Utc>,
}

impl TxCreate {
    #[allow(dead_code)]
    pub fn sign(pk: &PublicKey, sk: &SecretKey, id: &str, company_id: &str, name: &str, meta: &str, active: bool, created: &DateTime<Utc>) -> Signed<RawTransaction> {
        Message::sign_transaction(Self {id: id.to_owned(), company_id: company_id.to_owned(), name: name.to_owned(), meta: meta.to_owned(), active, created: created.clone() }, SERVICE_ID, *pk, sk)
    }
}

impl Transaction for TxCreate {
    fn execute(&self, context: TransactionContext) -> ExecutionResult {
        let pubkey = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        access::check(&mut schema, pubkey, Permission::ProductCreate)?;
        company::check(&mut schema, &self.company_id, pubkey, CompanyPermission::ProductCreate)?;

        if schema.get_product(&self.id).is_some() {
            Err(TransactionError::IDExists)?;
        }
        if !util::time::is_current(&self.created) {
            Err(CommonError::InvalidTime)?;
        }
        schema.products_create(&self.id, &self.company_id, &self.name, &self.meta, self.active, &self.created, &hash);
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::product::TxUpdate")]
pub struct TxUpdate {
    pub id: String,
    pub name: String,
    pub meta: String,
    pub active: bool,
    pub updated: DateTime<Utc>,
}

impl TxUpdate {
    #[allow(dead_code)]
    pub fn sign(pk: &PublicKey, sk: &SecretKey, id: &str, name: &str, meta: &str, active: bool, updated: &DateTime<Utc>) -> Signed<RawTransaction> {
        Message::sign_transaction(Self {id: id.to_owned(), name: name.to_owned(), meta: meta.to_owned(), active, updated: updated.clone() }, SERVICE_ID, *pk, sk)
    }
}

impl Transaction for TxUpdate {
    fn execute(&self, context: TransactionContext) -> ExecutionResult {
        let pubkey = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        let prod = schema.get_product(&self.id);
        if prod.is_none() {
            Err(TransactionError::ProductNotFound)?;
        }

        let product = prod.unwrap();
        access::check(&mut schema, pubkey, Permission::ProductUpdate)?;
        company::check(&mut schema, &product.company_id, pubkey, CompanyPermission::ProductUpdate)?;

        let name = empty_opt(&self.name).map(|x| x.as_str());
        let meta = empty_opt(&self.meta).map(|x| x.as_str());
        let active = Some(self.active);

        if !util::time::is_current(&self.updated) {
            Err(CommonError::InvalidTime)?;
        }
        schema.products_update(product, name, meta, active, &self.updated, &hash);
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::product::TxSetOption")]
pub struct TxSetOption {
    pub id: String,
    pub name: String,
    pub title: String,
    pub updated: DateTime<Utc>,
}

impl TxSetOption {
    #[allow(dead_code)]
    pub fn sign(pk: &PublicKey, sk: &SecretKey, id: &str, name: &str, title: &str, updated: &DateTime<Utc>) -> Signed<RawTransaction> {
        Message::sign_transaction(Self {id: id.to_owned(), name: name.to_owned(), title: title.to_owned(), updated: updated.clone() }, SERVICE_ID, *pk, sk)
    }
}

impl Transaction for TxSetOption {
    fn execute(&self, context: TransactionContext) -> ExecutionResult {
        let pubkey = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        let prod = schema.get_product(&self.id);
        if prod.is_none() {
            Err(TransactionError::ProductNotFound)?;
        }

        let product = prod.unwrap();
        access::check(&mut schema, pubkey, Permission::ProductUpdate)?;
        company::check(&mut schema, &product.company_id, pubkey, CompanyPermission::ProductUpdate)?;

        if !util::time::is_current(&self.updated) {
            Err(CommonError::InvalidTime)?;
        }
        schema.products_set_option(product, &self.name, &self.title, &self.updated, &hash);
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::product::TxRemoveOption")]
pub struct TxRemoveOption {
    pub id: String,
    pub name: String,
    pub updated: DateTime<Utc>,
}

impl TxRemoveOption {
    #[allow(dead_code)]
    pub fn sign(pk: &PublicKey, sk: &SecretKey, id: &str, name: &str, updated: &DateTime<Utc>) -> Signed<RawTransaction> {
        Message::sign_transaction(Self {id: id.to_owned(), name: name.to_owned(), updated: updated.clone() }, SERVICE_ID, *pk, sk)
    }
}

impl Transaction for TxRemoveOption {
    fn execute(&self, context: TransactionContext) -> ExecutionResult {
        let pubkey = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        let prod = schema.get_product(&self.id);
        if prod.is_none() {
            Err(TransactionError::ProductNotFound)?;
        }

        let product = prod.unwrap();
        access::check(&mut schema, pubkey, Permission::ProductUpdate)?;
        company::check(&mut schema, &product.company_id, pubkey, CompanyPermission::ProductUpdate)?;

        if !util::time::is_current(&self.updated) {
            Err(CommonError::InvalidTime)?;
        }
        schema.products_remove_option(product, &self.name, &self.updated, &hash);
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::product::TxSetVariant")]
pub struct TxSetVariant {
    pub id: String,
    pub variant: ProductVariant,
    pub updated: DateTime<Utc>,
}

impl TxSetVariant {
    #[allow(dead_code)]
    pub fn sign(pk: &PublicKey, sk: &SecretKey, id: &str, variant: &ProductVariant, updated: &DateTime<Utc>) -> Signed<RawTransaction> {
        Message::sign_transaction(Self {id: id.to_owned(), variant: variant.clone(), updated: updated.clone() }, SERVICE_ID, *pk, sk)
    }
}

impl Transaction for TxSetVariant {
    fn execute(&self, context: TransactionContext) -> ExecutionResult {
        let pubkey = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        let prod = schema.get_product(&self.id);
        if prod.is_none() {
            Err(TransactionError::ProductNotFound)?;
        }

        let product = prod.unwrap();
        access::check(&mut schema, pubkey, Permission::ProductUpdate)?;
        company::check(&mut schema, &product.company_id, pubkey, CompanyPermission::ProductUpdate)?;

        if !util::time::is_current(&self.updated) {
            Err(CommonError::InvalidTime)?;
        }
        schema.products_set_variant(product, &self.variant, &self.updated, &hash);
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::product::TxUpdateVariant")]
pub struct TxUpdateVariant {
    pub id: String,
    pub variant_id: String,
    pub name: String,
    pub active: bool,
    pub meta: String,
    pub updated: DateTime<Utc>,
}

impl TxUpdateVariant {
    #[allow(dead_code)]
    pub fn sign(pk: &PublicKey, sk: &SecretKey, id: &str, variant_id: &str, name: &str, active: bool, meta: &str, updated: &DateTime<Utc>) -> Signed<RawTransaction> {
        Message::sign_transaction(Self {id: id.to_owned(), variant_id: variant_id.to_owned(), name: name.to_owned(), active, meta: meta.to_owned(), updated: updated.clone() }, SERVICE_ID, *pk, sk)
    }
}

impl Transaction for TxUpdateVariant {
    fn execute(&self, context: TransactionContext) -> ExecutionResult {
        let pubkey = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        let prod = schema.get_product(&self.id);
        if prod.is_none() {
            Err(TransactionError::ProductNotFound)?;
        }

        let product = prod.unwrap();
        access::check(&mut schema, pubkey, Permission::ProductUpdate)?;
        company::check(&mut schema, &product.company_id, pubkey, CompanyPermission::ProductUpdate)?;

        if !util::time::is_current(&self.updated) {
            Err(CommonError::InvalidTime)?;
        }

        let name = empty_opt(&self.name).map(|x| x.as_str());
        let active = Some(self.active);
        let meta = empty_opt(&self.meta).map(|x| x.as_str());
        schema.products_update_variant(product, &self.variant_id, name, active, meta, &self.updated, &hash);
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::product::TxRemoveVariant")]
pub struct TxRemoveVariant {
    pub id: String,
    pub variant_id: String,
    pub updated: DateTime<Utc>,
}

impl TxRemoveVariant {
    #[allow(dead_code)]
    pub fn sign(pk: &PublicKey, sk: &SecretKey, id: &str, variant_id: &str, updated: &DateTime<Utc>) -> Signed<RawTransaction> {
        Message::sign_transaction(Self {id: id.to_owned(), variant_id: variant_id.to_owned(), updated: updated.clone() }, SERVICE_ID, *pk, sk)
    }
}

impl Transaction for TxRemoveVariant {
    fn execute(&self, context: TransactionContext) -> ExecutionResult {
        let pubkey = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        let prod = schema.get_product(&self.id);
        if prod.is_none() {
            Err(TransactionError::ProductNotFound)?;
        }

        let product = prod.unwrap();
        access::check(&mut schema, pubkey, Permission::ProductUpdate)?;
        company::check(&mut schema, &product.company_id, pubkey, CompanyPermission::ProductUpdate)?;

        if !util::time::is_current(&self.updated) {
            Err(CommonError::InvalidTime)?;
        }

        schema.products_remove_variant(product, &self.variant_id, &self.updated, &hash);
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::product::TxDelete")]
pub struct TxDelete {
    pub id: String,
    pub deleted: DateTime<Utc>,
}

impl TxDelete {
    #[allow(dead_code)]
    pub fn sign(pk: &PublicKey, sk: &SecretKey, id: &str, deleted: &DateTime<Utc>) -> Signed<RawTransaction> {
        Message::sign_transaction(Self {id: id.to_owned(), deleted: deleted.clone() }, SERVICE_ID, *pk, sk)
    }
}

impl Transaction for TxDelete {
    fn execute(&self, context: TransactionContext) -> ExecutionResult {
        let pubkey = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        if !util::time::is_current(&self.deleted) {
            Err(CommonError::InvalidTime)?;
        }

        let prod = schema.get_product(&self.id);
        if prod.is_none() {
            Err(TransactionError::ProductNotFound)?;
        }
        let product = prod.unwrap();

        access::check(&mut schema, pubkey, Permission::ProductDelete)?;
        company::check(&mut schema, &product.company_id, pubkey, CompanyPermission::ProductDelete)?;

        if product.is_deleted() {
            Err(TransactionError::AlreadyDeleted)?;
        }
        schema.products_delete(product, &self.deleted, &hash);
        Ok(())
    }
}

