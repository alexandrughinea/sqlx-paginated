use crate::PaginatedQueryBuilder;
use serde::Serialize;
use sqlx::postgres::PgArguments;
use sqlx::{FromRow, Postgres};

pub fn paginated_query_as<'q, T>(sql: &'q str) -> PaginatedQueryBuilder<'q, T, PgArguments>
where
    T: for<'r> FromRow<'r, <Postgres as sqlx::Database>::Row> + Send + Unpin + Serialize + Default + 'static,
{
    PaginatedQueryBuilder::new(sqlx::query_as::<_, T>(sql))
}
