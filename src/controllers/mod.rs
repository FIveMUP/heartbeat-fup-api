use crate::{
    error::{AppResult, ServerError},
    states::GlobalState,
};
use axum::extract::{Path, State};

#[inline(always)]
pub(crate) async fn heartbeat(
    State(state): State<GlobalState>,
    Path(cfx_license): Path<String>,
) -> AppResult<()> {
    if !state.threads_service.get(&cfx_license) {
        let Some(_) = state.server_repository.find_by_license(&cfx_license).await else {
            // send response hello world
            Err(ServerError::NotFound)?
        };

        state.threads_service.spawn_thread(&cfx_license);
    }

    state.threads_service.heartbeat(&cfx_license);

    Ok(())
}
