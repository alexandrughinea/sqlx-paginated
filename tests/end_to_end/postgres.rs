use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgPool, PgPoolOptions, Postgres};
use sqlx::FromRow;
use sqlx_paginated::{
    paginated_query_as, PaginatedResponse, QueryFilterOperator, QueryParamsBuilder,
    QuerySortDirection,
};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Default)]
struct TestUser {
    id: i32,
    first_name: String,
    last_name: String,
    email: String,
    confirmed: bool,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Default)]
struct TestProduct {
    id: i32,
    name: String,
    description: String,
    price: f64,
    stock: i32,
    category: String,
    status: String,
    created_at: DateTime<Utc>,
}

fn get_database_url() -> String {
    std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost:5432/sqlx_paginated_test".to_string()
    })
}

async fn setup_test_db() -> Result<PgPool, sqlx::Error> {
    let database_url = get_database_url();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let _ = sqlx::query("DROP TABLE IF EXISTS test_users CASCADE")
        .execute(&pool)
        .await;
    let _ = sqlx::query("DROP TABLE IF EXISTS test_products CASCADE")
        .execute(&pool)
        .await;
    let _ = sqlx::query("DROP TABLE IF EXISTS test_nulls CASCADE")
        .execute(&pool)
        .await;

    sqlx::query(
        r#"
        CREATE TABLE test_users (
            id SERIAL PRIMARY KEY,
            first_name VARCHAR(255) NOT NULL,
            last_name VARCHAR(255) NOT NULL,
            email VARCHAR(255) NOT NULL UNIQUE,
            confirmed BOOLEAN NOT NULL DEFAULT false,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE test_products (
            id SERIAL PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            description TEXT NOT NULL,
            price DOUBLE PRECISION NOT NULL,
            stock INTEGER NOT NULL,
            category VARCHAR(100) NOT NULL,
            status VARCHAR(50) NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query("CREATE INDEX idx_test_users_email ON test_users(email)")
        .execute(&pool)
        .await?;
    sqlx::query("CREATE INDEX idx_test_users_confirmed ON test_users(confirmed)")
        .execute(&pool)
        .await?;
    sqlx::query("CREATE INDEX idx_test_products_category ON test_products(category)")
        .execute(&pool)
        .await?;
    sqlx::query("CREATE INDEX idx_test_products_status ON test_products(status)")
        .execute(&pool)
        .await?;

    Ok(pool)
}

async fn seed_users(pool: &PgPool) -> Result<(), sqlx::Error> {
    let users = vec![
        ("John", "Smith", "john.smith@example.com", true),
        ("Jane", "Doe", "jane.doe@example.com", true),
        ("Johnny", "Appleseed", "johnny.appleseed@example.com", false),
        ("Alice", "Johnson", "alice.johnson@example.com", true),
        ("Bob", "Williams", "bob.williams@example.com", false),
        ("Charlie", "Brown", "charlie.brown@example.com", true),
        ("Diana", "Prince", "diana.prince@example.com", true),
        ("Eve", "Anderson", "eve.anderson@example.com", false),
    ];

    for (first_name, last_name, email, confirmed) in users {
        sqlx::query(
            "INSERT INTO test_users (first_name, last_name, email, confirmed) VALUES ($1, $2, $3, $4)",
        )
        .bind(first_name)
        .bind(last_name)
        .bind(email)
        .bind(confirmed)
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn seed_products(pool: &PgPool) -> Result<(), sqlx::Error> {
    let products = vec![
        (
            "Laptop Pro",
            "High-performance laptop",
            1299.99,
            15,
            "computers",
            "active",
        ),
        (
            "Wireless Mouse",
            "Ergonomic wireless mouse",
            29.99,
            50,
            "electronics",
            "active",
        ),
        (
            "Mechanical Keyboard",
            "RGB mechanical keyboard",
            149.99,
            30,
            "electronics",
            "active",
        ),
        (
            "USB-C Hub",
            "7-in-1 USB-C hub",
            49.99,
            100,
            "accessories",
            "active",
        ),
        (
            "Monitor 27\"",
            "4K UHD monitor",
            399.99,
            25,
            "computers",
            "active",
        ),
        (
            "Laptop Stand",
            "Adjustable aluminum stand",
            39.99,
            0,
            "accessories",
            "out_of_stock",
        ),
    ];

    for (name, desc, price, stock, category, status) in products {
        sqlx::query(
            "INSERT INTO test_products (name, description, price, stock, category, status) 
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(name)
        .bind(desc)
        .bind(price)
        .bind(stock)
        .bind(category)
        .bind(status)
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn cleanup_db(pool: &PgPool) {
    let _ = sqlx::query("DROP TABLE IF EXISTS test_users CASCADE")
        .execute(pool)
        .await;
    let _ = sqlx::query("DROP TABLE IF EXISTS test_products CASCADE")
        .execute(pool)
        .await;
    let _ = sqlx::query("DROP TABLE IF EXISTS test_nulls CASCADE")
        .execute(pool)
        .await;
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_basic_pagination() {
    let pool = setup_test_db().await.unwrap();
    seed_users(&pool).await.unwrap();

    let params = QueryParamsBuilder::<TestUser>::new()
        .with_pagination(1, 10)
        .build();

    let result: PaginatedResponse<TestUser> =
        paginated_query_as::<TestUser, Postgres>("SELECT * FROM test_users")
            .with_params(params)
            .fetch_paginated(&pool)
            .await
            .unwrap();

    assert_eq!(result.records.len(), 8);
    assert_eq!(result.pagination.as_ref().unwrap().page, 1);
    assert_eq!(result.pagination.as_ref().unwrap().page_size, 10);
    assert_eq!(result.total, Some(8));
    assert_eq!(result.total_pages, Some(1));

    cleanup_db(&pool).await;
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_pagination_second_page() {
    let pool = setup_test_db().await.unwrap();

    for i in 1..=25 {
        sqlx::query(
            "INSERT INTO test_users (first_name, last_name, email, confirmed) 
             VALUES ($1, $2, $3, $4)",
        )
        .bind(format!("First{}", i))
        .bind(format!("Last{}", i))
        .bind(format!("user{}@example.com", i))
        .bind(i % 2 == 0)
        .execute(&pool)
        .await
        .unwrap();
    }

    let params = QueryParamsBuilder::<TestUser>::new()
        .with_pagination(2, 10)
        .build();

    let result: PaginatedResponse<TestUser> =
        paginated_query_as::<TestUser, Postgres>("SELECT * FROM test_users")
            .with_params(params)
            .fetch_paginated(&pool)
            .await
            .unwrap();

    assert_eq!(result.records.len(), 10);
    assert_eq!(result.pagination.as_ref().unwrap().page, 2);
    assert_eq!(result.total, Some(25));
    assert_eq!(result.total_pages, Some(3));

    cleanup_db(&pool).await;
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_sort_ascending() {
    let pool = setup_test_db().await.unwrap();
    seed_users(&pool).await.unwrap();

    let params = QueryParamsBuilder::<TestUser>::new()
        .with_sort("first_name", QuerySortDirection::Ascending)
        .build();

    let result: PaginatedResponse<TestUser> =
        paginated_query_as::<TestUser, Postgres>("SELECT * FROM test_users")
            .with_params(params)
            .fetch_paginated(&pool)
            .await
            .unwrap();

    assert_eq!(result.records[0].first_name, "Alice");
    assert_eq!(result.records[1].first_name, "Bob");

    cleanup_db(&pool).await;
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_sort_descending() {
    let pool = setup_test_db().await.unwrap();
    seed_users(&pool).await.unwrap();

    let params = QueryParamsBuilder::<TestUser>::new()
        .with_sort("first_name", QuerySortDirection::Descending)
        .build();

    let result: PaginatedResponse<TestUser> =
        paginated_query_as::<TestUser, Postgres>("SELECT * FROM test_users")
            .with_params(params)
            .fetch_paginated(&pool)
            .await
            .unwrap();

    assert_eq!(result.records[0].first_name, "Johnny");
    assert_eq!(result.records[1].first_name, "John");

    cleanup_db(&pool).await;
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_search_single_column() {
    let pool = setup_test_db().await.unwrap();
    seed_users(&pool).await.unwrap();

    let params = QueryParamsBuilder::<TestUser>::new()
        .with_search("john", vec!["first_name"])
        .build();

    let result: PaginatedResponse<TestUser> =
        paginated_query_as::<TestUser, Postgres>("SELECT * FROM test_users")
            .with_params(params)
            .fetch_paginated(&pool)
            .await
            .unwrap();

    assert_eq!(result.records.len(), 2);
    assert!(result.records[0].first_name.to_lowercase().contains("john"));

    cleanup_db(&pool).await;
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_search_multiple_columns() {
    let pool = setup_test_db().await.unwrap();
    seed_users(&pool).await.unwrap();

    let params = QueryParamsBuilder::<TestUser>::new()
        .with_search("smith", vec!["first_name", "last_name", "email"])
        .build();

    let result: PaginatedResponse<TestUser> =
        paginated_query_as::<TestUser, Postgres>("SELECT * FROM test_users")
            .with_params(params)
            .fetch_paginated(&pool)
            .await
            .unwrap();

    assert_eq!(result.records.len(), 1);
    assert_eq!(result.records[0].last_name, "Smith");

    cleanup_db(&pool).await;
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_search_case_insensitive() {
    let pool = setup_test_db().await.unwrap();
    seed_users(&pool).await.unwrap();

    let params = QueryParamsBuilder::<TestUser>::new()
        .with_search("JOHN", vec!["first_name"])
        .build();

    let result: PaginatedResponse<TestUser> =
        paginated_query_as::<TestUser, Postgres>("SELECT * FROM test_users")
            .with_params(params)
            .fetch_paginated(&pool)
            .await
            .unwrap();

    assert_eq!(result.records.len(), 2);

    cleanup_db(&pool).await;
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_filter_equality() {
    let pool = setup_test_db().await.unwrap();
    seed_users(&pool).await.unwrap();

    let params = QueryParamsBuilder::<TestUser>::new()
        .with_filter("confirmed", Some("true"))
        .build();

    let result: PaginatedResponse<TestUser> =
        paginated_query_as::<TestUser, Postgres>("SELECT * FROM test_users")
            .with_params(params)
            .fetch_paginated(&pool)
            .await
            .unwrap();

    assert_eq!(result.records.len(), 5);
    assert!(result.records.iter().all(|u| u.confirmed));

    cleanup_db(&pool).await;
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_filter_greater_than() {
    let pool = setup_test_db().await.unwrap();
    seed_products(&pool).await.unwrap();

    let params = QueryParamsBuilder::<TestProduct>::new()
        .with_filter_operator("price", QueryFilterOperator::GreaterThan, "100.0")
        .build();

    let result: PaginatedResponse<TestProduct> =
        paginated_query_as::<TestProduct, Postgres>("SELECT * FROM test_products")
            .with_params(params)
            .fetch_paginated(&pool)
            .await
            .unwrap();

    assert!(result.records.len() > 0);
    assert!(result.records.iter().all(|p| p.price > 100.0));

    cleanup_db(&pool).await;
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_filter_in_operator() {
    let pool = setup_test_db().await.unwrap();
    seed_products(&pool).await.unwrap();

    let params = QueryParamsBuilder::<TestProduct>::new()
        .with_filter_in("category", vec!["computers", "electronics"])
        .build();

    let result: PaginatedResponse<TestProduct> =
        paginated_query_as::<TestProduct, Postgres>("SELECT * FROM test_products")
            .with_params(params)
            .fetch_paginated(&pool)
            .await
            .unwrap();

    assert!(result.records.len() >= 4);
    assert!(result
        .records
        .iter()
        .all(|p| p.category == "computers" || p.category == "electronics"));

    cleanup_db(&pool).await;
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_filter_between_range() {
    let pool = setup_test_db().await.unwrap();
    seed_products(&pool).await.unwrap();

    let params = QueryParamsBuilder::<TestProduct>::new()
        .with_filter_operator("price", QueryFilterOperator::GreaterOrEqual, "50.0")
        .build();

    let result: PaginatedResponse<TestProduct> =
        paginated_query_as::<TestProduct, Postgres>("SELECT * FROM test_products")
            .with_params(params)
            .fetch_paginated(&pool)
            .await
            .unwrap();

    assert!(result.records.len() > 0);
    assert!(result.records.iter().all(|p| p.price >= 50.0));

    cleanup_db(&pool).await;
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_filter_null_check() {
    let pool = setup_test_db().await.unwrap();

    sqlx::query(
        "CREATE TABLE test_nulls (
            id SERIAL PRIMARY KEY,
            name VARCHAR(255),
            deleted_at TIMESTAMPTZ
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query("INSERT INTO test_nulls (name, deleted_at) VALUES ($1, $2)")
        .bind("Active Item")
        .bind(Option::<DateTime<Utc>>::None)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO test_nulls (name, deleted_at) VALUES ($1, $2)")
        .bind("Deleted Item")
        .bind(Some(Utc::now()))
        .execute(&pool)
        .await
        .unwrap();

    #[derive(Debug, Serialize, FromRow, Default)]
    struct TestNull {
        id: i32,
        name: String,
        deleted_at: Option<DateTime<Utc>>,
    }

    let params = QueryParamsBuilder::<TestNull>::new()
        .with_filter_null("deleted_at", true)
        .with_sort("id", QuerySortDirection::Ascending)
        .build();

    let result: PaginatedResponse<TestNull> =
        paginated_query_as::<TestNull, Postgres>("SELECT * FROM test_nulls")
            .with_params(params)
            .fetch_paginated(&pool)
            .await
            .unwrap();

    assert_eq!(result.records.len(), 1);
    assert!(result.records[0].deleted_at.is_none());

    cleanup_db(&pool).await;
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_date_range_filter() {
    let pool = setup_test_db().await.unwrap();
    seed_users(&pool).await.unwrap();

    let now = Utc::now();
    let one_hour_ago = now - chrono::Duration::hours(1);
    let one_hour_later = now + chrono::Duration::hours(1);

    let params = QueryParamsBuilder::<TestUser>::new()
        .with_date_range(Some(one_hour_ago), Some(one_hour_later), Some("created_at"))
        .build();

    let result: PaginatedResponse<TestUser> =
        paginated_query_as::<TestUser, Postgres>("SELECT * FROM test_users")
            .with_params(params)
            .fetch_paginated(&pool)
            .await
            .unwrap();

    assert_eq!(result.records.len(), 8);

    cleanup_db(&pool).await;
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_combined_search_filter_sort_pagination() {
    let pool = setup_test_db().await.unwrap();
    seed_products(&pool).await.unwrap();

    let params = QueryParamsBuilder::<TestProduct>::new()
        .with_search("laptop", vec!["name", "description"])
        .with_filter("status", Some("active"))
        .with_filter_operator("price", QueryFilterOperator::GreaterOrEqual, "50.0")
        .with_sort("price", QuerySortDirection::Ascending)
        .with_pagination(1, 10)
        .build();

    let result: PaginatedResponse<TestProduct> =
        paginated_query_as::<TestProduct, Postgres>("SELECT * FROM test_products")
            .with_params(params)
            .fetch_paginated(&pool)
            .await
            .unwrap();

    assert!(result.records.len() > 0);
    assert!(result.records.iter().all(|p| p.status == "active"));
    assert!(result.records.iter().all(|p| p.price >= 50.0));

    for i in 1..result.records.len() {
        assert!(result.records[i].price >= result.records[i - 1].price);
    }

    cleanup_db(&pool).await;
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_complex_filtering_scenario() {
    let pool = setup_test_db().await.unwrap();
    seed_products(&pool).await.unwrap();

    let params = QueryParamsBuilder::<TestProduct>::new()
        .with_filter_in("category", vec!["computers", "electronics"])
        .with_filter_operator("stock", QueryFilterOperator::GreaterThan, "0")
        .with_filter_operator("price", QueryFilterOperator::GreaterOrEqual, "100.0")
        .with_sort("price", QuerySortDirection::Descending)
        .build();

    let result: PaginatedResponse<TestProduct> =
        paginated_query_as::<TestProduct, Postgres>("SELECT * FROM test_products")
            .with_params(params)
            .fetch_paginated(&pool)
            .await
            .unwrap();

    for product in &result.records {
        assert!(product.category == "computers" || product.category == "electronics");
        assert!(product.stock > 0);
        assert!(product.price >= 100.0);
    }

    cleanup_db(&pool).await;
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_sql_injection_attempt_in_search() {
    let pool = setup_test_db().await.unwrap();
    seed_users(&pool).await.unwrap();

    let params = QueryParamsBuilder::<TestUser>::new()
        .with_search("'; DROP TABLE test_users; --", vec!["first_name"])
        .build();

    let result = paginated_query_as::<TestUser, Postgres>("SELECT * FROM test_users")
        .with_params(params)
        .fetch_paginated(&pool)
        .await;

    assert!(result.is_ok());

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM test_users")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count.0, 8);

    cleanup_db(&pool).await;
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_sql_injection_attempt_in_filter() {
    let pool = setup_test_db().await.unwrap();
    seed_users(&pool).await.unwrap();

    let params = QueryParamsBuilder::<TestUser>::new()
        .with_filter("first_name", Some("John' OR '1'='1"))
        .build();

    let result = paginated_query_as::<TestUser, Postgres>("SELECT * FROM test_users")
        .with_params(params)
        .fetch_paginated(&pool)
        .await;

    assert!(result.is_ok());

    if let Ok(res) = result {
        assert!(res.records.len() < 8);
    }

    cleanup_db(&pool).await;
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_empty_table() {
    let pool = setup_test_db().await.unwrap();

    let params = QueryParamsBuilder::<TestUser>::new().build();

    let result: PaginatedResponse<TestUser> =
        paginated_query_as::<TestUser, Postgres>("SELECT * FROM test_users")
            .with_params(params)
            .fetch_paginated(&pool)
            .await
            .unwrap();

    assert_eq!(result.records.len(), 0);
    assert_eq!(result.total, Some(0));
    assert_eq!(result.total_pages, Some(0));

    cleanup_db(&pool).await;
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_disable_totals_count() {
    let pool = setup_test_db().await.unwrap();
    seed_users(&pool).await.unwrap();

    let params = QueryParamsBuilder::<TestUser>::new()
        .with_pagination(1, 10)
        .build();

    let result: PaginatedResponse<TestUser> =
        paginated_query_as::<TestUser, Postgres>("SELECT * FROM test_users")
            .with_params(params)
            .disable_totals_count()
            .fetch_paginated(&pool)
            .await
            .unwrap();

    assert!(result.records.len() > 0);
    assert_eq!(result.total, None);
    assert_eq!(result.total_pages, None);

    cleanup_db(&pool).await;
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_large_result_set() {
    let pool = setup_test_db().await.unwrap();

    for i in 1..=100 {
        sqlx::query(
            "INSERT INTO test_users (first_name, last_name, email, confirmed) 
             VALUES ($1, $2, $3, $4)",
        )
        .bind(format!("First{}", i))
        .bind(format!("Last{}", i))
        .bind(format!("user{}@example.com", i))
        .bind(i % 2 == 0)
        .execute(&pool)
        .await
        .unwrap();
    }

    let params = QueryParamsBuilder::<TestUser>::new()
        .with_pagination(1, 50)
        .build();

    let result: PaginatedResponse<TestUser> =
        paginated_query_as::<TestUser, Postgres>("SELECT * FROM test_users")
            .with_params(params)
            .fetch_paginated(&pool)
            .await
            .unwrap();

    assert_eq!(result.records.len(), 50);
    assert_eq!(result.total, Some(100));
    assert_eq!(result.total_pages, Some(2));

    cleanup_db(&pool).await;
}
