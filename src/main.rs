mod app;

// Local modules
use app::config::AppConfig;

// Crates
use std::net::SocketAddr;
use tokio::signal;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Crates
use axum::{error_handling::HandleErrorLayer, http::StatusCode, routing::get_service, Router};
use std::{str, time::Duration};
use tokio::net::TcpListener;
use tower::{BoxError, ServiceBuilder};
use tower_http::compression::CompressionLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

// Main /////////////////////////////

const CARGO_PKG_NAME: &str = env!("CARGO_PKG_NAME");
const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const REV: &str = env!("REV");
const BRANCH: &str = env!("BRANCH");
const BUILD_USER: &str = env!("BUILD_USER");
const RUST_VERSION: &str = env!("RUST_VERSION");

#[tokio::main]
async fn main() {
    match envy::from_env::<AppConfig>() {
        Ok(app_config) => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::EnvFilter::new(&app_config.log_level))
                .with(tracing_subscriber::fmt::layer())
                .init();
            tracing::info!("CARGO_PKG_NAME: {}", CARGO_PKG_NAME);
            tracing::info!("CARGO_PKG_VERSION: {}", CARGO_PKG_VERSION);
            tracing::info!("REV: {}", REV);
            tracing::info!("BRANCH: {}", BRANCH);
            tracing::info!("BUILD_USER: {}", BUILD_USER);
            tracing::info!("RUST_VERSION: {}", RUST_VERSION);
            tracing::debug!("APP_NAME: {:#?}", app_config.app_name);
            tracing::debug!("APP_ENVIRONMENT: {:#?}", app_config.app_environment);
            tracing::debug!("LOG_LEVEL: {:#?}", app_config.log_level);

            // Start the http server
            let app = Router::new()
                .nest_service(
                    "/",
                    get_service(
                        ServeDir::new("public").fallback(ServeFile::new("public/index.html")),
                    ),
                )
                // Add middleware to all routes
                .layer(
                    ServiceBuilder::new()
                        .layer(HandleErrorLayer::new(|error: BoxError| async move {
                            if error.is::<tower::timeout::error::Elapsed>() {
                                Ok(StatusCode::REQUEST_TIMEOUT)
                            } else {
                                Err((
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    format!("Unhandled internal error: {}", error),
                                ))
                            }
                        }))
                        .timeout(Duration::from_secs(10))
                        .layer(TraceLayer::new_for_http())
                        .layer(CompressionLayer::new())
                        .into_inner(),
                );

            let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
            tracing::debug!("listening on {}", addr);
            let listener = TcpListener::bind(addr).await.unwrap();
            if app_config.app_environment == "production" {
                axum::serve(listener, app)
                    .with_graceful_shutdown(shutdown_signal())
                    .await
                    .unwrap();
            } else {
                axum::serve(listener, app).await.unwrap();
            }
        }
        Err(error) => panic!("{:#?}", error),
    }
}

// Graceful shutdown //////////////////////////////////

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    //#[cfg(not(unix))]
    //let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::debug!("signal received, starting graceful shutdown");
}
