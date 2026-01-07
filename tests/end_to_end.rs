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
//! ### postgres (requires DB)
//! ```bash
//! docker run --name sqlx-test-postgres \
//!   -e POSTGRES_PASSWORD=postgres \
//!   -e POSTGRES_DB=sqlx_paginated_test \
//!   -p 5432:5432 -d postgres:15
//!
//! # Run tests
//! cargo test --test end_to_end --features postgres -- --ignored
//! ```

#[cfg(feature = "postgres")]
#[path = "end_to_end/postgres.rs"]
mod postgres;

#[cfg(feature = "sqlite")]
#[path = "end_to_end/sqlite.rs"]
mod sqlite;
