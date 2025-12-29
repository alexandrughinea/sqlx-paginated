use crate::paginated_query_as::internal::{
    deserialize_filter_map, QueryDateRangeParams, QueryPaginationParams, QuerySearchParams,
    QuerySortParams,
};
use crate::paginated_query_as::models::QueryFilterCondition;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::marker::PhantomData;

/// Flattened query parameters suitable for deserializing from HTTP query strings.
///
/// This struct is designed to work with web frameworks' query string deserializers.
/// All fields are optional to allow partial parameter specification.
///
/// # Filter Syntax
///
/// Supports both simple equality and advanced operator syntax:
/// - Simple: `field=value` → Equal operator
/// - With operator: `field[op]=value` → Specified operator
///
/// # Examples
///
/// ```rust
/// use sqlx_paginated::FlatQueryParams;
/// use serde::Deserialize;
///
/// // Simple equality filters:
/// // ?status=active&role=admin
///
/// // Advanced operator filters:
/// // ?price[gt]=10&stock[lte]=100&status[ne]=deleted
/// // ?role[in]=admin,moderator&deleted_at[is_null]=
///
/// // With Actix-web:
/// // async fn handler(Query(params): Query<FlatQueryParams>) { ... }
///
/// // With Axum:
/// // async fn handler(Query(params): Query<FlatQueryParams>) { ... }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FlatQueryParams {
    /// Pagination parameters (page, page_size)
    #[serde(flatten)]
    pub pagination: Option<QueryPaginationParams>,

    /// Sort parameters (sort_column, sort_direction)
    #[serde(flatten)]
    pub sort: Option<QuerySortParams>,

    /// Search parameters (search, search_columns)
    #[serde(flatten)]
    pub search: Option<QuerySearchParams>,

    /// Date range parameters (date_column, date_after, date_before)
    #[serde(flatten)]
    pub date_range: Option<QueryDateRangeParams>,

    /// Filter conditions with operators
    ///
    /// Supports operator syntax in query strings:
    /// - `field=value` → Equality
    /// - `field[gt]=value` → Greater than
    /// - `field[in]=val1,val2` → In list
    /// - `field[is_null]=` → Is null
    ///
    /// See `QueryFilterOperator` for all supported operators.
    #[serde(flatten, deserialize_with = "deserialize_filter_map")]
    pub filters: Option<HashMap<String, QueryFilterCondition>>,
}

/// Complete query parameters with all configuration options.
///
/// This is the main parameter structure used throughout the library.
/// It contains all pagination, sorting, filtering, and search configuration.
///
/// # Type Parameters
///
/// * `'q` - Lifetime for query parameter references
/// * `T` - The model type being queried
///
/// # Examples
///
/// ```rust
/// use sqlx_paginated::{QueryParams, QueryParamsBuilder, QuerySortDirection};
/// use serde::Serialize;
///
/// #[derive(Serialize, Default)]
/// struct User {
///     name: String,
///     email: String,
/// }
///
/// let params = QueryParamsBuilder::<User>::new()
///     .with_pagination(1, 20)
///     .with_sort("name", QuerySortDirection::Ascending)
///     .with_search("john", vec!["name", "email"])
///     .build();
/// ```
#[derive(Default, Clone)]
pub struct QueryParams<'q, T> {
    /// Pagination configuration (page, page_size)
    pub pagination: QueryPaginationParams,

    /// Sort configuration (column, direction)
    pub sort: QuerySortParams,

    /// Search configuration (term, columns)
    pub search: QuerySearchParams,

    /// Date range filtering configuration
    pub date_range: QueryDateRangeParams,

    /// Filter conditions with operators
    pub filters: HashMap<String, QueryFilterCondition>,

    /// Legacy simple filters (backward compatibility)
    ///
    /// Deprecated: Use `filters` with QueryFilterCondition instead
    #[deprecated(
        since = "0.3.0",
        note = "Use filters with QueryFilterCondition instead"
    )]
    pub simple_filters: HashMap<String, Option<String>>,

    /// Phantom data to hold the type parameter
    pub(crate) _phantom: PhantomData<&'q T>,
}

impl<'q, T> From<FlatQueryParams> for QueryParams<'q, T> {
    fn from(params: FlatQueryParams) -> Self {
        let filters = params.filters.unwrap_or_default();

        // Build simple_filters for backward compatibility (deprecated)
        #[allow(deprecated)]
        let simple_filters = filters
            .iter()
            .map(|(k, v)| (k.clone(), v.value.clone()))
            .collect();

        QueryParams {
            pagination: params.pagination.unwrap_or_default(),
            sort: params.sort.unwrap_or_default(),
            search: params.search.unwrap_or_default(),
            date_range: params.date_range.unwrap_or_default(),
            filters,
            #[allow(deprecated)]
            simple_filters,
            _phantom: PhantomData::<&'q T>,
        }
    }
}
