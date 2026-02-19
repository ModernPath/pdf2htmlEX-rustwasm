use std::sync::Arc;
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use ode_api::{
    db::Database,
    task_queue::TaskQueue,
    webhooks::WebhookService,
    storage::S3Storage,
    routes::AppState,
    auth_routes::create_final_router,
    auth::AuthState,
    rate_limit::RateLimitState,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ode_api=debug,tower_http=debug,axum=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost/ode".to_string());

    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());

    tracing::info!("Connecting to database...");
    let db = Database::new(&database_url).await?;
    tracing::info!("Database connected");

    tracing::info!("Connecting to Redis...");
    let task_queue = TaskQueue::new(&redis_url).await?;
    tracing::info!("Redis connected");

    let webhook_service = WebhookService::new();

    let s3_bucket = std::env::var("S3_BUCKET")
        .unwrap_or_else(|_| "ode-documents".to_string());

    tracing::info!("Connecting to S3...");
    let storage = Arc::new(S3Storage::new(s3_bucket).await?);
    tracing::info!("S3 connected");

    let auth_state = AuthState::new();
    let rate_limit_state = RateLimitState::new();

    let state = AppState {
        db,
        task_queue: Arc::new(Mutex::new(task_queue)),
        webhook_service,
        storage,
        auth_state: auth_state.clone(),
        rate_limit_state: rate_limit_state.clone(),
    };

    let app = create_final_router(state, rate_limit_state, auth_state)
        .layer(axum::extract::DefaultBodyLimit::max(50 * 1024 * 1024)) // 50MB
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
        );

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);

    let addr = format!("0.0.0.0:{}", port);
    
    tracing::info!("Starting ODE API server on {}", addr);
    tracing::info!("Swagger UI available at http://{}/docs", addr);
    tracing::info!("OpenAPI spec available at http://{}/api-docs/openapi.json", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}