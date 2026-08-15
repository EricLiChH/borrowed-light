use std::sync::Arc;

use monitor_core::{CheckPolicy, HealthChecker};
use monitor_store::SqliteRepository;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                tracing_subscriber::EnvFilter::new("monitor_web=info,tower_http=info")
            }),
        )
        .try_init()?;

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| String::from("sqlite://monitor.db"));
    let bind_address =
        std::env::var("BIND_ADDRESS").unwrap_or_else(|_| String::from("127.0.0.1:3000"));
    let repository = Arc::new(SqliteRepository::connect(&database_url).await?);
    let checker = HealthChecker::new(reqwest::Client::new(), CheckPolicy::default());
    let router = monitor_web::app(repository, checker).layer(
        TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
            .on_response(DefaultOnResponse::new().level(Level::INFO)),
    );
    let listener = tokio::net::TcpListener::bind(&bind_address).await?;

    tracing::info!(address = %bind_address, "monitor web server started");
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        tracing::error!(%error, "failed to listen for shutdown signal");
    }
}
