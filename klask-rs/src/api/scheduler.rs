use crate::auth::extractors::{AdminUser, AppState};
use anyhow::Result;
use axum::{extract::State, http::StatusCode, response::Json, routing::get, Router};
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct SchedulerStatusResponse {
    #[serde(rename = "isRunning")]
    pub is_running: bool,
    #[serde(rename = "scheduledRepositoriesCount")]
    pub scheduled_repositories_count: usize,
    #[serde(rename = "autoCrawlEnabledCount")]
    pub auto_crawl_enabled_count: usize,
    #[serde(rename = "nextRuns")]
    pub next_runs: Vec<NextScheduledRunResponse>,
}

#[derive(Debug, Serialize)]
pub struct NextScheduledRunResponse {
    #[serde(rename = "repositoryId")]
    pub repository_id: Uuid,
    #[serde(rename = "repositoryName")]
    pub repository_name: String,
    #[serde(rename = "nextRunAt")]
    pub next_run_at: Option<DateTime<Utc>>,
    #[serde(rename = "scheduleExpression")]
    pub schedule_expression: Option<String>,
}

pub async fn create_router() -> Result<Router<AppState>> {
    let router = Router::new().route("/status", get(get_scheduler_status));

    Ok(router)
}

async fn get_scheduler_status(
    State(app_state): State<AppState>,
    _admin_user: AdminUser, // Require admin authentication
) -> Result<Json<SchedulerStatusResponse>, StatusCode> {
    match &app_state.scheduler_service {
        Some(scheduler_service) => {
            let status = scheduler_service.get_status().await;
            let response = SchedulerStatusResponse {
                is_running: status.is_running,
                scheduled_repositories_count: status.scheduled_repositories_count,
                auto_crawl_enabled_count: status.auto_crawl_enabled_count,
                next_runs: status
                    .next_runs
                    .into_iter()
                    .map(|run| NextScheduledRunResponse {
                        repository_id: run.repository_id,
                        repository_name: run.repository_name,
                        next_run_at: Some(run.next_run_at),
                        schedule_expression: Some(run.schedule_expression),
                    })
                    .collect(),
            };
            Ok(Json(response))
        }
        None => {
            // Scheduler service not available
            let response = SchedulerStatusResponse {
                is_running: false,
                scheduled_repositories_count: 0,
                auto_crawl_enabled_count: 0,
                next_runs: vec![],
            };
            Ok(Json(response))
        }
    }
}
