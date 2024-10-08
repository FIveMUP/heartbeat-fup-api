use crate::{config::Database, controllers::heartbeat, states::GlobalState};
use axum::{
    error_handling::HandleErrorLayer,
    http::StatusCode,
    routing::{get, IntoMakeService},
    Router,
};
use std::time::Duration;
use tower::{load_shed::LoadShedLayer, ServiceBuilder};

#[inline(always)]
async fn handle(_: Box<dyn std::error::Error + Send + Sync>) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Internal Server Error".to_string(),
    )
}

#[inline(always)]
pub async fn routes() -> IntoMakeService<Router> {
    let db = Database::new().await;
    let global_state = GlobalState::new(db);

    Router::new()
        .route("/heartbeat/:cfx_license", get(heartbeat))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle))
                .layer(LoadShedLayer::new())
                .timeout(Duration::from_secs(5)),
        )
        .with_state(global_state)
        .into_make_service()
}
