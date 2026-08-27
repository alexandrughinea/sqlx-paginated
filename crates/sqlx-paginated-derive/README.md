# sqlx-paginated-derive

Internal proc-macro for [`sqlx-paginated`](https://crates.io/crates/sqlx-paginated). It provides `#[derive(Fields)]`, which generates a typed field enum used for pagination, search, sort, and filter column names.

Depend on `sqlx-paginated` and import `Fields` from there. You do not need this crate as a direct dependency.

```rust
use sqlx_paginated::Fields;

#[derive(Fields)]
struct User {
    id: i64,
    email: String,
}
```

See the [sqlx-paginated README](https://github.com/alexandrughinea/sqlx-paginated) for full usage.
