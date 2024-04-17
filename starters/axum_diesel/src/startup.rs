use crate::api::routes as api_routes;
use crate::common::context::ApiContext;
use crate::config::Settings;
use anyhow::{Context, Result};
use axum::Router;
use http::header::{ACCEPT, CONTENT_TYPE};
use http::{HeaderValue, Method};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

pub async fn build(settings: Settings) -> Result<()> {
    tracing::info!("Starting server...");
    let api_context = ApiContext {
        db: settings
            .database
            .get_connection_pool()
            .context("Failed to connect to database")?,
    };

    tracing::info!("Running migrations...");
    //TODO: Run migrations

    tracing::info!("Creating router...");

    let origins: Vec<HeaderValue> = settings
        .application
        .origins
        .iter()
        .map(|o| o.parse().unwrap())
        .collect();
    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST, Method::PUT])
        // allow requests from any origin
        .allow_origin(origins)
        .allow_credentials(true)
        .allow_headers([ACCEPT, CONTENT_TYPE]);

    let api_router = Router::new()
        .nest("/api", api_routes())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(api_context.clone());

    tracing::info!("Binding port {}...", settings.application.port);
    let address = format!(
        "{}:{}",
        settings.application.host, settings.application.port
    );
    let listener = TcpListener::bind(address)
        .await
        .context("Failed to bind to port")?;

    tracing::info!("Running api...");
    run(api_router, listener).await
}

async fn run(router: Router, listener: TcpListener) -> Result<()> {
    axum::serve(listener, router)
        .await
        .context("Failed to start server")
}
