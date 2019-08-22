use exonum::{
    api::{self, ServiceApiBuilder, ServiceApiState},
    blockchain,
    crypto::{self, Hash, PublicKey},
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
pub struct UsersQuery {
    pub after: Option<String>,
    pub per_page: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserQuery {
    pub id: Option<String>,
    pub pubkey: Option<PublicKey>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct UserApi;

impl UserApi {
    pub fn get_users(state: &ServiceApiState, query: UsersQuery) -> api::Result<ListResult<models::user::User>> {
        let snapshot = state.snapshot();
        let schema = Schema::new(&snapshot);

        let per_page = query.per_page.unwrap_or(10);
        let (from, skip) = if let Some(after) = query.after.as_ref() {
            (crypto::hash(after.as_bytes()), 1)
        } else {
            (Hash::default(), 0)
        };
        let users = schema.users().iter_from(&from)
            .skip(skip)
            .take(per_page)
            .map(|x| x.1)
            .collect::<Vec<_>>();
        Ok(ListResult {
            items: users,
        })
    }

    pub fn get_user(state: &ServiceApiState, query: UserQuery) -> api::Result<ProofResult<models::user::User>> {
        let snapshot = state.snapshot();
        let system_schema = blockchain::Schema::new(&snapshot);
        let schema = Schema::new(&snapshot);
        let user = if query.id.is_some() {
            schema.get_user(query.id.as_ref().unwrap())
        } else if query.pubkey.is_some() {
            schema.get_user_by_pubkey(query.pubkey.as_ref().unwrap())
        } else if query.email.is_some() {
            schema.get_user_by_email(query.email.as_ref().unwrap())
        } else {
            let err: failure::Error = From::from(ApiError::BadQuery);
            Err(err)?
        };
        let user_id = match user.as_ref() {
            Some(u) => u.id.clone(),
            None => String::from(""),
        };
        let max_height = system_schema.block_hashes_by_height().len() - 1;
        let block_proof = system_schema.block_and_precommits(Height(max_height));
        let table_proof: MapProof<Hash, Hash> = system_schema.get_proof_to_service_table(SERVICE_ID, 0);
        let user_proof: MapProof<Hash, models::user::User> = schema.users().get_proof(crypto::hash(user_id.as_bytes()));
        let object_proof = ObjectProof {
            table: table_proof,
            object: user_proof,
        };
        let explorer = BlockchainExplorer::new(state.blockchain());
        let user_history = user.as_ref().map(|_| {
            let history = schema.users_history(&user_id);
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
            item_history: user_history,
            item: user,
        })
    }

    pub fn wire(builder: &mut ServiceApiBuilder) {
        builder.public_scope()
            .endpoint("v1/users", Self::get_users)
            .endpoint("v1/users/info", Self::get_user);
    }
}

