use crate::error::ServerError;
use crate::states::GlobalState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use tracing::info;

#[inline(always)]
pub(crate) async fn heartbeat(
    State(state): State<GlobalState>,
    Path(cfx_license): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    let server_data = state.server_repository.find_by_license(&cfx_license).await;

    if server_data.is_empty() {
        return Err(ServerError::NOT_FOUND);
    }

    if !state.threads_service.get(&cfx_license) {
        state.threads_service.spawn_thread(
            &cfx_license,
            &server_data[0].id.as_deref().unwrap_or(""),
            &server_data[0].sv_licenseKeyToken.as_deref().unwrap_or(""),
        );
    }

    state.threads_service.heartbeat(&cfx_license);

    Ok(Json(json!({
        "status": "ok"
    })))
}
