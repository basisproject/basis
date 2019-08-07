use exonum::{
    api::{self, ServiceApiBuilder, ServiceApiState},
    blockchain::{self, BlockProof},
    crypto::{self, Hash},
    helpers::Height,
    storage::proof_map_index::MapProof,
    explorer::BlockchainExplorer,
};
use crate::block::{
    models,
    ApiError,
    ObjectProof,
    ObjectHistory,
    schema::Schema,
    SERVICE_ID,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct CompaniesQuery {
    pub after: Option<String>,
    pub per_page: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompaniesResult {
    pub companies: Vec<models::company::Company>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompanyQuery {
    pub id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompanyResult {
    pub block_proof: Option<BlockProof>,
    pub company_proof: ObjectProof<models::company::Company>,
    pub company_history: Option<ObjectHistory>,
    pub company: Option<models::company::Company>,
}

#[derive(Debug, Clone, Copy)]
pub struct CompanyApi;

impl CompanyApi {
    pub fn get_companies(state: &ServiceApiState, query: CompaniesQuery) -> api::Result<CompaniesResult> {
        let snapshot = state.snapshot();
        let schema = Schema::new(&snapshot);

        let per_page = query.per_page.unwrap_or(10);
        let (from, skip) = if let Some(after) = query.after.as_ref() {
            (crypto::hash(after.as_bytes()), 1)
        } else {
            (Hash::default(), 0)
        };
        let companies = schema.companies().iter_from(&from)
            .skip(skip)
            .take(per_page)
            .map(|x| x.1)
            .collect::<Vec<_>>();
        Ok(CompaniesResult {
            companies,
        })
    }

    pub fn get_company(state: &ServiceApiState, query: CompanyQuery) -> api::Result<CompanyResult> {
        let snapshot = state.snapshot();
        let system_schema = blockchain::Schema::new(&snapshot);
        let schema = Schema::new(&snapshot);
        let company = if query.id.is_some() {
            schema.get_company(query.id.as_ref().unwrap())
        } else {
            let err: failure::Error = From::from(ApiError::BadQuery);
            Err(err)?
        };
        let company_id = match company.as_ref() {
            Some(u) => u.id.clone(),
            None => String::from(""),
        };
        let max_height = system_schema.block_hashes_by_height().len() - 1;
        let block_proof = system_schema.block_and_precommits(Height(max_height));
        let table_proof: MapProof<Hash, Hash> = system_schema.get_proof_to_service_table(SERVICE_ID, 0);
        let company_proof: MapProof<Hash, models::company::Company> = schema.companies().get_proof(crypto::hash(company_id.as_bytes()));
        let object_proof = ObjectProof {
            table: table_proof,
            object: company_proof,
        };
        let explorer = BlockchainExplorer::new(state.blockchain());
        let company_history = company.as_ref().map(|_| {
            let history = schema.companies_history(&company_id);
            let proof = history.get_range_proof(0, history.len());

            let transactions = history
                .iter()
                .map(|record| explorer.transaction_without_proof(&record).unwrap())
                .collect::<Vec<_>>();

            ObjectHistory {
                proof,
                transactions,
            }
        });
        Ok(CompanyResult {
            block_proof,
            company_proof: object_proof,
            company_history,
            company,
        })
    }

    pub fn wire(builder: &mut ServiceApiBuilder) {
        builder.public_scope()
            .endpoint("v1/companies", Self::get_companies)
            .endpoint("v1/companies/info", Self::get_company);
    }
}


