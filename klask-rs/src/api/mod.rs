pub mod admin;
pub mod auth;
pub mod files;
pub mod repositories;
pub mod scheduler;
pub mod search;
pub mod users;

use crate::auth::extractors::AppState;
use anyhow::Result;
use axum::{Router, routing::get};

pub async fn create_router() -> Result<Router<AppState>> {
    let router = Router::new()
        .route("/status", get(status_handler))
        .nest("/admin", admin::create_router().await?)
        .nest("/auth", auth::create_router().await?)
        .nest("/files", files::create_router().await?)
        .nest("/search", search::create_router().await?)
        .nest("/repositories", repositories::create_router().await?)
        .nest("/scheduler", scheduler::create_router().await?)
        .nest("/users", users::create_router().await?);

    Ok(router)
}

async fn status_handler() -> &'static str {
    "API is running"
}
