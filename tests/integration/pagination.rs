use serde::Serialize;
use sqlx_paginated::{QueryBuilder, QueryParamsBuilder, QuerySortDirection};

#[derive(Serialize, Default, Debug)]
struct TestUser {
    id: i64,
    name: String,
    email: String,
    status: String,
    created_at: String,
    updated_at: String,
}

#[cfg(feature = "postgres")]
mod postgres_tests {
    use super::*;
    use sqlx::Postgres;

    #[test]
    fn test_basic_pagination() {
        let params = QueryParamsBuilder::<TestUser>::new()
            .with_pagination(2, 20)
            .build();

        assert_eq!(params.pagination.page, 2);
        assert_eq!(params.pagination.page_size, 20);
    }

    #[test]
    fn test_pagination_with_sorting() {
        let params = QueryParamsBuilder::<TestUser>::new()
            .with_pagination(1, 10)
            .with_sort("email", QuerySortDirection::Ascending)
            .build();

        assert_eq!(params.pagination.page, 1);
        assert_eq!(params.pagination.page_size, 10);
        assert_eq!(params.sort.sort_column, "email");
        assert_eq!(params.sort.sort_direction, QuerySortDirection::Ascending);
    }

    #[test]
    fn test_descending_sort() {
        let params = QueryParamsBuilder::<TestUser>::new()
            .with_sort("created_at", QuerySortDirection::Descending)
            .build();

        assert_eq!(params.sort.sort_column, "created_at");
        assert_eq!(params.sort.sort_direction, QuerySortDirection::Descending);
    }

    #[test]
    fn test_search_functionality() {
        let params = QueryParamsBuilder::<TestUser>::new()
            .with_search("john", vec!["name", "email"])
            .build();

        let (conditions, _args) = QueryBuilder::<TestUser, Postgres>::new()
            .with_search(&params)
            .build();

        assert!(!conditions.is_empty());
        let combined = conditions.join(" ");
        assert!(combined.contains("name") || combined.contains("email"));
    }

    #[test]
    fn test_date_range_filtering() {
        use chrono::DateTime;

        let start = DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
            .unwrap()
            .into();
        let end = DateTime::parse_from_rfc3339("2024-12-31T23:59:59Z")
            .unwrap()
            .into();

        let params = QueryParamsBuilder::<TestUser>::new()
            .with_date_range(Some(start), Some(end), Some("created_at"))
            .build();

        let (conditions, _args) = QueryBuilder::<TestUser, Postgres>::new()
            .with_date_range(&params)
            .build();

        assert!(!conditions.is_empty());
        let combined = conditions.join(" ");
        assert!(combined.contains("created_at"));
    }

    #[test]
    fn test_combined_pagination_filters_search() {
        let params = QueryParamsBuilder::<TestUser>::new()
            .with_pagination(1, 25)
            .with_filter("status", Some("active"))
            .with_search("test", vec!["name", "email"])
            .with_sort("created_at", QuerySortDirection::Descending)
            .build();

        let (conditions, _args) = QueryBuilder::<TestUser, Postgres>::new()
            .with_filters(&params)
            .with_search(&params)
            .build();

        assert!(conditions.len() >= 1);
    }

    #[test]
    fn test_page_size_clamping() {
        let params = QueryParamsBuilder::<TestUser>::new()
            .with_pagination(1, 1)
            .build();
        assert!(params.pagination.page_size >= 10);

        let params = QueryParamsBuilder::<TestUser>::new()
            .with_pagination(1, 10000)
            .build();
        assert!(params.pagination.page_size <= 100);
    }

    #[test]
    fn test_empty_search_no_conditions() {
        let params = QueryParamsBuilder::<TestUser>::new()
            .with_search("", vec!["name"])
            .build();

        let (conditions, _args) = QueryBuilder::<TestUser, Postgres>::new()
            .with_search(&params)
            .build();

        assert!(conditions.is_empty());
    }

    #[test]
    fn test_multiple_filters() {
        let params = QueryParamsBuilder::<TestUser>::new()
            .with_filter("status", Some("active"))
            .with_filter("email", Some("test@example.com"))
            .build();

        let (conditions, _args) = QueryBuilder::<TestUser, Postgres>::new()
            .with_filters(&params)
            .build();

        assert_eq!(conditions.len(), 2);
    }
}

#[cfg(feature = "sqlite")]
mod sqlite_tests {
    use super::*;
    use sqlx::Sqlite;

