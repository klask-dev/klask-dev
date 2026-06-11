//! MCP tool definitions and handlers.
//!
//! Tools are thin adapters over the existing services (`SearchService`,
//! `RepositoryRepository`): protocol-level failures (unknown tool, invalid
//! arguments) surface as JSON-RPC errors, while execution failures are
//! reported in-band via `isError: true` results, per the MCP specification.

use crate::auth::extractors::AppState;
use crate::repositories::repository_repository::RepositoryRepository;
use crate::services::SearchQuery;
use serde::Deserialize;
use serde_json::{Value, json};

/// Maximum number of search results a single tool call can return.
const MAX_SEARCH_LIMIT: u32 = 100;
const DEFAULT_SEARCH_LIMIT: u32 = 20;

/// Maximum number of lines returned by `get_file` in a single call.
const MAX_FILE_LINES: usize = 2000;

/// Protocol-level tool call failures (mapped to JSON-RPC errors by the caller).
#[derive(Debug)]
pub enum ToolCallError {
    UnknownTool(String),
    InvalidParams(String),
}

/// Tool definitions advertised by `tools/list`.
pub fn tool_definitions() -> Value {
    let string_array = |description: &str| {
        json!({
            "type": "array",
            "items": { "type": "string" },
            "description": description
        })
    };

    json!([
        {
            "name": "search_code",
            "description": "Full-text search across all indexed Git repositories and branches. \
                Supports plain terms, exact phrases (in double quotes) and regular expressions. \
                Returns matching files with a content snippet, the matching line number and a \
                doc_address usable with the get_file tool.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search terms, a \"quoted phrase\", or a regex pattern when regex is true"
                    },
                    "repositories": string_array("Restrict to these repository names"),
                    "projects": string_array("Restrict to these project names (GitLab/GitHub sub-projects)"),
                    "versions": string_array("Restrict to these branches or tags (e.g. [\"main\"])"),
                    "extensions": string_array("Restrict to these file extensions, without dot (e.g. [\"rs\", \"ts\"])"),
                    "regex": {
                        "type": "boolean",
                        "description": "Treat query as a regular expression (default false)"
                    },
                    "case_sensitive": {
                        "type": "boolean",
                        "description": "Case-sensitive matching (default false)"
                    },
                    "limit": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": MAX_SEARCH_LIMIT,
                        "description": "Maximum results to return (default 20, max 100)"
                    },
                    "page": {
                        "type": "integer",
                        "minimum": 1,
                        "description": "Result page, 1-based (default 1)"
                    }
                },
                "required": ["query"]
            }
        },
        {
            "name": "get_file",
            "description": "Retrieve the content of an indexed file, identified by the doc_address \
                (preferred, as returned by search_code) or by file_id. Large files are truncated; \
                use start_line/end_line to read a specific range.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "doc_address": {
                        "type": "string",
                        "description": "Document address from search_code results (format \"segment:doc\")"
                    },
                    "file_id": {
                        "type": "string",
                        "description": "File UUID from search_code results (alternative to doc_address)"
                    },
                    "start_line": {
                        "type": "integer",
                        "minimum": 1,
                        "description": "First line to return, 1-based inclusive (default 1)"
                    },
                    "end_line": {
                        "type": "integer",
                        "minimum": 1,
                        "description": "Last line to return, 1-based inclusive (default end of file)"
                    }
                }
            }
        },
        {
            "name": "list_repositories",
            "description": "List the repositories indexed by Klask with their type (Git, GitLab, \
                GitHub, FileSystem), URL and last crawl time.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        },
        {
            "name": "get_search_facets",
            "description": "Discover what is searchable: returns the available repositories, \
                projects, branches/tags and file extensions with their document counts, optionally \
                narrowed by a query and/or filters. Useful to scope a search_code call.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Optional search query to narrow the facets"
                    },
                    "repositories": string_array("Restrict to these repository names"),
                    "projects": string_array("Restrict to these project names"),
                    "versions": string_array("Restrict to these branches or tags"),
                    "extensions": string_array("Restrict to these file extensions, without dot")
                }
            }
        }
    ])
}

