use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;

pub fn init_tracing() {
    let filter = EnvFilter::new("heartbeat_api=trace");

    let subscriber = fmt::Subscriber::builder()
        .compact()
        .with_env_filter(filter)
        .finish();

    tracing::subscriber::set_global_default(subscriber).unwrap();
}
