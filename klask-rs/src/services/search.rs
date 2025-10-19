use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tantivy::collector::{Count, TopDocs};
use tantivy::directory::MmapDirectory;
use tantivy::query::{BooleanQuery, QueryParser, TermQuery};
use tantivy::schema::{Field, Schema, Value, FAST, STORED, STRING, TEXT};
use tantivy::snippet::SnippetGenerator;
use tantivy::{doc, Index, IndexReader, IndexWriter, Term};
use tokio::sync::RwLock;
use uuid::Uuid;

use tracing::{debug, warn};

#[derive(Debug, Clone)]
pub struct FileData<'a> {
    pub file_id: Uuid,
    pub file_name: &'a str,
    pub file_path: &'a str,
    pub content: &'a str,
    pub repository: &'a str, // Parent repository (for mass deletion and facets)
    pub project: &'a str,    // Individual project name (for GitLab/GitHub, same as repository for simple Git repos)
    pub version: &'a str,
    pub extension: &'a str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub file_id: Uuid,
    pub doc_address: String, // Format: "segment_ord:doc_id"
    pub file_name: String,
    pub file_path: String,
    pub content_snippet: String,
    pub repository: String,
    pub project: String,
    pub version: String,
    pub extension: String,
    pub score: f32,
    pub line_number: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultsWithTotal {
    pub results: Vec<SearchResult>,
    pub total: u64,
    pub facets: Option<SearchFacets>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFacets {
    pub repositories: Vec<(String, u64)>,
    pub projects: Vec<(String, u64)>,
    pub versions: Vec<(String, u64)>,
    pub extensions: Vec<(String, u64)>,
}

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub query: String,
    pub repository_filter: Option<String>,
    pub project_filter: Option<String>,
    pub version_filter: Option<String>,
    pub extension_filter: Option<String>,
    pub limit: usize,
    pub offset: usize,
    pub include_facets: bool,
}

#[derive(Clone)]
pub struct SearchService {
    index: Index,
    reader: IndexReader,
    writer: Arc<RwLock<IndexWriter>>,
    schema: Schema,
    fields: SearchFields,
    index_dir: std::path::PathBuf,
}

#[derive(Clone)]
struct SearchFields {
    file_id: Field,
    file_name: Field,
    file_path: Field,
    content: Field,
    repository: Field, // Parent repository name
    project: Field,    // Individual project name
    version: Field,
    extension: Field,
}

impl SearchService {
    pub fn new<P: AsRef<Path>>(index_dir: P) -> Result<Self> {
        let schema = Self::build_schema();
        let fields = Self::extract_fields(&schema);

        // Create directory if it doesn't exist
        std::fs::create_dir_all(&index_dir)?;

        // Use MmapDirectory with open_or_create - the elegant Tantivy way
        let mmap_directory = MmapDirectory::open(&index_dir)?;
        let index = Index::open_or_create(mmap_directory, schema.clone())?;

        let reader = index.reader()?;

        // Configure Tantivy IndexWriter with environment variables
        let memory_mb =
            std::env::var("KLASK_TANTIVY_MEMORY_MB").ok().and_then(|v| v.parse::<usize>().ok()).unwrap_or(200); // Default 200MB (4x more than previous 50MB)

        let memory_bytes = memory_mb * 1_000_000;

        let writer = if let Ok(num_threads_str) = std::env::var("KLASK_TANTIVY_NUM_THREADS") {
            if let Ok(num_threads) = num_threads_str.parse::<usize>() {
                tracing::info!(
                    "Creating Tantivy IndexWriter with {} threads and {}MB memory",
                    num_threads,
                    memory_mb
                );
                Arc::new(RwLock::new(index.writer_with_num_threads(num_threads, memory_bytes)?))
            } else {
                tracing::warn!("Invalid KLASK_TANTIVY_NUM_THREADS value: {}", num_threads_str);
                tracing::info!(
                    "Creating Tantivy IndexWriter with auto-threads and {}MB memory",
                    memory_mb
                );
                Arc::new(RwLock::new(index.writer(memory_bytes)?))
            }
        } else {
            // Use Tantivy's automatic thread detection (up to 8 threads)
            tracing::info!(
                "Creating Tantivy IndexWriter with auto-threads and {}MB memory",
                memory_mb
            );
            Arc::new(RwLock::new(index.writer(memory_bytes)?))
        };

        Ok(Self { index, reader, writer, schema, fields, index_dir: index_dir.as_ref().to_path_buf() })
    }

    fn build_schema() -> Schema {
        let mut schema_builder = Schema::builder();

        // File metadata fields
        schema_builder.add_text_field("file_id", TEXT | STORED | FAST);
        schema_builder.add_text_field("file_name", TEXT | STORED);
        schema_builder.add_text_field("file_path", TEXT | STORED);

        // Content field cargo clippy -- -D warningswith custom analyzer for code search
        schema_builder.add_text_field("content", TEXT | STORED);

        // Filter fields - use STRING for exact matching, not TEXT which tokenizes
        schema_builder.add_text_field("repository", STRING | STORED | FAST);
        schema_builder.add_text_field("project", STRING | STORED | FAST);
        schema_builder.add_text_field("version", STRING | STORED | FAST);
        schema_builder.add_text_field("extension", STRING | STORED | FAST);

        schema_builder.build()
    }

    fn extract_fields(schema: &Schema) -> SearchFields {
        SearchFields {
            file_id: schema.get_field("file_id").expect("file_id field should exist"),
            file_name: schema.get_field("file_name").expect("file_name field should exist"),
            file_path: schema.get_field("file_path").expect("file_path field should exist"),
            content: schema.get_field("content").expect("content field should exist"),
            repository: schema.get_field("repository").expect("repository field should exist"),
            project: schema.get_field("project").expect("project field should exist"),
            version: schema.get_field("version").expect("version field should exist"),
            extension: schema.get_field("extension").expect("extension field should exist"),
        }
    }

