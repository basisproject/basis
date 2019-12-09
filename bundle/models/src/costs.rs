use std::collections::HashMap;
use crate::proto;
use std::ops::{Add, Sub, Mul, Div};

#[derive(Clone, Debug, Default, PartialEq, ProtobufConvert)]
#[exonum(pb = "proto::costs::Costs", serde_pb_convert)]
pub struct Costs {
    pub products: HashMap<String, f64>,
    pub labor: HashMap<String, f64>,
}

impl Costs {
    pub fn new() -> Self {
        Self {
            labor: HashMap::new(),
            products: HashMap::new(),
        }
    }

    pub fn new_with_labor(ty: &str, labor: f64) -> Self {
        let mut costs = Self::new();
        costs.track_labor(ty, labor);
        costs
    }

    pub fn track(&mut self, prod: &str, val: f64) {
        if val < 0.0 {
            panic!("Costs::track() -- given value must be >= 0.0")
        }
        let entry = self.products.entry(prod.to_string()).or_insert(0.0);
        *entry += val;
    }

    pub fn track_labor(&mut self, ty: &str, val: f64) {
        if val < 0.0 {
            panic!("Costs::track_labor() -- given value must be >= 0.0")
        }
        let entry = self.labor.entry(ty.to_string()).or_insert(0.0);
        *entry += val;
    }

    pub fn labor(&self) -> &HashMap<String, f64> {
        &self.labor
    }

    pub fn products(&self) -> &HashMap<String, f64> {
        &self.products
    }

    #[allow(dead_code)]
    pub fn get(&self, product: &str) -> f64 {
        *self.products.get(product).unwrap_or(&0.0)
    }

    #[allow(dead_code)]
    pub fn get_labor(&self, ty: &str) -> f64 {
        *self.labor.get(ty).unwrap_or(&0.0)
    }

    pub fn is_zero(&self) -> bool {
        for (_, val) in self.labor.iter() {
            if val > &0.0 {
                return false;
            }
        }
        for (_, val) in self.products.iter() {
            if val > &0.0 {
                return false;
            }
        }
        true
    }

    /// given a set of costs, subtract them from our current costs, but only if
    /// the result is >= 0 for each cost tracked. then, return a costs object
    /// showing exactly how much was taken
    pub fn take(&mut self, costs: &Costs) -> Costs {
        let mut new_costs = Costs::new();
        for (k, lval) in self.labor.iter_mut() {
            let mut rval = costs.labor().get(k).unwrap_or(&0.0) + 0.0;
            let val = if lval > &mut rval { rval } else { lval.clone() };
            *lval -= val;
            new_costs.track_labor(k, val.clone());
        }
        for (k, lval) in self.products.iter_mut() {
            let mut rval = costs.products().get(k).unwrap_or(&0.0) + 0.0;
            let val = if lval > &mut rval { rval } else { lval.clone() };
            *lval -= val;
            new_costs.track(k, val.clone());
        }
        new_costs
    }
}

impl Add for Costs {
    type Output = Self;

    fn add(mut self, other: Self) -> Self {
        for k in other.labor().keys() {
            let entry = self.labor.entry(k.to_owned()).or_insert(0.0);
            *entry += other.labor().get(k).unwrap();
        }
        for k in other.products().keys() {
            let entry = self.products.entry(k.to_owned()).or_insert(0.0);
            *entry += other.products().get(k).unwrap();
        }
        self
    }
}

impl Sub for Costs {
    type Output = Self;

    fn sub(mut self, other: Self) -> Self {
        for k in other.labor().keys() {
            let entry = self.labor.entry(k.to_owned()).or_insert(0.0);
            *entry -= other.labor().get(k).unwrap();
        }
        for k in other.products().keys() {
            let entry = self.products.entry(k.to_owned()).or_insert(0.0);
            *entry -= other.products().get(k).unwrap();
        }
        self
    }
}

impl Mul for Costs {
    type Output = Self;

    fn mul(mut self, rhs: Self) -> Self {
        for (k, val) in self.labor.iter_mut() {
            *val *= rhs.labor().get(k).unwrap_or(&0.0);
        }
        for (k, val) in self.products.iter_mut() {
            *val *= rhs.products().get(k).unwrap_or(&0.0);
        }
        self
    }
}

impl Mul<f64> for Costs {
    type Output = Self;

    fn mul(mut self, rhs: f64) -> Self {
        for (_, val) in self.labor.iter_mut() {
            *val *= rhs;
        }
        for (_, val) in self.products.iter_mut() {
            *val *= rhs;
        }
        self
    }
}

