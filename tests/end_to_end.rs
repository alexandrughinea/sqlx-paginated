//! End-to-end tests requiring actual database connections.
//!
//! These are placeholder tests. Implement database setup utilities to enable.

#[cfg(feature = "postgres")]
#[path = "end_to_end/postgres.rs"]
mod postgres;

#[cfg(feature = "sqlite")]
#[path = "end_to_end/sqlite.rs"]
mod sqlite;
