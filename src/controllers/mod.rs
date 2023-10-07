use crate::error::{AppResult, ServerError};
use crate::states::GlobalState;
use axum::extract::{Path, State};

#[inline(always)]
pub(crate) async fn heartbeat(
    State(state): State<GlobalState>,
    Path(cfx_license): Path<String>,
) -> AppResult<()> {
    if !state.threads_service.get(&cfx_license) {
        let Some(server_data) = state.server_repository.find_by_license(&cfx_license).await else {
            Err(ServerError::NotFound)?
        };

        state.threads_service.spawn_thread(
            &cfx_license,
            &server_data.id.unwrap(),
            &server_data.sv_licenseKeyToken.unwrap(),
        );
    }

    state.threads_service.heartbeat(&cfx_license);

    Ok(())
}