impl Div for Costs {
    type Output = Self;

    fn div(mut self, rhs: Self) -> Self::Output {
        for (k, v) in self.labor.iter_mut() {
            let div = rhs.labor().get(k).unwrap_or(&0.0);
            #[cfg(feature = "panic-div0")]
            {
                if *div == 0.0 {
                    panic!("Costs::div() -- divide by zero for {:?}", k);
                }
            }
            *v /= div;
        }
        for (k, _) in rhs.labor().iter() {
            match self.labor.get(k) {
                None => {
                    self.labor.insert(k.clone(), 0.0);
                }
                _ => {}
            }
        }
        for (k, v) in self.products.iter_mut() {
            let div = rhs.products().get(k).unwrap_or(&0.0);
            #[cfg(feature = "panic-div0")]
            {
                if *div == 0.0 {
                    panic!("Costs::div() -- divide by zero for {:?}", k);
                }
            }
            *v /= div;
        }
        for (k, _) in rhs.products().iter() {
            match self.products.get(k) {
                None => {
                    self.products.insert(k.clone(), 0.0);
                }
                _ => {}
            }
        }
        self
    }
}

impl Div<f64> for Costs {
    type Output = Self;

    fn div(mut self, rhs: f64) -> Self::Output {
        #[cfg(feature = "panic-div0")]
        {
            if rhs == 0.0 {
                panic!("Costs::div() -- divide by zero");
            }
        }
        for (_, v) in self.labor.iter_mut() {
            *v /= rhs
        }
        for (_, v) in self.products.iter_mut() {
            *v /= rhs
        }
        self
    }
}

#[derive(Clone, Debug, Default, ProtobufConvert)]
#[exonum(pb = "proto::costs::CostsBucket", serde_pb_convert)]
pub struct CostsBucket {
    pub costs: Costs,
    pub count: u64,
}

impl CostsBucket {
    pub fn new() -> Self {
        Self {
            costs: Costs::new(),
            count: 0,
        }
    }

    /// Add an entry to the cost bucket
    pub fn add(&mut self, costs: &Costs) {
        self.costs = self.costs.clone() + costs.clone();
        self.count += 1;
    }

