use crate::auth::extractors::{AppState, AuthenticatedUser};
use anyhow::Result;
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::get,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileResponse {
    pub id: Uuid,
    pub name: String,
    pub path: String,
    pub content: Option<String>,
    pub project: String,
    pub version: String,
    pub extension: String,
    pub size: i64,
}

pub async fn create_router() -> Result<Router<AppState>> {
    let router = Router::new().route("/{id}", get(get_file)).route("/doc/{doc_address}", get(get_file_by_doc_address));

    Ok(router)
}

async fn get_file(
    _auth: AuthenticatedUser,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<FileResponse>, StatusCode> {
    tracing::debug!("Getting file by id: {}", id);

    // Get file from Tantivy search index only
    match app_state.search_service.get_file_by_id(id).await {
        Ok(Some(search_result)) => {
            tracing::debug!(
                "Found file in search index: {} (content size: {})",
                search_result.file_name,
                search_result.content_snippet.len()
            );

            let content_size = search_result.content_snippet.len() as i64;
            Ok(Json(FileResponse {
                id,
                name: search_result.file_name,
                path: search_result.file_path,
                content: Some(search_result.content_snippet), // This is the full content from Tantivy
                project: search_result.project,
                version: search_result.version,
                extension: search_result.extension,
                size: content_size,
            }))
        }
        Ok(None) => {
            tracing::debug!("File not found in search index");
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            tracing::error!("Search service error when fetching file: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_file_by_doc_address(
    _auth: AuthenticatedUser,
    State(app_state): State<AppState>,
    Path(doc_address): Path<String>,
) -> Result<Json<FileResponse>, StatusCode> {
    tracing::debug!("Getting file by doc_address: {}", doc_address);

    // Get file directly from Tantivy using DocAddress
    match app_state.search_service.get_file_by_doc_address(&doc_address).await {
        Ok(Some(search_result)) => {
            tracing::debug!(
                "Found file at doc_address: {} (content size: {})",
                doc_address,
                search_result.content_snippet.len()
            );

            let content_size = search_result.content_snippet.len() as i64;
            Ok(Json(FileResponse {
                id: search_result.file_id,
                name: search_result.file_name,
                path: search_result.file_path,
                content: Some(search_result.content_snippet), // This is the full content from Tantivy
                project: search_result.project,
                version: search_result.version,
                extension: search_result.extension,
                size: content_size,
            }))
        }
        Ok(None) => {
            tracing::debug!("File not found at doc_address: {}", doc_address);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            tracing::error!("Search service error when fetching file by doc_address: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}
