pub mod api;
pub mod auth;
pub mod config;
pub mod database;
pub mod models;
pub mod repositories;
pub mod services;
pub mod utils;

// Testing utilities - centralized test database setup
#[cfg(any(test, debug_assertions))]
pub mod testing;

pub use config::AppConfig;
pub use database::Database;
