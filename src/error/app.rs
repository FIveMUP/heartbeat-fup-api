use super::{CfxApiError, ServerError, ThreadError};
use axum::response::{IntoResponse, Response};
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Server(#[from] ServerError),
    #[error(transparent)]
    Thread(#[from] ThreadError),
    #[error(transparent)]
    CfxApi(#[from] CfxApiError),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Server(e) => e.into_response(),
            AppError::Thread(e) => e.into_response(),
            AppError::CfxApi(e) => e.into_response(),
            AppError::Sqlx(e) => {
                tracing::error!("Sqlx error: {}", e);
                Response::builder()
                    .status(500)
                    .body("Internal server error".into())
                    .unwrap()
            }
            AppError::Reqwest(e) => {
                tracing::error!("Reqwest error: {}", e);
                Response::builder()
                    .status(500)
                    .body("Internal server error".into())
                    .unwrap()
            }
        }
    }
}
