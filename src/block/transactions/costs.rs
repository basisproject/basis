//! Defines logic for generating and assigning costs to products

use std::collections::HashMap;
use exonum_merkledb::IndexAccess;
use costs;
use crate::block::schema::Schema;
use super::CommonError;
use models::order::ProcessStatus;

pub fn calculate_product_costs<T>(schema: &mut Schema<T>, company_id: &str) -> Result<(), CommonError>
    where T: IndexAccess
{
    let orders_incoming = schema.get_orders_incoming_recent(company_id);
    let orders_outgoing = schema.get_orders_outgoing_recent(company_id);
    // grab how many finalized incoming orders we have
    let num_orders_incoming_finalized = orders_incoming.iter()
        .filter(|x| x.process_status == ProcessStatus::Finalized)
        .count();
    let orders_incoming = if num_orders_incoming_finalized > 10 {
        // we have more than 10 finalized orders, so only use finalized orders
        // in our cAlCulAtIOnS BEEp bOOP
        orders_incoming.into_iter()
            .filter(|x| x.process_status == ProcessStatus::Finalized)
            .collect::<Vec<_>>()
    } else {
        // we don't have a lot of finalized orders so we're going to use all
        // pending orders for calculations. this helps to alleviate cases where
        // the first order that is made against a company can inflate costs to
        // extreme amounts
        orders_incoming
    };
    let mut products_dedupe = HashMap::new();
    schema.products_idx_company_active(company_id)
        .iter()
        .for_each(|x| { products_dedupe.insert(x, true); });
    for order in &orders_incoming {
        for product in &order.products {
            products_dedupe.insert(product.product_id.clone(), true);
        }
    }
    let mut products = HashMap::new();
    products_dedupe.keys()
        .map(|x| schema.get_product(x))
        .filter(|x| x.is_some())
        .map(|x| x.unwrap())
        .for_each(|x| { products.insert(x.id.clone(), x); });

    let labor = schema.get_labor_recent(company_id);
    let amortization = HashMap::new();
    let product_costs = match costs::calculate_costs(&orders_incoming, &orders_outgoing, &labor, &amortization, &products) {
        Ok(x) => x,
        Err(_) => Err(CommonError::CostError)?,
    };
    for (product_id, costs) in product_costs.iter() {
        schema.product_costs_attach(product_id, costs);
    }
    Ok(())
}