/// Dispatch a `tools/call` request to the matching tool handler.
pub async fn call_tool(state: &AppState, name: &str, arguments: &Value) -> Result<Value, ToolCallError> {
    match name {
        "search_code" => search_code(state, arguments).await,
        "get_file" => get_file(state, arguments).await,
        "list_repositories" => list_repositories(state).await,
        "get_search_facets" => get_search_facets(state, arguments).await,
        other => Err(ToolCallError::UnknownTool(other.to_string())),
    }
}

fn parse_args<T: serde::de::DeserializeOwned>(arguments: &Value) -> Result<T, ToolCallError> {
    // tools/call without an "arguments" field is valid for tools with no required input
    let value = if arguments.is_null() { json!({}) } else { arguments.clone() };
    serde_json::from_value(value).map_err(|e| ToolCallError::InvalidParams(format!("Invalid tool arguments: {e}")))
}

/// Join an optional list filter into the comma-separated form expected by `SearchService`.
fn join_filter(values: &Option<Vec<String>>) -> Option<String> {
    values.as_ref().and_then(|v| {
        let joined = v.iter().map(|s| s.trim()).filter(|s| !s.is_empty()).collect::<Vec<_>>().join(",");
        if joined.is_empty() { None } else { Some(joined) }
    })
}

#[derive(Debug, Deserialize)]
struct SearchCodeArgs {
    query: String,
    repositories: Option<Vec<String>>,
    projects: Option<Vec<String>>,
    versions: Option<Vec<String>>,
    extensions: Option<Vec<String>>,
    #[serde(default)]
    regex: bool,
    #[serde(default)]
    case_sensitive: bool,
    limit: Option<u32>,
    page: Option<u32>,
}

async fn search_code(state: &AppState, arguments: &Value) -> Result<Value, ToolCallError> {
    let args: SearchCodeArgs = parse_args(arguments)?;

    if args.query.trim().is_empty() {
        return Err(ToolCallError::InvalidParams("'query' must not be empty".to_string()));
    }

    if args.regex
        && let Err(e) = crate::api::regex_validator::validate_regex_pattern(&args.query)
    {
        return Err(ToolCallError::InvalidParams(format!("Invalid regex pattern: {e}")));
    }

    let limit = args.limit.unwrap_or(DEFAULT_SEARCH_LIMIT).clamp(1, MAX_SEARCH_LIMIT);
    let page = args.page.unwrap_or(1).max(1);
    // Compute in u64: (page - 1) * limit overflows u32 for very large pages
    let offset = ((page as u64 - 1) * limit as u64) as usize;

    let search_query = SearchQuery {
        query: args.query,
        repository_filter: join_filter(&args.repositories),
        project_filter: join_filter(&args.projects),
        version_filter: join_filter(&args.versions),
        extension_filter: join_filter(&args.extensions),
        min_size: None,
        max_size: None,
        limit: limit as usize,
        offset,
        include_facets: false,
        fuzzy_search: false,
        regex_search: args.regex,
        regex_flags: None,
        case_sensitive: args.case_sensitive,
    };

    match state.search_service.search(search_query).await {
        Ok(response) => {
            let results: Vec<Value> = response
                .results
                .into_iter()
                .map(|r| {
                    json!({
                        "repository": r.repository,
                        "project": r.project,
                        "version": r.version,
                        "path": r.file_path,
                        "extension": r.extension,
                        "line_number": r.line_number,
                        "score": r.score,
                        "snippet": r.content_snippet,
                        "doc_address": r.doc_address,
                        "file_id": r.file_id,
                    })
                })
                .collect();

            Ok(crate::mcp::protocol::tool_result(&json!({
                "total": response.total,
                "page": page,
                "limit": limit,
                "results": results,
            })))
        }
        Err(e) => {
            tracing::error!("MCP search_code failed: {e}");
            Ok(crate::mcp::protocol::tool_error(format!("Search failed: {e}")))
        }
    }
}

#[derive(Debug, Deserialize)]
struct GetFileArgs {
    doc_address: Option<String>,
    file_id: Option<String>,
    start_line: Option<usize>,
    end_line: Option<usize>,
}

