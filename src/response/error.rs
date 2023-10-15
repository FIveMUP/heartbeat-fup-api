use axum::{
    async_trait,
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub struct ApiErrorResponse {
    message: Option<String>,
    code: StatusCode,
}

impl ApiErrorResponse {
    pub fn send(code: StatusCode, message: Option<String>) -> Response {
        ApiErrorResponse { code, message }.into_response()
    }
}

#[async_trait]
impl IntoResponse for ApiErrorResponse {
    fn into_response(self) -> Response {
        (
            self.code,
            self.message
                .unwrap_or_else(|| "Internal Server Error".to_string()),
        )
            .into_response()
    }
}
