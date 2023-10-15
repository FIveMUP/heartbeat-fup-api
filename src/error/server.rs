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
        let status_code = match self {
            ServerError::NotFound => StatusCode::NOT_FOUND,
        };

        ApiErrorResponse::send(status_code, Some(self.to_string()))
    }
}
