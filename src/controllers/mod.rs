use crate::states::GlobalState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use crate::error::ServerError;

#[inline(always)]
pub(crate) async fn heartbeat(
    State(state): State<GlobalState>,
    Path(cfx_license): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    if !state.threads_service.get(&cfx_license) {
        let Some(_) = state.server_repository.find_by_license(&cfx_license).await else {
            return Err(ServerError::NOT_FOUND);
        };

        state.threads_service.spawn_thread(&cfx_license);
    }

    state.threads_service.heartbeat(&cfx_license);

    Ok(Json(json!({
        "status": "ok"
    })))
}
