use crate::PaginatedQueryBuilder;
use serde::Serialize;
use sqlx::FromRow;

#[cfg(feature = "postgres")]
pub fn paginated_query_as<'q, T>(sql: &'q str) -> PaginatedQueryBuilder<'q, T, sqlx::Postgres, sqlx::postgres::PgArguments>
where
    T: for<'r> FromRow<'r, <sqlx::Postgres as sqlx::Database>::Row> + Send + Unpin + Serialize + Default,
{
    <PaginatedQueryBuilder<'q, T, sqlx::Postgres, sqlx::postgres::PgArguments>>::new_with_defaults(sqlx::query_as::<_, T>(sql))
}

#[cfg(feature = "sqlite")]
pub fn paginated_query_as_sqlite<'q, T>(sql: &'q str) -> PaginatedQueryBuilder<'q, T, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>>
where
    T: for<'r> FromRow<'r, <sqlx::Sqlite as sqlx::Database>::Row> + Send + Unpin + Serialize + Default,
{
    <PaginatedQueryBuilder<'q, T, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>>>::new_with_defaults(sqlx::query_as::<_, T>(sql))
}
