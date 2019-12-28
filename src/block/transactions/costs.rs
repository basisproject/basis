//! Defines logic for generating and assigning costs to products

use std::collections::HashMap;
use exonum_merkledb::IndexAccess;
use costs;
use crate::block::schema::Schema;
use super::CommonError;
use models::{
    product::Product,
    order::{CostCategory, ProcessStatus},
    costs::{Costs, CostsTallyMap},
};

static USE_AGGREGATE_COSTS: bool = true;
static MIN_FINALIZED: u64 = 2;

fn get_products<T>(schema: &mut Schema<T>, prod_ids: &Vec<String>) -> HashMap<String, Product>
    where T: IndexAccess
{
    let mut products_dedupe = HashMap::new();
    prod_ids
        .iter()
        .for_each(|x| { products_dedupe.insert(x, true); });
    let mut products = HashMap::new();
    products_dedupe.keys()
        .map(|x| schema.get_product(x))
        .filter(|x| x.is_some())
        .map(|x| x.unwrap())
        .for_each(|x| { products.insert(x.id.clone(), x); });
    products
}

/// Calculate product costs for a company by pulling out their raw order lists
/// and using them in a direct cost calculation.
///
/// This is the alternative to calculate_product_costs_with_aggregate().
pub fn calculate_product_costs_with_raw<T>(schema: &mut Schema<T>, company_id: &str) -> Result<(HashMap<String, Costs>, usize), CommonError>
    where T: IndexAccess
{
    // grab incoming/outgoing orders
    let orders_incoming = schema.get_orders_incoming_recent(company_id);
    let orders_outgoing = schema.get_orders_outgoing_recent(company_id);

    // grab how many finalized incoming orders we have
    let num_orders_incoming_finalized = orders_incoming.iter()
        .filter(|x| x.process_status == ProcessStatus::Finalized)
        .count();

    // if we have some threshold of finalized orders, then we'll use just
    // finalized orders for our calculations, otherwise use the raw order
    // lists
    let (orders_incoming, orders_outgoing) = if (num_orders_incoming_finalized as u64) >= MIN_FINALIZED {
        // we have more than 10 finalized orders, so only use finalized orders
        // in our cAlCulAtIOnS BEEp bOOP
        let incoming = orders_incoming.into_iter()
            .filter(|x| x.process_status == ProcessStatus::Finalized)
            .collect::<Vec<_>>();
        let outgoing = orders_outgoing.into_iter()
            .filter(|x| x.process_status == ProcessStatus::Finalized)
            .collect::<Vec<_>>();
        (incoming, outgoing)
    } else {
        // we don't have a lot of finalized orders so we're going to use all
        // pending orders for calculations. this helps to alleviate cases where
        // the first order that is made against a company can inflate costs to
        // extreme amounts
        (orders_incoming, orders_outgoing)
    };

    // pull out the products (both active products and products that have
    // bee ordered from us)
    let mut product_ids = schema.products_idx_company_active(company_id).iter().collect::<Vec<_>>();
    for order in &orders_incoming {
        for product in &order.products {
            product_ids.push(product.product_id.clone());
        }
    }
    let products = get_products(schema, &product_ids);

    // grab our labor records
    let labor = schema.get_labor_recent(company_id);

    // grab wamortization tables
    let amortization = HashMap::new();

    // calculate our costs
    let costs = match costs::calculate_costs(&orders_incoming, &orders_outgoing, &labor, &amortization, &products) {
        Ok(x) => x,
        Err(_) => Err(CommonError::CostError)?,
    };
    Ok((costs, orders_incoming.len()))

}

/// Calculate product costs for a company by pulling out the aggregate costs,
/// which are updated as orders are rotated in/out of the costing window. These
/// aggregates are then fed directly into the second half of the costing algo,
/// which expects aggregate values.
///
/// This is the alternative to calculate_product_costs_with_raw().
pub fn calculate_product_costs_with_aggregate<T>(schema: &mut Schema<T>, company_id: &str) -> Result<(HashMap<String, Costs>, usize), CommonError>
    where T: IndexAccess
{
    // pull out the products (both active products and products that have
    // been ordered from us)
    let mut product_ids = schema.products_idx_company_active(company_id).iter().collect::<Vec<_>>();
    let cost_agg = schema.costs_aggregate(company_id);
    let bucket_map_outputs = match cost_agg.get("product_outputs.v1") {
        Some(x) => x,
        None => CostsTallyMap::new(),
    };
    let output_tally = bucket_map_outputs.get("outputs");
    let num_incoming_orders = output_tally.len();
    if num_incoming_orders < MIN_FINALIZED {
        return calculate_product_costs_with_raw(schema, company_id);
    }
    let output_tally_total = output_tally.total();
    for k in output_tally_total.products().keys() {
        product_ids.push(k.clone());
    }
    let products = get_products(schema, &product_ids);
    let sum_hours = schema.rolling_timeline(company_id);
    let costs_tally = match cost_agg.get("costs.v1") {
        Some(x) => x,
        None => CostsTallyMap::new(),
    };
    let costs_inputs_tally = match cost_agg.get("costs_inputs.v1") {
        Some(x) => x,
        None => CostsTallyMap::new(),
    };
    let mut sum_costs = HashMap::new();
    let labor_tally = match cost_agg.get("labor.v1") {
        Some(x) => x,
        None => CostsTallyMap::new(),
    };
    let labor_costs = labor_tally.get("hours").total();
    for (cost_cat_str, costs) in costs_tally.map_ref() {
        let cat = match CostCategory::set_from_str(cost_cat_str) {
            Some(x) => x,
            None => Err(CommonError::BadCostCategory)?,
        };
        sum_costs.insert(cat, costs.total());
    }
    let op_costs = sum_costs.entry(CostCategory::Operating).or_insert(Costs::new());
    *op_costs = op_costs.clone() + labor_costs;
    let sum_produced = output_tally_total.products();
    let mut avg_input_costs = HashMap::new();
    for (prod_id, tally) in costs_inputs_tally.map_ref() {
        let avg_cost = tally.total() / (tally.len() as f64);
        avg_input_costs.insert(prod_id.clone(), avg_cost);
    }
    // calculate our costs
    let costs = match costs::calculate_costs_with_aggregates(&products, sum_hours, &sum_costs, sum_produced, &avg_input_costs) {
        Ok(x) => x,
        Err(_) => Err(CommonError::CostError)?,
    };
    Ok((costs, num_incoming_orders as usize))
}

