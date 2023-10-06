use crate::response::ApiErrorResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Internal Server Error")]
    InternalServerError,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::InternalServerError => ApiErrorResponse::send(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some("Internal Server Error".to_string()),
            ),
        }
    }
}
