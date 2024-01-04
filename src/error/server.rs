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
    #[error("Invalid Server Data")]
    InvalidServerData,
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let status_code = match self {
            ServerError::NotFound => StatusCode::NOT_FOUND,
            ServerError::InvalidServerData => StatusCode::INTERNAL_SERVER_ERROR,
        };

        ApiErrorResponse::send(status_code, Some(self.to_string()))
    }
}
