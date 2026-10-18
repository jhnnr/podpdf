use podpdf::config::Config;
use podpdf::create_app;
use std::process::ExitCode;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> ExitCode {
    let config = Config::load();

    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.log_level)),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!(
        host = %config.host,
        port = %config.port,
        max_payload_bytes = config.max_payload_size,
        "Starting PodPdf server (Privacy: Zero Telemetry)"
    );

    let addr = match config.socket_addr() {
        Ok(addr) => addr,
        Err(err) => {
            error!(%err, "Failed to resolve socket address");
            return ExitCode::FAILURE;
        }
    };

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(err) => {
            error!(%err, %addr, "Failed to bind TCP listener");
            return ExitCode::FAILURE;
        }
    };

    info!(%addr, "Listening for HTTP requests");

    let app = create_app(&config);

    if let Err(err) = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        error!(%err, "Server encountered an unrecoverable error");
        return ExitCode::FAILURE;
    }

    info!("PodPdf server terminated gracefully");
    ExitCode::SUCCESS
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    info!("Termination signal received, beginning graceful shutdown");
}
