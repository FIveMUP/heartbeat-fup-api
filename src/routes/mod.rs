use crate::{config::Database, controllers::index, states::GlobalState};
use axum::{
    error_handling::HandleErrorLayer,
    http::StatusCode,
    routing::{get, IntoMakeService},
    Router,
};
use std::{sync::Arc, time::Duration};
use tower::{load_shed::LoadShedLayer, ServiceBuilder};

async fn handle(_: Box<dyn std::error::Error + Send + Sync>) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Internal Server Error".to_string(),
    )
}

pub(crate) async fn routes(db: Arc<Database>) -> IntoMakeService<Router> {
    let global_state = GlobalState::new(db);

    Router::new()
        .route("/", get(index))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle))
                .layer(LoadShedLayer::new())
                .timeout(Duration::from_secs(10)),
        )
        .with_state(global_state)
        .into_make_service()
}
