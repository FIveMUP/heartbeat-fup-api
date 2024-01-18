use crate::error::{AppResult, ServerError};
use crate::states::GlobalState;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use compact_str::CompactString;
use hyper::StatusCode;

#[inline(always)]
pub(crate) async fn heartbeat(
    State(state): State<GlobalState>,
    Path(cfx_license): Path<CompactString>,
) -> AppResult<Response> {
    let Some(server_data) = state
        .server_repository
        .find_by_license(&cfx_license)
        .await?
    else {
        Err(ServerError::NotFound)?
    };

    if !state.threads_service.get(&cfx_license) {
        state
            .threads_service
            .spawn_thread(
                cfx_license.clone(),
                server_data.id,
                server_data.sv_license_key_token,
                server_data.name,
            )
            .await?;
    }

    state.threads_service.heartbeat(cfx_license)?;
    Ok(StatusCode::OK.into_response())
}
