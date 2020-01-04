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
pub struct CostTagsQuery {
    pub after: Option<String>,
    pub per_page: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CostTagsByCompanyQuery {
    pub company_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CostTagQuery {
    pub id: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct CostTagApi;

impl CostTagApi {
    pub fn get_cost_tags(state: &ServiceApiState, query: CostTagsQuery) -> api::Result<ListResult<models::cost_tag::CostTag>> {
        let snapshot = state.snapshot();
        let schema = Schema::new(&snapshot);

        let per_page = query.per_page.unwrap_or(10);
        let (from, skip) = if let Some(after) = query.after.as_ref() {
            (crypto::hash(after.as_bytes()), 1)
        } else {
            (Hash::default(), 0)
        };
        let cost_tags = schema.cost_tags().iter_from(&from)
            .skip(skip)
            .take(per_page)
            .map(|x| x.1)
            .collect::<Vec<_>>();
        Ok(ListResult {
            items: cost_tags,
        })
    }

    pub fn get_cost_tags_by_company(state: &ServiceApiState, query: CostTagsByCompanyQuery) -> api::Result<ListResult<models::cost_tag::CostTag>> {
        let snapshot = state.snapshot();
        let schema = Schema::new(&snapshot);

        let company = if query.company_id.is_some() {
            schema.get_company(query.company_id.as_ref().unwrap())
        } else {
            let err: failure::Error = From::from(ApiError::BadQuery);
            Err(err)?
        };
        let company_id = match company.as_ref() {
            Some(u) => u.id.clone(),
            None => String::from(""),
        };
        let cost_tags = schema.get_cost_tags_by_company_id(&company_id);
        Ok(ListResult {
            items: cost_tags,
        })
    }

    pub fn get_cost_tag(state: &ServiceApiState, query: CostTagQuery) -> api::Result<ProofResult<models::cost_tag::CostTag>> {
        let snapshot = state.snapshot();
        let system_schema = blockchain::Schema::new(&snapshot);
        let schema = Schema::new(&snapshot);
        let cost_tag = if query.id.is_some() {
            schema.get_cost_tag(query.id.as_ref().unwrap())
        } else {
            let err: failure::Error = From::from(ApiError::BadQuery);
            Err(err)?
        };
        let cost_tag_id = match cost_tag.as_ref() {
            Some(u) => u.id.clone(),
            None => String::from(""),
        };
        let max_height = system_schema.block_hashes_by_height().len() - 1;
        let block_proof = system_schema.block_and_precommits(Height(max_height));
        let table_proof: MapProof<Hash, Hash> = system_schema.get_proof_to_service_table(SERVICE_ID, 0);
        let cost_tag_proof: MapProof<Hash, models::cost_tag::CostTag> = schema.cost_tags().get_proof(crypto::hash(cost_tag_id.as_bytes()));
        let object_proof = ObjectProof {
            table: table_proof,
            object: cost_tag_proof,
        };
        let explorer = BlockchainExplorer::new(state.blockchain());
        let cost_tag_history = cost_tag.as_ref().map(|_| {
            let history = schema.cost_tags_history(&cost_tag_id);
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
            item_history: cost_tag_history,
            item: cost_tag,
        })
    }

    pub fn wire(builder: &mut ServiceApiBuilder) {
        builder.public_scope()
            .endpoint("v1/cost-tags", Self::get_cost_tags)
            .endpoint("v1/cost-tags/by-company", Self::get_cost_tags_by_company)
            .endpoint("v1/cost-tags/info", Self::get_cost_tag);
    }
}

