# sqlx-paginated

[![Rust](https://github.com/alexandrughinea/sqlx-paginated/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/alexandrughinea/sqlx-paginated/actions/workflows/rust.yml)
[![crates.io](https://img.shields.io/crates/v/sqlx-paginated.svg)](https://crates.io/crates/sqlx-paginated)
[![docs](https://docs.rs/sqlx-paginated/badge.svg)](https://docs.rs/sqlx-paginated/latest/sqlx_paginated/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A blazingly fast, type-safe, fluid query builder for dynamic APIs, offering seamless pagination, sorting and dynamic filtering on top of [SQLx](https://docs.rs/sqlx/latest/sqlx).

## Table of Contents
- [Features](#features)
  - [Core Capabilities](#core-capabilities)
  - [Technical Features](#technical-features)
  - [Query Features](#query-features)
- [Database Support](#database-support)
  - [Current vs Planned Support](#current-vs-planned-support)
- [Market Analysis](#market-analysis)
  - [Ecosystem Gaps](#ecosystem-gaps)
  - [Unique Selling Points](#unique-selling-points)
  - [Target Audience](#target-audience)
- [Installation](#installation)
- [Quick Start](#quick-start)
  - [Basic Usage](#basic-usage)
  - [Response Example](#response-example)
- [API Reference](#api-reference)
  - [Pagination Parameters](#pagination-parameters)
  - [Sort Parameters](#sort-parameters)
  - [Search Parameters](#search-parameters)
  - [Date Range Parameters](#date-range-parameters)
  - [Filtering Parameters](#filtering-parameters)
- [Query Examples](#query-examples)
  - [Combined search, sort, date range, pagination and filter](#combined-search-sort-date-range-pagination-and-custom-filter)
  - [Date Range combined with two other filters](#date-range-filter-combined-with-two-other-custom-filters)
- [Performance Considerations](#performance-considerations)
  - [Query Pattern Optimization](#query-pattern-optimization)
  - [Recommended Indexes](#recommended-indexes)
  - [Pagination Performance](#pagination-performance)
- [Security Features](#security-features)
  - [Input Sanitization](#input-sanitization)
  - [Protected Patterns](#protected-patterns)
- [Contributing](#contributing)
- [License](#license)

## Features

### Core Capabilities
- Full-text search with column specification
- Smart pagination with customizable page size
- Dynamic sorting on any column
- Flexible filtering system 
- Date range filtering
- Type-safe operations
- High performance
- SQL injection protection

### Technical Features
- Builder patterns for query parameters and query construction
- Graceful error handling
- Logging with tracing (if enabled)
- Macro and function syntax support

### Query Features
- Case-insensitive search
- Multiple column search
- Complex filtering conditions
- Date-based filtering
- Dynamic sort direction
- Customizable page size
- Result count optimization (opt-out of total records lookup ahead)

## Database Support

### Current vs Planned Support
| Database    | Status      | Version | Features                  | Notes                                       |
|-------------|-------------|---------|---------------------------|---------------------------------------------|
| PostgreSQL  | ✅ Supported | 12+     | All features supported    | Production ready                            |
| SQLite      | 🚧 Testing  | 3.35+   | All features being tested | Testing                                     |
| MySQL       | 🚧 Planned  | 8.0+    | Core features planned     | On roadmap, development starting in Q2 2026 |


## Market Analysis

### Ecosystem Gaps
1. **Query builders**
   - Diesel: Full ORM, can be heavyweight
   - SeaQuery: Generic and can be verbose
   - sqlbuilder: Basic SQL building without pagination or security

2. **Missing features in existing solutions**
   - Easy integration with web frameworks
   - Automatic type casting
   - Typesafe search/filter/sort/pagination capabilities

### Unique Selling Points

1. **Quick Web Framework Integration with minimal footprint**

Actix Web handler example:
```rust
use sqlx_paginated::{paginated_query_as, FlatQueryParams};
use actix_web::{web, Responder, HttpResponse};

async fn list_users(web::Query(params): web::Query<FlatQueryParams>) -> impl Responder {
    let paginated_users = paginated_query_as!(User, "SELECT * FROM users")
        .with_params(params)
        .fetch_paginated(&pool)
        .await
        .unwrap();
    
    HttpResponse::Ok().json(json!(paginated_users))
}
```

2. **Type Safety & Ergonomics for parameter configuration**
```rust
let params = QueryParamsBuilder::<User>::new()
    .with_pagination(1, 10)
    .with_sort("created_at", QuerySortDirection::Descending)
    .with_search("john", vec!["name", "email"])
    .build();
```

3. **Advanced Builder Patterns**
- Optional fluent API for query parameters (QueryParams) which allow defining search, search location, date filtering, ordering, and custom filtering.
- Fluent API for the entire supported feature set, more here: [advanced example](src/paginated_query_as/examples/paginated_query_builder_advanced_examples.rs)

```rust
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
                   // ...
                .build()
        })
        .disable_totals_count() // Disables the calculation of total record count
        .fetch_paginated(&pool)
        .await
        .unwrap()
```


### Target Audience
1. **Primary users**
   - Rust web developers or API teams
   - Teams needing quick and secure query building
   - Projects requiring pagination and dynamic filtering APIs
   - SQLx users wanting higher-level abstractions for repetitive tasks

2. **Use cases**
   - REST APIs with pagination
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

## Quick Start

### Basic Usage

**PostgreSQL:**
```rust
use sqlx::PgPool;
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
        
    paginated_query_as!(User, "SELECT * FROM users")
        .with_params(params)
        .fetch_paginated(pool)
        .await
}
```

**SQLite:**
```rust
use sqlx::SqlitePool;
use sqlx_paginated::{QueryParamsBuilder, QuerySortDirection, paginated_query_as_sqlite};

async fn get_users(pool: &SqlitePool) -> Result<PaginatedResponse<User>, sqlx::Error> {
    let params = QueryParamsBuilder::<User>::new()
        .with_pagination(1, 10)
        .with_sort("created_at", QuerySortDirection::Descending)
        .with_search("john", vec!["first_name", "last_name", "email"])
        .build();
        
    paginated_query_as_sqlite!(User, "SELECT * FROM users")
        .with_params(params)
        .fetch_paginated(pool)
        .await
}
```

### Response Example
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

## API Reference

### Pagination Parameters
| Parameter  | Type    | Default | Min | Max | Description                    |
|------------|---------|---------|-----|-----|--------------------------------|
| page       | integer | 1       | 1   | n/a | Current page number            |
| page_size  | integer | 10      | 10  | 50  | Number of records per page     |

#### Example:
```
GET /v1/internal/users?page=2&page_size=20
```

### Sort Parameters
| Parameter      | Type   | Default    | Allowed Values              | Description                |
|----------------|--------|------------|----------------------------|----------------------------|
| sort_column    | string | created_at | Any valid table column     | Column name to sort by     |
| sort_direction | string | descending | ascending, descending      | Sort direction             |

#### Example:
```
GET /v1/internal/users?sort_column=last_name&sort_direction=ascending
```

### Search Parameters
| Parameter      | Type   | Default           | Max Length | Description                          |
|----------------|--------|-------------------|------------|--------------------------------------|
| search         | string | null             | 100        | Search term to filter results         |
| search_columns | string | name,description | n/a        | Comma-separated list of columns       |

#### Example:
```
GET /v1/internal/users?search=john&search_columns=first_name,last_name,email
```

### Date Range Parameters
| Parameter    | Type     | Default    | Format    | Description           |
|-------------|----------|------------|-----------|----------------------|
| date_column | string   | created_at | Column name| Column to filter on   |
| date_after  | datetime | null       | ISO 8601  | Start of date range   |
| date_before | datetime | null       | ISO 8601  | End of date range     |

#### Example:
```
GET /v1/internal/users?date_column=created_at&date_after=2024-01-01T00:00:00Z
```

### Filtering Parameters
| Parameter | Type                    | Default           | Max Length | Description                             |
|-----------|-------------------------|-------------------|------------|-----------------------------------------|
| *         | string,boolean,datetime | null             | 100        | Any valid table column for given struct |

#### Example:
```
GET /v1/internal/users?confirmed=true
```

## Query Examples

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

## Performance Considerations

### Query Pattern Optimization
| Query Pattern | Impact | Recommendation |
|--------------|---------|----------------|
| SELECT * | ❌ High Impact | Specify needed columns |
| Large Text Columns | ❌ High Impact | Use separate detail endpoint |
| Computed Columns | ⚠️ Medium Impact | Cache if possible |
| JSON Aggregation | ⚠️ Medium Impact | Limit array size |

### Recommended Indexes
```sql
-- Text search
CREATE INDEX idx_users_name_gin ON users USING gin(to_tsvector('english', name));

-- Composite indexes for common queries
CREATE INDEX idx_users_confirmed_created ON users(confirmed, created_at);

-- JSON indexes
CREATE INDEX idx_users_metadata ON users USING gin(metadata);
```

### Pagination Performance
| Page Size | Records | Performance Impact |
|-----------|---------|-------------------|
| 1-10      | Optimal | ✅ Best           |
| 11-50     | Good    | ✅ Good           |
| 51-100    | Caution | ⚠️ Monitor        |
| 100+      | Poor    | ❌ Not Recommended |


## Security Features

### Input Sanitization
- Search terms are cleaned and normalized
- Parameter input values are trimmed and/or clamped against their defaults
- Column names are validated against an allowlist:
  - The struct itself first;
  - Database specific table names second;
- SQL injection patterns are blocked
- System table access is prevented

### Protected Patterns
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
