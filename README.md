# sqlx-paginated

[![Rust](https://github.com/alexandrughinea/sqlx-paginated/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/alexandrughinea/sqlx-paginated/actions/workflows/rust.yml)
[![crates.io](https://img.shields.io/crates/v/sqlx-paginated.svg)](https://crates.io/crates/sqlx-paginated)
[![docs](https://docs.rs/sqlx-paginated/badge.svg)](https://docs.rs/sqlx-paginated/latest/sqlx_paginated/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A blazingly fast, type-safe, fluid query builder for dynamic APIs, offering seamless pagination, sorting and dynamic filtering on top of [SQLx](https://docs.rs/sqlx/latest/sqlx).

## Table of contents
- [Features](#features)
  - [Core capabilities](#core-capabilities)
  - [Technical features](#technical-features)
  - [Query features](#query-features)
- [Database support](#database-support)
  - [Current vs planned support](#current-vs-planned-support)
- [Market analysis](#market-analysis)
  - [Ecosystem gaps](#ecosystem-gaps)
  - [Unique selling points](#unique-selling-points)
  - [Target audience](#target-audience)
- [Installation](#installation)
- [Quick start](#quick-start)
  - [Basic usage](#basic-usage)
  - [Examples](#examples)
  - [Response example](#response-example)
- [API reference](#api-reference)
  - [Pagination parameters](#pagination-parameters)
  - [Sort parameters](#sort-parameters)
  - [Search parameters](#search-parameters)
  - [Date range parameters](#date-range-parameters)
  - [Filtering parameters](#filtering-parameters)
- [Query examples](#query-examples)
  - [Combined search, sort, date range, pagination and filter](#combined-search-sort-date-range-pagination-and-custom-filter)
  - [Date range combined with two other filters](#date-range-filter-combined-with-two-other-custom-filters)
- [Performance considerations](#performance-considerations)
  - [Query pattern optimization](#query-pattern-optimization)
  - [Recommended indexes](#recommended-indexes)
  - [Pagination performance](#pagination-performance)
- [Security features](#security-features)
  - [Input sanitization](#input-sanitization)
  - [Protected patterns](#protected-patterns)
- [Contributing](#contributing)
- [License](#license)

## Features

### Core capabilities
- Full-text search with multi-column specification
- Smart pagination with customizable page size
- Dynamic sorting on any column
- Flexible filtering system 
- Date range filtering
- Type-safe operations
- High performance
- SQL injection protection

### Technical features
- Builder patterns for query parameters and query construction
- Graceful error handling
- Logging with tracing (if enabled)
- Macro and function syntax support

### Query features
- Case-insensitive search
- Multiple column search
- Complex filtering conditions
- Date-based filtering
- Dynamic sort direction
- Customizable page size
- Result count optimization (opt-out of total records lookup ahead)

## Database support

### Current vs planned support
| Database    | Status   | Version | Features                 | Notes                                       |
|-------------|----------|---------|--------------------------|---------------------------------------------|
| PostgreSQL  | Supported | 12+     | All features supported   | Stable                                      |
| SQLite      | Testing  | 3.35+   | All features supported | Stable                                     |
| MySQL       | Planned | 8.0+    | Core features planned    | On roadmap, development starting in Q2 2026 |


## Market analysis

### Ecosystem gaps
1. **Query builders**
   - Diesel: Full ORM, can be heavyweight
   - SeaQuery: Generic and can be verbose
   - sqlbuilder: Basic SQL building without pagination or security

2. **Missing features in existing solutions**
   - Easy integration with web frameworks
   - Automatic type casting
   - Typesafe search/filter/sort/pagination capabilities

### Unique selling points

1. **Quick web framework integration with minimal footprint**

[Actix-web](https://actix.rs/) handler example:
```rust
use sqlx::Postgres;
use sqlx_paginated::{paginated_query_as, FlatQueryParams};
use actix_web::{web, Responder, HttpResponse};

async fn list_users(web::Query(params): web::Query<FlatQueryParams>) -> impl Responder {
    let paginated_users = paginated_query_as::<User, Postgres>("SELECT * FROM users")
        .with_params(params)
        .fetch_paginated(&pool)
        .await
        .unwrap();
    
    HttpResponse::Ok().json(json!(paginated_users))
}
```

2. **Type safety & ergonomics for parameter configuration**
```rust
let params = QueryParamsBuilder::<User>::new()
    .with_pagination(1, 10)
    .with_sort("created_at", QuerySortDirection::Descending)
    .with_search("john", vec!["name", "email"])
    .build();
```

3. **Advanced builder patterns**
- Optional fluent API for query parameters (QueryParams) which allow defining search, search location, date filtering, ordering, and custom filtering.
- Fluent API for the entire supported feature set, more here: [advanced example](src/paginated_query_as/examples/paginated_query_builder_advanced_examples.rs)

```rust
    paginated_query_as::<UserExample, Postgres>("SELECT * FROM users")
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
                   // ...
                .build()
        })
        .disable_totals_count() // Disables the calculation of total record count
        .fetch_paginated(&pool)
        .await
        .unwrap()
```


### Target audience
1. **Primary users**
   - Rust web developers or API teams
   - Teams needing quick and secure query building
   - Projects requiring pagination and dynamic filtering APIs
   - SQLx users wanting higher-level abstractions for repetitive tasks

2. **Use cases**
   - REST APIs with pagination
   - Complex backend filtering
   - Admin panels
   - Data exploration interfaces

## Installation

Add to `Cargo.toml`:

**For PostgreSQL:**
```toml
[dependencies]
sqlx_paginated = { version = "0.2.33", features = ["postgres"] }
```

**For SQLite:**
```toml
[dependencies]
sqlx_paginated = { version = "0.2.33", features = ["sqlite"] }
```

**For both:**
```toml
[dependencies]
sqlx_paginated = { version = "0.2.33", features = ["postgres", "sqlite"] }
```

## Quick start

### Basic usage

**PostgreSQL:**
```rust
use sqlx::{PgPool, Postgres};
use sqlx_paginated::{QueryParamsBuilder, QuerySortDirection, paginated_query_as};

#[derive(sqlx::FromRow, serde::Serialize, Default)]
struct User {
    id: i64,
    first_name: String,
    last_name: String,
    email: String,
    confirmed: bool,
    created_at: Option<DateTime<Utc>>,
}

async fn get_users(pool: &PgPool) -> Result<PaginatedResponse<User>, sqlx::Error> {
    let params = QueryParamsBuilder::<User>::new()
        .with_pagination(1, 10)
        .with_sort("created_at", QuerySortDirection::Descending)
        .with_search("john", vec!["first_name", "last_name", "email"])
        .build();
    
    // Function syntax (recommended)
    paginated_query_as::<User, Postgres>("SELECT * FROM users")
        .with_params(params)
        .fetch_paginated(pool)
        .await
    
    // Or using macro syntax
    // paginated_query_as!(User, Postgres, "SELECT * FROM users")
}
```

**SQLite:**
```rust
use sqlx::{SqlitePool, Sqlite};
use sqlx_paginated::{QueryParamsBuilder, QuerySortDirection, paginated_query_as};

async fn get_users(pool: &SqlitePool) -> Result<PaginatedResponse<User>, sqlx::Error> {
    let params = QueryParamsBuilder::<User>::new()
        .with_pagination(1, 10)
        .with_sort("created_at", QuerySortDirection::Descending)
        .with_search("john", vec!["first_name", "last_name", "email"])
        .build();
    
    // Function syntax (recommended)
    paginated_query_as::<User, Sqlite>("SELECT * FROM users")
        .with_params(params)
        .fetch_paginated(pool)
        .await
    
    // Or using macro syntax
    // paginated_query_as!(User, Sqlite, "SELECT * FROM users")
}
```

### Examples

#### Complete working example

See **[examples/sqlx-paginated-sqlite-example](examples/sqlx-paginated-sqlite-example/)** for a complete working REST API demonstrating:
- Actix-web integration with SQLite
- User and product endpoints with full CRUD operations
- Migrations and seed data
- Bruno API collection for testing
- Production-ready error handling

Run with: `cd examples/sqlx-paginated-sqlite-example && cargo run`

#### Query building code examples

For detailed query building patterns, see **[src/paginated_query_as/examples](src/paginated_query_as/examples/)**:

- **[query_filters_examples.rs](src/paginated_query_as/examples/query_filters_examples.rs)** - Examples of all filter operators including comparison operators, IN/NOT IN, NULL checks, LIKE patterns, and complex filtering scenarios
- **[query_builder_examples.rs](src/paginated_query_as/examples/query_builder_examples.rs)** - Query builder examples showing safe defaults and custom query construction for PostgreSQL and SQLite
- **[paginated_query_builder_advanced_examples.rs](src/paginated_query_as/examples/paginated_query_builder_advanced_examples.rs)** - Advanced query builder usage with custom conditions and protection disabling

### Response example
```json
{
  "records": [
    {
      "id": "409e3900-c190-4dad-882d-ec2d40245329",
      "first_name": "John",
      "last_name": "Smith",
      "email": "john@example.com",
      "confirmed": true,
      "created_at": "2024-01-01T00:00:00Z"
    }
  ],
  "page": 1,
  "page_size": 10,
  "total_pages": 1
}
```

## API reference

### Parameter overview

| Feature | HTTP parameters | Builder method |
|---------|----------------|----------------|
| Pagination | `page`, `page_size` | `.with_pagination(page, size)` |
| Sorting | `sort_column`, `sort_direction` | `.with_sort(column, direction)` |
| Search | `search`, `search_columns` | `.with_search(term, columns)` |
| Date range | `date_after`, `date_before`, `date_column` | `.with_date_range(after, before, column)` |
| Filters | `field=value`, `field[op]=value` | `.with_filter_operator(field, operator, value)` |

### Pagination

| Parameter | Type | Default | Range | Description |
|-----------|------|---------|-------|-------------|
| `page` | integer | `1` | 1+ | Page number (1-indexed) |
| `page_size` | integer | `10` | 10-50 | Records per page |

```
GET /users?page=2&page_size=20
```

```rust
.with_pagination(2, 20)
```

### Sorting

| Parameter | Type | Default | Values | Description |
|-----------|------|---------|--------|-------------|
| `sort_column` | string | `created_at` | Any valid column | Column to sort by |
| `sort_direction` | string | `descending` | `ascending`, `descending` | Sort order |

```
GET /users?sort_column=last_name&sort_direction=ascending
```

```rust
use sqlx_paginated::QuerySortDirection;

.with_sort("last_name", QuerySortDirection::Ascending)
```

### Search

| Parameter | Type | Default | Constraint | Description |
|-----------|------|---------|------------|-------------|
| `search` | string | null | Max 100 chars | Search term (sanitized: alphanumeric, spaces, hyphens) |
| `search_columns` | string | `name,description` | Comma-separated | Columns to search |

```
GET /users?search=john&search_columns=first_name,last_name,email
```

```rust
.with_search("john", vec!["first_name", "last_name", "email"])
```

**Generated SQL:**
```sql
WHERE (LOWER("first_name") LIKE LOWER('%john%') 
    OR LOWER("last_name") LIKE LOWER('%john%')
    OR LOWER("email") LIKE LOWER('%john%'))
```

### Date range

| Parameter | Type | Default | Format | Description |
|-----------|------|---------|--------|-------------|
| `date_after` | datetime | null | ISO 8601 | Range start (>=) |
| `date_before` | datetime | null | ISO 8601 | Range end (<=) |
| `date_column` | string | `created_at` | Column name | Target column |

```
GET /users?date_after=2024-01-01T00:00:00Z&date_before=2024-12-31T23:59:59Z
```

```rust
use chrono::{DateTime, Utc};

let start = DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z").unwrap().into();
let end = DateTime::parse_from_rfc3339("2024-12-31T23:59:59Z").unwrap().into();

.with_date_range(Some(start), Some(end), Some("created_at"))
```

### Filter operators

#### Operator reference

| Operator | HTTP syntax | Rust method | SQL output |
|----------|-------------|-------------|------------|
| Equal | `field=value` | `.with_filter("field", Some("value"))` | `field = $1` |
| Not equal | `field[ne]=value` | `.with_filter_operator("field", NotEqual, "value")` | `field != $1` |
| Greater than | `field[gt]=value` | `.with_filter_operator("field", GreaterThan, "value")` | `field > $1` |
| Greater or equal | `field[gte]=value` | `.with_filter_operator("field", GreaterOrEqual, "value")` | `field >= $1` |
| Less than | `field[lt]=value` | `.with_filter_operator("field", LessThan, "value")` | `field < $1` |
| Less or equal | `field[lte]=value` | `.with_filter_operator("field", LessOrEqual, "value")` | `field <= $1` |
| IN | `field[in]=a,b,c` | `.with_filter_in("field", vec!["a","b","c"])` | `field IN ($1,$2,$3)` |
| NOT IN | `field[nin]=a,b` | `.with_filter_not_in("field", vec!["a","b"])` | `field NOT IN ($1,$2)` |
| Is null | `field[is_null]=` | `.with_filter_null("field", true)` | `field IS NULL` |
| Is not null | `field[is_not_null]=` | `.with_filter_null("field", false)` | `field IS NOT NULL` |
| LIKE | `field[like]=%pattern%` | `.with_filter_like("field", "%pattern%")` | `field LIKE $1` |
| Not like | `field[nlike]=%pattern%` | `.with_filter_not_like("field", "%pattern%")` | `field NOT LIKE $1` |

#### HTTP examples

```
GET /products?price[gte]=10&price[lte]=100
GET /users?role[in]=admin,moderator&status[ne]=banned
GET /users?deleted_at[is_null]=&email[is_not_null]=
GET /users?email[like]=%@company.com
```

#### Rust examples

```rust
use sqlx_paginated::{QueryParamsBuilder, QueryFilterOperator};

// Basic operators
QueryParamsBuilder::<Product>::new()
    .with_filter_operator("price", QueryFilterOperator::GreaterThan, "10.00")
    .with_filter_operator("stock", QueryFilterOperator::LessOrEqual, "100")
    .with_filter("status", Some("active"))
    .build()

// Convenience methods
QueryParamsBuilder::<User>::new()
    .with_filter_in("role", vec!["admin", "moderator"])
    .with_filter_null("deleted_at", true)
    .build()

// Using QueryFilterCondition
use sqlx_paginated::QueryFilterCondition;
use std::collections::HashMap;

let mut filters = HashMap::new();
filters.insert("price".to_string(), QueryFilterCondition::greater_than("50.00"));
filters.insert("status".to_string(), QueryFilterCondition::not_equal("deleted"));

QueryParamsBuilder::<Product>::new()
    .with_filter_conditions(filters)
    .build()
```

### Web framework integration

**[Actix-web](https://actix.rs/):**
```rust
use actix_web::{web::Query, HttpResponse};
use sqlx_paginated::{FlatQueryParams, paginated_query_as};
use sqlx::Postgres;

async fn list_users(
    Query(params): Query<FlatQueryParams>,
    pool: web::Data<PgPool>
) -> HttpResponse {
    let result = paginated_query_as::<User, Postgres>("SELECT * FROM users")
        .with_params(params)
        .fetch_paginated(pool.get_ref())
        .await
        .unwrap();
    
    HttpResponse::Ok().json(result)
}
```

**[Axum](https://docs.rs/axum/latest/axum/):**
```rust
use axum::{extract::Query, Json, Extension};
use sqlx_paginated::{FlatQueryParams, paginated_query_as, PaginatedResponse};
use sqlx::{PgPool, Postgres};

async fn list_users(
    Query(params): Query<FlatQueryParams>,
    Extension(pool): Extension<PgPool>
) -> Json<PaginatedResponse<User>> {
    let result = paginated_query_as::<User, Postgres>("SELECT * FROM users")
        .with_params(params)
        .fetch_paginated(&pool)
        .await
        .unwrap();
    
    Json(result)
}
```

### Complete example

**HTTP:**
```
GET /products?search=laptop&search_columns=name,description
    &price[gte]=500&price[lte]=2000&stock[gt]=0
    &category[in]=computers,electronics
    &status=active&deleted_at[is_null]=
    &sort_column=price&sort_direction=ascending
    &page=1&page_size=24
```

**Rust:**
```rust
use sqlx_paginated::{QueryParamsBuilder, QuerySortDirection, QueryFilterOperator};

let params = QueryParamsBuilder::<Product>::new()
    .with_search("laptop", vec!["name", "description"])
    .with_filter_operator("price", QueryFilterOperator::GreaterOrEqual, "500")
    .with_filter_operator("price", QueryFilterOperator::LessOrEqual, "2000")
    .with_filter_operator("stock", QueryFilterOperator::GreaterThan, "0")
    .with_filter_in("category", vec!["computers", "electronics"])
    .with_filter("status", Some("active"))
    .with_filter_null("deleted_at", true)
    .with_sort("price", QuerySortDirection::Ascending)
    .with_pagination(1, 24)
    .build();
```

## Query examples

- Given the following `struct`, we can then perform search and filtering
against its own fields. 
- We should also receive a paginated response back with the matching records.

```rust
#[derive(Serialize, Deserialize, FromRow, Default)]
pub struct User {
    pub id: Option<Uuid>,
    pub first_name: String,
    pub last_name: String,
    pub confirmed: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}
```

1. ### Combined search, sort, date range, pagination and custom filter

- Notice the `confirmed=true` filter.

#### Request:
```
GET /v1/internal/users
    ?search=john
    &search_columns=first_name,last_name,email
    &sort_column=created_at
    &sort_direction=descending
    &date_before=2024-11-03T12:30:12.081598Z
    &date_after=2024-11-02T12:30:12.081598Z
    &page=1
    &page_size=20
    &confirmed=true
```

#### Response:
```json
{
  "page": 1,
  "page_size": 20,
  "total": 2,
  "total_pages": 1,
  "records": [
    {
      "id": "409e3900-c190-4dad-882d-ec2d40245329",
      "first_name": "John",
      "last_name": "Smith",
      "email": "john.smith@example.com",
      "confirmed": true,
      "created_at": "2024-11-03T12:30:12.081598Z",
      "updated_at": "2024-11-03T12:30:12.081598Z"
    },
    {
      "id": "9167d825-8944-4428-bf91-3c5531728b5e",
      "first_name": "Johnny",
      "last_name": "Doe",
      "email": "johnny.doe@example.com",
      "confirmed": true,
      "created_at": "2024-10-28T19:14:49.064626Z",
      "updated_at": "2024-10-28T19:14:49.064626Z"
    }
  ]
}
```

2. ### Date range filter combined with two other custom filters

- Notice the `confirmed=true` and `first_name=Alex` filters.
- For the `first_name` filter the value will be an exact match (case-sensitive).
- You can extend your struct as you please while the query parameters will also be available automatically. 

#### Request:
```
GET /v1/internal/users
    ?date_before=2024-11-03T12:30:12.081598Z
    &date_after=2024-11-02T12:30:12.081598Z
    &confirmed=true
    &first_name=Alex
```

#### Response:
```json
{
  "page": 1,
  "page_size": 20,
  "total": 1,
  "total_pages": 1,
  "records": [
    {
      "id": "509e3900-c190-4dad-882d-ec2d40245329",
      "first_name": "Alex",
      "last_name": "Johnson",
      "email": "alex.johnson@example.com",
      "confirmed": true,
      "created_at": "2024-11-02T12:30:12.081598Z"
    }
  ]
}
```

## Performance considerations

### Query pattern optimization
| Query pattern | Impact | Recommendation |
|--------------|---------|----------------|
| SELECT * | High impact | Specify needed columns |
| Large text columns | High impact | Use separate detail endpoint |
| Computed columns | Medium impact | Cache if possible |
| JSON aggregation | Medium impact | Limit array size |

### Recommended indexes
```sql
-- Text search
CREATE INDEX idx_users_name_gin ON users USING gin(to_tsvector('english', name));

-- Composite indexes for common queries
CREATE INDEX idx_users_confirmed_created ON users(confirmed, created_at);

-- JSON indexes
CREATE INDEX idx_users_metadata ON users USING gin(metadata);
```

### Pagination performance
| Page size | Records | Performance impact |
|-----------|---------|-------------------|
| 1-10      | Optimal | Best           |
| 11-50     | Good    | Good           |
| 51-100    | Caution | Monitor        |
| 100+      | Poor    | Not recommended |


## Security features

### Input sanitization
- Search terms are cleaned and normalized
- Parameter input values are trimmed and/or clamped against their defaults
- Column names are validated against an allowlist:
  - The struct itself first;
  - Database specific table names second;
- SQL injection patterns are blocked
- System table access is prevented

### Protected patterns
- System schemas (pg_, information_schema)
- System columns (oid, xmin, etc.)
- SQL injection attempts
- Invalid characters in identifiers

## Contributing

I warmly welcome contributions from the community! 
If you have ideas, improvements, or fixes, we encourage you to submit a Pull Request. 
Your input is highly valued, and I'm excited to collaborate with you to make this project even better.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