    #[test]
    fn test_basic_pagination_sqlite() {
        let params = QueryParamsBuilder::<TestUser>::new()
            .with_pagination(3, 15)
            .build();

        assert_eq!(params.pagination.page, 3);
        assert_eq!(params.pagination.page_size, 15);
    }

    #[test]
    fn test_sorting_sqlite() {
        let params = QueryParamsBuilder::<TestUser>::new()
            .with_sort("name", QuerySortDirection::Ascending)
            .build();

        assert_eq!(params.sort.sort_column, "name");
        assert_eq!(params.sort.sort_direction, QuerySortDirection::Ascending);
    }

    #[test]
    fn test_search_sqlite() {
        let params = QueryParamsBuilder::<TestUser>::new()
            .with_search("search term", vec!["name"])
            .build();

        let (conditions, _args) = QueryBuilder::<TestUser, Sqlite>::new()
            .with_search(&params)
            .build();

        if !conditions.is_empty() {
            let combined = conditions.join(" ");
            assert!(combined.contains("?") || combined.contains("name"));
        }
    }

    #[test]
    fn test_filters_sqlite() {
        let params = QueryParamsBuilder::<TestUser>::new()
            .with_filter("status", Some("active"))
            .build();

        let (conditions, _args) = QueryBuilder::<TestUser, Sqlite>::new()
            .with_filters(&params)
            .build();

        assert_eq!(conditions.len(), 1);
        assert!(conditions[0].contains("\"status\" = ?"));
    }

    #[test]
    fn test_search_multiple_columns_binding_count() {
        use sqlx::Arguments;

        let params = QueryParamsBuilder::<TestUser>::new()
            .with_search("Smith", vec!["name", "email", "status"])
            .build();

        let (conditions, args) = QueryBuilder::<TestUser, Sqlite>::new()
            .with_search(&params)
            .build();

        assert_eq!(
            conditions.len(),
            1,
            "Should have one search condition group"
        );
        let condition = &conditions[0];
        let placeholder_count = condition.matches('?').count();
        assert_eq!(
            placeholder_count, 3,
            "Should have one ? placeholder per search column"
        );

        assert_eq!(
            args.len(),
            3,
            "SQLite requires one binding per ? placeholder (was 1, should be 3)"
        );

        assert!(
            condition.contains(" OR "),
            "Multiple columns should be joined with OR"
        );
    }

    #[test]
    fn test_search_different_column_counts() {
        use sqlx::Arguments;

        // One column = one binding
        let params_1 = QueryParamsBuilder::<TestUser>::new()
            .with_search("test", vec!["name"])
            .build();
        let (_, args_1) = QueryBuilder::<TestUser, Sqlite>::new()
            .with_search(&params_1)
            .build();
        assert_eq!(args_1.len(), 1);

        // Two columns = two bindings
        let params_2 = QueryParamsBuilder::<TestUser>::new()
            .with_search("test", vec!["name", "email"])
            .build();
        let (_, args_2) = QueryBuilder::<TestUser, Sqlite>::new()
            .with_search(&params_2)
            .build();
        assert_eq!(args_2.len(), 2);

        // Three columns = three bindings
        let params_3 = QueryParamsBuilder::<TestUser>::new()
            .with_search("test", vec!["name", "email", "status"])
            .build();
        let (_, args_3) = QueryBuilder::<TestUser, Sqlite>::new()
            .with_search(&params_3)
            .build();
        assert_eq!(args_3.len(), 3);
    }
}

#[test]
fn test_query_params_builder_defaults() {
    let params = QueryParamsBuilder::<TestUser>::new().build();

    assert_eq!(params.pagination.page, 1);
    assert_eq!(params.pagination.page_size, 10);
    assert_eq!(params.sort.sort_column, "created_at");
    assert_eq!(params.sort.sort_direction, QuerySortDirection::Descending);
}

#[test]
fn test_sort_direction_enum() {
    let asc = QuerySortDirection::Ascending;
    let desc = QuerySortDirection::Descending;
    assert_ne!(asc, desc);
}

#[test]
fn test_builder_chaining() {
    let params = QueryParamsBuilder::<TestUser>::new()
        .with_pagination(2, 50)
        .with_sort("email", QuerySortDirection::Ascending)
        .with_filter("status", Some("active"))
        .with_search("john", vec!["name", "email"])
        .build();

    assert_eq!(params.pagination.page, 2);
    assert_eq!(params.pagination.page_size, 50);
    assert_eq!(params.sort.sort_column, "email");
    assert_eq!(params.sort.sort_direction, QuerySortDirection::Ascending);
}
