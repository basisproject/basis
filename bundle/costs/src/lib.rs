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
            Order::new( "d1024686-d9ff-4afa-871b-5d9ad43b3b04", "6eb292de-35b7-46f3-9214-5ac8fe074a36", "9f54262a-5765-4252-bb5b-8bba6efe25dc", &CostCategory::Operating, &vec![{ let mut costs = Costs::new(); costs.track_labor("technician", 64 as f64); costs.track_labor("plant president", 16 as f64); ProductEntry::new("b69089f7-8d9c-4925-956f-516e0d41fcec", 910000 as f64, &costs, false) }], &ProcessStatus::Finalized, &time::from_timestamp(1574569609), &time::from_timestamp(1574863200), 1, &fakehash ),
            Order::new( "8eaa94ae-ce29-4ff6-b962-ce69d5a6bb39", "ad4b34ff-534b-47ef-a3a7-6c6f4c702611", "9f54262a-5765-4252-bb5b-8bba6efe25dc", &CostCategory::Operating, &vec![{ let mut costs = Costs::new(); costs.track_labor("technician", 96 as f64); costs.track_labor("plant president", 24 as f64); ProductEntry::new("b69089f7-8d9c-4925-956f-516e0d41fcec", 455000 as f64, &costs, false) }], &ProcessStatus::Finalized, &time::from_timestamp(1574569614), &time::from_timestamp(1574942400), 1, &fakehash ),
            Order::new( "238cac5b-36c2-4f8a-99a4-311e9fb73527", "f2d76111-d7c2-4a9b-9b78-0427eb5ab960", "9f54262a-5765-4252-bb5b-8bba6efe25dc", &CostCategory::Operating, &vec![{ let mut costs = Costs::new(); costs.track_labor("plant president", 0.00003516483516483517 as f64); costs.track_labor("technician", 0.00014065934065934067 as f64); ProductEntry::new("b69089f7-8d9c-4925-956f-516e0d41fcec", 1137500 as f64, &costs, false) }], &ProcessStatus::Finalized, &time::from_timestamp(1574569619), &time::from_timestamp(1575032400), 1, &fakehash ),
        ]
    }

    fn test_orders_outgoing() -> Vec<Order> {
        let fakehash = make_hash();
        vec![
        ]
    }

    fn test_labor() -> Vec<Labor> {
        let fakehash = make_hash();
        vec![
            Labor::new( "c7f5cc32-2a1a-4797-b41a-6837502676a8", "9f54262a-5765-4252-bb5b-8bba6efe25dc", "e93fbea0-dc92-4508-a0e5-442d5a49fc4c", "technician", Some(&time::from_timestamp(1574614800)), Some(&time::from_timestamp(1574643600)), &time::from_timestamp(1574614800), &time::from_timestamp(1574643600), 1, &fakehash ),
            Labor::new( "6162cb90-a600-43f7-870c-e3ae37de4881", "9f54262a-5765-4252-bb5b-8bba6efe25dc", "f08ca5a4-47dd-461c-a7a8-3feaff111b12", "technician", Some(&time::from_timestamp(1574614800)), Some(&time::from_timestamp(1574643600)), &time::from_timestamp(1574614800), &time::from_timestamp(1574643600), 1, &fakehash ),
            Labor::new( "1e405dce-1f98-44fd-99b0-a49cba7c6704", "9f54262a-5765-4252-bb5b-8bba6efe25dc", "d2121e75-a70d-4233-af9b-19811c462e34", "technician", Some(&time::from_timestamp(1574614800)), Some(&time::from_timestamp(1574643600)), &time::from_timestamp(1574614800), &time::from_timestamp(1574643600), 1, &fakehash ),
            Labor::new( "8fbb4562-3e9c-4599-9590-1fee8ccf4975", "9f54262a-5765-4252-bb5b-8bba6efe25dc", "16ab2c66-0902-40e2-86b1-bd825ebeac97", "plant president", Some(&time::from_timestamp(1574614800)), Some(&time::from_timestamp(1574643600)), &time::from_timestamp(1574614800), &time::from_timestamp(1574643600), 1, &fakehash ),
            Labor::new( "0416b20a-24cd-44a9-87c5-83ee48094760", "9f54262a-5765-4252-bb5b-8bba6efe25dc", "5b149932-b807-4784-8c1d-d1dc15b350a6", "technician", Some(&time::from_timestamp(1574614800)), Some(&time::from_timestamp(1574643600)), &time::from_timestamp(1574614800), &time::from_timestamp(1574643600), 1, &fakehash ),
            Labor::new( "e61c08c6-296a-4339-bc80-7e7870e537d1", "9f54262a-5765-4252-bb5b-8bba6efe25dc", "e93fbea0-dc92-4508-a0e5-442d5a49fc4c", "technician", Some(&time::from_timestamp(1574701200)), Some(&time::from_timestamp(1574730000)), &time::from_timestamp(1574701200), &time::from_timestamp(1574730000), 1, &fakehash ),
            Labor::new( "4e261c70-f40c-4b1f-baea-33ba0761cd9a", "9f54262a-5765-4252-bb5b-8bba6efe25dc", "d2121e75-a70d-4233-af9b-19811c462e34", "technician", Some(&time::from_timestamp(1574701200)), Some(&time::from_timestamp(1574730000)), &time::from_timestamp(1574701200), &time::from_timestamp(1574730000), 1, &fakehash ),
            Labor::new( "00dba20d-a95e-4ac1-bd0e-95f06757919a", "9f54262a-5765-4252-bb5b-8bba6efe25dc", "5b149932-b807-4784-8c1d-d1dc15b350a6", "technician", Some(&time::from_timestamp(1574701200)), Some(&time::from_timestamp(1574730000)), &time::from_timestamp(1574701200), &time::from_timestamp(1574730000), 1, &fakehash ),
            Labor::new( "24c175b7-fd19-4851-a030-b7170790a2a3", "9f54262a-5765-4252-bb5b-8bba6efe25dc", "f08ca5a4-47dd-461c-a7a8-3feaff111b12", "technician", Some(&time::from_timestamp(1574701200)), Some(&time::from_timestamp(1574730000)), &time::from_timestamp(1574701200), &time::from_timestamp(1574730000), 1, &fakehash ),
            Labor::new( "2393bd5f-f4a2-48bc-b53b-032e6120c821", "9f54262a-5765-4252-bb5b-8bba6efe25dc", "16ab2c66-0902-40e2-86b1-bd825ebeac97", "plant president", Some(&time::from_timestamp(1574701200)), Some(&time::from_timestamp(1574730000)), &time::from_timestamp(1574701200), &time::from_timestamp(1574730000), 1, &fakehash ),
        ]
    }

    fn test_products() -> HashMap<String, Product> {
        let fakehash = make_hash();
        let mut products = HashMap::new();
        products.insert("b69089f7-8d9c-4925-956f-516e0d41fcec".to_owned(), Product::new( "b69089f7-8d9c-4925-956f-516e0d41fcec", "9f54262a-5765-4252-bb5b-8bba6efe25dc", "electricity", &Unit::WattHour, 0 as f64, &Dimensions::new( 1 as f64, 1 as f64, 1 as f64 ), &vec![Input::new("950c7ee4-56da-43e5-9694-0ee2796fa2f7", 0.002711685944438878 as f64)], &Effort::new(&EffortTime::Hours, 1 as u64), true, "{}", &time::from_timestamp(1574569588), &time::from_timestamp(1574569588), None, 1, &fakehash ));
        products
    }
}

