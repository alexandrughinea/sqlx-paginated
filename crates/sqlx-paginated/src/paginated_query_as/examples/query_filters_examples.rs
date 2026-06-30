//! Examples demonstrating the use of filter operators
//!
//! This module provides comprehensive examples of how to use the various
//! filter operators supported by sqlx-paginated.

#![allow(dead_code)]

#[cfg(feature = "postgres")]
pub mod postgres_examples {
    use crate::{
        QueryBuilder, QueryFilterCondition, QueryFilterOperator, QueryParamsBuilder,
        QuerySortDirection,
    };
    use serde::Serialize;
    use sqlx::Postgres;
    use std::collections::HashMap;

    #[allow(dead_code)]
    #[derive(Serialize, Default)]
    pub struct Product {
        pub id: i64,
        pub name: String,
        pub price: f64,
        pub stock: i32,
        pub status: String,
        pub category: String,
        pub deleted_at: Option<String>,
    }

    /// Example: Using comparison operators for filtering
    ///
    /// This example shows how to use >, >=, <, <= operators
    /// to filter numeric fields like price and stock.
    pub fn example_comparison_operators() {
        let params = QueryParamsBuilder::<Product>::new()
            .with_filter_operator("price", QueryFilterOperator::GreaterThan, "10.00")
            .with_filter_operator("stock", QueryFilterOperator::GreaterOrEqual, "1")
            .build();

        // Generates: WHERE "price" > $1 AND "stock" >= $2
        let (_conditions, _args) = QueryBuilder::<Product, Postgres>::new()
            .with_filters(&params)
            .build();
    }

    /// Example: Using IN and NOT IN operators
    ///
    /// Filter records where a field matches (or doesn't match)
    /// a list of values.
    pub fn example_in_operators() {
        let params = QueryParamsBuilder::<Product>::new()
            .with_filter_in("status", vec!["active", "pending", "approved"])
            .with_filter_not_in("category", vec!["deprecated", "archived"])
            .build();

        // Generates: WHERE "status" IN ($1, $2, $3) AND "category" NOT IN ($4, $5)
        let (_conditions, _args) = QueryBuilder::<Product, Postgres>::new()
            .with_filters(&params)
            .build();
    }

    /// Example: Using NULL checks
    ///
    /// Filter records based on whether a field is NULL or NOT NULL.
    pub fn example_null_checks() {
        let params = QueryParamsBuilder::<Product>::new()
            .with_filter_null("deleted_at", true) // IS NULL
            .build();

        // Generates: WHERE "deleted_at" IS NULL
        let (_conditions, _args) = QueryBuilder::<Product, Postgres>::new()
            .with_filters(&params)
            .build();

        // For NOT NULL:
        let params_not_null = QueryParamsBuilder::<Product>::new()
            .with_filter_null("deleted_at", false) // IS NOT NULL
            .build();

        // Generates: WHERE "deleted_at" IS NOT NULL
        let (_conditions, _args) = QueryBuilder::<Product, Postgres>::new()
            .with_filters(&params_not_null)
            .build();
    }

    /// Example: Using LIKE and NOT LIKE for pattern matching
    ///
    /// Search for records using SQL wildcards.
    /// % matches any sequence of characters
    /// _ matches a single character
    pub fn example_like_patterns() {
        let params = QueryParamsBuilder::<Product>::new()
            .with_filter_operator("name", QueryFilterOperator::Like, "%laptop%")
            .with_filter_operator("category", QueryFilterOperator::NotLike, "%test%")
            .build();

        // Generates: WHERE LOWER("name") LIKE LOWER($1) AND LOWER("category") NOT LIKE LOWER($2)
        let (_conditions, _args) = QueryBuilder::<Product, Postgres>::new()
            .with_filters(&params)
            .build();
    }

    /// Example: Using NOT EQUAL operator
    ///
    /// Filter records where a field does not equal a specific value.
    pub fn example_not_equal() {
        let params = QueryParamsBuilder::<Product>::new()
            .with_filter_operator("status", QueryFilterOperator::NotEqual, "deleted")
            .build();

        // Generates: WHERE "status" != $1
        let (_conditions, _args) = QueryBuilder::<Product, Postgres>::new()
            .with_filters(&params)
            .build();
    }

