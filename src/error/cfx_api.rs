use crate::response::ApiErrorResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CfxApiError {
    #[error("Failed to send entitlement heartbeat")]
    EntitlementHeartbeatFailed,
    #[error("Status code was not 200")]
    StatusCodeNot200,
    #[error("Ticket response was null")]
    TicketResponseNull,
}

impl IntoResponse for CfxApiError {
    fn into_response(self) -> Response {
        let status_code = match self {
            CfxApiError::EntitlementHeartbeatFailed => StatusCode::NOT_FOUND,
            CfxApiError::StatusCodeNot200 => StatusCode::INTERNAL_SERVER_ERROR,
            CfxApiError::TicketResponseNull => StatusCode::NOT_FOUND,
        };

        ApiErrorResponse::send(status_code, Some(self.to_string()))
    }
}
