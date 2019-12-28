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
pub struct CompaniesMembersQuery {
    pub company_id: Option<String>,
    pub after: Option<String>,
    pub per_page: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompanyMemberQuery {
    pub id: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct CompanyMemberApi;

impl CompanyMemberApi {
    pub fn get_members(state: &ServiceApiState, query: CompaniesMembersQuery) -> api::Result<ListResult<models::company_member::CompanyMember>> {
        let snapshot = state.snapshot();
        let schema = Schema::new(&snapshot);

        if query.company_id.is_none() {
            let err: failure::Error = From::from(ApiError::BadQuery);
            Err(err)?
        }
        let company_id = query.company_id.as_ref().unwrap().clone();

        let per_page = query.per_page.unwrap_or(10);
        let (from, skip) = if let Some(after) = query.after.as_ref() {
            (after.as_str(), 1)
        } else {
            ("", 0)
        };
        let members = schema.companies_members_idx_company_id(&company_id).iter_from(from)
            .skip(skip)
            .take(per_page)
            .map(|(_, member_id)| schema.get_company_member(&member_id))
            .filter(|m| m.is_some())
            .map(|m| m.unwrap())
            .collect::<Vec<_>>();
        Ok(ListResult {
            items: members,
        })
    }

    pub fn get_member(state: &ServiceApiState, query: CompanyMemberQuery) -> api::Result<ProofResult<models::company_member::CompanyMember>> {
        let snapshot = state.snapshot();
        let system_schema = blockchain::Schema::new(&snapshot);
        let schema = Schema::new(&snapshot);
        let member = if query.id.is_some() {
            schema.get_company_member(query.id.as_ref().unwrap())
        } else {
            let err: failure::Error = From::from(ApiError::BadQuery);
            Err(err)?
        };
        let member_id = match member.as_ref() {
            Some(u) => u.id.clone(),
            None => String::from(""),
        };
        let max_height = system_schema.block_hashes_by_height().len() - 1;
        let block_proof = system_schema.block_and_precommits(Height(max_height));
        let table_proof: MapProof<Hash, Hash> = system_schema.get_proof_to_service_table(SERVICE_ID, 0);
        let member_proof: MapProof<Hash, models::company_member::CompanyMember> = schema.companies_members().get_proof(crypto::hash(member_id.as_bytes()));
        let object_proof = ObjectProof {
            table: table_proof,
            object: member_proof,
        };
        let explorer = BlockchainExplorer::new(state.blockchain());
        let member_history = member.as_ref().map(|_| {
            let history = schema.companies_members_history(&member_id);
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
            item_history: member_history,
            item: member,
        })
    }

    pub fn wire(builder: &mut ServiceApiBuilder) {
        builder.public_scope()
            .endpoint("v1/companies/members", Self::get_members)
            .endpoint("v1/companies/members/info", Self::get_member);
    }
}

