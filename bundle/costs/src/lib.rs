//! This library holds the algorithm that costs products and services.

use std::cmp;
use std::collections::HashMap;
use chrono::Utc;
use error::{BResult, BError};
use models::costs::Costs;
use models::order::{CostCategory, Order};
use models::amortization::Amortization;
use models::product::Product;
use models::labor::Labor;

/// Takes two sets of orders: a company's incoming orders ("sales" in the
/// current vernacular) and outgoing orders ("purchases").
///
/// The orders *must* be filtered such that both sets are a particular window
/// in time (ex, the last 365 days) and must be ordered from oldest to newest.
pub fn calculate_costs(orders_incoming: &Vec<Order>, orders_outgoing: &Vec<Order>, labor: &Vec<Labor>, _wamortization: &HashMap<String, Amortization>, products: &HashMap<String, Product>) -> BResult<HashMap<String, Costs>> {
    // grab how many hours our orders cover
    let sum_hours = {
        let incoming_start_time = if orders_incoming.len() > 0 { orders_incoming[0].updated.timestamp() } else { Utc::now().timestamp() };
        let outgoing_start_time = if orders_outgoing.len() > 0 { orders_outgoing[0].updated.timestamp() } else { Utc::now().timestamp() };
        let start_time = cmp::min(incoming_start_time, outgoing_start_time) as f64;
        let incoming_end_time = if orders_incoming.len() > 0 { orders_incoming[orders_incoming.len() - 1].updated.timestamp() } else { Utc::now().timestamp() };
        let outgoing_end_time = if orders_outgoing.len() > 0 { orders_outgoing[orders_outgoing.len() - 1].updated.timestamp() } else { Utc::now().timestamp() };
        let end_time = cmp::max(incoming_end_time, outgoing_end_time) as f64;
        let seconds = end_time - start_time;
        let hours = if seconds < 3600.0 {
            1.0
        } else {
            seconds / 3600.0
        };
        hours
    };
    // holds a mapping for cost_type -> sum costs for all of our costs
    let mut sum_costs: HashMap<CostCategory, Costs> = HashMap::new();
    // maps product_id -> number produced over order period
    let mut sum_produced: HashMap<String, f64> = HashMap::new();
    // maps product_id -> vec[costs] for each product we bought for *inventory*
    let mut sum_inventory_costs: HashMap<String, Vec<Costs>> = HashMap::new();
    // holds product_id -> average_costs for products we bought for inventory
    let mut avg_input_costs: HashMap<String, Costs> = HashMap::new();

    // labor is an operating cost
    {
        let op_costs = sum_costs.entry(CostCategory::Operating).or_insert(Default::default());
        for entry in labor {
            *op_costs = op_costs.clone() + Costs::new_with_labor(&entry.occupation, entry.hours());
        }
    }

    // for all "purchase" orders, sum the costs of the different categories:
    // inventory and operating costs. also, if inventory, track a vector of the
    // costs for each product. we'll use this later to get an "average input
    // cost" for each inventory product we bought.
    for order in orders_outgoing {
        let cat = order.cost_category.clone();
        let current = sum_costs.entry(cat).or_insert(Default::default());
        for prod in &order.products {
            let mut prod_costs = prod.costs.clone() * prod.quantity;
            // if this product is a resource, add its id and quantity to the
            // cost list
            if prod.is_resource() {
                let mut tmp_costs = Costs::new();
                tmp_costs.track(&prod.product_id, prod.quantity);
                prod_costs = prod_costs + tmp_costs;
            }
            *current = current.clone() + prod_costs.clone();
            if cat == CostCategory::Inventory {
                let prod_inp_costs = sum_inventory_costs.entry(prod.product_id.clone()).or_insert(vec![]);
                prod_inp_costs.push(prod_costs);
            }
        }
    }

    // average the vec'ed costs from `sum_inventory_costs` and put the costs in
    // `avg_input_costs`
    for (prod_id, costvec) in sum_inventory_costs.iter() {
        let costlen = costvec.len();
        let costsum = costvec.iter()
            .fold(Costs::new(), |acc, x| acc + x.clone());
        avg_input_costs.insert(prod_id.clone(), costsum / costlen as f64);
    }

    // sum how many of each product we have produced
    for order in orders_incoming {
        for prod in &order.products {
            let current = sum_produced.entry(prod.product_id.clone()).or_insert(Default::default());
            *current += prod.quantity;
        }
    }

    // if we haven't made anything, just assume we've made one of each
    if sum_produced.len() == 0 {
        for prod in products.values() {
            sum_produced.insert(prod.id.clone(), 1.0);
        }
    }

    // grab our cost category sums
    let costs_operating = sum_costs.get(&CostCategory::Operating).unwrap_or(&Costs::new()).clone();
    let costs_inputs = sum_costs.get(&CostCategory::Inventory).unwrap_or(&Costs::new()).clone();

    // prod_ratios holds product_id -> prod_ratio for each product we make. the
    // `prod_ratio` is a value 0 < x < 1 that determines how much production was
    // devoted to that particular product.
    let mut prod_ratios = HashMap::new();

    // inp_ratios holds product_id -> inp_ratio for each product we make. the
    // `inp_ratio` is a cost *ratio*:
    //
    //   average costs product inputs / sum of input costs
    //
    // this gives us a comparative value we can use apportion the costs of
    // inputs to each product
    let mut inp_ratios = HashMap::new();

    // loop over the products we've made and populate our prod/inp ratios
    for (prod_id, num_produced) in sum_produced.iter() {
        let num_produced = num_produced.clone();
        // we need a product to grab values from.
        //
        // NOTE: eventually it might make sense to solidify all costs and
        // parameters needed to generate our ratios *in the order itself*.
        let prod = match products.get(prod_id) {
            Some(x) => x,
            None => Err(BError::CostMissingProduct)?,
        };

        // this is how much of this product we could have made if *all*
        // production was devoted to it
		let max_theoretical_production = (sum_hours as f64) / prod.effort_hours();
        // here's our prod ratio: what we made vs what we could have made
        let prod_ratio = num_produced / max_theoretical_production;
        // generate the inp ratio
        let inp_ratio = if costs_inputs.is_zero() {
            // if we have made no inventory orders, this is blank
            Costs::new()
        } else {
            // for each of our product's inputs, find the matching average cost
            // of those inputs and sum them, giving us a "total avarage costs of
            // inputs" for our product
            //
            // NOTE: we use the *current* product input. eventually it would be
            // more accurate to use the inputs *at the time the order was made*
            // which would preserve cost history more accurately.
            let mut prod_inp_costs = Costs::new();
            for input in &prod.inputs {
                // NOTE: we use the average cost of the product's inputs over time
                // here. another measure might be more accurate
                let avg_costs = match avg_input_costs.get(&input.product_id) {
                    Some(x) => x.clone(),
                    None => Costs::new(),
                };
                prod_inp_costs = prod_inp_costs + avg_costs;
            }
            (prod_inp_costs * num_produced) / costs_inputs.clone()
        };
        prod_ratios.insert(prod_id.clone(), prod_ratio);
        inp_ratios.insert(prod_id.clone(), inp_ratio);
    }

    // now, sum our prod/input ratios to get a grand total.
    let prod_ratio_sum = prod_ratios.iter().fold(0.0, |acc, (_, x)| acc + x);
    let inp_ratio_sum = inp_ratios.iter().fold(Costs::new(), |acc, (_, x)| acc + x.clone());

    let mut final_costs: HashMap<String, Costs> = HashMap::new();
    for prod_id in products.keys() {
        let num_produced = sum_produced.get(prod_id).unwrap_or(&0.0).clone();
        if num_produced == 0.0 {
            // NOTE: can we possibly estimate this?
            final_costs.insert(prod_id.clone(), Costs::new());
        } else {
            let prod_ratio = if prod_ratio_sum == 0.0 { 0.0 } else { prod_ratios.get(prod_id).unwrap_or(&0.0) / prod_ratio_sum.clone() };
            let inp_ratio = if inp_ratio_sum.is_zero() { Costs::new() } else { inp_ratios.get(prod_id).unwrap_or(&Costs::new()).clone() / inp_ratio_sum.clone() };

            let operating_cost = costs_operating.clone() * prod_ratio;
            let inp_cost = costs_inputs.clone() * inp_ratio;
            let product_costs = (operating_cost + inp_cost) / num_produced;
            final_costs.insert(prod_id.clone(), product_costs);
        }
    }
    Ok(final_costs)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn calculates() {
        let orders_incoming = vec![];
        let orders_outgoing = vec![];
        let labor = vec![];
        let amortization = HashMap::new();
        let products = HashMap::new();
        calculate_costs(&orders_incoming, &orders_outgoing, &labor, &amortization, &products).expect("costs failed");
    }
}