    #[allow(dead_code)]
    pub async fn index_file(&self, file_data: FileData<'_>) -> Result<()> {
        let writer = self.writer.write().await;

        let doc = doc!(
            self.fields.file_id => file_data.file_id.to_string(),
            self.fields.file_name => file_data.file_name,
            self.fields.file_path => file_data.file_path,
            self.fields.content => file_data.content,
            self.fields.repository => file_data.repository,
            self.fields.project => file_data.project,
            self.fields.version => file_data.version,
            self.fields.extension => file_data.extension,
        );

        writer.add_document(doc)?;
        Ok(())
    }

    /// Upsert a file - delete existing and add new version if it exists, otherwise just add
    pub async fn upsert_file(&self, file_data: FileData<'_>) -> Result<()> {
        let writer = self.writer.write().await;

        // Delete ALL existing documents with the same file_id to ensure no duplicates
        let file_id_str = file_data.file_id.to_string();
        let term = tantivy::Term::from_field_text(self.fields.file_id, &file_id_str);

        // Use a query to delete all matching documents
        let query = TermQuery::new(term.clone(), tantivy::schema::IndexRecordOption::Basic);
        let _ = writer.delete_query(Box::new(query));

        debug!(
            "Indexing file '{}' with file_id='{}', repository='{}', project='{}'",
            file_data.file_path, file_id_str, file_data.repository, file_data.project
        );

        // Add the new document
        let doc = doc!(
            self.fields.file_id => file_id_str,
            self.fields.file_name => file_data.file_name,
            self.fields.file_path => file_data.file_path,
            self.fields.content => file_data.content,
            self.fields.repository => file_data.repository,
            self.fields.project => file_data.project,
            self.fields.version => file_data.version,
            self.fields.extension => file_data.extension,
        );

        writer.add_document(doc)?;
        Ok(())
    }

    /// Check if a document exists by file_id (lightweight check)
    #[allow(dead_code)]
    pub async fn document_exists(&self, file_id: Uuid) -> Result<bool> {
        // Use the existing get_file_by_id method but only check if result is Some
        let result = self.get_file_by_id(file_id).await?;
        Ok(result.is_some())
    }

    pub async fn commit(&self) -> Result<()> {
        let mut writer = self.writer.write().await;
        writer.commit()?;
        // Reload reader to ensure latest changes are visible
        self.reader.reload()?;
        Ok(())
    }

    /// Delete all documents for a specific repository (parent repository)
    pub async fn delete_project_documents(&self, repository: &str) -> Result<u64> {
        debug!("delete_project_documents called with repository='{}'", repository);
        let mut writer = self.writer.write().await;

        // Create a query to match all documents with this repository
        let term = tantivy::Term::from_field_text(self.fields.repository, repository);
        let query = TermQuery::new(term, tantivy::schema::IndexRecordOption::Basic);

        // Get count before deletion for logging
        let searcher = self.reader.searcher();
        let count_before = searcher.search(&query, &Count)? as u64;
        debug!(
            "Found {} documents to delete for repository='{}'",
            count_before, repository
        );

        // Delete all matching documents
        let _ = writer.delete_query(Box::new(query));
        writer.commit()?;
        debug!(
            "Committed deletion of {} documents for repository='{}'",
            count_before, repository
        );

        // Reload reader to see changes
        self.reader.reload()?;

        // Verify deletion worked by checking count after
        let term_verify = tantivy::Term::from_field_text(self.fields.repository, repository);
        let query_verify = TermQuery::new(term_verify, tantivy::schema::IndexRecordOption::Basic);
        let searcher_after = self.reader.searcher();
        let count_after = searcher_after.search(&query_verify, &Count)? as u64;
        if count_after > 0 {
            warn!("After deletion and reload, still found {} documents for repository='{}' - this suggests Tantivy deletion might not be working as expected", count_after, repository);
        } else {
            debug!(
                "Verified: 0 documents remain for repository='{}' after deletion",
                repository
            );
        }

        Ok(count_before)
    }

    /// Update project name for all documents (used when repository is renamed)
    pub async fn update_project_name(&self, old_project: &str, new_project: &str) -> Result<u64> {
        let searcher = self.reader.searcher();
        let mut writer = self.writer.write().await;

        // Find all documents with the old project name
        let term = tantivy::Term::from_field_text(self.fields.project, old_project);
        let query = TermQuery::new(term, tantivy::schema::IndexRecordOption::Basic);

        // Get all matching documents
        let top_docs = searcher.search(&query, &TopDocs::with_limit(10000))?;
        let updated_count = top_docs.len() as u64;

        // For each document, create a new version with updated project name
        for (_score, doc_address) in top_docs {
            if let Ok(doc) = searcher.doc::<tantivy::TantivyDocument>(doc_address) {
                // Extract current document fields
                let file_id = doc.get_first(self.fields.file_id).and_then(|v| v.as_str()).unwrap_or_default();
                let file_name = doc.get_first(self.fields.file_name).and_then(|v| v.as_str()).unwrap_or_default();
                let file_path = doc.get_first(self.fields.file_path).and_then(|v| v.as_str()).unwrap_or_default();
                let content = doc.get_first(self.fields.content).and_then(|v| v.as_str()).unwrap_or_default();
                let version = doc.get_first(self.fields.version).and_then(|v| v.as_str()).unwrap_or_default();
                let extension = doc.get_first(self.fields.extension).and_then(|v| v.as_str()).unwrap_or_default();

                // Extract repository or use new_project as default
                let repository = doc.get_first(self.fields.repository).and_then(|v| v.as_str()).unwrap_or(new_project);

                // Create new document with updated project name
                let new_doc = doc!(
                    self.fields.file_id => file_id,
                    self.fields.file_name => file_name,
                    self.fields.file_path => file_path,
                    self.fields.content => content,
                    self.fields.repository => repository,
                    self.fields.project => new_project,
                    self.fields.version => version,
                    self.fields.extension => extension,
                );

                writer.add_document(new_doc)?;
            }
        }

        // Delete all documents with the old project name
        let delete_term = tantivy::Term::from_field_text(self.fields.project, old_project);
        let delete_query = TermQuery::new(delete_term, tantivy::schema::IndexRecordOption::Basic);
        let _ = writer.delete_query(Box::new(delete_query));

        writer.commit()?;
        self.reader.reload()?;

        Ok(updated_count)
    }