async fn get_file(state: &AppState, arguments: &Value) -> Result<Value, ToolCallError> {
    let args: GetFileArgs = parse_args(arguments)?;

    let lookup = if let Some(doc_address) = args.doc_address.as_deref() {
        state.search_service.get_file_by_doc_address(doc_address).await
    } else if let Some(file_id) = args.file_id.as_deref() {
        let uuid = file_id
            .parse::<uuid::Uuid>()
            .map_err(|_| ToolCallError::InvalidParams(format!("'file_id' is not a valid UUID: {file_id}")))?;
        state.search_service.get_file_by_id(uuid).await
    } else {
        return Err(ToolCallError::InvalidParams(
            "Either 'doc_address' or 'file_id' must be provided".to_string(),
        ));
    };

    let file = match lookup {
        Ok(Some(file)) => file,
        Ok(None) => return Ok(crate::mcp::protocol::tool_error("File not found in the search index")),
        Err(e) => {
            tracing::error!("MCP get_file failed: {e}");
            return Ok(crate::mcp::protocol::tool_error(format!("Failed to fetch file: {e}")));
        }
    };

    // For full documents fetched by id/address, content_snippet holds the entire file content
    let slice = slice_lines(&file.content_snippet, args.start_line, args.end_line, MAX_FILE_LINES);

    Ok(crate::mcp::protocol::tool_result(&json!({
        "file_id": file.file_id,
        "name": file.file_name,
        "path": file.file_path,
        "repository": file.repository,
        "project": file.project,
        "version": file.version,
        "extension": file.extension,
        "total_lines": slice.total_lines,
        "start_line": slice.start_line,
        "end_line": slice.end_line,
        "truncated": slice.truncated,
        "content": slice.content,
    })))
}

struct LineSlice {
    content: String,
    total_lines: usize,
    start_line: usize,
    end_line: usize,
    truncated: bool,
}

/// Extract a 1-based inclusive line range from `content`, capped at `max_lines`.
fn slice_lines(content: &str, start_line: Option<usize>, end_line: Option<usize>, max_lines: usize) -> LineSlice {
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();

    let start = start_line.unwrap_or(1).max(1);
    let requested_end = end_line.unwrap_or(total_lines).min(total_lines);

    if start > requested_end || total_lines == 0 {
        return LineSlice {
            content: String::new(),
            total_lines,
            start_line: start,
            end_line: start.saturating_sub(1),
            truncated: false,
        };
    }

    let capped_end = requested_end.min(start + max_lines - 1);
    let truncated = capped_end < requested_end;

    LineSlice {
        content: lines[start - 1..capped_end].join("\n"),
        total_lines,
        start_line: start,
        end_line: capped_end,
        truncated,
    }
}

async fn list_repositories(state: &AppState) -> Result<Value, ToolCallError> {
    let repo_repository = RepositoryRepository::new(state.database.pool().clone());

    match repo_repository.list_repositories().await {
        Ok(repositories) => {
            // Explicit read-only projection: never expose tokens or crawl configuration
            let repositories: Vec<Value> = repositories
                .into_iter()
                .map(|r| {
                    json!({
                        "name": r.name,
                        "url": r.url,
                        "repository_type": r.repository_type,
                        "branch": r.branch,
                        "enabled": r.enabled,
                        "last_crawled": r.last_crawled,
                    })
                })
                .collect();

            Ok(crate::mcp::protocol::tool_result(&json!({
                "total": repositories.len(),
                "repositories": repositories,
            })))
        }
        Err(e) => {
            tracing::error!("MCP list_repositories failed: {e}");
            Ok(crate::mcp::protocol::tool_error(format!(
                "Failed to list repositories: {e}"
            )))
        }
    }
}

#[derive(Debug, Deserialize)]
struct GetSearchFacetsArgs {
    query: Option<String>,
    repositories: Option<Vec<String>>,
    projects: Option<Vec<String>>,
    versions: Option<Vec<String>>,
    extensions: Option<Vec<String>>,
}

