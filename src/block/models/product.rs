use std::collections::HashMap;
use exonum::crypto::Hash;
use chrono::{DateTime, Utc};
use crate::block::models::proto;
use crate::util;

proto_enum! {
    enum PhysicalPropertyUnit {
        Unknown = 0,
        Mm = 1,
        Ml = 2,
    };
    proto::product::ProductVariant_PhysicalProperties_Units
}

#[derive(Clone, Debug, Default, PartialEq, ProtobufConvert)]
#[exonum(pb = "proto::product::ProductVariant_PhysicalProperties_Dimensions", serde_pb_convert)]
pub struct PhysicalPropertyDimensions {
    pub width: f64,
    pub height: f64,
    pub length: f64,
}

impl PhysicalPropertyDimensions {
    pub fn new(width: f64, height: f64, length: f64) -> Self {
        Self {
            width,
            height,
            length,
        }
    }
}

#[derive(Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::product::ProductVariant_PhysicalProperties", serde_pb_convert)]
pub struct PhysicalProperties {
    pub unit: PhysicalPropertyUnit,
    pub weight_mg: f64,
    pub dimensions: PhysicalPropertyDimensions,
}

impl PhysicalProperties {
    pub fn new(unit: PhysicalPropertyUnit, weight_mg: f64, dimensions: Option<&PhysicalPropertyDimensions>) -> Self {
        Self {
            unit,
            weight_mg,
            dimensions: dimensions.unwrap_or(&Default::default()).clone(),
        }
    }
}

#[derive(Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::product::ProductVariant_Input", serde_pb_convert)]
pub struct Input {
    pub product_variant_id: String,
    pub quantity: u64,
}

impl Input {
    pub fn new(product_variant_id: &str, quantity: u64) -> Self {
        Self {
            product_variant_id: product_variant_id.to_owned(),
            quantity,
        }
    }
}

proto_enum! {
    enum EffortTime {
        Unknown = 0,
        Nanoseconds = 1,
        Milliseconds = 2,
        Seconds = 3,
        Minutes = 4,
        Hours = 5,
        Days = 6,
        Weeks = 7,
        Years = 8,
    };
    proto::product::ProductVariant_Effort_Time
}

#[derive(Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::product::ProductVariant_Effort", serde_pb_convert)]
pub struct Effort {
    pub time: EffortTime,
    pub quantity: u64,
}

impl Effort {
    pub fn new(time: EffortTime, quantity: u64) -> Self {
        Self {
            time,
            quantity,
        }
    }
}

#[derive(Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::product::ProductVariant", serde_pb_convert)]
pub struct ProductVariant {
    pub id: String,
    pub product_id: String,
    pub name: String,
    pub properties: PhysicalProperties,
    pub inputs: Vec<Input>,
    pub options: HashMap<String, String>,
    pub effort: Effort,
    pub active: bool,
    pub meta: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub deleted: DateTime<Utc>,
}

impl ProductVariant {
    pub fn new(id: &str, product_id: &str, name: &str, properties: &PhysicalProperties, inputs: &Vec<Input>, options: &HashMap<String, String>, effort: &Effort, active: bool, meta: &str, created: &DateTime<Utc>, updated: &DateTime<Utc>, deleted: Option<&DateTime<Utc>>) -> Self {
        Self {
            id: id.to_owned(),
            product_id: product_id.to_owned(),
            name: name.to_owned(),
            properties: properties.clone(),
            inputs: inputs.clone(),
            options: options.clone(),
            effort: effort.clone(),
            active,
            meta: meta.to_owned(),
            created: created.clone(),
            updated: updated.clone(),
            deleted: deleted.unwrap_or(&util::time::default_time()).clone(),
        }
    }
}

#[derive(Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::product::Product", serde_pb_convert)]
pub struct Product {
    pub id: String,
    pub company_id: String,
    pub name: String,
    pub options: HashMap<String, String>,
    pub variants: HashMap<String, ProductVariant>,
    pub meta: String,
    pub active: bool,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub deleted: DateTime<Utc>,
    pub history_len: u64,
    pub history_hash: Hash,
}

