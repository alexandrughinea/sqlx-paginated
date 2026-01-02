use actix_web::{web, App, HttpResponse, HttpServer};
use log::info;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{Sqlite, SqlitePool, SqlitePoolOptions};
use sqlx::FromRow;
use sqlx_paginated::{paginated_query_as, FlatQueryParams, PaginatedResponse};

const DATABASE_URL: &str = "sqlite://test_database.db";
const BIND_ADDRESS: &str = "127.0.0.1:8080";
const MAX_CONNECTIONS: u32 = 5;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Default)]
struct User {
    id: String,
    first_name: String,
    last_name: String,
    email: String,
    confirmed: bool,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Default)]
struct Product {
    id: String,
    name: String,
    description: String,
    price: f64,
    stock: i32,
    category: String,
    status: String,
    created_at: String,
}

struct AppState {
    db: SqlitePool,
}

async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    info!("Running database migrations...");
    sqlx::migrate!("./migrations").run(pool).await?;
    info!("Migrations completed successfully");
    Ok(())
}

fn log_pagination_info<T>(resource: &str, result: &PaginatedResponse<T>) {
    let page = result.pagination.as_ref().map(|p| p.page).unwrap_or(1);
    let total_pages = result.total_pages.unwrap_or(1);
    info!(
        "Returning {} {} (page {}/{})",
        result.records.len(),
        resource,
        page,
        total_pages
    );
}

fn error_response(message: &str, error: sqlx::Error) -> HttpResponse {
    log::error!("{}: {}", message, error);
    HttpResponse::InternalServerError().json(serde_json::json!({
        "error": message
    }))
}

async fn list_users(
    query: web::Query<FlatQueryParams>,
    state: web::Data<AppState>,
) -> HttpResponse {
    info!("GET /api/users");

    match paginated_query_as::<User, Sqlite>("SELECT * FROM users")
        .with_params(query.into_inner())
        .fetch_paginated(&state.db)
        .await
    {
        Ok(result) => {
            log_pagination_info("users", &result);
            HttpResponse::Ok().json(result)
        }
        Err(e) => error_response("Failed to fetch users", e),
    }
}

async fn list_products(
    query: web::Query<FlatQueryParams>,
    state: web::Data<AppState>,
) -> HttpResponse {
    info!("GET /api/products");

    match paginated_query_as::<Product, Sqlite>("SELECT * FROM products")
        .with_params(query.into_inner())
        .fetch_paginated(&state.db)
        .await
    {
        Ok(result) => {
            log_pagination_info("products", &result);
            HttpResponse::Ok().json(result)
        }
        Err(e) => error_response("Failed to fetch products", e),
    }
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "message": "sqlx-paginated-sqlite-example API is running"
    }))
}

async fn index() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "message": "sqlx-paginated-sqlite-example API",
        "endpoints": {
            "health": "GET /health",
            "users": "GET /api/users",
            "products": "GET /api/products"
        },
        "examples": {
            "pagination": "/api/users?page=1&page_size=5",
            "search": "/api/users?search=john&search_columns=first_name,last_name,email",
            "filter": "/api/users?confirmed=true",
            "advanced_filter": "/api/products?price[gte]=50&price[lte]=200&stock[gt]=0",
            "filter_in": "/api/products?category[in]=computers,electronics",
            "filter_null": "/api/products?status[is_not_null]=",
            "sort": "/api/users?sort_column=created_at&sort_direction=descending",
            "combined": "/api/products?search=laptop&category=computers&price[gte]=100&sort_column=price&page=1&page_size=10"
        }
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    info!("Starting SQLx-Paginated SQLite Test API");

    let pool = SqlitePoolOptions::new()
        .max_connections(MAX_CONNECTIONS)
        .connect(DATABASE_URL)
        .await
        .expect("Failed to connect to database");

    info!("Connected to database: {}", DATABASE_URL);

    run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    let app_state = web::Data::new(AppState { db: pool });

    info!("Starting server at http://{}", BIND_ADDRESS);

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/", web::get().to(index))
            .route("/health", web::get().to(health_check))
            .route("/api/users", web::get().to(list_users))
            .route("/api/products", web::get().to(list_products))
    })
    .bind(BIND_ADDRESS)?
    .run()
    .await
}
