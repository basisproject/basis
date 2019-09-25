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
pub struct LaborListQuery {
    pub company_id: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LaborQuery {
    pub id: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct LaborApi;

impl LaborApi {
    pub fn get_labor_list(state: &ServiceApiState, query: LaborListQuery) -> api::Result<ListResult<models::labor::Labor>> {
        let snapshot = state.snapshot();
        let schema = Schema::new(&snapshot);

        let company_id = if let Some(cid) = query.company_id.as_ref() {
            cid.clone()
        } else {
            let err: failure::Error = From::from(ApiError::BadQuery);
            Err(err)?
        };
        let per_page = query.per_page.unwrap_or(10);
        let page = query.page.as_ref().unwrap_or(&1).clone();
        let labor = schema.labor_idx_company_id(&company_id)
            .iter()
            .skip((page - 1) * per_page)
            .take(per_page)
            .map(|x| schema.get_labor(&x))
            .filter(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect::<Vec<_>>();
        Ok(ListResult {
            items: labor,
        })
    }

    pub fn get_labor(state: &ServiceApiState, query: LaborQuery) -> api::Result<ProofResult<models::labor::Labor>> {
        let snapshot = state.snapshot();
        let system_schema = blockchain::Schema::new(&snapshot);
        let schema = Schema::new(&snapshot);
        let labor = if query.id.is_some() {
            schema.get_labor(query.id.as_ref().unwrap())
        } else {
            let err: failure::Error = From::from(ApiError::BadQuery);
            Err(err)?
        };
        let labor_id = match labor.as_ref() {
            Some(u) => u.id.clone(),
            None => String::from(""),
        };
        let max_height = system_schema.block_hashes_by_height().len() - 1;
        let block_proof = system_schema.block_and_precommits(Height(max_height));
        let table_proof: MapProof<Hash, Hash> = system_schema.get_proof_to_service_table(SERVICE_ID, 0);
        let labor_proof: MapProof<Hash, models::labor::Labor> = schema.labor().get_proof(crypto::hash(labor_id.as_bytes()));
        let object_proof = ObjectProof {
            table: table_proof,
            object: labor_proof,
        };
        let explorer = BlockchainExplorer::new(state.blockchain());
        let labor_history = labor.as_ref().map(|_| {
            let history = schema.labor_history(&labor_id);
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
            item_history: labor_history,
            item: labor,
        })
    }

    pub fn wire(builder: &mut ServiceApiBuilder) {
        builder.public_scope()
            .endpoint("v1/labor", Self::get_labor_list)
            .endpoint("v1/labor/info", Self::get_labor);
    }
}