    /// Example: Complex filtering combining multiple operators
    ///
    /// This demonstrates a real-world e-commerce scenario with
    /// multiple filter conditions.
    pub fn example_complex_filtering() {
        let params = QueryParamsBuilder::<Product>::new()
            // Price range
            .with_filter_operator("price", QueryFilterOperator::GreaterOrEqual, "10.00")
            // Available stock
            .with_filter_operator("stock", QueryFilterOperator::GreaterThan, "0")
            // Active status
            .with_filter("status", Some("active"))
            // Not deleted
            .with_filter_null("deleted_at", true)
            // Specific categories
            .with_filter_in("category", vec!["electronics", "computers", "accessories"])
            // Sorting
            .with_sort("price", QuerySortDirection::Ascending)
            // Pagination
            .with_pagination(1, 20)
            .build();

        let (_conditions, _args) = QueryBuilder::<Product, Postgres>::new()
            .with_filters(&params)
            .build();

        // Generates complex WHERE clause with all conditions
    }

    /// Example: Using QueryFilterCondition directly
    ///
    /// For advanced use cases, you can create QueryFilterCondition objects
    /// directly and pass them in a HashMap.
    pub fn example_filter_conditions() {
        let mut filters = HashMap::new();
        filters.insert("price", QueryFilterCondition::greater_than("50.00"));
        filters.insert("stock", QueryFilterCondition::less_or_equal("100"));
        filters.insert("status", QueryFilterCondition::not_equal("deleted"));
        filters.insert("deleted_at", QueryFilterCondition::is_null());

        let params = QueryParamsBuilder::<Product>::new()
            .with_filter_conditions(filters)
            .build();

        let (_conditions, _args) = QueryBuilder::<Product, Postgres>::new()
            .with_filters(&params)
            .build();
    }

    /// Example: Backward compatibility with simple filters
    ///
    /// The old API still works - simple filters default to equality.
    pub fn example_backward_compatibility() {
        // Old style (still works)
        let params = QueryParamsBuilder::<Product>::new()
            .with_filter("status", Some("active"))
            .with_filter("category", Some("electronics"))
            .build();

        // Generates: WHERE "status" = $1 AND "category" = $2
        let (_conditions, _args) = QueryBuilder::<Product, Postgres>::new()
            .with_filters(&params)
            .build();
    }

    /// Example: E-commerce product search with filters
    ///
    /// Complete example showing how to build an API endpoint
    /// for product search with advanced filtering.
    pub fn example_ecommerce_search() {
        // Simulate query parameters from API:
        // GET /products?search=laptop&price[gte]=500&price[lte]=2000&stock[gt]=0
        //                &status=active&category[in]=computers,electronics

        let params = QueryParamsBuilder::<Product>::new()
            // Text search across name and category
            .with_search("laptop", vec!["name", "category"])
            // Price range: $500 to $2000
            .with_filter_operator("price", QueryFilterOperator::GreaterOrEqual, "500")
            // In stock only
            .with_filter_operator("stock", QueryFilterOperator::GreaterThan, "0")
            // Active products
            .with_filter("status", Some("active"))
            // Specific categories
            .with_filter_in("category", vec!["computers", "electronics"])
            // Not deleted
            .with_filter_null("deleted_at", true)
            // Sort by price ascending
            .with_sort("price", QuerySortDirection::Ascending)
            // First page, 24 items
            .with_pagination(1, 24)
            .build();

        let (_conditions, _args) = QueryBuilder::<Product, Postgres>::new()
            .with_search(&params)
            .with_filters(&params)
            .build();
    }

