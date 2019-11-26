//! Defines logic for generating and assigning costs to products

use std::collections::HashMap;
use exonum_merkledb::IndexAccess;
use costs;
use crate::block::schema::Schema;
use super::CommonError;
use models::{
    order::ProcessStatus,
    costs::Costs,
};

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
    let empty_costs = Costs::new();
    for (product_id, costs) in product_costs.iter() {
        // if we have no incoming orders, effectively set costs to 0
        let costs = if orders_incoming.len() > 0 { costs } else { &empty_costs };
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
        let tx_co1 = transactions::company::TxCreatePrivate::sign(
            &co1_id,
            &String::from("company1@basis.org"),
            &String::from("Widget Builders Inc"),
            &String::from("Widget builder"),
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        let tx_co2 = transactions::company::TxCreatePrivate::sign(
            &co2_id,
            &String::from("company2@basis.org"),
            &String::from("Widget Distributors Inc"),
            &String::from("Widget distributor"),
            &util::time::now(),
            &root_pub,
            &root_sec
        );
        let tx_co3 = transactions::company::TxCreatePrivate::sign(
            &co3_id,
            &String::from("company3lol@basis.org"),
            &String::from("Widget BLOWOUT EMPORIUM!!!1"),
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

