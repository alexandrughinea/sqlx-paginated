use crate::paginated_query_as::internal::{get_struct_field_meta, ColumnProtection, SqliteDialect};
use crate::QueryBuilder;
use serde::Serialize;
use std::collections::HashMap;
use std::marker::PhantomData;

impl<'q, T> Default for QueryBuilder<'q, T, sqlx::Sqlite>
where
    T: Default + Serialize,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'q, T> QueryBuilder<'q, T, sqlx::Sqlite>
where
    T: Default + Serialize,
{
    pub fn new() -> Self {
        let field_meta = get_struct_field_meta::<T>();
        let valid_columns: Vec<String> = field_meta.keys().cloned().collect();
        Self {
            conditions: Vec::new(),
            arguments: sqlx::sqlite::SqliteArguments::default(),
            mappers: HashMap::new(),
            valid_columns,
            field_meta,
            protection: Some(ColumnProtection::default()),
            protection_enabled: true,
            column_validation_enabled: true,
            dialect: Box::new(SqliteDialect),
            _phantom: PhantomData,
            computed_properties: HashMap::new(),
            active_joins: Vec::new(),
            table_prefix: None,
        }
    }
}
