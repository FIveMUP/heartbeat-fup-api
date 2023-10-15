use crate::response::ApiErrorResponse;
use axum::response::{IntoResponse, Response};
use hyper::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ThreadError {
    #[error("Thread Not Found")]
    NotFound,
    #[error("Thread already exists")]
    AlreadyExists,
}

impl IntoResponse for ThreadError {
    fn into_response(self) -> Response {
        let status_code = match self {
            ThreadError::NotFound => StatusCode::NOT_FOUND,
            ThreadError::AlreadyExists => StatusCode::CONFLICT,
        };

        ApiErrorResponse::send(status_code, Some(self.to_string()))
    }
}