pub fn calculate_product_costs<T>(schema: &mut Schema<T>, company_id: &str) -> Result<(), CommonError>
    where T: IndexAccess
{
    let (product_costs, num_incoming_orders) = if USE_AGGREGATE_COSTS {
        calculate_product_costs_with_aggregate(schema, company_id)?
    } else {
        calculate_product_costs_with_raw(schema, company_id)?
    };
    let empty_costs = Costs::new();
    for (product_id, costs) in product_costs.iter() {
        // if we have no incoming orders, effectively set costs to 0
        let costs = if num_incoming_orders > 0 { costs } else { &empty_costs };
        schema.product_costs_attach(product_id, costs);
    }
    Ok(())
}

#[cfg(test)]
pub mod tests {
    use chrono::Duration;
    use models;
    use util;
    use crate::block::{transactions, schema::Schema};
    use crate::test::{self, gen_uuid};

    #[test]
    fn order_costs() {
        let mut testkit = test::init_testkit();
        let uid = gen_uuid();
        let (tx_user, root_pub, root_sec) = test::tx_superuser(&uid);
        testkit.create_block_with_transactions(txvec![tx_user]);

        let co1_id = gen_uuid();
        let co2_id = gen_uuid();
        let co3_id = gen_uuid();
        let co1_founder_id = gen_uuid();
        let co2_founder_id = gen_uuid();
        let co3_founder_id = gen_uuid();
        let tx_co1 = transactions::company::TxCreatePrivate::sign(
            &co1_id,
            &String::from("company1@basis.org"),
            &String::from("Widget Builders Inc"),
            &co1_founder_id,
            &String::from("Widget builder"),
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        let tx_co2 = transactions::company::TxCreatePrivate::sign(
            &co2_id,
            &String::from("company2@basis.org"),
            &String::from("Widget Distributors Inc"),
            &co2_founder_id,
            &String::from("Widget distributor"),
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        let tx_co3 = transactions::company::TxCreatePrivate::sign(
            &co3_id,
            &String::from("company3lol@basis.org"),
            &String::from("Widget BLOWOUT EMPORIUM!!!1"),
            &co3_founder_id,
            &String::from("Widget distributor"),
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_co1, tx_co2, tx_co3]);

        let prod_id = gen_uuid();
        let tx_prod = transactions::product::TxCreate::sign(
            &prod_id,
            &co1_id,
            &String::from("Red widget"),
            &models::product::Unit::Millimeter,
            &3.0,
            &models::product::Dimensions::new(100.0, 100.0, 100.0),
            &Vec::new(),
            &models::product::Effort::new(&models::product::EffortTime::Minutes, 7),
            &true,
            &String::from("{}"),
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_prod]);

        let labor_id = gen_uuid();
        let tx_labor1 = transactions::labor::TxCreate::sign(
            &labor_id,
            &co1_id,
            &uid,
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_labor1]);

        let now = util::time::now();
        let then = now - Duration::hours(8);
        let tx_labor2 = transactions::labor::TxSetTime::sign(
            &labor_id,
            &then,
            &now,
            &now,
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_labor2]);

        let ord1_id = gen_uuid();
        let tx_ord1 = transactions::order::TxCreate::sign(
            &ord1_id,
            &co2_id,
            &co1_id,
            &models::order::CostCategory::Operating,
            &vec![models::order::ProductEntry::new(&prod_id, 20334.0, &models::costs::Costs::new(), false)],
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        let ord2_id = gen_uuid();
        let tx_ord2 = transactions::order::TxCreate::sign(
            &ord2_id,
            &co3_id,
            &co1_id,
            &models::order::CostCategory::Operating,
            &vec![models::order::ProductEntry::new(&prod_id, 10000.0, &models::costs::Costs::new(), false)],
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        testkit.create_block_with_transactions(txvec![tx_ord1, tx_ord2]);

        let snapshot = testkit.snapshot();
        let schema = Schema::new(&snapshot);
        let (_, costs, _) = schema.get_product_with_costs_tagged(&prod_id);
        let costs = costs.unwrap();
        assert_eq!(costs.labor().get("Widget builder").unwrap().clone(), 8.0 / (10000.0 + 20334.0));
    }
}

