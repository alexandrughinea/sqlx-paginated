# Paginated queries for SQLx

[![Rust](https://github.com/alexandrughinea/sqlx-paginated/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/alexandrughinea/sqlx-paginated/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/sqlx-paginate.svg)](https://crates.io/crates/sqlx-paginate)
[![Documentation](https://docs.rs/sqlx-paginate/badge.svg)](https://docs.rs/sqlx-paginate)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A flexible, type-safe SQLx query builder for dynamic web APIs, offering seamless pagination, searching, filtering, and sorting.

## Table of Contents
- [Paginated queries for SQLx](#paginated-queries-for-sqlx)
  - [Table of Contents](#table-of-contents)
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
  - [Complex Query Examples](#complex-query-examples)
    - [Combined Search, Sort, and Pagination](#combined-search-sort-and-pagination)
    - [Filtered Date Range with another field](#filtered-date-range-with-another-field)
  - [Performance Considerations](#performance-considerations)
    - [Query Pattern Optimization](#query-pattern-optimization)
    - [Recommended Indexes](#recommended-indexes)
    - [Pagination Performance](#pagination-performance)
  - [Advanced Builders](#advanced-builders)
  - [Security Features](#security-features)
    - [Input Sanitization](#input-sanitization)
    - [Protected Patterns](#protected-patterns)
  - [Contributing](#contributing)
  - [License](#license)

## Features

### Core Capabilities
- 🔍 Full-text search with column specification
- 📑 Smart pagination with customizable page size
- 🔄 Dynamic sorting on any column
- 🎯 Flexible filtering system
- 📅 Date range filtering
- 🔒 Type-safe operations
- ⚡ High performance
- 🛡️ SQL injection protection

### Technical Features
- Builder patterns for query parameters and query construction
- Graceful error handling
- Logging with tracing (if enabled)
- Macro and function support

### Query Features
- Case-insensitive search
- Multiple column search
- Complex filtering conditions
- Date-based filtering
- Dynamic sort direction
- Customizable page size
- Result count optimization

## Database Support

### Current vs Planned Support
| Database    | Status      | Version | Features                           | Notes                                   |
|-------------|-------------|---------|-----------------------------------|-----------------------------------------|
| PostgreSQL  | ✅ Supported | 12+     | All features supported            | Production ready, fully tested          |
| SQLite      | 🚧 Planned  | 3.35+   | Basic features planned           | Development starting after mid Feb 2025 |
| MySQL       | 🚧 Planned  | 8.0+    | Core features planned            | On roadmap                              |
| MSSQL       | 🚧 Planned  | 2019+   | Core features planned            | On roadmap                              |

⚠️ Note: `This documentation covers PostgreSQL features only, as it's currently the only fully supported database.`

## Market Analysis

### Ecosystem Gaps
1. **Query builders**
   - Diesel: Full ORM, can be heavyweight
   - SQLx: Great low-level toolkit but no high-level query building
   - SeaQuery: Generic and verbose
   - sqlbuilder: Basic SQL building without pagination or security

2. **Missing features in existing solutions**
   - Type-safe pagination
   - Easy integration with web frameworks
   - Automatic type casting
   - Typesafe search/filter/sort/pagination capabilities

### Unique Selling Points

1. **Quick Web Framework Integration**

[Actix Web](https://actix.rs/) handler example
```rust
async fn list_users(Query(params): Query<QueryParams>) -> impl Responder {
    let query = paginated_query_as!(User, "SELECT * FROM users")
        .with_params(params)
        .disable_totals_count()
        .fetch_paginated(&pool);
}
```

2. **Type Safety & Ergonomics**
```rust
// Type inference and validation
let params = QueryParamsBuilder::<User>::new()
    .with_pagination(1, 10)
    .with_sort("created_at", QuerySortDirection::Descending)
    .with_search("john", vec!["name", "email"])
    .build();
```

3. **Advanced Builder Patterns**
- Optional fluent API for query parameters (QueryParams) which allow defining search, search location, date filtering, ordering, and custom filtering.
- Fluent API for the entire supported feature set, more here: [advanced example](src/paginated_query_as/examples/paginated_query_builder_advanced_examples.rs)

### Target Audience
1. **Primary users**
   - Rust web developers
   - Teams needing secure query building
   - Projects requiring pagination APIs
   - SQLx users wanting higher-level abstractions

2. **Use cases**
   - REST APIs with pagination
   - Admin panels
   - Data exploration interfaces

## Installation

Add to `Cargo.toml`:
```toml
[dependencies]
sqlx_paginated = { version = "0.1.0", features = ["postgres"] }
```

## Quick Start

### Basic Usage
```rust
#[derive(sqlx::FromRow, serde::Serialize)]
struct User {
    id: i64,
    first_name: String,
    last_name: String,
    email: String,
    confirmed: bool,
    created_at: Option<DateTime<Utc>>,
}

/// Macro usage example
async fn get_users(pool: &PgPool) -> Result<PaginatedResponse<User>, sqlx::Error> {
    let paginated_response = paginated_query_as!(User, "SELECT * FROM users")
        .with_params(params)
        .fetch_paginated(&pool)
        .await
        .unwrap();

    paginated_response
}

/// Alternative function call example (if macros cannot be applied to your use case)
async fn get_users(pool: &PgPool) -> Result<PaginatedResponse<User>, sqlx::Error> {
    let paginated_response = paginated_query_as::<User>("SELECT * FROM users")
        .with_params(params)
        .fetch_paginated(&pool)
        .await
        .unwrap();

    paginated_response
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
  "total": 1,
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

Example:
```
GET /v1/internal/users?page=2&page_size=20
```

### Sort Parameters
| Parameter      | Type   | Default    | Allowed Values              | Description                |
|----------------|--------|------------|----------------------------|----------------------------|
| sort_column    | string | created_at | Any valid table column     | Column name to sort by     |
| sort_direction | string | descending | ascending, descending      | Sort direction             |

Example:
```
GET /v1/internal/users?sort_column=last_name&sort_direction=ascending
```

### Search Parameters
| Parameter      | Type   | Default           | Max Length | Description                          |
|----------------|--------|-------------------|------------|--------------------------------------|
| search         | string | null             | 100        | Search term to filter results         |
| search_columns | string | name,description | n/a        | Comma-separated list of columns       |

Example:
```
GET /v1/internal/users?search=john&search_columns=first_name,last_name,email
```

### Date Range Parameters
| Parameter    | Type     | Default    | Format    | Description           |
|-------------|----------|------------|-----------|----------------------|
| date_column | string   | created_at | Column name| Column to filter on   |
| date_after  | datetime | null       | ISO 8601  | Start of date range   |
| date_before | datetime | null       | ISO 8601  | End of date range     |

Example:
```
GET /v1/internal/users?date_column=created_at&date_after=2024-01-01T00:00:00Z
```

### Filtering Parameters
| Parameter | Type                    | Default           | Max Length | Description                             |
|-----------|-------------------------|-------------------|------------|-----------------------------------------|
| *         | string,boolean,datetime | null             | 100        | Any valid table column for given struct |

Example:
```
GET /v1/internal/users?confirmed=true
```

## Complex Query Examples

### Combined Search, Sort, and Pagination
```
GET /v1/internal/users
    ?search=john
    &search_columns=first_name,last_name,email
    &sort_column=created_at
    &sort_direction=descending
    &page=1
    &page_size=20
```

Response:
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
      "created_at": "2024-11-03T12:30:12.081598Z"
    },
    {
      "id": "9167d825-8944-4428-bf91-3c5531728b5e",
      "first_name": "Johnny",
      "last_name": "Doe",
      "email": "johnny.doe@example.com",
      "confirmed": true,
      "created_at": "2024-10-28T19:14:49.064626Z"
    }
  ]
}
```

### Filtered Date Range with another field
```
GET /v1/internal/users
    ?date_column=created_at
    &date_after=2024-01-01T00:00:00Z
    &date_before=2024-12-31T23:59:59Z
    &confirmed=active
    &sort_column=last_login
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

## Advanced Builders
If you need to, you can also construct programmatically all the conditions you might ever need on a struct.
Below is an exhaustive example where not only the core DB query builder is being programmatically built, but also the query params.

```rust

#[derive(Default, Serialize, FromRow)]
#[allow(dead_code)]
pub struct UserExample {
    id: String,
    name: String,
    email: String,
    status: String,
    score: i32,
}

#[allow(dead_code)]
pub async fn paginated_query_builder_advanced_example(
    pool: PgPool,
) -> PaginatedResponse<UserExample> {
    let some_extra_filters =
        HashMap::from([("a", Some("1".to_string())), ("b", Some("2".to_string()))]);
    let initial_params = QueryParamsBuilder::<UserExample>::new()
        .with_search("john", vec!["name", "email"])
        .with_pagination(1, 10)
        .with_date_range(Some(Utc::now()), None, None::<String>)
        .with_filter("status", Some("active"))
        .with_filters(some_extra_filters)
        .with_sort("created_at", QuerySortDirection::Descending)
        .build();

    paginated_query_as!(UserExample, "SELECT * FROM users")
        .with_params(initial_params)
        .with_query_builder(|params| {
            QueryBuilder::<UserExample, Postgres>::new()
                .with_search(params)
                .with_filters(params)
                .with_date_range(params)
                .with_raw_condition("") // Add raw condition, no checks
                .disable_protection() // This removes all column safety checks
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
        .disable_totals_count() // Disables the calculation of total record count.
        .fetch_paginated(&pool)
        .await
        .unwrap()
}
```

## Security Features

### Input Sanitization
- Search terms are cleaned and normalized
- Column names are validated against allowlist
- SQL injection patterns are blocked
- System table access is prevented

### Protected Patterns
- System schemas (pg_, information_schema)
- System columns (oid, xmin, etc.)
- SQL injection attempts
- Invalid characters in identifiers

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
