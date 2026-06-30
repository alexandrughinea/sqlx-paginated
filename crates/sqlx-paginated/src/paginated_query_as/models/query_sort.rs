use serde::{Deserialize, Serialize};

/// Represents the direction for sorting query results.
///
/// Used in conjunction with a column name to specify how results should be ordered.
///
/// # Examples
///
/// ```rust
/// use sqlx_paginated::{QuerySortDirection, QueryParamsBuilder};
/// use serde::Serialize;
///
/// #[derive(Serialize, Default)]
/// struct User {
///     name: String,
///     created_at: String,
/// }
///
/// let params = QueryParamsBuilder::<User>::new()
///     .with_sort("created_at", QuerySortDirection::Descending)
///     .build();
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum QuerySortDirection {
    /// Sort in ascending order (A-Z, 0-9, oldest-newest)
    Ascending,

    /// Sort in descending order (Z-A, 9-0, newest-oldest)
    #[default]
    Descending,
}
