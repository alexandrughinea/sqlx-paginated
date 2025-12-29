use crate::QueryParams;
use serde::Serialize;
use sqlx::Database;

/// Trait for providing database-specific default query building behavior.
///
/// This trait allows `paginated_query_as` to work generically across different
/// database backends while maintaining database-specific default configurations.
pub trait DatabaseQueryDefaults: Database {
    /// Builds default query conditions and arguments for this database type.
    ///
    /// # Arguments
    ///
    /// * `params` - Query parameters containing search, filter, and date range settings
    ///
    /// # Returns
    ///
    /// Returns a tuple of (conditions, arguments) to be used in the paginated query.
    fn build_default_query<'p, T>(params: &'p QueryParams<T>) -> (Vec<String>, Self::Arguments<'p>)
    where
        T: Default + Serialize;
}

#[cfg(feature = "postgres")]
impl DatabaseQueryDefaults for sqlx::Postgres {
    fn build_default_query<'p, T>(params: &'p QueryParams<T>) -> (Vec<String>, Self::Arguments<'p>)
    where
        T: Default + Serialize,
    {
        use crate::paginated_query_as::examples::postgres_examples::build_query_with_safe_defaults;
        build_query_with_safe_defaults::<T, sqlx::Postgres>(params)
    }
}

#[cfg(feature = "sqlite")]
impl DatabaseQueryDefaults for sqlx::Sqlite {
    fn build_default_query<'p, T>(params: &'p QueryParams<T>) -> (Vec<String>, Self::Arguments<'p>)
    where
        T: Default + Serialize,
    {
        use crate::QueryBuilder;
        QueryBuilder::<T, sqlx::Sqlite>::new()
            .with_search(params)
            .with_filters(params)
            .with_date_range(params)
            .build()
    }
}
