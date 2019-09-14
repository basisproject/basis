use exonum::crypto::Hash;
use chrono::{DateTime, Utc};
use crate::proto;
use util;

proto_enum! {
    enum Unit {
        Unknown = 0,
        Millimeter = 1,
        Milliliter = 2,
        WattHour = 3,
    };
    proto::product::Product_Unit
}

#[derive(Clone, Debug, Default, PartialEq, ProtobufConvert)]
#[exonum(pb = "proto::product::Product_Dimensions", serde_pb_convert)]
pub struct Dimensions {
    pub width: f64,
    pub height: f64,
    pub length: f64,
}

impl Dimensions {
    pub fn new(width: f64, height: f64, length: f64) -> Self {
        Self {
            width,
            height,
            length,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, ProtobufConvert)]
#[exonum(pb = "proto::product::Product_Input", serde_pb_convert)]
pub struct Input {
    pub product_id: String,
    pub quantity: f64,
}

impl Input {
    pub fn new(product_id: &str, quantity: f64) -> Self {
        Self {
            product_id: product_id.to_owned(),
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
    proto::product::Product_Effort_Time
}

#[derive(Clone, Debug, Default, PartialEq, ProtobufConvert)]
#[exonum(pb = "proto::product::Product_Effort", serde_pb_convert)]
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
#[exonum(pb = "proto::product::Product", serde_pb_convert)]
pub struct Product {
    pub id: String,
    pub company_id: String,
    pub name: String,
    pub unit: Unit,
    pub mass_mg: f64,
    pub dimensions: Dimensions,
    pub inputs: Vec<Input>,
    pub effort: Effort,
    pub active: bool,
    pub meta: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub deleted: DateTime<Utc>,
    pub history_len: u64,
    pub history_hash: Hash,
}

impl Product {
    pub fn new(id: &str, company_id: &str, name: &str, unit: &Unit, mass_mg: f64, dimensions: &Dimensions, inputs: &Vec<Input>, effort: &Effort, active: bool, meta: &str, created: &DateTime<Utc>, updated: &DateTime<Utc>, deleted: Option<&DateTime<Utc>>, history_len: u64, history_hash: &Hash) -> Self {
        Self {
            id: id.to_owned(),
            company_id: company_id.to_owned(),
            name: name.to_owned(),
            unit: unit.clone(),
            mass_mg,
            dimensions: dimensions.clone(),
            inputs: inputs.clone(),
            effort: effort.clone(),
            active,
            meta: meta.to_owned(),
            created: created.clone(),
            updated: updated.clone(),
            deleted: deleted.unwrap_or(&util::time::default_time()).clone(),
            history_len,
            history_hash: history_hash.clone(),
        }
    }

    pub fn update(&self, name: Option<&str>, unit: Option<&Unit>, mass_mg: Option<f64>, dimensions: Option<&Dimensions>, inputs: Option<&Vec<Input>>, effort: Option<&Effort>, active: Option<bool>, meta: Option<&str>, updated: &DateTime<Utc>, history_hash: &Hash) -> Self {
        Self::new(
            &self.id,
            &self.company_id,
            name.unwrap_or(&self.name),
            unit.unwrap_or(&self.unit),
            mass_mg.unwrap_or(self.mass_mg),
            dimensions.unwrap_or(&self.dimensions),
            inputs.unwrap_or(&self.inputs),
            effort.unwrap_or(&self.effort),
            active.unwrap_or(self.active),
            meta.unwrap_or(&self.meta),
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
            &self.unit,
            self.mass_mg,
            &self.dimensions,
            &self.inputs,
            &self.effort,
            self.active,
            &self.meta,
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
    use util;

    fn make_date() -> DateTime<Utc> {
        chrono::offset::Utc::now()
    }

    fn make_hash() -> Hash {
        Hash::new([1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4])
    }

    fn make_product() -> Product {
        let date = make_date();
        let inputs = vec![];
        let effort = Effort::new(EffortTime::Hours, 1);
        Product::new(
            "4266954b-c5c0-43e4-a740-9e36c726451d",
            "b9eb0cc2-5b37-4fd1-83fd-8597625aee95",
            "XXXLarge RED TSHIRT!",
            &Unit::Millimeter,
            600.00,
            &Dimensions::new(100.0, 100.0, 100.0),
            &inputs,
            &effort,
            true,
            "",
            &date,
            &date,
            None,
            0,
            &make_hash(),
        )
    }

    #[test]
    fn updates() {
        let product = make_product();
        util::sleep(100);
        let date2 = make_date();
        let hash2 = Hash::new([1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4]);
        let inputs = vec![
            Input::new("4722d6bc-953d-4e3a-b1df-c133fc088710", 10.0),
        ];
        let effort = Effort::new(EffortTime::Hours, 2);
        let product2 = product.clone().update(
            Some("Liquid shirt, dogs love it"),
            Some(&Unit::Milliliter),
            None,
            Some(&Default::default()),
            Some(&inputs),
            Some(&effort),
            None,
            Some(r#"{"convert":"gallons"}"#),
            &date2,
            &hash2
        );
        assert_eq!(product2.company_id, product.company_id);
        assert_eq!(product.name, "XXXLarge RED TSHIRT!");
        assert_eq!(product2.name, "Liquid shirt, dogs love it");
        assert_eq!(product2.unit, Unit::Milliliter);
        assert_eq!(product2.mass_mg, product.mass_mg);
        assert_eq!(product2.dimensions, Default::default());
        assert_eq!(product2.inputs, inputs);
        assert_eq!(product2.inputs.len(), 1);
        assert_eq!(product.inputs.len(), 0);
        assert_eq!(product2.effort, effort);
        assert_eq!(product.created, product2.created);
        assert!(product.updated != product2.updated);
        assert_eq!(product2.updated, date2);
        assert_eq!(product2.history_len, product.history_len + 1);
        assert_eq!(product2.history_hash, hash2);
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

