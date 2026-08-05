use crate::paginated_query_as::internal::{ColumnProtection, SqliteDialect};
use crate::{FieldSource, QueryBuilder};
use std::marker::PhantomData;

impl<'q, T> Default for QueryBuilder<'q, T, sqlx::Sqlite>
where
    T: FieldSource,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'q, T> QueryBuilder<'q, T, sqlx::Sqlite>
where
    T: FieldSource,
{
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
            arguments: sqlx::sqlite::SqliteArguments::default(),
            protection: Some(ColumnProtection::for_sqlite()),
            protection_enabled: true,
            dialect: Box::new(SqliteDialect),
            _phantom: PhantomData,
        }
    }
}
