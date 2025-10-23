// Module declarations for crawler submodules
pub mod branch_processor;
pub mod file_processing;
pub mod git_operations;
pub mod git_tree_walker;
pub mod github_crawler;
pub mod gitlab_crawler;
pub mod service;

// Re-export main service and commonly used types
#[allow(unused_imports)]
pub use branch_processor::CrawlProgress;
pub use service::CrawlerService;
