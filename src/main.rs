mod models;
mod handlers;
mod database;
mod auth;
mod error;
mod utils;

use axum::{
    routing::{get, post, put, delete},
    middleware,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use sqlx::PgPool;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

use error::AppError;
use auth::{AuthService, auth_middleware};
use handlers::*;

#[derive(Clone)]
struct AppState {
    db: PgPool,
    auth_service: AuthService,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost/recruitment".to_string());

    let pool = database::create_connection_pool().await?;
    
    // Run database migrations
    database::run_migrations(&pool).await?;

    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "your-super-secret-jwt-key-here".to_string());

    let app_state = AppState {
        db: pool.clone(),
        auth_service: AuthService::new(jwt_secret),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        // Health check
        .route("/", get(health_check))
        
        // Auth routes
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        .route("/api/auth/me", get(get_current_user))
        .route("/api/auth/update-password", post(update_password))
        
        // Client routes
        .route("/api/clients", get(get_clients).post(create_client))
        .route("/api/clients/:id", get(get_client).put(update_client).delete(delete_client))
        
        // Worker routes
        .route("/api/workers", get(get_workers).post(create_worker))
        .route("/api/workers/:id", get(get_worker).put(update_worker).delete(delete_worker))
        .route("/api/workers/skills", get(get_worker_skills))
        
        // Job routes
        .route("/api/jobs", get(get_jobs).post(create_job))
        .route("/api/jobs/:id", get(get_job).put(update_job).delete(delete_job))
        .route("/api/jobs/:id/applications", get(get_job_applications))
        
        // Meeting routes
        .route("/api/meetings", get(get_meetings).post(create_meeting))
        .route("/api/meetings/upcoming", get(get_upcoming_meetings))
        .route("/api/meetings/:id", get(get_meeting).put(update_meeting).delete(delete_meeting))
        .route("/api/meetings/:id/status", post(update_meeting_status))
        
        // Matching routes
        .route("/api/match/job/:job_id", get(find_matches))
        .route("/api/match/worker/:worker_id", get(find_jobs_for_worker))
        .route("/api/match/stats", get(get_matching_stats))
        
        // Apply auth middleware to protected routes
        .route_layer(middleware::from_fn_with_state(
            app_state.auth_service.clone(),
            auth_middleware,
        ))
        
        .layer(cors)
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    
    println!("🚀 Recruitment API Server running on http://{}", addr);
    println!("📊 Health check: http://{}/", addr);
    println!("🔧 API Documentation: http://{}/api/docs", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "version": "1.0.0"
    }))
}
