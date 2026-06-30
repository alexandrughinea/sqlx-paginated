use crate::paginated_query_as::internal::{
    get_struct_field_names, ColumnProtection, SqliteDialect,
};
use crate::{PaginatedInfo, QueryBuilder};
use std::marker::PhantomData;

impl<'q, T> Default for QueryBuilder<'q, T, sqlx::Sqlite>
where
    T: PaginatedInfo,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'q, T> QueryBuilder<'q, T, sqlx::Sqlite>
where
    T: PaginatedInfo,
{
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
            arguments: sqlx::sqlite::SqliteArguments::default(),
            valid_columns: get_struct_field_names::<T>(),
            protection: Some(ColumnProtection::for_sqlite()),
            protection_enabled: true,
            dialect: Box::new(SqliteDialect),
            _phantom: PhantomData,
        }
    }
}
