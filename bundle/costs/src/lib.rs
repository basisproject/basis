//! This library holds the algorithm that costs products and services

mod costs;

use std::collections::HashMap;
use crate::costs::Costs;

pub struct Order {
}

pub fn calculate_costs(orders_incoming: Vec<Order>, orders_outgoing: Vec<Order>) -> HashMap<String, Costs> {
    HashMap::new()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
