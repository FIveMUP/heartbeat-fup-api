use crate::response::ApiErrorResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Server Not Found")]
    NotFound,
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        match self {
            ServerError::NotFound => {
                ApiErrorResponse::send(StatusCode::NOT_FOUND, Some("server Not Found".to_string()))
            }
        }
    }
}