impl Product {
    pub fn new(id: &str, company_id: &str, name: &str, options: &HashMap<String, String>, variants: &HashMap<String, ProductVariant>, meta: &str, active: bool, created: &DateTime<Utc>, updated: &DateTime<Utc>, deleted: Option<&DateTime<Utc>>, history_len: u64, &history_hash: &Hash) -> Self {
        Self {
            id: id.to_owned(),
            company_id: company_id.to_owned(),
            name: name.to_owned(),
            options: options.clone(),
            variants: variants.clone(),
            meta: meta.to_owned(),
            active,
            created: created.clone(),
            updated: updated.clone(),
            deleted: deleted.unwrap_or(&util::time::default_time()).clone(),
            history_len,
            history_hash,
        }
    }

    pub fn update(&self, name: Option<&str>, meta: Option<&str>, active: Option<bool>, updated: &DateTime<Utc>, history_hash: &Hash) -> Self {
        Self::new(
            &self.id,
            &self.company_id,
            name.unwrap_or(&self.name),
            &self.options,
            &self.variants,
            meta.unwrap_or(&self.meta),
            active.unwrap_or(self.active),
            &self.created,
            updated,
            Some(&self.deleted),
            self.history_len + 1,
            history_hash
        )
    }

    pub fn set_option(&self, name: &str, title: &str, updated: &DateTime<Utc>, history_hash: &Hash) -> Self {
        let mut options = self.options.clone();
        options.insert(name.to_owned(), title.to_owned());
        Self::new(
            &self.id,
            &self.company_id,
            &self.name,
            &options,
            &self.variants,
            &self.meta,
            self.active,
            &self.created,
            updated,
            Some(&self.deleted),
            self.history_len + 1,
            history_hash
        )
    }

    pub fn remove_option(&self, name: &str, updated: &DateTime<Utc>, history_hash: &Hash) -> Self {
        // remove the option...
        let mut options = self.options.clone();
        options.remove(name);

        // ...but also remove the option from all variants >=]
        let mut variants = self.variants.clone();
        for var in variants.values_mut() {
            var.options.remove(name);
            var.updated = updated.clone();
        }

        Self::new(
            &self.id,
            &self.company_id,
            &self.name,
            &options,
            &variants,
            &self.meta,
            self.active,
            &self.created,
            updated,
            Some(&self.deleted),
            self.history_len + 1,
            history_hash
        )
    }

    pub fn set_variant(&self, variant: &ProductVariant, updated: &DateTime<Utc>, history_hash: &Hash) -> Self {
        let mut variants = self.variants.clone();
        let mut variant_cloned = variant.clone();
        variant_cloned.product_id = self.id.clone();
        if !variants.contains_key(&variant.id) {
            variant_cloned.created = updated.clone();
        }
        variant_cloned.updated = updated.clone();
        variant_cloned.deleted = util::time::default_time();
        variants.insert(variant.id.clone(), variant_cloned);
        Self::new(
            &self.id,
            &self.company_id,
            &self.name,
            &self.options,
            &variants,
            &self.meta,
            self.active,
            &self.created,
            updated,
            Some(&self.deleted),
            self.history_len + 1,
            history_hash
        )
    }

    pub fn update_variant(&self, variant_id: &str, name: Option<&str>, active: Option<bool>, meta: Option<&str>, updated: &DateTime<Utc>, history_hash: &Hash) -> Self {
        let mut variants = self.variants.clone();
        if let Some(mut var) = variants.get_mut(variant_id) {
            var.name = name.unwrap_or(var.name.as_str()).to_owned();
            var.active = active.unwrap_or(var.active);
            var.meta = meta.unwrap_or(var.meta.as_str()).to_owned();
            var.updated = updated.clone();
        }
        Self::new(
            &self.id,
            &self.company_id,
            &self.name,
            &self.options,
            &variants,
            &self.meta,
            self.active,
            &self.created,
            updated,
            Some(&self.deleted),
            self.history_len + 1,
            history_hash
        )
    }

    pub fn remove_variant(&self, variant_id: &str, updated: &DateTime<Utc>, history_hash: &Hash) -> Self {
        let mut variants = self.variants.clone();
        if let Some(mut var) = variants.get_mut(variant_id) {
            var.deleted = updated.clone();
        }
        Self::new(
            &self.id,
            &self.company_id,
            &self.name,
            &self.options,
            &variants,
            &self.meta,
            self.active,
            &self.created,
            updated,
            Some(&self.deleted),
            self.history_len + 1,
            history_hash
        )
    }

