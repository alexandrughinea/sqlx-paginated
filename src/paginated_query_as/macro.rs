/// Creates a paginated query builder with database-specific defaults.
///
/// # Syntax
///
/// ```ignore
/// paginated_query_as!(Type, DatabaseType, "SQL query")
/// ```
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
/// let builder = paginated_query_as!(User, Postgres, "SELECT * FROM users");
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
/// let builder = paginated_query_as!(User, Sqlite, "SELECT * FROM users");
/// # }
/// ```
#[macro_export]
macro_rules! paginated_query_as {
    ($type:ty, $db:ty, $query:expr) => {{
        $crate::paginated_query_as::<$type, $db>($query)
    }};
}
