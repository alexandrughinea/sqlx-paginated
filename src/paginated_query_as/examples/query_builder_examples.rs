use crate::{QueryBuilder, QueryParams};
use serde::Serialize;
use sqlx::postgres::PgArguments;
use sqlx::sqlite::SqliteArguments;
use sqlx::Sqlite;
use sqlx::{Database, Postgres};

#[cfg(feature = "sqlite")]
#[allow(dead_code)]
pub fn builder_new_query_with_disabled_protection_for_sqlite<'q, T, DB>(
    params: &'q QueryParams<T>,
) -> (Vec<String>, DB::Arguments<'q>)
where
    T: Default + Serialize,
    DB: Database<Arguments<'q> = SqliteArguments<'q>>,
{
    QueryBuilder::<'q, T, Sqlite>::new()
        .with_search(params)
        .with_filters(params)
        .with_date_range(params)
        .disable_protection()
        .build()
}

#[cfg(feature = "postgres")]
#[allow(dead_code)]
pub fn build_query_with_disabled_protection<'q, T, DB>(
    params: &QueryParams<T>,
) -> (Vec<String>, DB::Arguments<'q>)
where
    T: Default + Serialize,
    DB: Database<Arguments<'q> = PgArguments>,
{
    QueryBuilder::<T, Postgres>::new()
        .with_search(params)
        .with_filters(params)
        .with_date_range(params)
        .disable_protection()
        .build()
}

#[cfg(feature = "postgres")]
#[allow(dead_code)]
pub fn build_query_with_safe_defaults<'q, T, DB>(
    params: &QueryParams<T>,
) -> (Vec<String>, DB::Arguments<'q>)
where
    T: Default + Serialize,
    DB: Database<Arguments<'q> = PgArguments>,
{
    QueryBuilder::<T, Postgres>::new()
        .with_search(params)
        .with_filters(params)
        .with_date_range(params)
        .build()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::paginated_query_as::QueryParamsBuilder;
    use chrono::{DateTime, Utc};

    #[derive(Debug, Default, Serialize)]
    struct TestModel {
        name: String,
        title: String,
        description: String,
        status: String,
        category: String,
        updated_at: DateTime<Utc>,
        created_at: DateTime<Utc>,
    }

    #[test]
    fn test_search_query_generation() {
        let params = QueryParamsBuilder::<TestModel>::new()
            .with_search("XXX", vec!["description"])
            .build();

        let (conditions, _) = build_query_with_safe_defaults::<TestModel, Postgres>(&params);

        assert!(!conditions.is_empty());
        assert!(conditions.iter().any(|c| c.contains("LOWER")));
        assert!(conditions.iter().any(|c| c.contains("LIKE LOWER")));
    }

    #[test]
    fn test_empty_search_query() {
        let params = QueryParamsBuilder::<TestModel>::new()
            .with_search("   ", vec!["name"])
            .build();

        let (conditions, _) = build_query_with_safe_defaults::<TestModel, Postgres>(&params);
        assert!(!conditions.iter().any(|c| c.contains("LIKE")));
    }

    #[test]
    fn test_empty_search_query_sqlite() {
        let params = QueryParamsBuilder::<TestModel>::new()
            .with_search("   ", vec!["name"])
            .build();

        let (conditions, _) =
            builder_new_query_with_disabled_protection_for_sqlite::<TestModel, Sqlite>(&params);
        assert!(!conditions.iter().any(|c| c.contains("LIKE")));
    }
}
