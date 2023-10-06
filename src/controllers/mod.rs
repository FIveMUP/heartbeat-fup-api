use crate::error::AppResult;
use axum::extract::Path;
use tracing::info;

#[inline(always)]
pub(crate) async fn heartbeat(Path(cfx_license): Path<String>) -> AppResult<()> {
    info!("cfx_license: {}", cfx_license);

    Ok(())
}
