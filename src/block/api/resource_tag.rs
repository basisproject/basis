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
pub struct ResourceTagListQuery {
    pub after: Option<String>,
    pub per_page: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceTagQuery {
    pub id: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct ResourceTagApi;

impl ResourceTagApi {
    pub fn get_resource_tags(state: &ServiceApiState, query: ResourceTagListQuery) -> api::Result<ListResult<models::user::User>> {
        let snapshot = state.snapshot();
        let schema = Schema::new(&snapshot);

        let per_page = query.per_page.unwrap_or(10);
        let (from, skip) = if let Some(after) = query.after.as_ref() {
            (crypto::hash(after.as_bytes()), 1)
        } else {
            (Hash::default(), 0)
        };
        let resource_tags = schema.users().iter_from(&from)
            .skip(skip)
            .take(per_page)
            .map(|x| x.1)
            .collect::<Vec<_>>();
        Ok(ListResult {
            items: resource_tags,
        })
    }

    pub fn get_resource_tag(state: &ServiceApiState, query: ResourceTagQuery) -> api::Result<ProofResult<models::resource_tag::ResourceTag>> {
        let snapshot = state.snapshot();
        let system_schema = blockchain::Schema::new(&snapshot);
        let schema = Schema::new(&snapshot);
        let resource_tag = if query.id.is_some() {
            schema.get_resource_tag(query.id.as_ref().unwrap())
        } else {
            let err: failure::Error = From::from(ApiError::BadQuery);
            Err(err)?
        };
        let resource_tag_id = match resource_tag.as_ref() {
            Some(u) => u.id.clone(),
            None => String::from(""),
        };
        let max_height = system_schema.block_hashes_by_height().len() - 1;
        let block_proof = system_schema.block_and_precommits(Height(max_height));
        let table_proof: MapProof<Hash, Hash> = system_schema.get_proof_to_service_table(SERVICE_ID, 0);
        let resource_tag_proof: MapProof<Hash, models::resource_tag::ResourceTag> = schema.resource_tags().get_proof(crypto::hash(resource_tag_id.as_bytes()));
        let object_proof = ObjectProof {
            table: table_proof,
            object: resource_tag_proof,
        };
        let explorer = BlockchainExplorer::new(state.blockchain());
        let resource_tag_history = resource_tag.as_ref().map(|_| {
            let history = schema.resource_tags_history(&resource_tag_id);
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
            item_history: resource_tag_history,
            item: resource_tag,
        })
    }

    pub fn wire(builder: &mut ServiceApiBuilder) {
        builder.public_scope()
            .endpoint("v1/resource-tags", Self::get_resource_tags)
            .endpoint("v1/resource-tags/info", Self::get_resource_tag);
    }
}


