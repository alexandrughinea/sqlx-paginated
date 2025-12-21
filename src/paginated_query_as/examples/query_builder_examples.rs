use crate::paginated_query_as::QueryBuildResult;
use crate::{QueryBuilder, QueryParams};
use serde::Serialize;

#[cfg(feature = "postgres")]
pub mod postgres_examples {
    use super::*;
    use sqlx::Postgres;

    #[allow(dead_code)]
    pub fn build_query_with_disabled_protection<T>(
        params: &QueryParams<T>,
    ) -> QueryBuildResult<'static, Postgres>
    where
        T: Default + Serialize + 'static,
    {
        QueryBuilder::<T, Postgres>::new()
            .with_table_prefix("base_query")
            .with_search(params)
            .with_filters(params)
            .disable_protection()
            .build()
    }

    #[allow(dead_code)]
    pub fn build_query_with_safe_defaults<T>(
        params: &QueryParams<T>,
    ) -> QueryBuildResult<'static, Postgres>
    where
        T: Default + Serialize + 'static,
    {
        QueryBuilder::<T, Postgres>::new()
            .with_table_prefix("base_query")
            .with_search(params)
            .with_filters(params)
            .build()
    }

    #[cfg(test)]
    mod test {
        use super::*;
        use crate::QueryParamsBuilder;

        #[derive(Debug, Default, Serialize)]
        struct TestModel {
            name: String,
            title: String,
            description: String,
            status: String,
            category: String,
        }

        #[test]
        fn test_search_query_generation() {
            let params = QueryParamsBuilder::<TestModel>::new()
                .with_search("XXX", vec!["description"])
                .build();

            let result = build_query_with_safe_defaults::<TestModel>(&params);

            assert!(!result.conditions.is_empty());
            assert!(result.conditions.iter().any(|c| c.contains("LOWER")));
            assert!(result.conditions.iter().any(|c| c.contains("LIKE LOWER")));
        }

        #[test]
        fn test_empty_search_query() {
            let params = QueryParamsBuilder::<TestModel>::new()
                .with_search("   ", vec!["name"])
                .build();

            let result = build_query_with_safe_defaults::<TestModel>(&params);
            assert!(!result.conditions.iter().any(|c| c.contains("LIKE")));
        }
    }
}

#[cfg(feature = "sqlite")]
pub mod sqlite_examples {
    use super::*;
    use sqlx::Sqlite;

    #[allow(dead_code)]
    pub fn builder_new_query_with_disabled_protection_for_sqlite<'q, T>(
        params: &'q QueryParams<T>,
    ) -> QueryBuildResult<'q, Sqlite>
    where
        T: Default + Serialize,
    {
        QueryBuilder::<'q, T, Sqlite>::new()
            .with_search(params)
            .with_filters(params)
            .disable_protection()
            .build()
    }

    #[cfg(test)]
    mod test {
        use super::*;
        use crate::QueryParamsBuilder;

        #[derive(Debug, Default, Serialize)]
        struct TestModel {
            name: String,
            title: String,
            description: String,
            status: String,
            category: String,
        }

        #[test]
        fn test_empty_search_query_sqlite() {
            let params = QueryParamsBuilder::<TestModel>::new()
                .with_search("   ", vec!["name"])
                .build();

            let result =
                builder_new_query_with_disabled_protection_for_sqlite::<TestModel>(&params);
            assert!(!result.conditions.iter().any(|c| c.contains("LIKE")));
        }
    }
}
