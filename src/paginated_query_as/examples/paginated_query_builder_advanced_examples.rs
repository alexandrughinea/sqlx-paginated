use crate::paginated_query_as::QueryParamsBuilder;
use crate::{paginated_query_as, QueryBuilder, QueryDateTime};
use crate::{PaginatedResponse, QuerySortDirection};
use chrono::Utc;
use serde::Serialize;
use sqlx::{Arguments, FromRow, PgPool, Postgres};
use std::collections::HashMap;

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
    let some_extra_filters =
        HashMap::from([("a", Some("1".to_string())), ("b", Some("2".to_string()))]);
    let date_after = QueryDateTime::TimestampTz(Utc::now());
    let initial_params = QueryParamsBuilder::<UserExample>::new()
        .with_search("john", vec!["name", "email"])
        .with_pagination(1, 10)
        .with_date_range(Some(date_after), None, None::<String>)
        .with_filter("status", Some("active"))
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
                .with_date_range(params) // Add or remove data range;
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