    /// Reset the entire search index (delete all documents)
    pub async fn reset_index(&self) -> Result<()> {
        // Delete the index directory and recreate it
        if self.index_dir.exists() {
            std::fs::remove_dir_all(&self.index_dir)?;
        }
        std::fs::create_dir_all(&self.index_dir)?;

        // Recreate the index
        let _new_index = Index::create_in_dir(&self.index_dir, self.schema.clone())?;

        // Note: We can't replace self.index directly since it's not mutable
        // Instead, we'll delete all documents from the existing index
        let mut writer = self.writer.write().await;
        writer.delete_all_documents()?;
        writer.commit()?;

        // Reload the reader to see the changes
        self.reader.reload()?;

        Ok(())
    }

    pub async fn search(&self, search_query: SearchQuery) -> Result<SearchResultsWithTotal> {
        let searcher = self.reader.searcher();

        // Build query parser for content search
        let query_parser = QueryParser::for_index(
            &self.index,
            vec![self.fields.content, self.fields.file_name, self.fields.file_path],
        );

        // Parse the main query
        let base_query = query_parser
            .parse_query(&search_query.query)
            .map_err(|e| anyhow!("Failed to parse query '{}': {}", search_query.query, e))?;

        // Create snippet generator once for the entire search
        let snippet_generator = if search_query.limit > 0 {
            Some(self.create_snippet_generator(&searcher, &*base_query)?)
        } else {
            None
        };

        // Build filter queries if filters are provided
        let mut filter_queries = Vec::new();

        // Handle repository filters (supports comma-separated multi-select)
        if let Some(repository_filter) = &search_query.repository_filter {
            let repository_values: Vec<&str> = repository_filter.split(',').map(|s| s.trim()).collect();
            if repository_values.len() == 1 {
                // Single filter - use TermQuery
                let term = Term::from_field_text(self.fields.repository, repository_values[0]);
                filter_queries.push(
                    Box::new(TermQuery::new(term, tantivy::schema::IndexRecordOption::Basic))
                        as Box<dyn tantivy::query::Query>,
                );
            } else {
                // Multiple filters - use OR BooleanQuery
                let mut repository_clauses = Vec::new();
                for repository_value in repository_values {
                    let term = Term::from_field_text(self.fields.repository, repository_value);
                    let term_query = Box::new(TermQuery::new(term, tantivy::schema::IndexRecordOption::Basic))
                        as Box<dyn tantivy::query::Query>;
                    repository_clauses.push((tantivy::query::Occur::Should, term_query));
                }
                filter_queries.push(Box::new(BooleanQuery::new(repository_clauses)) as Box<dyn tantivy::query::Query>);
            }
        }

        // Handle project filters (supports comma-separated multi-select)
        if let Some(project_filter) = &search_query.project_filter {
            let project_values: Vec<&str> = project_filter.split(',').map(|s| s.trim()).collect();
            if project_values.len() == 1 {
                // Single filter - use TermQuery
                let term = Term::from_field_text(self.fields.project, project_values[0]);
                filter_queries.push(
                    Box::new(TermQuery::new(term, tantivy::schema::IndexRecordOption::Basic))
                        as Box<dyn tantivy::query::Query>,
                );
            } else {
                // Multiple filters - use OR BooleanQuery
                let mut project_clauses = Vec::new();
                for project_value in project_values {
                    let term = Term::from_field_text(self.fields.project, project_value);
                    let term_query = Box::new(TermQuery::new(term, tantivy::schema::IndexRecordOption::Basic))
                        as Box<dyn tantivy::query::Query>;
                    project_clauses.push((tantivy::query::Occur::Should, term_query));
                }
                filter_queries.push(Box::new(BooleanQuery::new(project_clauses)) as Box<dyn tantivy::query::Query>);
            }
        }

        // Handle version filters (supports comma-separated multi-select)
        if let Some(version_filter) = &search_query.version_filter {
            let version_values: Vec<&str> = version_filter.split(',').map(|s| s.trim()).collect();
            if version_values.len() == 1 {
                // Single filter - use TermQuery
                let term = Term::from_field_text(self.fields.version, version_values[0]);
                filter_queries.push(
                    Box::new(TermQuery::new(term, tantivy::schema::IndexRecordOption::Basic))
                        as Box<dyn tantivy::query::Query>,
                );
            } else {
                // Multiple filters - use OR BooleanQuery
                let mut version_clauses = Vec::new();
                for version_value in version_values {
                    let term = Term::from_field_text(self.fields.version, version_value);
                    let term_query = Box::new(TermQuery::new(term, tantivy::schema::IndexRecordOption::Basic))
                        as Box<dyn tantivy::query::Query>;
                    version_clauses.push((tantivy::query::Occur::Should, term_query));
                }
                filter_queries.push(Box::new(BooleanQuery::new(version_clauses)) as Box<dyn tantivy::query::Query>);
            }
        }

        // Handle extension filters (supports comma-separated multi-select)
        if let Some(extension_filter) = &search_query.extension_filter {
            let extension_values: Vec<&str> = extension_filter.split(',').map(|s| s.trim()).collect();
            if extension_values.len() == 1 {
                // Single filter - use TermQuery
                let term = Term::from_field_text(self.fields.extension, extension_values[0]);
                filter_queries.push(
                    Box::new(TermQuery::new(term, tantivy::schema::IndexRecordOption::Basic))
                        as Box<dyn tantivy::query::Query>,
                );
            } else {
                // Multiple filters - use OR BooleanQuery
                let mut extension_clauses = Vec::new();
                for extension_value in extension_values {
                    let term = Term::from_field_text(self.fields.extension, extension_value);
                    let term_query = Box::new(TermQuery::new(term, tantivy::schema::IndexRecordOption::Basic))
                        as Box<dyn tantivy::query::Query>;
                    extension_clauses.push((tantivy::query::Occur::Should, term_query));
                }
                filter_queries.push(Box::new(BooleanQuery::new(extension_clauses)) as Box<dyn tantivy::query::Query>);
            }
        }

        // Combine base query with filters using BooleanQuery if we have filters
        let final_query: Box<dyn tantivy::query::Query> = if !filter_queries.is_empty() {
            let mut clauses = vec![(tantivy::query::Occur::Must, base_query)];
            for filter in filter_queries {
                clauses.push((tantivy::query::Occur::Must, filter));
            }
            Box::new(BooleanQuery::new(clauses))
        } else {
            base_query
        };

        // For performance with large indices, use Count collector for total
        let total = searcher.search(&final_query, &Count)? as u64;

        // Ensure limit is at least 1 to avoid Tantivy panic
        let effective_limit = if search_query.limit == 0 { 1 } else { search_query.limit };

        // Execute search with pagination
        let top_docs = searcher.search(
            &final_query,
            &TopDocs::with_limit(effective_limit).and_offset(search_query.offset),
        )?;

        let mut results = Vec::new();

        // Only process results if limit > 0 (for facets-only searches, we don't need results)
        if search_query.limit > 0 {
            for (score, doc_address) in top_docs {
                let retrieved_doc = searcher.doc::<tantivy::TantivyDocument>(doc_address)?;

                let file_id_str = retrieved_doc
                    .get_first(self.fields.file_id)
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing file_id in search result"))?;

                let file_id = Uuid::parse_str(file_id_str)
                    .map_err(|_| anyhow!("Invalid UUID format in file_id: {}", file_id_str))?;

                let file_name =
                    retrieved_doc.get_first(self.fields.file_name).and_then(|v| v.as_str()).unwrap_or("").to_string();

                let file_path =
                    retrieved_doc.get_first(self.fields.file_path).and_then(|v| v.as_str()).unwrap_or("").to_string();

                let repository =
                    retrieved_doc.get_first(self.fields.repository).and_then(|v| v.as_str()).unwrap_or("").to_string();

                let project =
                    retrieved_doc.get_first(self.fields.project).and_then(|v| v.as_str()).unwrap_or("").to_string();

                let version =
                    retrieved_doc.get_first(self.fields.version).and_then(|v| v.as_str()).unwrap_or("").to_string();

                let extension =
                    retrieved_doc.get_first(self.fields.extension).and_then(|v| v.as_str()).unwrap_or("").to_string();

                // No need for post-query filtering anymore since we're using query-time filters

                // Generate content snippet and extract line number
                let (content_snippet, line_number) = if let Some(ref generator) = snippet_generator {
                    self.generate_optimized_snippet(generator, &search_query.query, &retrieved_doc)?
                } else {
                    ("".to_string(), None)
                };

                // Format DocAddress as "segment_ord:doc_id"
                let doc_address_str = format!("{}:{}", doc_address.segment_ord, doc_address.doc_id);

                results.push(SearchResult {
                    file_id,
                    doc_address: doc_address_str,
                    file_name,
                    file_path,
                    content_snippet,
                    repository,
                    project,
                    version,
                    extension,
                    score,
                    line_number,
                });
            }
        }

        // Collect facets - calculate from search results when requested
        let facets = if search_query.include_facets {
            Some(self.collect_facets_from_search_results(&searcher, &final_query, &search_query).await?)
        } else {
            None
        };

        Ok(SearchResultsWithTotal { results, total, facets })
    }

