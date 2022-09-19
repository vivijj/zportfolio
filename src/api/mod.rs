use axum::extract::State;
use axum::{routing::get, Json, Router};
use tower_http::trace::TraceLayer;

use crate::storage::{chain::ChainInfo, token::TokenInfo, StorageProcessor};

use self::error::ApiError;

mod error;
mod user;

#[derive(Clone)]
pub struct ApiV1State {
    pub storage_core: StorageProcessor,
}
impl ApiV1State {
    pub async fn new() -> Self {
        Self {
            storage_core: StorageProcessor::new_from_pool().await,
        }
    }
}

async fn chain_list(State(state): State<ApiV1State>) -> Result<Json<Vec<ChainInfo>>, ApiError> {
    let res = state.storage_core.load_chains().await?;
    Ok(Json(res))
}

async fn token_list(State(state): State<ApiV1State>) -> Result<Json<Vec<TokenInfo>>, ApiError> {
    let res = state.storage_core.load_tokens().await?;
    Ok(Json(res))
}

async fn api_v1_scope() -> Router<ApiV1State> {
    Router::with_state(ApiV1State::new().await)
        .nest("/user", user::api_scope().await)
        .route("/token/list", get(token_list))
        .route("/chain/list", get(chain_list))
        .route("/favicon", get(|| async { "Hello, World!" }))
}

pub async fn start_server() {
    let app = Router::new()
        .nest("/api/v1", api_v1_scope().await)
        .layer(TraceLayer::new_for_http());

    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
