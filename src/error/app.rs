use super::{ServerError, ThreadError};
use axum::response::{IntoResponse, Response};
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Server(#[from] ServerError),
    #[error(transparent)]
    Thread(#[from] ThreadError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Server(e) => e.into_response(),
            AppError::Thread(e) => e.into_response(),
        }
    }
}