async fn get_search_facets(state: &AppState, arguments: &Value) -> Result<Value, ToolCallError> {
    let args: GetSearchFacetsArgs = parse_args(arguments)?;

    let query = match args.query {
        Some(q) if !q.trim().is_empty() => q,
        _ => "*".to_string(),
    };

    let search_query = SearchQuery {
        query,
        repository_filter: join_filter(&args.repositories),
        project_filter: join_filter(&args.projects),
        version_filter: join_filter(&args.versions),
        extension_filter: join_filter(&args.extensions),
        min_size: None,
        max_size: None,
        limit: 0,
        offset: 0,
        include_facets: true,
        fuzzy_search: false,
        regex_search: false,
        regex_flags: None,
        case_sensitive: false,
    };

    match state.search_service.search(search_query).await {
        Ok(response) => {
            let to_values = |facet: Vec<(String, u64)>| -> Vec<Value> {
                facet.into_iter().map(|(value, count)| json!({ "value": value, "count": count })).collect()
            };

            let facets = response.facets.map(|f| {
                json!({
                    "repositories": to_values(f.repositories),
                    "projects": to_values(f.projects),
                    "versions": to_values(f.versions),
                    "extensions": to_values(f.extensions),
                })
            });

            Ok(crate::mcp::protocol::tool_result(&facets.unwrap_or_else(
                || json!({ "repositories": [], "projects": [], "versions": [], "extensions": [] }),
            )))
        }
        Err(e) => {
            tracing::error!("MCP get_search_facets failed: {e}");
            Ok(crate::mcp::protocol::tool_error(format!(
                "Failed to compute facets: {e}"
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_definitions_lists_all_tools() {
        let definitions = tool_definitions();
        let names: Vec<&str> = definitions.as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert_eq!(
            names,
            vec!["search_code", "get_file", "list_repositories", "get_search_facets"]
        );

        // Every tool must declare an object input schema (required by MCP clients)
        for tool in definitions.as_array().unwrap() {
            assert_eq!(tool["inputSchema"]["type"], "object", "tool {} schema", tool["name"]);
            assert!(tool["description"].as_str().unwrap().len() > 20);
        }
    }

    #[test]
    fn test_join_filter() {
        assert_eq!(
            join_filter(&Some(vec!["a".to_string(), " b ".to_string()])),
            Some("a,b".to_string())
        );
        assert_eq!(join_filter(&Some(vec![])), None);
        assert_eq!(join_filter(&Some(vec!["  ".to_string()])), None);
        assert_eq!(join_filter(&None), None);
    }

    #[test]
    fn test_slice_lines_full_content() {
        let slice = slice_lines("a\nb\nc", None, None, 100);
        assert_eq!(slice.content, "a\nb\nc");
        assert_eq!((slice.start_line, slice.end_line, slice.total_lines), (1, 3, 3));
        assert!(!slice.truncated);
    }

    #[test]
    fn test_slice_lines_range() {
        let slice = slice_lines("a\nb\nc\nd", Some(2), Some(3), 100);
        assert_eq!(slice.content, "b\nc");
        assert_eq!((slice.start_line, slice.end_line), (2, 3));
    }

    #[test]
    fn test_slice_lines_truncation() {
        let content = (1..=10).map(|i| i.to_string()).collect::<Vec<_>>().join("\n");
        let slice = slice_lines(&content, None, None, 4);
        assert_eq!(slice.content, "1\n2\n3\n4");
        assert!(slice.truncated);
        assert_eq!(slice.end_line, 4);
        assert_eq!(slice.total_lines, 10);
    }

    #[test]
    fn test_slice_lines_out_of_range() {
        let slice = slice_lines("a\nb", Some(5), None, 100);
        assert_eq!(slice.content, "");
        assert!(!slice.truncated);
        assert_eq!(slice.total_lines, 2);
    }

    #[test]
    fn test_search_code_args_parse() {
        let args: SearchCodeArgs = parse_args(&serde_json::json!({
            "query": "fn main",
            "extensions": ["rs"],
            "regex": false,
            "limit": 5
        }))
        .expect("valid args");
        assert_eq!(args.query, "fn main");
        assert_eq!(args.extensions, Some(vec!["rs".to_string()]));
        assert_eq!(args.limit, Some(5));
        assert!(!args.case_sensitive);
    }

    #[test]
    fn test_get_file_args_parse_null_arguments() {
        let args: GetFileArgs = parse_args(&Value::Null).expect("null arguments are valid");
        assert!(args.doc_address.is_none());
        assert!(args.file_id.is_none());
    }
}
