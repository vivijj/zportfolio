use serde_json;
use thiserror::Error;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::storage::StorageError;
use crate::{debank::openapi::DebankApiError, etherscan::EtherscanApiError};

#[derive(Debug, Error)]
pub enum ApiError {
    #[error(transparent)]
    AssetApiError(#[from] DebankApiError),
    #[error(transparent)]
    StorageError(#[from] StorageError),
    #[error(transparent)]
    AccountApiError(#[from] EtherscanApiError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::error::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = format!("error: {:?}", self);
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}