    pub fn delete(&self, deleted: &DateTime<Utc>, history_hash: &Hash) -> Self {
        Self::new(
            &self.id,
            &self.company_id,
            &self.name,
            &self.options,
            &self.variants,
            &self.meta,
            self.active,
            &self.created,
            &self.updated,
            Some(deleted),
            self.history_len + 1,
            history_hash
        )
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted != util::time::default_time()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::util;
    use std::collections::HashMap;

    fn make_date() -> DateTime<Utc> {
        chrono::offset::Utc::now()
    }

    fn make_hash() -> Hash {
        Hash::new([1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4])
    }

    fn make_product() -> Product {
        let date = make_date();
        let options = HashMap::new();
        let variants = HashMap::new();
        Product::new(
            "6969ec02-1c6d-4791-8ba5-eb9e16964c26",
            "ef9f714b-f598-4176-a223-9075d86361a4",
            "Widget",
            &options,
            &variants,
            "",
            true,
            &date,
            &date,
            None,
            0,
            &make_hash()
        )
    }

    #[test]
    fn updates() {
        let product = make_product();
        util::sleep(100);
        let date2 = make_date();
        let hash2 = Hash::new([1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4]);
        let product2 = product.clone().update(None, None, None, &date2, &hash2);
        assert_eq!(product.id, product2.id);
        assert_eq!(product.company_id, product2.company_id);
        assert_eq!(product.name, product2.name);
        assert_eq!(product.meta, product2.meta);
        assert_eq!(product.active, product2.active);
        assert_eq!(product.created, product2.created);
        assert!(product.updated != product2.updated);
        assert_eq!(product2.updated, date2);
        assert_eq!(product.history_len, product2.history_len - 1);
        assert!(product.history_hash != product2.history_hash);
        assert_eq!(product2.history_hash, hash2);
        util::sleep(100);
        let date3 = make_date();
        let hash3 = Hash::new([1, 37, 6, 4, 1, 37, 6, 4, 1, 37, 6, 4, 1, 37, 6, 4, 1, 37, 6, 4, 1, 37, 6, 4, 1, 37, 6, 4, 1, 37, 6, 4]);
        let product3 = product2.clone().update(Some("Widget2"), Some(r#"{"description":"better widget"}"#), Some(false), &date3, &hash3);
        assert_eq!(product2.id, product3.id);
        assert_eq!(product2.company_id, product3.company_id);
        assert!(product2.name != product3.name);
        assert!(product2.active != product3.active);
        assert!(!product3.active);
        assert_eq!(product3.name, "Widget2");
        assert_eq!(product3.meta, r#"{"description":"better widget"}"#);
        assert_eq!(product2.created, product3.created);
        assert!(product2.updated != product3.updated);
        assert_eq!(product3.updated, date3);
        assert_eq!(product2.history_len, product3.history_len - 1);
        assert!(product2.history_hash != product3.history_hash);
        assert_eq!(product3.history_hash, hash3);
    }

    #[test]
    fn options() {
        let product = make_product();
        assert_eq!(product.options.len(), 0);
        util::sleep(100);
        let date2 = make_date();
        let hash2 = Hash::new([1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4]);
        let product2 = product.clone().set_option("size", "Size", &date2, &hash2);
        assert_eq!(product.id, product2.id);
        assert_eq!(product.company_id, product2.company_id);
        assert_eq!(product.name, product2.name);
        assert_eq!(product.meta, product2.meta);
        assert_eq!(product.active, product2.active);
        assert_eq!(product.created, product2.created);
        assert!(product.updated != product2.updated);
        assert_eq!(product2.updated, date2);
        assert_eq!(product.history_len, product2.history_len - 1);
        assert!(product.history_hash != product2.history_hash);
        assert_eq!(product2.history_hash, hash2);
        assert_eq!(product2.options.len(), 1);
        util::sleep(100);
        let date3 = make_date();
        let hash3 = Hash::new([1, 37, 6, 4, 1, 37, 6, 4, 1, 37, 6, 4, 1, 37, 6, 4, 1, 37, 6, 4, 1, 37, 6, 4, 1, 37, 6, 4, 1, 37, 6, 4]);
        let product3 = product2.clone().set_option("size", "Sizes", &date3, &hash3);
        assert_eq!(product2.id, product3.id);
        assert_eq!(product2.company_id, product3.company_id);
        assert_eq!(product2.name, product3.name);
        assert_eq!(product2.active, product3.active);
        assert_eq!(product2.created, product3.created);
        assert!(product2.updated != product3.updated);
        assert_eq!(product3.updated, date3);
        assert_eq!(product2.history_len, product3.history_len - 1);
        assert!(product2.history_hash != product3.history_hash);
        assert_eq!(product3.history_hash, hash3);
        assert_eq!(product3.options.len(), 1);
        util::sleep(100);
        let date4 = make_date();
        let hash4 = Hash::new([1, 47, 6, 4, 1, 47, 6, 4, 1, 47, 6, 4, 1, 47, 6, 4, 1, 47, 6, 4, 1, 47, 6, 4, 1, 47, 6, 4, 1, 47, 6, 4]);
        let product4 = product3.clone().remove_option("size", &date4, &hash4);
        assert_eq!(product3.id, product4.id);
        assert_eq!(product3.company_id, product4.company_id);
        assert_eq!(product3.name, product4.name);
        assert_eq!(product3.active, product4.active);
        assert_eq!(product3.created, product4.created);
        assert!(product3.updated != product4.updated);
        assert_eq!(product4.updated, date4);
        assert_eq!(product3.history_len, product4.history_len - 1);
        assert!(product3.history_hash != product4.history_hash);
        assert_eq!(product4.history_hash, hash4);
        assert_eq!(product4.options.len(), 0);
    }

    #[test]
    fn variants() {
        let product = make_product();
        assert_eq!(product.options.len(), 0);
        assert_eq!(product.variants.len(), 0);
        util::sleep(100);
        let date2 = make_date();
        let hash2_1 = Hash::new([1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4]);
        let hash2_2 = Hash::new([4, 87, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4]);
        let hash2_3 = Hash::new([5, 87, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4]);
        let props = PhysicalProperties::new(PhysicalPropertyUnit::Mm, 600.0, Some(&PhysicalPropertyDimensions::new(100.0, 100.0, 100.0)));
        let inputs = vec![
            Input::new("4722d6bc-953d-4e3a-b1df-c133fc088710", 10),
        ];
        let mut voptions = HashMap::new();
        voptions.insert("size".to_owned(), "XXXLarge".to_owned());
        voptions.insert("color".to_owned(), "Red".to_owned());
        let effort = Effort::new(EffortTime::Hours, 1);
        let variant = ProductVariant::new(
            "4266954b-c5c0-43e4-a740-9e36c726451d",
            &product.id,
            "SECKKK XXXLarge RED shirt braaaahh",
            &props,
            &inputs,
            &voptions,
            &effort,
            true,
            "",
            &date2,
            &date2,
            None
        );
        let product2 = product.clone()
            .set_option("size", "Size", &date2, &hash2_1)
            .set_option("color", "Color", &date2, &hash2_2)
            .set_variant(&variant, &date2, &hash2_3);
        assert_eq!(product.id, product2.id);
        assert_eq!(product.company_id, product2.company_id);
        assert_eq!(product.name, product2.name);
        assert_eq!(product.meta, product2.meta);
        assert_eq!(product.active, product2.active);
        assert_eq!(product.created, product2.created);
        assert!(product.updated != product2.updated);
        assert_eq!(product2.updated, date2);
        assert_eq!(product.history_len, product2.history_len - 3);
        assert!(product.history_hash != product2.history_hash);
        assert_eq!(product2.history_hash, hash2_3);
        assert_eq!(product2.options.len(), 2);
        assert_eq!(product2.variants.len(), 1);
        let pvariant = product2.variants.get("4266954b-c5c0-43e4-a740-9e36c726451d").unwrap();
        assert_eq!(pvariant.deleted, util::time::default_time());
        assert_eq!(pvariant.name, "SECKKK XXXLarge RED shirt braaaahh");
        assert_eq!(pvariant.meta, "");
        assert_eq!(pvariant.product_id, product.id);
        util::sleep(100);
        let date3 = make_date();
        let hash3 = Hash::new([1, 37, 6, 33, 1, 37, 6, 4, 1, 37, 6, 4, 1, 37, 6, 4, 1, 37, 6, 4, 1, 37, 6, 4, 1, 37, 6, 4, 1, 37, 6, 4]);
        let product3 = product2.clone().update_variant("4266954b-c5c0-43e4-a740-9e36c726451d", None, None, Some(r#"{"get":"a job"}"#), &date3, &hash3);
        let pvariant = product3.variants.get("4266954b-c5c0-43e4-a740-9e36c726451d").unwrap();
        assert_eq!(product3.updated, date3);
        assert_eq!(product2.history_len, product3.history_len - 1);
        assert!(product2.history_hash != product3.history_hash);
        assert_eq!(product3.history_hash, hash3);
        assert_eq!(product3.options.len(), 2);
        assert_eq!(product3.variants.len(), 1);
        assert_eq!(pvariant.updated, date3);
        assert_eq!(pvariant.deleted, util::time::default_time());
        assert_eq!(pvariant.name, "SECKKK XXXLarge RED shirt braaaahh");
        assert_eq!(pvariant.meta, r#"{"get":"a job"}"#);
        assert_eq!(pvariant.product_id, product.id);
        util::sleep(100);
        let date4 = make_date();
        let hash4 = Hash::new([1, 47, 6, 44, 1, 47, 6, 4, 1, 47, 6, 4, 1, 47, 6, 4, 1, 47, 6, 4, 1, 47, 6, 4, 1, 47, 6, 4, 1, 47, 6, 4]);
        let product4 = product3.clone().update_variant("4266954b-c5c0-43e4-a740-9e36c726451d", Some("XXXLARGE RED BROHEIM"), Some(false), Some(""), &date4, &hash4);
        let pvariant = product4.variants.get("4266954b-c5c0-43e4-a740-9e36c726451d").unwrap();
        assert_eq!(product4.updated, date4);
        assert_eq!(product3.history_len, product4.history_len - 1);
        assert!(product3.history_hash != product4.history_hash);
        assert_eq!(product4.history_hash, hash4);
        assert_eq!(product4.options.len(), 2);
        assert_eq!(product4.variants.len(), 1);
        assert_eq!(pvariant.updated, date4);
        assert_eq!(pvariant.deleted, util::time::default_time());
        assert_eq!(pvariant.name, "XXXLARGE RED BROHEIM");
        assert_eq!(pvariant.meta, "");
        assert_eq!(pvariant.active, false);
        assert_eq!(pvariant.product_id, product.id);
        util::sleep(100);
        let date5 = make_date();
        let hash5 = Hash::new([1, 57, 6, 55, 1, 57, 6, 5, 1, 57, 6, 5, 1, 57, 6, 5, 1, 57, 6, 5, 1, 57, 6, 5, 1, 57, 6, 5, 1, 57, 6, 5]);
        let product5 = product4.clone().remove_variant("4266954b-c5c0-43e4-a740-9e36c726451d", &date5, &hash5);
        let pvariant = product5.variants.get("4266954b-c5c0-43e4-a740-9e36c726451d").unwrap().clone();
        assert_eq!(product5.updated, date5);
        assert_eq!(product4.history_len, product5.history_len - 1);
        assert!(product4.history_hash != product5.history_hash);
        assert_eq!(product5.history_hash, hash5);
        assert_eq!(product5.options.len(), 2);
        assert_eq!(product5.variants.len(), 1);
        assert_eq!(pvariant.updated, date4);
        assert_eq!(pvariant.deleted, date5);
        assert_eq!(pvariant.name, "XXXLARGE RED BROHEIM");
        assert_eq!(pvariant.meta, "");
        assert_eq!(pvariant.active, false);
        assert_eq!(pvariant.product_id, product.id);
        util::sleep(100);
        let date6 = make_date();
        let hash6 = Hash::new([1, 57, 6, 55, 1, 57, 6, 5, 1, 57, 6, 5, 1, 57, 6, 5, 1, 57, 6, 6, 1, 67, 6, 6, 1, 67, 6, 6, 1, 67, 6, 6]);
        let product6 = product5.clone().remove_option("color", &date6, &hash6);
        let pvariant2 = product6.variants.get("4266954b-c5c0-43e4-a740-9e36c726451d").unwrap().clone();
        assert!(pvariant.options.contains_key("color"));
        assert!(!pvariant2.options.contains_key("color"));
        assert_eq!(product6.updated, date6);
        assert_eq!(product5.history_len, product6.history_len - 1);
        assert!(product5.history_hash != product6.history_hash);
        assert_eq!(product6.history_hash, hash6);
        assert_eq!(product6.options.len(), 1);
        assert_eq!(product6.variants.len(), 1);
        assert_eq!(pvariant2.updated, date6);
    }

    #[test]
    fn deletes() {
        let product = make_product();
        assert_eq!(product.deleted, util::time::default_time());
        assert!(!product.is_deleted());
        let date2 = make_date();
        let hash2 = Hash::new([56, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4]);
        let product2 = product.delete(&date2, &hash2);
        assert_eq!(product2.deleted, date2);
        assert!(product2.deleted != util::time::default_time());
        assert!(product2.is_deleted());
    }
}

