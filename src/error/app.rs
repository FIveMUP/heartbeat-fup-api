use crate::states::GlobalState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    response::Response,
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Server not found")]
    NOT_FOUND,
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        match self {
            ServerError::NOT_FOUND => (
                StatusCode::CONFLICT,
                Json(json!({"status": "Server not found"})),
            )
                .into_response(),
        }
    }
}
