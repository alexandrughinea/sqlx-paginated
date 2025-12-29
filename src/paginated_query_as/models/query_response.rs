use crate::paginated_query_as::internal::QueryPaginationParams;
use serde::{Deserialize, Serialize};

/// Represents a paginated response with records and metadata.
///
/// This is the standard response structure returned by all paginated queries.
/// It includes the actual records, pagination information, and optionally
/// the total count and total pages.
///
/// # Type Parameters
///
/// * `T` - The type of records being paginated
///
/// # Examples
///
/// ```rust
/// use sqlx_paginated::PaginatedResponse;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct User {
///     id: i64,
///     name: String,
/// }
///
/// // Response will be serialized as:
/// // {
/// //   "records": [...],
/// //   "page": 1,
/// //   "page_size": 10,
/// //   "total": 100,
/// //   "total_pages": 10
/// // }
/// ```
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PaginatedResponse<T> {
    /// The records for the current page
    pub records: Vec<T>,

    /// Pagination metadata (page and page_size)
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub pagination: Option<QueryPaginationParams>,

    /// Total number of records across all pages
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,

    /// Total number of pages
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_pages: Option<i64>,
}
