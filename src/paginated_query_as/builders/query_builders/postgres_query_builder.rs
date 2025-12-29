use crate::paginated_query_as::internal::{
    get_struct_field_names, ColumnProtection, PostgresDialect,
};
use crate::QueryBuilder;
use serde::Serialize;
use std::marker::PhantomData;

impl<T> Default for QueryBuilder<'_, T, sqlx::Postgres>
where
    T: Default + Serialize,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> QueryBuilder<'_, T, sqlx::Postgres>
where
    T: Default + Serialize,
{
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
            arguments: sqlx::postgres::PgArguments::default(),
            valid_columns: get_struct_field_names::<T>(),
            protection: Some(ColumnProtection::for_postgres()),
            protection_enabled: true,
            dialect: Box::new(PostgresDialect),
            _phantom: PhantomData,
        }
    }
}
