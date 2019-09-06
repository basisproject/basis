use exonum::{
    api::{self, ServiceApiBuilder, ServiceApiState},
    blockchain,
    crypto::{self, Hash},
    helpers::Height,
    explorer::BlockchainExplorer,
};
use exonum_merkledb::MapProof;
use crate::block::{
    models,
    ApiError,
    ObjectProof,
    ObjectHistory,
    ListResult,
    ProofResult,
    schema::Schema,
    SERVICE_ID,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductsQuery {
    pub after: Option<String>,
    pub per_page: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductQuery {
    pub id: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct ProductApi;

impl ProductApi {
    pub fn get_products(state: &ServiceApiState, query: ProductsQuery) -> api::Result<ListResult<models::product::Product>> {
        let snapshot = state.snapshot();
        let schema = Schema::new(&snapshot);

        let per_page = query.per_page.unwrap_or(10);
        let (from, skip) = if let Some(after) = query.after.as_ref() {
            (crypto::hash(after.as_bytes()), 1)
        } else {
            (Hash::default(), 0)
        };
        let products = schema.products().iter_from(&from)
            .skip(skip)
            .take(per_page)
            .map(|x| x.1)
            .collect::<Vec<_>>();
        Ok(ListResult {
            items: products,
        })
    }

    pub fn get_product(state: &ServiceApiState, query: ProductQuery) -> api::Result<ProofResult<models::product::Product>> {
        let snapshot = state.snapshot();
        let system_schema = blockchain::Schema::new(&snapshot);
        let schema = Schema::new(&snapshot);
        let product = if query.id.is_some() {
            schema.get_product(query.id.as_ref().unwrap())
        } else {
            let err: failure::Error = From::from(ApiError::BadQuery);
            Err(err)?
        };
        let product_id = match product.as_ref() {
            Some(u) => u.id.clone(),
            None => String::from(""),
        };
        let max_height = system_schema.block_hashes_by_height().len() - 1;
        let block_proof = system_schema.block_and_precommits(Height(max_height));
        let table_proof: MapProof<Hash, Hash> = system_schema.get_proof_to_service_table(SERVICE_ID, 0);
        let product_proof: MapProof<Hash, models::product::Product> = schema.products().get_proof(crypto::hash(product_id.as_bytes()));
        let object_proof = ObjectProof {
            table: table_proof,
            object: product_proof,
        };
        let explorer = BlockchainExplorer::new(state.blockchain());
        let product_history = product.as_ref().map(|_| {
            let history = schema.products_history(&product_id);
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
            item_history: product_history,
            item: product,
        })
    }

    pub fn wire(builder: &mut ServiceApiBuilder) {
        builder.public_scope()
            .endpoint("v1/products", Self::get_products)
            .endpoint("v1/products/info", Self::get_product);
    }
}