    /// Example: Using automatic operator deserialization from query strings
    ///
    /// This example demonstrates how the operator syntax is automatically
    /// parsed from URL query parameters when using `FlatQueryParams`.
    ///
    /// ## Query String Examples
    ///
    /// ```text
    /// # Simple equality (backward compatible)
    /// GET /products?status=active
    ///
    /// # Comparison operators
    /// GET /products?price[gt]=100&stock[lte]=50
    ///
    /// # IN operator with comma-separated values
    /// GET /products?category[in]=electronics,books,toys
    ///
    /// # NULL checks
    /// GET /products?deleted_at[is_null]=&featured[is_not_null]=
    ///
    /// # LIKE patterns
    /// GET /products?name[like]=%gaming%
    ///
    /// # Combined filters
    /// GET /products?status=active&price[gte]=50&price[lte]=500
    ///     &category[in]=electronics,computers&deleted_at[is_null]=
    /// ```
    ///
    /// ## Web Framework Integration
    ///
    /// With Actix-web:
    /// ```rust,ignore
    /// use actix_web::{web, HttpResponse};
    /// use sqlx_paginated::{FlatQueryParams, QueryParams};
    ///
    /// async fn list_products(
    ///     Query(flat_params): Query<FlatQueryParams>,
    /// ) -> HttpResponse {
    ///     // Automatically converts to QueryParams with filter operators
    ///     let params: QueryParams<Product> = flat_params.into();
    ///     
    ///     // URL: /products?price[gt]=100&status[ne]=deleted
    ///     // Automatically creates:
    ///     // - price filter with GreaterThan operator
    ///     // - status filter with NotEqual operator
    ///     
    ///     // Build and execute query
    ///     // ...
    /// }
    /// ```
    ///
    /// With Axum:
    /// ```rust,ignore
    /// use axum::extract::Query;
    /// use sqlx_paginated::{FlatQueryParams, QueryParams};
    ///
    /// async fn list_products(
    ///     Query(flat_params): Query<FlatQueryParams>,
    /// ) -> Json<PaginatedResponse<Product>> {
    ///     let params: QueryParams<Product> = flat_params.into();
    ///     
    ///     // The operator syntax is automatically parsed:
    ///     // ?price[gte]=10&price[lte]=100 → price >= 10 AND price <= 100
    ///     // ?role[in]=admin,moderator → role IN ('admin', 'moderator')
    ///     // ?deleted_at[is_null]= → deleted_at IS NULL
    ///     
    ///     // Build and execute query
    ///     // ...
    /// }
    /// ```
    ///
    /// ## All Supported Operators
    ///
    /// | Operator Syntax | SQL Equivalent | Example |
    /// |----------------|----------------|---------|
    /// | `field=value` | `field = 'value'` | `status=active` |
    /// | `field[eq]=value` | `field = 'value'` | `status[eq]=active` |
    /// | `field[ne]=value` | `field != 'value'` | `status[ne]=deleted` |
    /// | `field[gt]=value` | `field > value` | `price[gt]=100` |
    /// | `field[gte]=value` | `field >= value` | `age[gte]=18` |
    /// | `field[lt]=value` | `field < value` | `stock[lt]=5` |
    /// | `field[lte]=value` | `field <= value` | `price[lte]=1000` |
    /// | `field[in]=v1,v2` | `field IN ('v1','v2')` | `role[in]=admin,mod` |
    /// | `field[not_in]=v1,v2` | `field NOT IN ('v1','v2')` | `status[not_in]=banned` |
    /// | `field[is_null]=` | `field IS NULL` | `deleted_at[is_null]=` |
    /// | `field[is_not_null]=` | `field IS NOT NULL` | `email[is_not_null]=` |
    /// | `field[like]=pattern` | `field LIKE 'pattern'` | `name[like]=%phone%` |
    /// | `field[not_like]=pattern` | `field NOT LIKE 'pattern'` | `email[not_like]=%spam%` |
    pub fn example_automatic_operator_parsing() {
        // No code needed here - this demonstrates URL query string usage
        // The actual parsing happens automatically when using FlatQueryParams
        // in your web framework handlers
    }
}

#[cfg(feature = "sqlite")]
pub mod sqlite_examples {
    use crate::{QueryBuilder, QueryFilterOperator, QueryParamsBuilder};
    use serde::Serialize;
    use sqlx::Sqlite;

    #[derive(Serialize, Default)]
    pub struct Product {
        pub id: i64,
        pub name: String,
        pub price: f64,
        pub stock: i32,
        pub status: String,
    }

    /// Example: Using filter operators with SQLite
    ///
    /// The same API works across different databases.
    /// SQLite uses ? placeholders instead of $1, $2, etc.
    pub fn example_sqlite_filtering() {
        let params = QueryParamsBuilder::<Product>::new()
            .with_filter_operator("price", QueryFilterOperator::GreaterThan, "10.00")
            .with_filter_in("status", vec!["active", "pending"])
            .with_filter_null("deleted_at", true)
            .build();

        // Generates: WHERE "price" > ? AND "status" IN (?, ?) AND "deleted_at" IS NULL
        let (_conditions, _args) = QueryBuilder::<Product, Sqlite>::new()
            .with_filters(&params)
            .build();
    }
}
