//! This library holds the algorithm that costs products and services

pub mod costs;

use std::collections::HashMap;
use models::product::Product;
use crate::costs::Costs;

pub struct Order {
}

/// Takes two sets of orders: a company's incoming orders ("sales" in the
/// current vernacular) and outgoing orders ("purchases").
///
/// The orders *must* be filtered such that both sets are a particular window
/// in time (ex, the last 365 days) and must be ordered from oldest to newest.
pub fn calculate_costs(orders_incoming: Vec<Order>, orders_outgoing: Vec<Order>) -> HashMap<String, Costs> {
    HashMap::new()
}

/*
fn product_cost(comp: &Company, prod: &Product) -> Costs {
    let mut sum_hours = 0;
    let mut sum_costs = HashMap::new();
    let mut sum_outputs = HashMap::new();
    for t in &comp.track {
        sum_hours += t.hours;
        for (key, val) in t.costs.iter() {
            let current = sum_costs.get(key).unwrap_or(&Costs::new()).clone();
            sum_costs.insert(key.to_owned(), current + val.clone());
        };
        for (key, val) in t.outputs.iter() {
            let current = sum_outputs.get(key).unwrap_or(&0);
            sum_outputs.insert(key.to_owned(), current + val);
        };
    }
    let total_output = sum_outputs.get(prod.name());
    if total_output.is_none() || total_output == Some(&0) {
        return default_cost(comp, prod);
    }
    let costs_operating = sum_costs.get("operating").unwrap_or(&Costs::new()).clone();
    let costs_inputs = sum_costs.get("inventory").unwrap_or(&Costs::new()).clone();

    let mut prod_ratios = HashMap::new();
    let mut inp_ratios = HashMap::new();
    for prod in &Product::list() {
		let max_theoretical_production = (comp.concurrency as f64) * ((sum_hours as f64) / prod.effort());
		let num_produced = (sum_outputs.get(prod.name()).unwrap_or(&0) + 0) as f64;
        let prod_ratio = num_produced / max_theoretical_production;
        let inp_ratio = if costs_inputs.is_zero() {
            Costs::new()
        } else {
            (costs::input_costs(prod, 0.0) * num_produced) / costs_inputs.clone()
        };
        prod_ratios.insert(prod.name().to_string(), prod_ratio);
        inp_ratios.insert(prod.name().to_string(), inp_ratio);
    }
    let prod_ratio_sum = prod_ratios.iter().fold(0.0, |acc, (_, x)| acc + x);
    let inp_ratio_sum = inp_ratios.iter().fold(Costs::new(), |acc, (_, x)| acc + x.clone());
    let prod_ratio = if prod_ratio_sum == 0.0 { 0.0 } else { prod_ratios.get(prod.name()).unwrap_or(&0.0) / prod_ratio_sum };
    let inp_ratio = if inp_ratio_sum.is_zero() { Costs::new() } else { inp_ratios.get(prod.name()).unwrap_or(&Costs::new()).clone() / inp_ratio_sum };
    let num_produced = (sum_outputs.get(prod.name()).unwrap() + 0) as f64;

    let operating_cost = costs_operating * prod_ratio;
    let inp_cost = costs_inputs * inp_ratio;
    ((operating_cost + inp_cost) / num_produced)
}
*/
