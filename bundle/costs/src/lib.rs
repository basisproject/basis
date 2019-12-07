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
        let incoming_start_time = if orders_incoming.len() > 0 { orders_incoming[0].created.timestamp() } else { Utc::now().timestamp() };
        let outgoing_start_time = if orders_outgoing.len() > 0 { orders_outgoing[0].created.timestamp() } else { Utc::now().timestamp() };
        let start_time = cmp::min(incoming_start_time, outgoing_start_time) as f64;
        let incoming_end_time = if orders_incoming.len() > 0 { orders_incoming[orders_incoming.len() - 1].created.timestamp() } else { Utc::now().timestamp() };
        let outgoing_end_time = if orders_outgoing.len() > 0 { orders_outgoing[orders_outgoing.len() - 1].created.timestamp() } else { Utc::now().timestamp() };
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
    use exonum::crypto::Hash;
    use std::collections::HashMap;
    use models::order::{Order, ProcessStatus, CostCategory, ProductEntry};
    use models::labor::Labor;
    use models::product::{Product, Unit, Dimensions, Input, EffortTime, Effort};
    use util::time;

    fn make_hash() -> Hash {
        Hash::new([1, 28, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4, 1, 27, 6, 4])
    }

    #[test]
    fn calculates() {
        let orders_incoming = test_orders_incoming();
        let orders_outgoing = test_orders_outgoing();
        let labor = test_labor();
        let amortization = HashMap::new();
        let products = test_products();
        let costs = calculate_costs(&orders_incoming, &orders_outgoing, &labor, &amortization, &products).expect("costs failed");
        println!(">>> final costs: {:?}", costs);
    }

    fn test_orders_incoming() -> Vec<Order> {
        let fakehash = make_hash();
        vec![
            Order::new( "ad23570e-c036-4ea9-9dfa-021508b94419", "dd803927-3c4a-4f42-9de4-38a258f96e27", "df42a90a-1fb1-4f9f-a56a-23ec45ecbf46", &CostCategory::Operating, &vec![{ let mut costs = Costs::new(); costs.track_labor("plant president", 8 as f64); costs.track_labor("technician", 32 as f64); ProductEntry::new("58bd7cbb-b7f2-4ae5-ac46-c69438d7733f", 910000 as f64, &costs, false) }], &ProcessStatus::Finalized, &time::from_timestamp(1574665849), &time::from_timestamp(1574838000), 1, &fakehash ),
            Order::new( "a570c6c5-71d3-4ed5-9214-7ee35cc5109e", "8a7cb9be-d7b8-4d16-ab03-5ac815e7faa5", "df42a90a-1fb1-4f9f-a56a-23ec45ecbf46", &CostCategory::Operating, &vec![{ let mut costs = Costs::new(); costs.track_labor("plant president", 0.000017582417582417584 as f64); costs.track_labor("mine president", 0.000004419443392918096 as f64); costs.track_labor("technician", 0.00007032967032967034 as f64); costs.track_labor("miner", 0.000022097216964590478 as f64); costs.track("d6b603c8-dcf1-4aac-a827-5c03b22a5b82", 5.52430424114762e-7 as f64); ProductEntry::new("58bd7cbb-b7f2-4ae5-ac46-c69438d7733f", 455000 as f64, &costs, false) }], &ProcessStatus::Finalized, &time::from_timestamp(1574665854), &time::from_timestamp(1574935200), 1, &fakehash ),
            Order::new( "d04af594-07af-4c3a-82c8-c29f298ee4e5", "631dfaaf-79be-48c4-9c49-8501af3fbf05", "df42a90a-1fb1-4f9f-a56a-23ec45ecbf46", &CostCategory::Operating, &vec![{ let mut costs = Costs::new(); costs.track_labor("mine president", 0.0000029462955952787305 as f64); costs.track_labor("plant president", 0.000017582417582417584 as f64); costs.track_labor("technician", 0.00007032967032967034 as f64); costs.track_labor("miner", 0.000014731477976393652 as f64); costs.track("d6b603c8-dcf1-4aac-a827-5c03b22a5b82", 3.682869494098413e-7 as f64); ProductEntry::new("58bd7cbb-b7f2-4ae5-ac46-c69438d7733f", 1137500 as f64, &costs, false) }], &ProcessStatus::Completed, &time::from_timestamp(1574665859), &time::from_timestamp(1575028800), 1, &fakehash ),
        ]
    }

    fn test_orders_outgoing() -> Vec<Order> {
        let fakehash = make_hash();
        vec![
            Order::new( "825dee45-225e-40e0-bf76-da12f958e673", "df42a90a-1fb1-4f9f-a56a-23ec45ecbf46", "631dfaaf-79be-48c4-9c49-8501af3fbf05", &CostCategory::Inventory, &vec![{ let mut costs = Costs::new(); costs.track_labor("mine president", 8 as f64); costs.track_labor("miner", 40 as f64); ProductEntry::new("d6b603c8-dcf1-4aac-a827-5c03b22a5b82", 0.5027116859444334 as f64, &costs, true) }], &ProcessStatus::Accepted, &time::from_timestamp(1574665850), &time::from_timestamp(1574820000), 1, &fakehash ),
        ]
    }

    fn test_labor() -> Vec<Labor> {
        let fakehash = make_hash();
        vec![
            Labor::new( "f3478ac7-4ee2-41a1-b75c-2c059fc2613b", "df42a90a-1fb1-4f9f-a56a-23ec45ecbf46", "5f6e4ab4-eaa4-4225-8142-d8e9ea023117", "technician", Some(&time::from_timestamp(1574701200)), Some(&time::from_timestamp(1574730000)), &time::from_timestamp(1574701200), &time::from_timestamp(1574730000), 1, &fakehash ),
            Labor::new( "280fa1e3-6b21-4e58-a3cb-0d2d587a491d", "df42a90a-1fb1-4f9f-a56a-23ec45ecbf46", "54093698-475f-464f-b0c2-799fab8da354", "technician", Some(&time::from_timestamp(1574701200)), Some(&time::from_timestamp(1574730000)), &time::from_timestamp(1574701200), &time::from_timestamp(1574730000), 1, &fakehash ),
            Labor::new( "15d082ac-b92c-4d4b-821b-28332d2a5688", "df42a90a-1fb1-4f9f-a56a-23ec45ecbf46", "61d1e24d-e441-442b-bd75-7665a3026d3c", "technician", Some(&time::from_timestamp(1574701200)), Some(&time::from_timestamp(1574730000)), &time::from_timestamp(1574701200), &time::from_timestamp(1574730000), 1, &fakehash ),
            Labor::new( "48652aaf-3406-4a97-b1c5-8ca90e047067", "df42a90a-1fb1-4f9f-a56a-23ec45ecbf46", "0d89b0f9-f2b6-427a-b79f-23f026ad896c", "technician", Some(&time::from_timestamp(1574701200)), Some(&time::from_timestamp(1574730000)), &time::from_timestamp(1574701200), &time::from_timestamp(1574730000), 1, &fakehash ),
            Labor::new( "e5729805-95a7-44e5-9553-204810ab96be", "df42a90a-1fb1-4f9f-a56a-23ec45ecbf46", "17e54a77-b526-425f-9227-06a31f8ed3bf", "plant president", Some(&time::from_timestamp(1574701200)), Some(&time::from_timestamp(1574730000)), &time::from_timestamp(1574701200), &time::from_timestamp(1574730000), 1, &fakehash ),
            Labor::new( "add98c12-b10f-4645-9e94-c440ae2ec97c", "df42a90a-1fb1-4f9f-a56a-23ec45ecbf46", "54093698-475f-464f-b0c2-799fab8da354", "technician", Some(&time::from_timestamp(1574787600)), Some(&time::from_timestamp(1574816400)), &time::from_timestamp(1574787600), &time::from_timestamp(1574816400), 1, &fakehash ),
            Labor::new( "27a2a0ca-2d55-4e37-a994-5a2472e7d087", "df42a90a-1fb1-4f9f-a56a-23ec45ecbf46", "0d89b0f9-f2b6-427a-b79f-23f026ad896c", "technician", Some(&time::from_timestamp(1574787600)), Some(&time::from_timestamp(1574816400)), &time::from_timestamp(1574787600), &time::from_timestamp(1574816400), 1, &fakehash ),
            Labor::new( "4811b1f1-9355-42d7-9974-83e63d828ef4", "df42a90a-1fb1-4f9f-a56a-23ec45ecbf46", "5f6e4ab4-eaa4-4225-8142-d8e9ea023117", "technician", Some(&time::from_timestamp(1574787600)), Some(&time::from_timestamp(1574816400)), &time::from_timestamp(1574787600), &time::from_timestamp(1574816400), 1, &fakehash ),
            Labor::new( "ead743fc-e6be-4461-abc8-bb9f8d9a4629", "df42a90a-1fb1-4f9f-a56a-23ec45ecbf46", "61d1e24d-e441-442b-bd75-7665a3026d3c", "technician", Some(&time::from_timestamp(1574787600)), Some(&time::from_timestamp(1574816400)), &time::from_timestamp(1574787600), &time::from_timestamp(1574816400), 1, &fakehash ),
            Labor::new( "b5ff881d-1291-4b49-b24d-6682811cd8f3", "df42a90a-1fb1-4f9f-a56a-23ec45ecbf46", "17e54a77-b526-425f-9227-06a31f8ed3bf", "plant president", Some(&time::from_timestamp(1574787600)), Some(&time::from_timestamp(1574816400)), &time::from_timestamp(1574787600), &time::from_timestamp(1574816400), 1, &fakehash ),
        ]
    }

    fn test_products() -> HashMap<String, Product> {
        let fakehash = make_hash();
        let mut products = HashMap::new();
        products.insert("58bd7cbb-b7f2-4ae5-ac46-c69438d7733f".to_owned(), Product::new( "58bd7cbb-b7f2-4ae5-ac46-c69438d7733f", "df42a90a-1fb1-4f9f-a56a-23ec45ecbf46", "electricity", &Unit::WattHour, 0 as f64, &Dimensions::new( 1 as f64, 1 as f64, 1 as f64 ), &vec![Input::new("d6b603c8-dcf1-4aac-a827-5c03b22a5b82", 0.002711685944438878 as f64)], &Effort::new(&EffortTime::Hours, 1 as u64), true, "{}", &time::from_timestamp(1574665834), &time::from_timestamp(1574665834), None, 1, &fakehash ));
        products
    }
}

