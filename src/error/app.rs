use super::ServerError;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Server(#[from] ServerError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Server(e) => e.into_response(),
        }
    }
}
