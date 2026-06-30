use crate::{QueryBuilder, QueryParams};

#[cfg(feature = "postgres")]
pub mod postgres_examples {
    use super::*;
    use sqlx::postgres::PgArguments;
    use sqlx::Postgres;
    use crate::{PaginatedInfo};

    #[allow(dead_code)]
    pub fn build_query_with_disabled_protection<T>(
        params: &QueryParams<T>,
    ) -> (Vec<String>, PgArguments)
    where
        T: PaginatedInfo,
    {
        QueryBuilder::<T, Postgres>::new()
            .with_search(params)
            .with_filters(params)
            .with_date_range(params)
            .disable_protection()
            .build()
    }

    #[allow(dead_code)]
    pub fn build_query_with_safe_defaults<T>(params: &QueryParams<T>) -> (Vec<String>, PgArguments)
    where
        T: PaginatedInfo,
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
        use crate::QueryParamsBuilder;
        use chrono::{DateTime, Utc};
        use sqlx_paginated_derive::{Fields, Paginated};

        #[derive(Paginated, Fields)]
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

            let (conditions, _) = build_query_with_safe_defaults::<TestModel>(&params);

            assert!(!conditions.is_empty());
            assert!(conditions.iter().any(|c| c.contains("LOWER")));
            assert!(conditions.iter().any(|c| c.contains("LIKE LOWER")));
        }

        #[test]
        fn test_empty_search_query() {
            let params = QueryParamsBuilder::<TestModel>::new()
                .with_search("   ", vec!["name"])
                .build();

            let (conditions, _) = build_query_with_safe_defaults::<TestModel>(&params);
            assert!(!conditions.iter().any(|c| c.contains("LIKE")));
        }
    }
}

#[cfg(feature = "sqlite")]
pub mod sqlite_examples {
    use super::*;
    use sqlx::sqlite::SqliteArguments;
    use sqlx::Sqlite;
    use crate::PaginatedInfo;

    #[allow(dead_code)]
    pub fn build_query_with_safe_defaults<T>(
        params: &QueryParams<T>,
    ) -> (Vec<String>, SqliteArguments)
    where
        T: PaginatedInfo,
    {
        QueryBuilder::<T, Sqlite>::new()
            .with_search(params)
            .with_filters(params)
            .with_date_range(params)
            .build()
    }

    #[allow(dead_code)]
    pub fn builder_new_query_with_disabled_protection_for_sqlite<T>(
        params: &QueryParams<T>,
    ) -> (Vec<String>, SqliteArguments)
    where
        T: PaginatedInfo,
    {
        QueryBuilder::<T, Sqlite>::new()
            .with_search(params)
            .with_filters(params)
            .with_date_range(params)
            .disable_protection()
            .build()
    }

    #[cfg(test)]
    mod test {
        use super::*;
        use crate::QueryParamsBuilder;
        use chrono::{DateTime, Utc};
        use sqlx_paginated_derive::{Fields, Paginated};

        #[derive(Debug, Paginated, Fields)]
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
        fn test_empty_search_query_sqlite() {
            let params = QueryParamsBuilder::<TestModel>::new()
                .with_search("   ", vec!["name"])
                .build();

            let (conditions, _) =
                builder_new_query_with_disabled_protection_for_sqlite::<TestModel>(&params);
            assert!(!conditions.iter().any(|c| c.contains("LIKE")));
        }
    }
}
