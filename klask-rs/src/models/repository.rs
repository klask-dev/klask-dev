use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct Repository {
    pub id: Uuid,
    pub name: String,
    pub url: String,
    #[serde(rename = "repositoryType")]
    pub repository_type: RepositoryType,
    pub branch: Option<String>,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(skip_deserializing)]
    #[serde(rename = "accessToken")]
    pub access_token: Option<String>, // Store encrypted in DB, never send to frontend
    #[serde(rename = "gitlabNamespace")]
    pub gitlab_namespace: Option<String>,
    #[serde(rename = "isGroup")]
    pub is_group: bool,
    #[serde(rename = "lastCrawled")]
    pub last_crawled: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "createdAt")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    // Scheduling fields
    #[serde(rename = "autoCrawlEnabled")]
    pub auto_crawl_enabled: bool,
    #[serde(rename = "cronSchedule")]
    pub cron_schedule: Option<String>,
    #[serde(rename = "nextCrawlAt")]
    pub next_crawl_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "crawlFrequencyHours")]
    pub crawl_frequency_hours: Option<i32>,
    #[serde(rename = "maxCrawlDurationMinutes")]
    pub max_crawl_duration_minutes: Option<i32>,
    #[serde(rename = "lastCrawlDurationSeconds")]
    pub last_crawl_duration_seconds: Option<i32>,
    // GitLab exclusion fields
    #[serde(rename = "gitlabExcludedProjects")]
    pub gitlab_excluded_projects: Option<String>,
    #[serde(rename = "gitlabExcludedPatterns")]
    pub gitlab_excluded_patterns: Option<String>,
    // GitHub fields
    #[serde(rename = "githubNamespace")]
    pub github_namespace: Option<String>,
    #[serde(rename = "githubExcludedRepositories")]
    pub github_excluded_repositories: Option<String>,
    #[serde(rename = "githubExcludedPatterns")]
    pub github_excluded_patterns: Option<String>,
    // Crash resumption fields
    #[serde(rename = "crawlState")]
    pub crawl_state: Option<String>, // "idle", "in_progress", "failed"
    #[serde(rename = "lastProcessedProject")]
    pub last_processed_project: Option<String>, // For GitLab: project path, For Git: branch, For FileSystem: null
    #[serde(rename = "crawlStartedAt")]
    pub crawl_started_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
#[sqlx(type_name = "varchar")]
#[sqlx(rename_all = "PascalCase")]
pub enum RepositoryType {
    Git,
    GitLab,
    GitHub,
    FileSystem,
}
