use crate::{DatabaseQueryDefaults, PaginatedQueryBuilder};
use serde::Serialize;
use sqlx::{Database, FromRow, IntoArguments};

/// Creates a new `PaginatedQueryBuilder` with database-specific defaults.
///
/// This function works generically across different database backends (Postgres, SQLite, etc.)
/// by using the database type parameter to determine the appropriate defaults.
///
/// # Type Parameters
///
/// * `T` - The model type that implements `FromRow`, `Serialize`, and `Default`
/// * `DB` - The database type (e.g., `sqlx::Postgres`, `sqlx::Sqlite`)
///
/// # Arguments
///
/// * `sql` - The SQL query string
///
/// # Examples
///
/// **PostgreSQL:**
/// ```rust
/// use sqlx::Postgres;
/// use sqlx_paginated::paginated_query_as;
///
/// # use sqlx::FromRow;
/// # use serde::Serialize;
/// # #[derive(FromRow, Serialize, Default)]
/// # struct User { name: String }
/// let builder = paginated_query_as::<User, Postgres>("SELECT * FROM users");
/// ```
///
/// **SQLite:**
/// ```
/// # #[cfg(feature = "sqlite")]
/// # {
/// use sqlx::Sqlite;
/// use sqlx_paginated::paginated_query_as;
///
/// # use sqlx::FromRow;
/// # use serde::Serialize;
/// # #[derive(FromRow, Serialize, Default)]
/// # struct User { name: String }
/// let builder = paginated_query_as::<User, Sqlite>("SELECT * FROM users");
/// # }
/// ```
pub fn paginated_query_as<'q, T, DB>(
    sql: &'q str,
) -> PaginatedQueryBuilder<'q, T, DB, DB::Arguments<'q>>
where
    DB: Database + DatabaseQueryDefaults,
    T: for<'r> FromRow<'r, DB::Row> + Send + Unpin + Serialize + Default,
    DB::Arguments<'q>: IntoArguments<'q, DB>,
    for<'c> &'c sqlx::Pool<DB>: sqlx::Executor<'c, Database = DB>,
    usize: sqlx::ColumnIndex<DB::Row>,
    i64: sqlx::Type<DB> + for<'r> sqlx::Decode<'r, DB> + Send + Unpin,
{
    PaginatedQueryBuilder::new(sqlx::query_as::<DB, T>(sql), |params| {
        DB::build_default_query(params)
    })
}
