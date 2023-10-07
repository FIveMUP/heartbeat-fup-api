use crate::{error::AppResult, states::GlobalState};
use axum::extract::{Path, State};
use tracing::info;

#[inline(always)]
pub(crate) async fn heartbeat(
    State(state): State<GlobalState>,
    Path(cfx_license): Path<String>,
) -> AppResult<()> {
    if !state.threads_service.get(&cfx_license) {
        state.threads_service.spawn_thread(&cfx_license);
    }

    state.threads_service.heartbeat(&cfx_license);

    info!("Heartbeat from {}", cfx_license);
    // Send a heartbeat to the thread

    Ok(())
}
