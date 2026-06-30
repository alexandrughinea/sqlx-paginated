use crate::paginated_query_as::internal::{ColumnProtection, PostgresDialect, };
use crate::{PaginatedInfo, QueryBuilder};
use std::marker::PhantomData;

impl<T> Default for QueryBuilder<'_, T, sqlx::Postgres>
where
    T: PaginatedInfo,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> QueryBuilder<'_, T, sqlx::Postgres>
where
    T: PaginatedInfo,
{
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
            arguments: sqlx::postgres::PgArguments::default(),
            protection: Some(ColumnProtection::for_postgres()),
            protection_enabled: true,
            dialect: Box::new(PostgresDialect),
            _phantom: PhantomData,
        }
    }
}
