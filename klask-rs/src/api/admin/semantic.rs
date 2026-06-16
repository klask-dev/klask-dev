//! Admin endpoints for the semantic search index (Phase 3).
//!
//! Lets an administrator rebuild the vector index from the documents already in
//! Tantivy ("Build semantic index"), poll its progress, and cancel a running
//! rebuild. All routes are admin-only (the [`AdminUser`] extractor enforces it).
//!
//! These compile in every build mode. When semantic search is disabled or the
//! binary was built without the `semantic-search` feature, the backfill
//! controller in `AppState` is `None` and the handlers report that cleanly
//! (503 for actions, a `enabled: false` status) instead of failing to build.

use crate::auth::extractors::{AdminUser, AppState};
use anyhow::Result;
use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use serde::Serialize;
#[cfg(feature = "semantic-search")]
use tracing::{info, warn};

/// Status payload returned by `GET /admin/semantic/status`.
///
/// `enabled` is the one field always meaningful: when false, the rest are
/// defaults and the rebuild actions are unavailable.
#[derive(Debug, Serialize)]
pub struct SemanticStatusResponse {
    /// True when semantic indexing is active (feature compiled in + enabled +
    /// the vector store opened successfully).
    pub enabled: bool,
    /// A rebuild is currently running.
    pub running: bool,
    /// Documents enqueued for re-embedding so far in the current/last run.
    pub processed: u64,
    /// Total documents to process (Tantivy count at start), if known.
    pub total: Option<u64>,
    /// Chunks currently stored in the vector index.
    pub chunks_indexed: u64,
    /// Embedding model id the index is built with.
    pub model: Option<String>,
    /// Embedding dimension.
    pub dimension: Option<usize>,
    /// Last error message, if the last run failed.
    pub error: Option<String>,
    /// True if the last run was cancelled.
    pub cancelled: bool,
    /// ISO-8601 start time of the current/last run.
    pub started_at: Option<String>,
    /// ISO-8601 finish time of the current/last run.
    pub finished_at: Option<String>,
}

impl SemanticStatusResponse {
    /// Status shown when semantic search is unavailable in this build/config.
    fn disabled() -> Self {
        Self {
            enabled: false,
            running: false,
            processed: 0,
            total: None,
            chunks_indexed: 0,
            model: None,
            dimension: None,
            error: None,
            cancelled: false,
            started_at: None,
            finished_at: None,
        }
    }
}

/// Generic action acknowledgement.
#[derive(Debug, Serialize)]
pub struct SemanticActionResponse {
    pub success: bool,
    pub message: String,
}

pub async fn create_router() -> Result<Router<AppState>> {
    Ok(Router::new()
        .route("/status", get(get_semantic_status))
        .route("/backfill", post(start_backfill))
        .route("/cancel", post(cancel_backfill)))
}

/// GET /admin/semantic/status — current backfill / index status.
async fn get_semantic_status(
    _admin_user: AdminUser,
    State(app_state): State<AppState>,
) -> Json<SemanticStatusResponse> {
    #[cfg(feature = "semantic-search")]
    if let Some(controller) = &app_state.semantic_backfill {
        let s = controller.status().await;
        return Json(SemanticStatusResponse {
            enabled: true,
            running: s.running,
            processed: s.processed,
            total: s.total,
            chunks_indexed: s.chunks_indexed,
            model: Some(s.model),
            dimension: Some(s.dimension),
            error: s.error,
            cancelled: s.cancelled,
            started_at: s.started_at.map(|t| t.to_rfc3339()),
            finished_at: s.finished_at.map(|t| t.to_rfc3339()),
        });
    }

    // Feature off, semantic disabled, or store failed to open.
    let _ = &app_state;
    Json(SemanticStatusResponse::disabled())
}

/// POST /admin/semantic/backfill — start a rebuild from Tantivy documents.
///
/// Returns 202 Accepted on start, 409 Conflict if one is already running, and
/// 503 Service Unavailable when semantic search is not active.
async fn start_backfill(
    _admin_user: AdminUser,
    State(app_state): State<AppState>,
) -> Result<(StatusCode, Json<SemanticActionResponse>), (StatusCode, Json<SemanticActionResponse>)> {
    #[cfg(feature = "semantic-search")]
    if let Some(controller) = &app_state.semantic_backfill {
        return match controller.start().await {
            Ok(()) => {
                info!("Admin started a semantic index backfill");
                Ok((
                    StatusCode::ACCEPTED,
                    Json(SemanticActionResponse {
                        success: true,
                        message: "Semantic index rebuild started".to_string(),
                    }),
                ))
            }
            Err(e) => {
                warn!("Semantic backfill rejected: {e}");
                Err((
                    StatusCode::CONFLICT,
                    Json(SemanticActionResponse { success: false, message: e.to_string() }),
                ))
            }
        };
    }

    let _ = &app_state;
    Err((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(SemanticActionResponse {
            success: false,
            message: "Semantic search is not enabled on this server".to_string(),
        }),
    ))
}

/// POST /admin/semantic/cancel — request cancellation of a running rebuild.
async fn cancel_backfill(
    _admin_user: AdminUser,
    State(app_state): State<AppState>,
) -> (StatusCode, Json<SemanticActionResponse>) {
    #[cfg(feature = "semantic-search")]
    if let Some(controller) = &app_state.semantic_backfill {
        controller.cancel();
        info!("Admin requested semantic backfill cancellation");
        return (
            StatusCode::OK,
            Json(SemanticActionResponse {
                success: true,
                message: "Cancellation requested; the rebuild will stop shortly".to_string(),
            }),
        );
    }

    let _ = &app_state;
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(SemanticActionResponse {
            success: false,
            message: "Semantic search is not enabled on this server".to_string(),
        }),
    )
}