    fn create_snippet_generator(
        &self,
        searcher: &tantivy::Searcher,
        query: &dyn tantivy::query::Query,
    ) -> Result<SnippetGenerator> {
        let mut snippet_generator = SnippetGenerator::create(searcher, query, self.fields.content)?;
        snippet_generator.set_max_num_chars(600);
        Ok(snippet_generator)
    }

    fn generate_optimized_snippet(
        &self,
        generator: &SnippetGenerator,
        query: &str,
        doc: &tantivy::TantivyDocument,
    ) -> Result<(String, Option<u32>)> {
        // Generate the snippet with HTML highlighting - this is now much faster
        let snippet = generator.snippet_from_doc(doc);
        let mut highlighted_html = snippet.to_html();

        // Adjust snippet to only include complete lines (start after first newline, end at last newline)
        if let Some(first_newline_pos) = highlighted_html.find('\n') {
            // Find the last newline in the snippet
            if let Some(last_newline_pos) = highlighted_html.rfind('\n') {
                if last_newline_pos > first_newline_pos {
                    // Trim to only include complete lines (after first \n, up to last \n)
                    highlighted_html = highlighted_html[first_newline_pos + 1..last_newline_pos].to_string();
                }
            }
        }

        // For line number, use a simple approach to avoid scanning the entire content
        let line_number = if let Some(content) = doc.get_first(self.fields.content).and_then(|v| v.as_str()) {
            // Only search for the first term to avoid performance issues
            if let Some(first_term) = query.split_whitespace().next() {
                content.to_lowercase().find(&first_term.to_lowercase()).and_then(|pos| {
                    // Ensure we don't slice in the middle of a UTF-8 character
                    if content.is_char_boundary(pos) {
                        Some(content[..pos].chars().filter(|&c| c == '\n').count() as u32 + 1)
                    } else {
                        // If pos is not a char boundary, find the nearest valid boundary
                        let valid_pos = (0..=pos).rev().find(|&p| content.is_char_boundary(p))?;
                        Some(content[..valid_pos].chars().filter(|&c| c == '\n').count() as u32 + 1)
                    }
                })
            } else {
                None
            }
        } else {
            None
        };

        Ok((highlighted_html, line_number))
    }

