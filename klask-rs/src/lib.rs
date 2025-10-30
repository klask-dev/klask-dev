pub mod api;
pub mod auth;
pub mod config;
pub mod database;
pub mod models;
pub mod repositories;
pub mod services;
pub mod utils;

// Always available for integration tests but marked as test-only
#[cfg(any(test, debug_assertions))]
pub mod test_utils;

pub use config::AppConfig;
pub use database::Database;
