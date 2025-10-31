use crate::auth::extractors::{AppState, AuthenticatedUser};
use crate::services::SearchQuery;
use anyhow::Result;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};

const MAX_FILTER_LENGTH: usize = 1000; // Maximum length for filter parameters

/// Validates filter parameters for search endpoints.
///
/// Checks that filter parameters:
/// - Are not longer than MAX_FILTER_LENGTH (1000 characters)
/// - Do not contain empty values when split by comma (e.g., "value1,,value2" is invalid)
/// - Are properly formatted as comma-separated strings
fn validate_filter_param(param_name: &str, param_value: &str) -> Result<(), String> {
    // Check length
    if param_value.len() > MAX_FILTER_LENGTH {
        return Err(format!(
            "Filter parameter '{}' exceeds maximum length of {} characters",
            param_name, MAX_FILTER_LENGTH
        ));
    }

    // Check for empty values when split by comma
    let trimmed = param_value.trim();
    if trimmed.is_empty() {
        return Err(format!(
            "Filter parameter '{}' cannot be empty after trimming",
            param_name
        ));
    }

    // Split by comma and check for empty values
    for part in trimmed.split(',') {
        let trimmed_part = part.trim();
        if trimmed_part.is_empty() {
            return Err(format!(
                "Filter parameter '{}' contains empty values (e.g., comma-separated items with no content)",
                param_name
            ));
        }
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    pub q: Option<String>,
    pub query: Option<String>,
    #[serde(alias = "max_results")]
    pub limit: Option<u32>,
    pub page: Option<u32>,
    // Multi-select filters as comma-separated strings
    pub repositories: Option<String>,
    pub projects: Option<String>,
    pub versions: Option<String>,
    pub extensions: Option<String>,
    // Size filters in bytes
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub include_facets: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FacetsRequest {
    // Optional search query to combine with filters
    pub query: Option<String>,
    // Multi-select filters as comma-separated strings
    pub repositories: Option<String>,
    pub projects: Option<String>,
    pub versions: Option<String>,
    pub extensions: Option<String>,
    // Size filters in bytes
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total: u64,
    pub page: u32,
    pub limit: u32,
    pub facets: Option<SearchFacets>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchFacets {
    pub repositories: Vec<FacetValue>,
    pub projects: Vec<FacetValue>,
    pub versions: Vec<FacetValue>,
    pub extensions: Vec<FacetValue>,
    pub size_ranges: Vec<FacetValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacetValue {
    pub value: String,
    pub count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub file_id: String,
    pub doc_address: String,
    pub name: String,
    pub path: String,
    pub content_snippet: String,
    pub project: String,
    pub version: String,
    pub extension: String,
    pub score: f32,
    pub line_number: Option<u32>,
}

pub async fn create_router() -> Result<Router<AppState>> {
    let router = Router::new().route("/", get(search_files)).route("/facets", get(get_facets_with_filters));

    Ok(router)
}

async fn search_files(
    _auth: AuthenticatedUser,
    State(app_state): State<AppState>,
    Query(params): Query<SearchRequest>,
) -> Result<Json<SearchResponse>, StatusCode> {
    tracing::debug!("Search request params: {:?}", params);

    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(50).min(1000); // Cap at 1000 results max
    let offset = (page - 1) * limit;

    // Get search query from either 'q' or 'query' parameter
    let query_string = params.q.or(params.query).ok_or(StatusCode::BAD_REQUEST)?;

    // Build search query - filters are already comma-separated strings
    let search_query = SearchQuery {
        query: query_string,
        repository_filter: params.repositories,
        project_filter: params.projects,
        version_filter: params.versions,
        extension_filter: params.extensions,
        min_size: params.min_size,
        max_size: params.max_size,
        limit: limit as usize,
        offset: offset as usize,
        include_facets: params.include_facets.unwrap_or(false),
    };

    // Perform search using Tantivy
    match app_state.search_service.search(search_query).await {
        Ok(search_response) => {
            let results: Vec<SearchResult> = search_response
                .results
                .into_iter()
                .map(|r| SearchResult {
                    id: r.file_id.to_string(),
                    file_id: r.file_id.to_string(),
                    doc_address: r.doc_address,
                    name: r.file_name,
                    path: r.file_path,
                    content_snippet: r.content_snippet,
                    project: r.project,
                    version: r.version,
                    extension: r.extension,
                    score: r.score,
                    line_number: r.line_number,
                })
                .collect();

            // Convert facets to API format if present
            let facets = search_response.facets.map(|service_facets| SearchFacets {
                repositories: service_facets
                    .repositories
                    .into_iter()
                    .map(|(value, count)| FacetValue { value, count })
                    .collect(),
                projects: service_facets
                    .projects
                    .into_iter()
                    .map(|(value, count)| FacetValue { value, count })
                    .collect(),
                versions: service_facets
                    .versions
                    .into_iter()
                    .map(|(value, count)| FacetValue { value, count })
                    .collect(),
                extensions: service_facets
                    .extensions
                    .into_iter()
                    .map(|(value, count)| FacetValue { value, count })
                    .collect(),
                size_ranges: service_facets
                    .size_ranges
                    .into_iter()
                    .map(|(value, count)| FacetValue { value, count })
                    .collect(),
            });

            let response = SearchResponse { total: search_response.total, results, page, limit, facets };

            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("Search failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_facets_with_filters(
    _auth: AuthenticatedUser,
    State(app_state): State<AppState>,
    Query(params): Query<FacetsRequest>,
) -> Result<Json<SearchFacets>, StatusCode> {
    tracing::debug!("Facets request params: {:?}", params);

    // Validate filter parameters
    if let Some(ref repos) = params.repositories {
        if let Err(e) = validate_filter_param("repositories", repos) {
            tracing::warn!("Invalid filter parameter - repositories: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    if let Some(ref projects) = params.projects {
        if let Err(e) = validate_filter_param("projects", projects) {
            tracing::warn!("Invalid filter parameter - projects: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    if let Some(ref versions) = params.versions {
        if let Err(e) = validate_filter_param("versions", versions) {
            tracing::warn!("Invalid filter parameter - versions: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    if let Some(ref extensions) = params.extensions {
        if let Err(e) = validate_filter_param("extensions", extensions) {
            tracing::warn!("Invalid filter parameter - extensions: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    // Build search query with optional search query and filters
    // If no query provided, use "*" to match all documents
    let search_query = SearchQuery {
        query: params.query.clone().unwrap_or_else(|| "*".to_string()),
        repository_filter: params.repositories,
        project_filter: params.projects,
        version_filter: params.versions,
        extension_filter: params.extensions,
        min_size: params.min_size,
        max_size: params.max_size,
        limit: 0, // We only need facets, not results
        offset: 0,
        include_facets: true, // Always include facets for this endpoint
    };

    // Perform search using Tantivy
    match app_state.search_service.search(search_query).await {
        Ok(search_response) => {
            let facets = search_response
                .facets
                .map(|service_facets| SearchFacets {
                    repositories: service_facets
                        .repositories
                        .into_iter()
                        .map(|(value, count)| FacetValue { value, count })
                        .collect(),
                    projects: service_facets
                        .projects
                        .into_iter()
                        .map(|(value, count)| FacetValue { value, count })
                        .collect(),
                    versions: service_facets
                        .versions
                        .into_iter()
                        .map(|(value, count)| FacetValue { value, count })
                        .collect(),
                    extensions: service_facets
                        .extensions
                        .into_iter()
                        .map(|(value, count)| FacetValue { value, count })
                        .collect(),
                    size_ranges: service_facets
                        .size_ranges
                        .into_iter()
                        .map(|(value, count)| FacetValue { value, count })
                        .collect(),
                })
                .unwrap_or_else(|| SearchFacets {
                    repositories: vec![],
                    projects: vec![],
                    versions: vec![],
                    extensions: vec![],
                    size_ranges: vec![],
                });

            Ok(Json(facets))
        }
        Err(e) => {
            tracing::error!("Failed to get facets with filters: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[cfg(test)]
mod validation_tests {
    use super::*;

    #[test]
    fn test_validate_filter_param_valid_single_value() {
        let result = validate_filter_param("test", "value1");
        assert!(result.is_ok(), "Single value should be valid");
    }

    #[test]
    fn test_validate_filter_param_valid_multiple_values() {
        let result = validate_filter_param("test", "value1,value2,value3");
        assert!(result.is_ok(), "Multiple comma-separated values should be valid");
    }

    #[test]
    fn test_validate_filter_param_valid_with_spaces() {
        let result = validate_filter_param("test", "value1, value2, value3");
        assert!(result.is_ok(), "Values with spaces around commas should be valid");
    }

    #[test]
    fn test_validate_filter_param_valid_with_leading_trailing_whitespace() {
        let result = validate_filter_param("test", "  value1,value2  ");
        assert!(result.is_ok(), "Leading and trailing whitespace should be trimmed");
    }

    #[test]
    fn test_validate_filter_param_empty_after_trim() {
        let result = validate_filter_param("test", "   ");
        assert!(result.is_err(), "Empty string after trimming should be invalid");
    }

    #[test]
    fn test_validate_filter_param_empty_values() {
        let result = validate_filter_param("test", "value1,,value2");
        assert!(result.is_err(), "Empty values (consecutive commas) should be invalid");
    }

    #[test]
    fn test_validate_filter_param_empty_values_with_spaces() {
        let result = validate_filter_param("test", "value1, , value2");
        assert!(result.is_err(), "Empty values with only spaces should be invalid");
    }

    #[test]
    fn test_validate_filter_param_exceeds_max_length() {
        let long_value = "x".repeat(MAX_FILTER_LENGTH + 1);
        let result = validate_filter_param("test", &long_value);
        assert!(result.is_err(), "Value exceeding max length should be invalid");
    }

    #[test]
    fn test_validate_filter_param_at_max_length() {
        let long_value = "x".repeat(MAX_FILTER_LENGTH);
        let result = validate_filter_param("test", &long_value);
        assert!(result.is_ok(), "Value at max length should be valid");
    }

    #[test]
    fn test_validate_filter_param_comma_at_end() {
        let result = validate_filter_param("test", "value1,value2,");
        assert!(result.is_err(), "Trailing comma should be invalid (empty value)");
    }

    #[test]
    fn test_validate_filter_param_comma_at_start() {
        let result = validate_filter_param("test", ",value1,value2");
        assert!(result.is_err(), "Leading comma should be invalid (empty value)");
    }

    #[test]
    fn test_validate_filter_param_error_message_contains_param_name() {
        let result = validate_filter_param("repositories", "   ");
        match result {
            Err(e) => {
                assert!(
                    e.contains("repositories"),
                    "Error message should contain parameter name"
                );
            }
            Ok(_) => panic!("Should have returned an error"),
        }
    }
}
