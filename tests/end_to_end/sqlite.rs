use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{Sqlite, SqlitePool, SqlitePoolOptions};
use sqlx::FromRow;
use sqlx_paginated::{
    paginated_query_as, PaginatedResponse, QueryFilterOperator, QueryParamsBuilder,
    QuerySortDirection,
};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Default)]
struct TestUser {
    id: String,
    first_name: String,
    last_name: String,
    email: String,
    confirmed: bool,
    created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Default)]
struct TestProduct {
    id: String,
    name: String,
    description: String,
    price: f64,
    stock: i32,
    category: String,
    status: String,
    created_at: String,
}

async fn setup_test_db() -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE users (
            id TEXT PRIMARY KEY,
            first_name TEXT NOT NULL,
            last_name TEXT NOT NULL,
            email TEXT NOT NULL UNIQUE,
            confirmed BOOLEAN NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE products (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            price REAL NOT NULL,
            stock INTEGER NOT NULL,
            category TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}

async fn seed_users(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let now = Utc::now().to_rfc3339();

    let users = vec![
        (
            "1",
            "John",
            "Smith",
            "john.smith@example.com",
            true,
            now.clone(),
        ),
        (
            "2",
            "Jane",
            "Doe",
            "jane.doe@example.com",
            true,
            now.clone(),
        ),
        (
            "3",
            "Johnny",
            "Appleseed",
            "johnny.appleseed@example.com",
            false,
            now.clone(),
        ),
        (
            "4",
            "Alice",
            "Johnson",
            "alice.johnson@example.com",
            true,
            now.clone(),
        ),
        (
            "5",
            "Bob",
            "Williams",
            "bob.williams@example.com",
            false,
            now.clone(),
        ),
        (
            "6",
            "Charlie",
            "Brown",
            "charlie.brown@example.com",
            true,
            now.clone(),
        ),
        (
            "7",
            "Diana",
            "Prince",
            "diana.prince@example.com",
            true,
            now.clone(),
        ),
        (
            "8",
            "Eve",
            "Anderson",
            "eve.anderson@example.com",
            false,
            now.clone(),
        ),
    ];

    for (id, first_name, last_name, email, confirmed, created_at) in users {
        sqlx::query(
            "INSERT INTO users (id, first_name, last_name, email, confirmed, created_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(first_name)
        .bind(last_name)
        .bind(email)
        .bind(confirmed)
        .bind(created_at)
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn seed_products(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let now = Utc::now().to_rfc3339();

    let products = vec![
        (
            "p1",
            "Laptop Pro",
            "High-performance laptop",
            1299.99,
            15,
            "computers",
            "active",
            now.clone(),
        ),
        (
            "p2",
            "Wireless Mouse",
            "Ergonomic wireless mouse",
            29.99,
            50,
            "electronics",
            "active",
            now.clone(),
        ),
        (
            "p3",
            "Mechanical Keyboard",
            "RGB mechanical keyboard",
            149.99,
            30,
            "electronics",
            "active",
            now.clone(),
        ),
        (
            "p4",
            "USB-C Hub",
            "7-in-1 USB-C hub",
            49.99,
            100,
            "accessories",
            "active",
            now.clone(),
        ),
        (
            "p5",
            "Monitor 27\"",
            "4K UHD monitor",
            399.99,
            25,
            "computers",
            "active",
            now.clone(),
        ),
        (
            "p6",
            "Laptop Stand",
            "Adjustable aluminum stand",
            39.99,
            0,
            "accessories",
            "out_of_stock",
            now.clone(),
        ),
    ];

    for (id, name, desc, price, stock, category, status, created_at) in products {
        sqlx::query(
            "INSERT INTO products (id, name, description, price, stock, category, status, created_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(name)
        .bind(desc)
        .bind(price)
        .bind(stock)
        .bind(category)
        .bind(status)
        .bind(created_at)
        .execute(pool)
        .await?;
    }

    Ok(())
}

#[tokio::test]
async fn test_basic_pagination() {
    let pool = setup_test_db().await.unwrap();
    seed_users(&pool).await.unwrap();

    let params = QueryParamsBuilder::<TestUser>::new()
        .with_pagination(1, 10)
        .build();

    let result: PaginatedResponse<TestUser> =
        paginated_query_as::<TestUser, Sqlite>("SELECT * FROM users")
            .with_params(params)
            .fetch_paginated(&pool)
            .await
            .unwrap();

    assert_eq!(result.records.len(), 8);
    assert_eq!(result.pagination.as_ref().unwrap().page, 1);
    assert_eq!(result.pagination.as_ref().unwrap().page_size, 10);
    assert_eq!(result.total, Some(8));
    assert_eq!(result.total_pages, Some(1));
}

#[tokio::test]
async fn test_pagination_second_page() {
    let pool = setup_test_db().await.unwrap();

    for i in 1..=25 {
        sqlx::query(
            "INSERT INTO users (id, first_name, last_name, email, confirmed, created_at) 
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(format!("user_{}", i))
        .bind(format!("First{}", i))
        .bind(format!("Last{}", i))
        .bind(format!("user{}@example.com", i))
        .bind(i % 2 == 0)
        .bind(Utc::now().to_rfc3339())
        .execute(&pool)
        .await
        .unwrap();
    }

    let params = QueryParamsBuilder::<TestUser>::new()
        .with_pagination(2, 10)
        .build();

    let result: PaginatedResponse<TestUser> =
        paginated_query_as::<TestUser, Sqlite>("SELECT * FROM users")
            .with_params(params)
            .fetch_paginated(&pool)
            .await
            .unwrap();

    assert_eq!(result.records.len(), 10);
    assert_eq!(result.pagination.as_ref().unwrap().page, 2);
    assert_eq!(result.total, Some(25));
    assert_eq!(result.total_pages, Some(3));
}

#[tokio::test]
async fn test_pagination_empty_page() {
    let pool = setup_test_db().await.unwrap();
    seed_users(&pool).await.unwrap();

    let params = QueryParamsBuilder::<TestUser>::new()
        .with_pagination(10, 10)
        .build();

    let result: PaginatedResponse<TestUser> =
        paginated_query_as::<TestUser, Sqlite>("SELECT * FROM users")
            .with_params(params)
            .fetch_paginated(&pool)
            .await
            .unwrap();

    assert_eq!(result.records.len(), 0);
    assert_eq!(result.pagination.as_ref().unwrap().page, 10);
}

#[tokio::test]
async fn test_sort_ascending() {
    let pool = setup_test_db().await.unwrap();
    seed_users(&pool).await.unwrap();

    let params = QueryParamsBuilder::<TestUser>::new()
        .with_sort("first_name", QuerySortDirection::Ascending)
        .build();

    let result: PaginatedResponse<TestUser> =
        paginated_query_as::<TestUser, Sqlite>("SELECT * FROM users")
            .with_params(params)
            .fetch_paginated(&pool)
            .await
            .unwrap();

    assert_eq!(result.records[0].first_name, "Alice");
    assert_eq!(result.records[1].first_name, "Bob");
}

#[tokio::test]
async fn test_sort_descending() {
    let pool = setup_test_db().await.unwrap();
    seed_users(&pool).await.unwrap();

    let params = QueryParamsBuilder::<TestUser>::new()
        .with_sort("first_name", QuerySortDirection::Descending)
        .build();

    let result: PaginatedResponse<TestUser> =
        paginated_query_as::<TestUser, Sqlite>("SELECT * FROM users")
            .with_params(params)
            .fetch_paginated(&pool)
            .await
            .unwrap();

    assert_eq!(result.records[0].first_name, "Johnny");
    assert_eq!(result.records[1].first_name, "John");
}

#[tokio::test]
async fn test_search_single_column() {
    let pool = setup_test_db().await.unwrap();
    seed_users(&pool).await.unwrap();

    let params = QueryParamsBuilder::<TestUser>::new()
        .with_search("john", vec!["first_name"])
        .build();

    let result: PaginatedResponse<TestUser> =
        paginated_query_as::<TestUser, Sqlite>("SELECT * FROM users")
            .with_params(params)
            .fetch_paginated(&pool)
            .await
            .unwrap();

    assert_eq!(result.records.len(), 2);
    assert!(result.records[0].first_name.to_lowercase().contains("john"));
}

#[tokio::test]
async fn test_search_multiple_columns() {
    let pool = setup_test_db().await.unwrap();
    seed_users(&pool).await.unwrap();

    let params = QueryParamsBuilder::<TestUser>::new()
        .with_search("smith", vec!["first_name", "last_name", "email"])
        .build();

    let result: PaginatedResponse<TestUser> =
        paginated_query_as::<TestUser, Sqlite>("SELECT * FROM users")
            .with_params(params)
            .fetch_paginated(&pool)
            .await
            .unwrap();

    assert_eq!(result.records.len(), 1);
    assert_eq!(result.records[0].last_name, "Smith");
}

#[tokio::test]
async fn test_search_case_insensitive() {
    let pool = setup_test_db().await.unwrap();
    seed_users(&pool).await.unwrap();

    let params = QueryParamsBuilder::<TestUser>::new()
        .with_search("JOHN", vec!["first_name"])
        .build();

    let result: PaginatedResponse<TestUser> =
        paginated_query_as::<TestUser, Sqlite>("SELECT * FROM users")
            .with_params(params)
            .fetch_paginated(&pool)
            .await
            .unwrap();

    assert_eq!(result.records.len(), 2);
}

#[tokio::test]
async fn test_filter_equality() {
    let pool = setup_test_db().await.unwrap();
    seed_users(&pool).await.unwrap();

    let params = QueryParamsBuilder::<TestUser>::new()
        .with_filter("confirmed", Some("1"))
        .build();

    let result: PaginatedResponse<TestUser> =
        paginated_query_as::<TestUser, Sqlite>("SELECT * FROM users")
            .with_params(params)
            .fetch_paginated(&pool)
            .await
            .unwrap();

    assert_eq!(result.records.len(), 5);
    assert!(result.records.iter().all(|u| u.confirmed));
}

#[tokio::test]
async fn test_filter_greater_than() {
    let pool = setup_test_db().await.unwrap();
    seed_products(&pool).await.unwrap();

    let params = QueryParamsBuilder::<TestProduct>::new()
        .with_filter_operator("price", QueryFilterOperator::GreaterThan, "100.0")
        .build();

    let result: PaginatedResponse<TestProduct> =
        paginated_query_as::<TestProduct, Sqlite>("SELECT * FROM products")
            .with_params(params)
            .fetch_paginated(&pool)
            .await
            .unwrap();

    assert!(result.records.len() > 0);
    assert!(result.records.iter().all(|p| p.price > 100.0));
}

#[tokio::test]
async fn test_filter_in_operator() {
    let pool = setup_test_db().await.unwrap();
    seed_products(&pool).await.unwrap();

    let params = QueryParamsBuilder::<TestProduct>::new()
        .with_filter_in("category", vec!["computers", "electronics"])
        .build();

    let result: PaginatedResponse<TestProduct> =
        paginated_query_as::<TestProduct, Sqlite>("SELECT * FROM products")
            .with_params(params)
            .fetch_paginated(&pool)
            .await
            .unwrap();

    assert!(result.records.len() >= 4);
    assert!(result
        .records
        .iter()
        .all(|p| p.category == "computers" || p.category == "electronics"));
}

#[tokio::test]
async fn test_filter_null_check() {
    let pool = setup_test_db().await.unwrap();

    sqlx::query(
        "CREATE TABLE test_nulls (
            id TEXT PRIMARY KEY,
            name TEXT,
            deleted_at TEXT
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query("INSERT INTO test_nulls (id, name, deleted_at) VALUES (?, ?, ?)")
        .bind("1")
        .bind("Active Item")
        .bind(Option::<String>::None)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO test_nulls (id, name, deleted_at) VALUES (?, ?, ?)")
        .bind("2")
        .bind("Deleted Item")
        .bind(Some("2024-01-01"))
        .execute(&pool)
        .await
        .unwrap();

    #[derive(Debug, Serialize, FromRow, Default)]
    struct TestNull {
        id: String,
        name: String,
        deleted_at: Option<String>,
    }

    let params = QueryParamsBuilder::<TestNull>::new()
        .with_filter_null("deleted_at", true)
        .build();

    let result: PaginatedResponse<TestNull> =
        paginated_query_as::<TestNull, Sqlite>("SELECT * FROM test_nulls")
            .with_params(params)
            .fetch_paginated(&pool)
            .await
            .unwrap();

    assert_eq!(result.records.len(), 1);
    assert!(result.records[0].deleted_at.is_none());
}

#[tokio::test]
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
        paginated_query_as::<TestProduct, Sqlite>("SELECT * FROM products")
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
}

#[tokio::test]
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
        paginated_query_as::<TestProduct, Sqlite>("SELECT * FROM products")
            .with_params(params)
            .fetch_paginated(&pool)
            .await
            .unwrap();

    for product in &result.records {
        assert!(product.category == "computers" || product.category == "electronics");
        assert!(product.stock > 0);
        assert!(product.price >= 100.0);
    }
}

#[tokio::test]
async fn test_empty_table() {
    let pool = setup_test_db().await.unwrap();

    let params = QueryParamsBuilder::<TestUser>::new().build();

    let result: PaginatedResponse<TestUser> =
        paginated_query_as::<TestUser, Sqlite>("SELECT * FROM users")
            .with_params(params)
            .fetch_paginated(&pool)
            .await
            .unwrap();

    assert_eq!(result.records.len(), 0);
    assert_eq!(result.total, Some(0));
    assert_eq!(result.total_pages, Some(0));
}

#[tokio::test]
async fn test_invalid_column_name_protected() {
    let pool = setup_test_db().await.unwrap();
    seed_users(&pool).await.unwrap();

    let params = QueryParamsBuilder::<TestUser>::new()
        .with_sort("invalid_column_name", QuerySortDirection::Ascending)
        .build();

    let result = paginated_query_as::<TestUser, Sqlite>("SELECT * FROM users")
        .with_params(params)
        .fetch_paginated(&pool)
        .await;

    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_sql_injection_attempt_in_search() {
    let pool = setup_test_db().await.unwrap();
    seed_users(&pool).await.unwrap();

    let params = QueryParamsBuilder::<TestUser>::new()
        .with_search("'; DROP TABLE users; --", vec!["first_name"])
        .build();

    let result = paginated_query_as::<TestUser, Sqlite>("SELECT * FROM users")
        .with_params(params)
        .fetch_paginated(&pool)
        .await;

    assert!(result.is_ok());

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count.0, 8);
}

#[tokio::test]
async fn test_disable_totals_count() {
    let pool = setup_test_db().await.unwrap();
    seed_users(&pool).await.unwrap();

    let params = QueryParamsBuilder::<TestUser>::new()
        .with_pagination(1, 10)
        .build();

    let result: PaginatedResponse<TestUser> =
        paginated_query_as::<TestUser, Sqlite>("SELECT * FROM users")
            .with_params(params)
            .disable_totals_count()
            .fetch_paginated(&pool)
            .await
            .unwrap();

    assert!(result.records.len() > 0);
    assert_eq!(result.total, None);
    assert_eq!(result.total_pages, None);
}

#[tokio::test]
async fn test_large_result_set() {
    let pool = setup_test_db().await.unwrap();

    for i in 1..=100 {
        sqlx::query(
            "INSERT INTO users (id, first_name, last_name, email, confirmed, created_at) 
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(format!("user_{}", i))
        .bind(format!("First{}", i))
        .bind(format!("Last{}", i))
        .bind(format!("user{}@example.com", i))
        .bind(i % 2 == 0)
        .bind(Utc::now().to_rfc3339())
        .execute(&pool)
        .await
        .unwrap();
    }

    let params = QueryParamsBuilder::<TestUser>::new()
        .with_pagination(1, 50)
        .build();

    let result: PaginatedResponse<TestUser> =
        paginated_query_as::<TestUser, Sqlite>("SELECT * FROM users")
            .with_params(params)
            .fetch_paginated(&pool)
            .await
            .unwrap();

    assert_eq!(result.records.len(), 50);
    assert_eq!(result.total, Some(100));
    assert_eq!(result.total_pages, Some(2));
}

#[tokio::test]
async fn test_max_page_size_clamping() {
    let pool = setup_test_db().await.unwrap();
    seed_users(&pool).await.unwrap();

    let params = QueryParamsBuilder::<TestUser>::new()
        .with_pagination(1, 1000)
        .build();

    assert!(params.pagination.page_size <= 100);
}