    /// Subtract an entry from the cost bucket
    pub fn subtract(&mut self, costs: &Costs) {
        self.costs = self.costs.clone() - costs.clone();
        self.count -= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add() {
        let mut costs1 = Costs::new();
        let mut costs2 = Costs::new();

        costs1.track_labor("miner", 6.0);
        costs1.track("widget", 3.1);
        costs1.track("iron", 8.5);
        costs2.track_labor("miner", 2.0);
        costs2.track_labor("widgetmaker", 3.0);
        costs2.track("widget", 1.8);
        costs2.track("oil", 5.6);

        let costs = costs1 + costs2;
        assert_eq!(costs.get_labor("miner"), 6.0 + 2.0);
        assert_eq!(costs.get_labor("widgetmaker"), 3.0);
        assert_eq!(costs.get_labor("joker"), 0.0);
        assert_eq!(costs.get("widget"), 3.1 + 1.8);
        assert_eq!(costs.get("iron"), 8.5 + 0.0);
        assert_eq!(costs.get("oil"), 5.6 + 0.0);
    }

    #[test]
    fn mul() {
        let mut costs1 = Costs::new();
        costs1.track_labor("miner", 6.0);
        costs1.track_labor("widgetmaker", 3.0);
        costs1.track("widget", 3.1);
        costs1.track("iron", 8.5);

        let costs = costs1 * 5.2;
        assert_eq!(costs.get_labor("miner"), 6.0 * 5.2);
        assert_eq!(costs.get_labor("widgetmaker"), 3.0 * 5.2);
        assert_eq!(costs.get("widget"), 3.1 * 5.2);
        assert_eq!(costs.get("iron"), 8.5 * 5.2);

        let mut costs1 = Costs::new();
        let mut costs2 = Costs::new();
        costs1.track_labor("miner", 1.3);
        costs1.track("widget", 8.7);
        costs2.track_labor("miner", 6.0);
        costs2.track_labor("widgetmaker", 5.0);
        costs2.track("widget", 3.1);
        costs2.track("iron", 8.5);

        let costs = costs1 * costs2;
        assert_eq!(costs.get_labor("miner"), 1.3 * 6.0);
        assert_eq!(costs.get_labor("widgetmaker"), 0.0 * 5.0);
        assert_eq!(costs.get("widget"), 8.7 * 3.1);
        assert_eq!(costs.get("iron"), 0.0 * 8.5);
    }

    #[test]
    fn div_costs() {
        let mut costs1 = Costs::new();
        let mut costs2 = Costs::new();

        costs1.track_labor("miner", 6.0);
        costs1.track_labor("singer", 2.0);
        costs1.track("widget", 3.1);
        costs2.track_labor("miner", 2.0);
        costs2.track_labor("singer", 6.0);
        costs2.track("widget", 1.8);
        costs2.track("oil", 5.6);

        let costs = costs1 / costs2;
        assert_eq!(costs.get_labor("miner"), 6.0 / 2.0);
        assert_eq!(costs.get_labor("singer"), 2.0 / 6.0);
        assert_eq!(costs.get("widget"), 3.1 / 1.8);
        assert_eq!(costs.get("oil"), 0.0 / 5.6);
    }

    #[test]
    fn div_f64() {
        let mut costs1 = Costs::new();

        costs1.track_labor("widgetmaker", 6.0);
        costs1.track("widget", 3.1);
        costs1.track("oil", 5.6);

        let costs = costs1 / 1.3;
        assert_eq!(costs.get_labor("widgetmaker"), 6.0 / 1.3);
        assert_eq!(costs.get("widget"), 3.1 / 1.3);
        assert_eq!(costs.get("oil"), 5.6 / 1.3);
    }

    #[cfg(feature = "panic-div0")]
    #[test]
    #[should_panic]
    fn div_by_0() {
        let mut costs1 = Costs::new();
        let costs2 = Costs::new();

        costs1.track("iron", 8.5);

        let costs = costs1 / costs2;
        assert_eq!(costs.get("iron"), 8.5 / 0.0);
    }

    #[cfg(not(feature = "panic-div0"))]
    #[test]
    fn div_by_0() {
        let mut costs1 = Costs::new();
        let costs2 = Costs::new();

        costs1.track("iron", 8.5);

        let costs = costs1 / costs2;
        assert_eq!(costs.get("iron"), 8.5 / 0.0);
    }

    #[cfg(feature = "panic-div0")]
    #[test]
    #[should_panic]
    fn div_f64_by_0() {
        let mut costs1 = Costs::new();

        costs1.track_labor("dancer", 6.0);
        costs1.track("widget", 3.1);
        costs1.track("oil", 5.6);

        let costs = costs1 / 0.0;
        assert_eq!(costs.get_labor("dancer"), 6.0 / 0.0);
        assert_eq!(costs.get("widget"), 3.1 / 0.0);
        assert_eq!(costs.get("oil"), 5.6 / 0.0);
    }

    #[cfg(not(feature = "panic-div0"))]
    #[test]
    fn div_f64_by_0() {
        let mut costs1 = Costs::new();

        costs1.track_labor("dancer", 6.0);
        costs1.track("widget", 3.1);
        costs1.track("oil", 5.6);

        let costs = costs1 / 0.0;
        assert_eq!(costs.get_labor("dancer"), 6.0 / 0.0);
        assert_eq!(costs.get("widget"), 3.1 / 0.0);
        assert_eq!(costs.get("oil"), 5.6 / 0.0);
    }

    #[test]
    fn is_zero() {
        let mut costs = Costs::new();
        assert!(costs.is_zero());
        costs.track("widget", 5.0);
        assert!(!costs.is_zero());
        assert!(!Costs::new_with_labor("dictator", 4.0).is_zero());
    }

    #[test]
    fn cost_buckets() {
        let mut bucket = CostsBucket::new();
        assert_eq!(bucket.costs, Costs::new());
        assert_eq!(bucket.count, 0);

        let mut costs = Costs::new();
        costs.track("widget", 69.0);
        bucket.add(&costs);

        assert_eq!(bucket.costs.get("widget"), 69.0);
        assert_eq!(bucket.count, 1);

        let mut costs = Costs::new();
        costs.track("widget", 42.0);
        bucket.add(&costs);

        assert_eq!(bucket.costs.get("widget"), 69.0 + 42.0);
        assert_eq!(bucket.count, 2);

        let mut costs = Costs::new();
        costs.track("widget", 69.0);
        bucket.subtract(&costs);

        assert_eq!(bucket.costs.get("widget"), 42.0);
        assert_eq!(bucket.count, 1);
    }
}

