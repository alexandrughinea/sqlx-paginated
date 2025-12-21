use crate::paginated_query_as::models::{Filter, FilterOperator, FilterValue};
use crate::paginated_query_as::QueryParamsBuilder;
use crate::{paginated_query_as, QueryBuilder};
use crate::{PaginatedResponse, QuerySortDirection};
use serde::Serialize;
use sqlx::{Arguments, FromRow, PgPool, Postgres};

#[derive(Default, Serialize, FromRow)]
#[allow(dead_code)]
pub struct UserExample {
    id: String,
    name: String,
    email: String,
    status: String,
    role: String,
    score: i32,
}

#[allow(dead_code)]
pub async fn paginated_query_builder_advanced_example(
    pool: PgPool,
) -> PaginatedResponse<UserExample> {
    let some_extra_filters = vec![
        Filter {
            field: "role".to_string(),
            operator: FilterOperator::Eq,
            value: FilterValue::String("admin".to_string()),
        },
    ];
    let initial_params = QueryParamsBuilder::<UserExample>::new()
        .with_search("john", vec!["name", "email"])
        .with_pagination(1, 10)
        .with_eq_filter("status", "active")
        .with_filters(some_extra_filters)
        .with_sort("created_at", QuerySortDirection::Descending)
        .build();

    paginated_query_as!(UserExample, "SELECT * FROM users")
        .with_params(initial_params)
        .with_query_builder(|params| {
            // Can override the default query builder (build_query_with_safe_defaults) with a complete custom one:
            QueryBuilder::<UserExample, Postgres>::new()
                .with_search(params) // Add or remove search feature from the query;
                .with_filters(params) // Add or remove custom filters from the query;
                .with_raw_condition("") // Add raw condition, no checks.
                .disable_protection() // This removes all column safety checks.
                .with_combined_conditions(|builder| {
                    if builder.has_column("status") && builder.has_column("role") {
                        builder
                            .conditions
                            .push("(status = 'active' AND role IN ('admin', 'user'))".to_string());
                    }
                    if builder.has_column("score") {
                        builder
                            .conditions
                            .push("score BETWEEN $1 AND $2".to_string());
                        let _ = builder.arguments.add(50);
                        let _ = builder.arguments.add(100);
                    }
                })
                .build()
        })
        .fetch_paginated(&pool)
        .await
        .unwrap()
}