    #[allow(dead_code)]
    pub async fn delete_file(&self, file_id: Uuid) -> Result<()> {
        let writer = self.writer.write().await;
        let term = tantivy::Term::from_field_text(self.fields.file_id, &file_id.to_string());
        writer.delete_term(term);
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn clear_index(&self) -> Result<()> {
        let mut writer = self.writer.write().await;
        writer.delete_all_documents()?;
        writer.commit()?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_stats(&self) -> Result<SearchStats> {
        let searcher = self.reader.searcher();
        let num_docs = searcher.num_docs();

        Ok(SearchStats {
            total_documents: num_docs,
            index_size_bytes: 0, // TODO: Calculate actual index size
        })
    }

    pub async fn get_file_by_doc_address(&self, doc_address_str: &str) -> Result<Option<SearchResult>> {
        // Parse "segment_ord:doc_id" format
        let parts: Vec<&str> = doc_address_str.split(':').collect();
        if parts.len() != 2 {
            return Err(anyhow!("Invalid doc_address format, expected 'segment_ord:doc_id'"));
        }

        let segment_ord: u32 =
            parts[0].parse().map_err(|_| anyhow!("Invalid segment_ord in doc_address: {}", parts[0]))?;
        let doc_id: u32 = parts[1].parse().map_err(|_| anyhow!("Invalid doc_id in doc_address: {}", parts[1]))?;

        let doc_address = tantivy::DocAddress::new(segment_ord, doc_id);
        let searcher = self.reader.searcher();

        // Try to get the document directly using DocAddress
        match searcher.doc::<tantivy::TantivyDocument>(doc_address) {
            Ok(retrieved_doc) => {
                let file_id_str = retrieved_doc
                    .get_first(self.fields.file_id)
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing file_id in document"))?;

                let file_id = Uuid::parse_str(file_id_str)
                    .map_err(|_| anyhow!("Invalid UUID format in file_id: {}", file_id_str))?;

                let file_name =
                    retrieved_doc.get_first(self.fields.file_name).and_then(|v| v.as_str()).unwrap_or("").to_string();

                let file_path =
                    retrieved_doc.get_first(self.fields.file_path).and_then(|v| v.as_str()).unwrap_or("").to_string();

                let content =
                    retrieved_doc.get_first(self.fields.content).and_then(|v| v.as_str()).unwrap_or("").to_string();

                let repository =
                    retrieved_doc.get_first(self.fields.repository).and_then(|v| v.as_str()).unwrap_or("").to_string();

                let project =
                    retrieved_doc.get_first(self.fields.project).and_then(|v| v.as_str()).unwrap_or("").to_string();

                let version =
                    retrieved_doc.get_first(self.fields.version).and_then(|v| v.as_str()).unwrap_or("").to_string();

                let extension =
                    retrieved_doc.get_first(self.fields.extension).and_then(|v| v.as_str()).unwrap_or("").to_string();

                Ok(Some(SearchResult {
                    file_id,
                    doc_address: doc_address_str.to_string(),
                    file_name,
                    file_path,
                    content_snippet: content,
                    repository,
                    project,
                    version,
                    extension,
                    score: 1.0,
                    line_number: None,
                }))
            }
            Err(_) => {
                // Document not found at this address
                Ok(None)
            }
        }
    }

    pub async fn get_file_by_id(&self, file_id: Uuid) -> Result<Option<SearchResult>> {
        let searcher = self.reader.searcher();
        debug!("Getting file by id: {}", file_id);

        // Use a targeted query to find the document with the matching file_id
        let file_id_str = file_id.to_string();
        let term = tantivy::Term::from_field_text(self.fields.file_id, &file_id_str);
        let query = TermQuery::new(term, tantivy::schema::IndexRecordOption::Basic);

        // Search for the specific document
        let top_docs = searcher.search(&query, &TopDocs::with_limit(1))?;

        if let Some((score, doc_address)) = top_docs.first() {
            let retrieved_doc = searcher.doc::<tantivy::TantivyDocument>(*doc_address)?;

            debug!("Found matching document for file_id: {}", file_id);

            let file_name =
                retrieved_doc.get_first(self.fields.file_name).and_then(|v| v.as_str()).unwrap_or("").to_string();

            let file_path =
                retrieved_doc.get_first(self.fields.file_path).and_then(|v| v.as_str()).unwrap_or("").to_string();

            let content =
                retrieved_doc.get_first(self.fields.content).and_then(|v| v.as_str()).unwrap_or("").to_string();

            let repository =
                retrieved_doc.get_first(self.fields.repository).and_then(|v| v.as_str()).unwrap_or("").to_string();

            let project =
                retrieved_doc.get_first(self.fields.project).and_then(|v| v.as_str()).unwrap_or("").to_string();

            let version =
                retrieved_doc.get_first(self.fields.version).and_then(|v| v.as_str()).unwrap_or("").to_string();

            let extension =
                retrieved_doc.get_first(self.fields.extension).and_then(|v| v.as_str()).unwrap_or("").to_string();

            // Format DocAddress as "segment_ord:doc_id"
            let doc_address_str = format!("{}:{}", doc_address.segment_ord, doc_address.doc_id);

            return Ok(Some(SearchResult {
                file_id,
                doc_address: doc_address_str,
                file_name,
                file_path,
                content_snippet: content, // Return full content instead of snippet
                repository,
                project,
                version,
                extension,
                score: *score,
                line_number: None,
            }));
        }

        // If no matching document found
        debug!("No document found with file_id: {}", file_id);
        Ok(None)
    }

    pub fn get_document_count(&self) -> Result<u64> {
        // Reload the reader to see the latest changes
        self.reader.reload()?;
        let searcher = self.reader.searcher();
        Ok(searcher.num_docs())
    }

    /// Calculate the total size of a directory recursively in bytes
    fn calculate_directory_size(dir_path: &Path) -> Result<u64> {
        let mut total_size = 0u64;

        if !dir_path.exists() {
            debug!("Directory does not exist: {:?}", dir_path);
            return Ok(0);
        }

        if !dir_path.is_dir() {
            debug!("Path is not a directory: {:?}", dir_path);
            return Ok(0);
        }

        let read_dir =
            std::fs::read_dir(dir_path).map_err(|e| anyhow!("Failed to read directory {:?}: {}", dir_path, e))?;

        for entry_result in read_dir {
            let entry = entry_result.map_err(|e| anyhow!("Failed to read directory entry in {:?}: {}", dir_path, e))?;

            let path = entry.path();
            let metadata = entry.metadata().map_err(|e| anyhow!("Failed to get metadata for {:?}: {}", path, e))?;

            if metadata.is_file() {
                total_size = total_size.saturating_add(metadata.len());
            } else if metadata.is_dir() {
                // Recursively calculate subdirectory size
                let subdir_size = Self::calculate_directory_size(&path)?;
                total_size = total_size.saturating_add(subdir_size);
            }
        }

        Ok(total_size)
    }

    /// Get the search index size in megabytes
    pub fn get_index_size_mb(&self) -> f64 {
        match Self::calculate_directory_size(&self.index_dir) {
            Ok(size_bytes) => {
                let size_mb = size_bytes as f64 / 1_048_576.0; // Convert bytes to MB
                debug!("Index directory size: {} bytes ({:.2} MB)", size_bytes, size_mb);
                size_mb
            }
            Err(e) => {
                warn!("Failed to calculate index size for {:?}: {}", self.index_dir, e);
                0.0
            }
        }
    }

    /// Collect facets from the entire search index for filtering
    #[allow(dead_code)]
    async fn collect_facets_from_index(&self, searcher: &tantivy::Searcher) -> Result<SearchFacets> {
        // Use a match-all query to get facets from the entire index
        let match_all_query = tantivy::query::AllQuery;

        // Get all documents for facet calculation (up to a reasonable limit)
        const FACET_CALCULATION_LIMIT: usize = 50000;
        let top_docs = searcher.search(&match_all_query, &TopDocs::with_limit(FACET_CALCULATION_LIMIT))?;

        let mut repository_counts: HashMap<String, u64> = HashMap::new();
        let mut project_counts: HashMap<String, u64> = HashMap::new();
        let mut version_counts: HashMap<String, u64> = HashMap::new();
        let mut extension_counts: HashMap<String, u64> = HashMap::new();

        // Count facets from all documents in the index
        for (_score, doc_address) in top_docs {
            let doc = searcher.doc::<tantivy::TantivyDocument>(doc_address)?;

            // Count repository facets
            if let Some(repository) = doc.get_first(self.fields.repository).and_then(|v| v.as_str()) {
                if !repository.is_empty() {
                    *repository_counts.entry(repository.to_string()).or_insert(0) += 1;
                }
            }

            // Count project facets
            if let Some(project) = doc.get_first(self.fields.project).and_then(|v| v.as_str()) {
                if !project.is_empty() {
                    *project_counts.entry(project.to_string()).or_insert(0) += 1;
                }
            }

            // Count version facets (don't skip empty ones, use "main" as default)
            if let Some(version) = doc.get_first(self.fields.version).and_then(|v| v.as_str()) {
                let version_str = if version.is_empty() { "main" } else { version };
                *version_counts.entry(version_str.to_string()).or_insert(0) += 1;
            } else {
                // If no version field exists, use "main" as default
                *version_counts.entry("main".to_string()).or_insert(0) += 1;
            }

            // Count extension facets
            if let Some(extension) = doc.get_first(self.fields.extension).and_then(|v| v.as_str()) {
                if !extension.is_empty() {
                    *extension_counts.entry(extension.to_string()).or_insert(0) += 1;
                }
            }
        }

        // Sort and limit to top 50 each for UI performance
        let mut repositories: Vec<(String, u64)> = repository_counts.into_iter().collect();
        repositories.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        repositories.truncate(50);

        let mut projects: Vec<(String, u64)> = project_counts.into_iter().collect();
        projects.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        projects.truncate(50);

        let mut versions: Vec<(String, u64)> = version_counts.into_iter().collect();
        versions.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        versions.truncate(50);

        let mut extensions: Vec<(String, u64)> = extension_counts.into_iter().collect();
        extensions.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        extensions.truncate(50);

        Ok(SearchFacets { repositories, projects, versions, extensions })
    }

    /// Collect facets using Tantivy native aggregations API
    /// ULTRA-OPTIMIZED VERSION: Using terms aggregations for <100ms performance
    async fn collect_facets_from_search_results(
        &self,
        searcher: &tantivy::Searcher,
        _query: &dyn tantivy::query::Query,
        search_query: &SearchQuery,
    ) -> Result<SearchFacets> {
        use tantivy::aggregation::agg_req::Aggregations;
        use tantivy::aggregation::agg_result::AggregationResults;
        use tantivy::aggregation::AggregationCollector;
        use tantivy::query::{AllQuery, BooleanQuery, Occur, QueryParser, TermQuery};

        // Helper to build query with specific filters
        let build_query_with_filters = |include_repository: bool,
                                        include_project: bool,
                                        include_version: bool,
                                        include_extension: bool|
         -> Result<Box<dyn tantivy::query::Query>> {
            let mut clauses = vec![];

            // Always include text query
            let text_query: Box<dyn tantivy::query::Query> =
                if search_query.query.trim().is_empty() || search_query.query == "*" {
                    Box::new(AllQuery)
                } else {
                    let query_parser = QueryParser::for_index(
                        searcher.index(),
                        vec![self.fields.content, self.fields.file_name, self.fields.file_path],
                    );
                    match query_parser.parse_query(&search_query.query) {
                        Ok(parsed) => parsed,
                        Err(_) => Box::new(AllQuery),
                    }
                };
            clauses.push((Occur::Must, text_query));

            // Add repository filter if requested
            if include_repository {
                if let Some(ref repository_filter) = search_query.repository_filter {
                    let repositories: Vec<&str> =
                        repository_filter.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
                    if !repositories.is_empty() {
                        let mut repository_clauses = vec![];
                        for repository in repositories {
                            let term = tantivy::Term::from_field_text(self.fields.repository, repository);
                            repository_clauses.push((
                                Occur::Should,
                                Box::new(TermQuery::new(term, tantivy::schema::IndexRecordOption::Basic))
                                    as Box<dyn tantivy::query::Query>,
                            ));
                        }
                        clauses.push((
                            Occur::Must,
                            Box::new(BooleanQuery::from(repository_clauses)) as Box<dyn tantivy::query::Query>,
                        ));
                    }
                }
            }

            // Add project filter if requested
            if include_project {
                if let Some(ref project_filter) = search_query.project_filter {
                    let projects: Vec<&str> =
                        project_filter.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
                    if !projects.is_empty() {
                        let mut project_clauses = vec![];
                        for project in projects {
                            let term = tantivy::Term::from_field_text(self.fields.project, project);
                            project_clauses.push((
                                Occur::Should,
                                Box::new(TermQuery::new(term, tantivy::schema::IndexRecordOption::Basic))
                                    as Box<dyn tantivy::query::Query>,
                            ));
                        }
                        clauses.push((
                            Occur::Must,
                            Box::new(BooleanQuery::from(project_clauses)) as Box<dyn tantivy::query::Query>,
                        ));
                    }
                }
            }

            // Add version filter if requested
            if include_version {
                if let Some(ref version_filter) = search_query.version_filter {
                    let versions: Vec<&str> =
                        version_filter.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
                    if !versions.is_empty() {
                        let mut version_clauses = vec![];
                        for version in versions {
                            let term = tantivy::Term::from_field_text(self.fields.version, version);
                            version_clauses.push((
                                Occur::Should,
                                Box::new(TermQuery::new(term, tantivy::schema::IndexRecordOption::Basic))
                                    as Box<dyn tantivy::query::Query>,
                            ));
                        }
                        clauses.push((
                            Occur::Must,
                            Box::new(BooleanQuery::from(version_clauses)) as Box<dyn tantivy::query::Query>,
                        ));
                    }
                }
            }

            // Add extension filter if requested
            if include_extension {
                if let Some(ref extension_filter) = search_query.extension_filter {
                    let extensions: Vec<&str> =
                        extension_filter.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
                    if !extensions.is_empty() {
                        let mut extension_clauses = vec![];
                        for extension in extensions {
                            let term = tantivy::Term::from_field_text(self.fields.extension, extension);
                            extension_clauses.push((
                                Occur::Should,
                                Box::new(TermQuery::new(term, tantivy::schema::IndexRecordOption::Basic))
                                    as Box<dyn tantivy::query::Query>,
                            ));
                        }
                        clauses.push((
                            Occur::Must,
                            Box::new(BooleanQuery::from(extension_clauses)) as Box<dyn tantivy::query::Query>,
                        ));
                    }
                }
            }

            if clauses.len() == 1 {
                Ok(clauses.into_iter().next().unwrap().1)
            } else {
                Ok(Box::new(BooleanQuery::from(clauses)))
            }
        };

        // For faceted search, we need to calculate each facet with the appropriate filters:
        // - Repository facets: apply project, version & extension filters (but not repository filter)
        // - Project facets: apply repository, version & extension filters (but not project filter)
        // - Version facets: apply repository, project & extension filters (but not version filter)
        // - Extension facets: apply repository, project & version filters (but not extension filter)

        // Calculate repository facets (with project, version & extension filters)
        let repository_facets = {
            let query = build_query_with_filters(false, true, true, true)?;

            // Build aggregation request using JSON
            let agg_req: Aggregations = serde_json::from_value(serde_json::json!({
                "repository_terms": {
                    "terms": {
                        "field": "repository",
                        "size": 1000
                    }
                }
            }))?;

            let collector = AggregationCollector::from_aggs(agg_req, Default::default());
            let agg_res: AggregationResults = searcher.search(&*query, &collector)?;

            // Extract results
            let mut facets = Vec::new();
            if let Some(tantivy::aggregation::agg_result::AggregationResult::BucketResult(
                tantivy::aggregation::agg_result::BucketResult::Terms { buckets, .. },
            )) = agg_res.0.get("repository_terms")
            {
                for entry in buckets {
                    if let tantivy::aggregation::Key::Str(term) = &entry.key {
                        facets.push((term.to_string(), entry.doc_count));
                    }
                }
            }
            facets
        };

        // Calculate project facets (with repository, version & extension filters)
        let project_facets = {
            let query = build_query_with_filters(true, false, true, true)?;

            // Build aggregation request using JSON
            let agg_req: Aggregations = serde_json::from_value(serde_json::json!({
                "project_terms": {
                    "terms": {
                        "field": "project",
                        "size": 10000
                    }
                }
            }))?;

            let collector = AggregationCollector::from_aggs(agg_req, Default::default());
            let agg_res: AggregationResults = searcher.search(&*query, &collector)?;

            // Extract results
            let mut facets = Vec::new();
            if let Some(tantivy::aggregation::agg_result::AggregationResult::BucketResult(
                tantivy::aggregation::agg_result::BucketResult::Terms { buckets, .. },
            )) = agg_res.0.get("project_terms")
            {
                for entry in buckets {
                    if let tantivy::aggregation::Key::Str(term) = &entry.key {
                        facets.push((term.to_string(), entry.doc_count));
                    }
                }
            }
            facets
        };

        // Calculate version facets (with repository, project & extension filters)
        let version_facets = {
            let query = build_query_with_filters(true, true, false, true)?;

            let agg_req: Aggregations = serde_json::from_value(serde_json::json!({
                "version_terms": {
                    "terms": {
                        "field": "version",
                        "size": 10000
                    }
                }
            }))?;

            let collector = AggregationCollector::from_aggs(agg_req, Default::default());
            let agg_res: AggregationResults = searcher.search(&*query, &collector)?;

            let mut facets = Vec::new();
            if let Some(tantivy::aggregation::agg_result::AggregationResult::BucketResult(
                tantivy::aggregation::agg_result::BucketResult::Terms { buckets, .. },
            )) = agg_res.0.get("version_terms")
            {
                for entry in buckets {
                    if let tantivy::aggregation::Key::Str(term) = &entry.key {
                        facets.push((term.to_string(), entry.doc_count));
                    }
                }
            }
            facets
        };

        // Calculate extension facets (with repository, project & version filters)
        let extension_facets = {
            let query = build_query_with_filters(true, true, true, false)?;

            let agg_req: Aggregations = serde_json::from_value(serde_json::json!({
                "extension_terms": {
                    "terms": {
                        "field": "extension",
                        "size": 10000
                    }
                }
            }))?;

            let collector = AggregationCollector::from_aggs(agg_req, Default::default());
            let agg_res: AggregationResults = searcher.search(&*query, &collector)?;

            let mut facets = Vec::new();
            if let Some(tantivy::aggregation::agg_result::AggregationResult::BucketResult(
                tantivy::aggregation::agg_result::BucketResult::Terms { buckets, .. },
            )) = agg_res.0.get("extension_terms")
            {
                for entry in buckets {
                    if let tantivy::aggregation::Key::Str(term) = &entry.key {
                        facets.push((term.to_string(), entry.doc_count));
                    }
                }
            }
            facets
        };

        Ok(SearchFacets {
            repositories: repository_facets,
            projects: project_facets,
            versions: version_facets,
            extensions: extension_facets,
        })
    }

    /// Legacy method for backward compatibility with tests - maps to index_file
    #[allow(dead_code)]
    #[allow(clippy::too_many_arguments)]
    pub fn add_document(
        &self,
        file_id: &str,
        content: &str,
        file_name: &str,
        extension: &str,
        _size: i64, // Not used in current implementation
        project: &str,
        version: &str,
    ) -> Result<()> {
        let uuid = if file_id.len() == 36 && file_id.contains('-') {
            Uuid::parse_str(file_id)?
        } else {
            // Generate a deterministic UUID from the file_id string using a hash
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            let mut hasher = DefaultHasher::new();
            file_id.hash(&mut hasher);
            let hash = hasher.finish();

            // Create a UUID from the hash (not cryptographically secure but deterministic for tests)
            let bytes = hash.to_be_bytes();
            let mut uuid_bytes = [0u8; 16];
            uuid_bytes[0..8].copy_from_slice(&bytes);
            uuid_bytes[8..16].copy_from_slice(&bytes); // Repeat the hash to fill 16 bytes

            Uuid::from_bytes(uuid_bytes)
        };

        let file_data = FileData {
            file_id: uuid,
            file_name,
            file_path: file_name, // Use file_name as path if not provided separately
            content,
            repository: project, // Use project as repository for backward compatibility
            project,
            version,
            extension,
        };

        // This is sync, so we need to use a runtime block
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async { self.index_file(file_data).await })
        })
    }

    /// Legacy method for backward compatibility with tests - maps to commit
    #[allow(dead_code)]
    pub fn commit_writer(&self) -> Result<()> {
        tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(async { self.commit().await }))
    }

    /// Get advanced index metrics with repository-level statistics
    /// Uses Tantivy aggregation API for accurate counts across ALL documents (no limits)
    #[allow(dead_code)]
    pub fn get_advanced_metrics(&self) -> Result<AdvancedIndexMetrics> {
        use tantivy::aggregation::agg_req::Aggregations;
        use tantivy::aggregation::agg_result::AggregationResults;
        use tantivy::aggregation::AggregationCollector;
        use tantivy::query::AllQuery;

        let searcher = self.reader.searcher();
        let total_documents = searcher.num_docs();
        let total_size_mb = self.get_index_size_mb();

        // Use a match-all query to count all documents in the index
        let match_all_query = AllQuery;

        // Build aggregation request for repository counts using JSON
        let repo_agg_req: Aggregations = serde_json::from_value(serde_json::json!({
            "repository_terms": {
                "terms": {
                    "field": "repository",
                    "size": 10000  // Large enough to get all repositories
                }
            }
        }))?;

        let repo_collector = AggregationCollector::from_aggs(repo_agg_req, Default::default());
        let repo_agg_res: AggregationResults = searcher.search(&match_all_query, &repo_collector)?;

        // Extract repository counts from aggregation results
        let mut documents_by_repository: HashMap<String, u64> = HashMap::new();
        if let Some(tantivy::aggregation::agg_result::AggregationResult::BucketResult(
            tantivy::aggregation::agg_result::BucketResult::Terms { buckets, .. },
        )) = repo_agg_res.0.get("repository_terms")
        {
            for entry in buckets {
                if let tantivy::aggregation::Key::Str(term) = &entry.key {
                    documents_by_repository.insert(term.to_string(), entry.doc_count);
                }
            }
        }

        // Build aggregation request for extension counts using JSON
        let ext_agg_req: Aggregations = serde_json::from_value(serde_json::json!({
            "extension_terms": {
                "terms": {
                    "field": "extension",
                    "size": 1000  // Should be enough for file types
                }
            }
        }))?;

        let ext_collector = AggregationCollector::from_aggs(ext_agg_req, Default::default());
        let ext_agg_res: AggregationResults = searcher.search(&match_all_query, &ext_collector)?;

        // Extract extension counts from aggregation results
        let mut file_types_distribution: HashMap<String, u64> = HashMap::new();
        if let Some(tantivy::aggregation::agg_result::AggregationResult::BucketResult(
            tantivy::aggregation::agg_result::BucketResult::Terms { buckets, .. },
        )) = ext_agg_res.0.get("extension_terms")
        {
            for entry in buckets {
                if let tantivy::aggregation::Key::Str(term) = &entry.key {
                    file_types_distribution.insert(term.to_string(), entry.doc_count);
                }
            }
        }

        // Create top repositories list (sorted by document count)
        let mut top_repositories: Vec<(String, u64)> =
            documents_by_repository.iter().map(|(k, v)| (k.clone(), *v)).collect();
        top_repositories.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        top_repositories.truncate(20); // Top 20 repositories

        Ok(AdvancedIndexMetrics {
            total_documents,
            total_size_mb,
            documents_by_repository,
            top_repositories,
            file_types_distribution,
        })
    }
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct SearchStats {
    pub total_documents: u64,
    pub index_size_bytes: u64,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct AdvancedIndexMetrics {
    pub total_documents: u64,
    pub total_size_mb: f64,
    pub documents_by_repository: HashMap<String, u64>,
    pub top_repositories: Vec<(String, u64)>,
    pub file_types_distribution: HashMap<String, u64>,
}
