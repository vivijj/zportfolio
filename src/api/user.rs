use std::env;

use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::storage::StorageProcessor;

use super::error::ApiError;

use crate::debank::openapi::{DebankOpenAPI, DebankTokenBalance, DebankTotalBalance};
use crate::etherscan;

#[derive(Clone)]
pub struct ApiUserData {
    storage_core: StorageProcessor,
    support_chains: Vec<String>,
    acc_api: etherscan::EtherscanAPi,
    ass_api: DebankOpenAPI,
}

impl ApiUserData {
    pub async fn new() -> Self {
        let storage = StorageProcessor::new_from_pool().await;
        let chains = storage.load_support_chain_ids().await.expect("fail in db");

        let etherscan_key = env::var("ETHERSCAN_KEY").expect("no valid etherscan key");
        let debank_key = env::var("DEBANK_KEY").expect("no valid debank key");

        let acc_api = etherscan::EtherscanAPi::new(&etherscan_key, "https://api.etherscan.io/api");
        let ass_api = DebankOpenAPI::new("https://pro-openapi.debank.com/v1", &debank_key);
        Self {
            storage_core: storage,
            support_chains: chains,
            acc_api,
            ass_api,
        }
    }
}
#[derive(Debug, Deserialize)]
pub struct QueryWithId {
    id: String,
}

#[derive(Debug, Deserialize)]
pub struct QueryWithTokenName {
    id: String,
    token_name: String,
}

#[derive(Debug, Deserialize)]
pub struct QueryTokenWithId {
    id: String,
    token_id: String,
    chain_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AccountInfo {
    activation_time: i64,
}

pub async fn api_scope() -> Router<ApiUserData> {
    Router::with_state(ApiUserData::new().await)
        .route("/", get(account_info))
        .route("/token", get(token_balance))
        .route("/total_balance", get(total_balance))
        .route("/vote_token_amount", get(token_total_amount))
}

async fn account_info(
    State(state): State<ApiUserData>,
    Query(info): Query<QueryWithId>,
) -> Result<Json<AccountInfo>, ApiError> {
    let activation_time = match state.storage_core.load_user_info(&info.id).await {
        Ok(user_info) => user_info.activation_time,
        _ => {
            let activation_time = state.acc_api.account_age(&info.id).await?;
            state
                .storage_core
                .set_user_info(&info.id, activation_time)
                .await
                .unwrap();
            activation_time
        }
    };
    Ok(Json(AccountInfo { activation_time }))
}

async fn total_balance(
    State(state): State<ApiUserData>,
    Query(info): Query<QueryWithId>,
) -> Result<Json<DebankTotalBalance>, ApiError> {
    let res = state
        .ass_api
        .muti_chain_balance(&info.id, &state.support_chains)
        .await?;

    Ok(Json(res))
}

async fn token_balance(
    State(state): State<ApiUserData>,
    Query(info): Query<QueryTokenWithId>,
) -> Result<Json<DebankTokenBalance>, ApiError> {
    let res = state
        .ass_api
        .token_balance(&info.id, &info.chain_id, &info.token_id)
        .await?;
    Ok(Json(res))
}

async fn token_total_amount(
    State(state): State<ApiUserData>,
    Query(info): Query<QueryWithTokenName>,
) -> Result<Json<Value>, ApiError> {
    let token_ids = state
        .storage_core
        .load_token_ids_by_name(info.token_name)
        .await?;
    let mut token_amount = 0.0;
    for (k, v) in &token_ids {
        let balance = state.ass_api.token_balance(&info.id, &k, &v).await?;
        token_amount += balance.amount;
    }
    Ok(Json(json!({ "amount": token_amount })))
}
