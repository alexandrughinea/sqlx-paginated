//! End-to-end tests requiring actual database connections.
//!
//!
//! ## Running tests
//!
//! ### sqlite
//! ```bash
//! cargo test --test end_to_end --features sqlite
//! ```
//!
//! ### postgres
//! Utilizes a test container for each test, requires a Docker-compatible runtime.
//! ```bash
//!
//! # Run tests
//! cargo test --test end_to_end --features postgres
//! ```

#[cfg(feature = "postgres")]
#[path = "end_to_end/postgres.rs"]
mod postgres;

#[cfg(feature = "sqlite")]
#[path = "end_to_end/sqlite.rs"]
mod sqlite;
