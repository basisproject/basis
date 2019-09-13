use exonum::{
    api::{self, ServiceApiBuilder, ServiceApiState},
    blockchain,
    crypto::{self, Hash},
    helpers::Height,
    explorer::BlockchainExplorer,
};
use exonum_merkledb::MapProof;
use models;
use crate::block::{
    ApiError,
    ObjectProof,
    ObjectHistory,
    ListResult,
    ProofResult,
    schema::Schema,
    SERVICE_ID,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct OrdersQuery {
    pub after: Option<String>,
    pub per_page: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderQuery {
    pub id: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct OrderApi;

impl OrderApi {
    pub fn get_orders(state: &ServiceApiState, query: OrdersQuery) -> api::Result<ListResult<models::order::Order>> {
        let snapshot = state.snapshot();
        let schema = Schema::new(&snapshot);

        let per_page = query.per_page.unwrap_or(10);
        let (from, skip) = if let Some(after) = query.after.as_ref() {
            (crypto::hash(after.as_bytes()), 1)
        } else {
            (Hash::default(), 0)
        };
        let orders = schema.orders().iter_from(&from)
            .skip(skip)
            .take(per_page)
            .map(|x| x.1)
            .collect::<Vec<_>>();
        Ok(ListResult {
            items: orders,
        })
    }

    pub fn get_order(state: &ServiceApiState, query: OrderQuery) -> api::Result<ProofResult<models::order::Order>> {
        let snapshot = state.snapshot();
        let system_schema = blockchain::Schema::new(&snapshot);
        let schema = Schema::new(&snapshot);
        let order = if query.id.is_some() {
            schema.get_order(query.id.as_ref().unwrap())
        } else {
            let err: failure::Error = From::from(ApiError::BadQuery);
            Err(err)?
        };
        let order_id = match order.as_ref() {
            Some(u) => u.id.clone(),
            None => String::from(""),
        };
        let max_height = system_schema.block_hashes_by_height().len() - 1;
        let block_proof = system_schema.block_and_precommits(Height(max_height));
        let table_proof: MapProof<Hash, Hash> = system_schema.get_proof_to_service_table(SERVICE_ID, 0);
        let order_proof: MapProof<Hash, models::order::Order> = schema.orders().get_proof(crypto::hash(order_id.as_bytes()));
        let object_proof = ObjectProof {
            table: table_proof,
            object: order_proof,
        };
        let explorer = BlockchainExplorer::new(state.blockchain());
        let order_history = order.as_ref().map(|_| {
            let history = schema.orders_history(&order_id);
            let proof = history.get_range_proof(0..history.len());

            let transactions = history
                .iter()
                .map(|record| explorer.transaction_without_proof(&record).unwrap())
                .collect::<Vec<_>>();

            ObjectHistory {
                proof,
                transactions,
            }
        });
        Ok(ProofResult {
            block_proof,
            item_proof: object_proof,
            item_history: order_history,
            item: order,
        })
    }

    pub fn wire(builder: &mut ServiceApiBuilder) {
        builder.public_scope()
            .endpoint("v1/orders", Self::get_orders)
            .endpoint("v1/orders/info", Self::get_order);
    }
}

