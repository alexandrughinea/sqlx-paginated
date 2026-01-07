use crate::paginated_query_as::internal::{
    get_struct_field_names, QueryDateRangeParams, QueryPaginationParams, QuerySearchParams,
    QuerySortParams, DEFAULT_DATE_RANGE_COLUMN_NAME, DEFAULT_MAX_PAGE_SIZE, DEFAULT_MIN_PAGE_SIZE,
    DEFAULT_PAGE,
};
use crate::paginated_query_as::models::QuerySortDirection;
use crate::paginated_query_as::models::{QueryFilterCondition, QueryFilterOperator};
use crate::QueryParams;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::HashMap;

pub struct QueryParamsBuilder<'q, T> {
    query: QueryParams<'q, T>,
}

impl<T: Default + Serialize> Default for QueryParamsBuilder<'_, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'q, T: Default + Serialize> QueryParamsBuilder<'q, T> {
    /// Creates a new `QueryParamsBuilder` with default values.
    ///
    /// Default values include:
    /// - Page: 1
    /// - Page size: 10
    /// - Sort column: "created_at"
    /// - Sort direction: Descending
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serde::{Serialize};
    /// use sqlx_paginated::{QueryParamsBuilder};
    ///
    /// #[derive(Serialize, Default)]
    /// struct UserExample {
    ///     name: String
    /// }
    /// let builder = QueryParamsBuilder::<UserExample>::new();
    /// ```
    pub fn new() -> Self {
        Self {
            query: QueryParams::default(),
        }
    }

    /// Creates a new `QueryParamsBuilder` with default values.
    ///
    /// Default values include:
    /// - Page: 1
    /// - Page size: 10
    /// - Sort column: "created_at"
    /// - Sort direction: Descending
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serde::{Serialize};
    /// use sqlx_paginated::{QueryParamsBuilder};
    ///
    /// #[derive(Serialize, Default)]
    /// struct UserExample {
    ///     name: String
    /// }
    /// let builder = QueryParamsBuilder::<UserExample>::new();
    /// ```
    pub fn with_pagination(mut self, page: i64, page_size: i64) -> Self {
        self.query.pagination = QueryPaginationParams {
            page: page.max(DEFAULT_PAGE),
            page_size: page_size.clamp(DEFAULT_MIN_PAGE_SIZE, DEFAULT_MAX_PAGE_SIZE),
        };
        self
    }

    /// Sets sorting parameters.
    ///
    /// # Arguments
    ///
    /// * `sort_column` - Column name to sort by
    /// * `sort_direction` - Direction of sort (Ascending or Descending)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serde::{Serialize};
    /// use sqlx_paginated::{QueryParamsBuilder, QuerySortDirection};
    ///
    /// #[derive(Serialize, Default)]
    /// struct UserExample {
    ///     name: String
    /// }
    ///
    /// let params = QueryParamsBuilder::<UserExample>::new()
    ///     .with_sort("updated_at", QuerySortDirection::Ascending)
    ///     .build();
    /// ```
    pub fn with_sort(
        mut self,
        sort_column: impl Into<String>,
        sort_direction: QuerySortDirection,
    ) -> Self {
        self.query.sort = QuerySortParams {
            sort_column: sort_column.into(),
            sort_direction,
        };
        self
    }

    /// Sets search parameters with multiple columns support.
    ///
    /// # Arguments
    ///
    /// * `search` - Search term to look for
    /// * `search_columns` - Vector of column names to search in
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serde::{Serialize};
    /// use sqlx_paginated::{QueryParamsBuilder, QuerySortDirection};
    ///
    /// #[derive(Serialize, Default)]
    /// struct UserExample {
    ///     name: String
    /// }
    ///
    /// let params = QueryParamsBuilder::<UserExample>::new()
    ///     .with_search("john", vec!["name", "email", "username"])
    ///     .build();
    /// ```
    pub fn with_search(
        mut self,
        search: impl Into<String>,
        search_columns: Vec<impl Into<String>>,
    ) -> Self {
        self.query.search = QuerySearchParams {
            search: Some(search.into()),
            search_columns: Some(search_columns.into_iter().map(Into::into).collect()),
        };
        self
    }

    /// Sets date range parameters for filtering by date.
    ///
    /// # Arguments
    ///
    /// * `date_after` - Optional start date (inclusive)
    /// * `date_before` - Optional end date (inclusive)
    /// * `column_name` - Optional column name to apply date range filter (defaults to created_at)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use chrono::{DateTime, Utc};
    /// use serde::{Serialize};
    /// use sqlx_paginated::{QueryParamsBuilder, QuerySortDirection};
    ///
    /// #[derive(Serialize, Default)]
    /// struct UserExample {
    ///     name: String,
    ///     updated_at: DateTime<Utc>
    /// }
    ///
    /// let start = DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z").unwrap().into();
    /// let end = DateTime::parse_from_rfc3339("2024-12-31T23:59:59Z").unwrap().into();
    ///
    /// let params = QueryParamsBuilder::<UserExample>::new()
    ///     .with_date_range(Some(start), Some(end), Some("updated_at"))
    ///     .build();
    /// ```
    pub fn with_date_range(
        mut self,
        date_after: Option<DateTime<Utc>>,
        date_before: Option<DateTime<Utc>>,
        column_name: Option<impl Into<String>>,
    ) -> Self {
        self.query.date_range = QueryDateRangeParams {
            date_after,
            date_before,
            date_column: column_name.map_or_else(
                || Some(DEFAULT_DATE_RANGE_COLUMN_NAME.to_string()),
                |column_name| Some(column_name.into()),
            ),
        };
        self
    }

    /// Adds a single filter condition with an operator.
    ///
    /// # Arguments
    ///
    /// * `key` - Column name to filter on
    /// * `operator` - The comparison operator to use
    /// * `value` - Value to filter by (required for most operators except IS NULL/IS NOT NULL)
    ///
    /// # Details
    ///
    /// Only adds the filter if the column exists in the model struct.
    /// Logs a warning if tracing is enabled and the column is invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serde::{Serialize};
    /// use sqlx_paginated::{QueryParamsBuilder, QueryFilterOperator};
    ///
    /// #[derive(Serialize, Default)]
    /// struct Product {
    ///     name: String,
    ///     price: f64,
    ///     stock: i32,
    ///     status: String,
    /// }
    ///
    /// let params = QueryParamsBuilder::<Product>::new()
    ///     .with_filter_operator("price", QueryFilterOperator::GreaterThan, "10.00")
    ///     .with_filter_operator("stock", QueryFilterOperator::LessOrEqual, "100")
    ///     .with_filter_operator("status", QueryFilterOperator::NotEqual, "deleted")
    ///     .build();
    /// ```
    pub fn with_filter_operator(
        mut self,
        key: impl Into<String>,
        operator: QueryFilterOperator,
        value: impl Into<String>,
    ) -> Self {
        let key = key.into();
        let valid_fields = get_struct_field_names::<T>();

        if valid_fields.contains(&key) {
            self.query
                .filters
                .insert(key, QueryFilterCondition::new(operator, Some(value)));
        } else {
            #[cfg(feature = "tracing")]
            tracing::warn!(column = %key, "Skipping invalid filter column");
        }
        self
    }

    /// Adds a filter condition for IS NULL or IS NOT NULL checks.
    ///
    /// # Arguments
    ///
    /// * `key` - Column name to filter on
    /// * `is_null` - If true, checks IS NULL; if false, checks IS NOT NULL
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serde::{Serialize};
    /// use sqlx_paginated::{QueryParamsBuilder};
    ///
    /// #[derive(Serialize, Default)]
    /// struct User {
    ///     name: String,
    ///     deleted_at: Option<String>,
    /// }
    ///
    /// let params = QueryParamsBuilder::<User>::new()
    ///     .with_filter_null("deleted_at", true)  // IS NULL
    ///     .build();
    /// ```
    pub fn with_filter_null(mut self, key: impl Into<String>, is_null: bool) -> Self {
        let key = key.into();
        let valid_fields = get_struct_field_names::<T>();

        if valid_fields.contains(&key) {
            let condition = if is_null {
                QueryFilterCondition::is_null()
            } else {
                QueryFilterCondition::is_not_null()
            };
            self.query.filters.insert(key, condition);
        } else {
            #[cfg(feature = "tracing")]
            tracing::warn!(column = %key, "Skipping invalid filter column");
        }
        self
    }

    /// Adds an IN filter condition with multiple values.
    ///
    /// # Arguments
    ///
    /// * `key` - Column name to filter on
    /// * `values` - Vector of values to check against
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serde::{Serialize};
    /// use sqlx_paginated::{QueryParamsBuilder};
    ///
    /// #[derive(Serialize, Default)]
    /// struct User {
    ///     name: String,
    ///     role: String,
    /// }
    ///
    /// let params = QueryParamsBuilder::<User>::new()
    ///     .with_filter_in("role", vec!["admin", "moderator", "user"])
    ///     .build();
    /// ```
    pub fn with_filter_in(
        mut self,
        key: impl Into<String>,
        values: Vec<impl Into<String>>,
    ) -> Self {
        let key = key.into();
        let valid_fields = get_struct_field_names::<T>();

        if valid_fields.contains(&key) {
            self.query
                .filters
                .insert(key, QueryFilterCondition::in_list(values));
        } else {
            #[cfg(feature = "tracing")]
            tracing::warn!(column = %key, "Skipping invalid filter column");
        }
        self
    }

    /// Adds a NOT IN filter condition with multiple values.
    ///
    /// # Arguments
    ///
    /// * `key` - Column name to filter on
    /// * `values` - Vector of values to exclude
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serde::{Serialize};
    /// use sqlx_paginated::{QueryParamsBuilder};
    ///
    /// #[derive(Serialize, Default)]
    /// struct User {
    ///     name: String,
    ///     role: String,
    /// }
    ///
    /// let params = QueryParamsBuilder::<User>::new()
    ///     .with_filter_not_in("role", vec!["banned", "suspended"])
    ///     .build();
    /// ```
    pub fn with_filter_not_in(
        mut self,
        key: impl Into<String>,
        values: Vec<impl Into<String>>,
    ) -> Self {
        let key = key.into();
        let valid_fields = get_struct_field_names::<T>();

        if valid_fields.contains(&key) {
            self.query
                .filters
                .insert(key, QueryFilterCondition::not_in_list(values));
        } else {
            #[cfg(feature = "tracing")]
            tracing::warn!(column = %key, "Skipping invalid filter column");
        }
        self
    }

    /// Adds a simple equality filter condition (backward compatible).
    ///
    /// # Arguments
    ///
    /// * `key` - Column name to filter on
    /// * `value` - Optional value to filter by
    ///
    /// # Details
    ///
    /// Only adds the filter if the column exists in the model struct.
    /// Logs a warning if tracing is enabled and the column is invalid.
    /// This method maintains backward compatibility with the original API.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::any::Any;
    /// use serde::{Serialize};
    /// use sqlx_paginated::{QueryParamsBuilder};
    ///
    /// #[derive(Serialize, Default)]
    /// struct UserExample {
    ///     name: String,
    ///     status: String,
    ///     role: String
    /// }
    ///
    /// let params = QueryParamsBuilder::<UserExample>::new()
    ///     .with_filter("status", Some("active"))
    ///     .with_filter("role", Some("admin"))
    ///     .build();
    /// ```
    pub fn with_filter(mut self, key: impl Into<String>, value: Option<impl Into<String>>) -> Self {
        let key = key.into();
        let valid_fields = get_struct_field_names::<T>();

        if valid_fields.contains(&key) {
            if let Some(val) = value {
                self.query
                    .filters
                    .insert(key, QueryFilterCondition::equal(val));
            }
        } else {
            #[cfg(feature = "tracing")]
            tracing::warn!(column = %key, "Skipping invalid filter column");
        }
        self
    }

    /// Adds multiple filter conditions from a HashMap (backward compatible).
    ///
    /// # Arguments
    ///
    /// * `filters` - HashMap of column names and their filter values (equality only)
    ///
    /// # Details
    ///
    /// Only adds filters for columns that exist in the model struct.
    /// Logs a warning if tracing is enabled and a column is invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use serde::{Serialize};
    /// use sqlx_paginated::{QueryParamsBuilder};
    ///
    /// #[derive(Serialize, Default)]
    /// struct UserExample {
    ///     name: String,
    ///     status: String,
    ///     role: String
    /// }
    ///
    /// let mut filters = HashMap::new();
    /// filters.insert("status", Some("active"));
    /// filters.insert("role", Some("admin"));
    ///
    /// let params = QueryParamsBuilder::<UserExample>::new()
    ///     .with_filters(filters)
    ///     .build();
    /// ```
    pub fn with_filters(
        mut self,
        filters: HashMap<impl Into<String>, Option<impl Into<String>>>,
    ) -> Self {
        let valid_fields = get_struct_field_names::<T>();

        self.query
            .filters
            .extend(filters.into_iter().filter_map(|(key, value)| {
                let key = key.into();
                if valid_fields.contains(&key) {
                    value.map(|v| (key, QueryFilterCondition::equal(v)))
                } else {
                    #[cfg(feature = "tracing")]
                    tracing::warn!(column = %key, "Skipping invalid filter column");
                    None
                }
            }));

        self
    }

    /// Adds multiple filter conditions with operators from a HashMap.
    ///
    /// # Arguments
    ///
    /// * `filters` - HashMap of column names and their FilterConditions
    ///
    /// # Details
    ///
    /// Only adds filters for columns that exist in the model struct.
    /// Logs a warning if tracing is enabled and a column is invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use serde::{Serialize};
    /// use sqlx_paginated::{QueryParamsBuilder, QueryFilterCondition, QueryFilterOperator};
    ///
    /// #[derive(Serialize, Default)]
    /// struct Product {
    ///     name: String,
    ///     price: f64,
    ///     stock: i32,
    /// }
    ///
    /// let mut filters = HashMap::new();
    /// filters.insert("price", QueryFilterCondition::greater_than("10.00"));
    /// filters.insert("stock", QueryFilterCondition::less_or_equal("100"));
    ///
    /// let params = QueryParamsBuilder::<Product>::new()
    ///     .with_filter_conditions(filters)
    ///     .build();
    /// ```
    pub fn with_filter_conditions(
        mut self,
        filters: HashMap<impl Into<String>, QueryFilterCondition>,
    ) -> Self {
        let valid_fields = get_struct_field_names::<T>();

        self.query
            .filters
            .extend(filters.into_iter().filter_map(|(key, condition)| {
                let key = key.into();
                if valid_fields.contains(&key) {
                    Some((key, condition))
                } else {
                    #[cfg(feature = "tracing")]
                    tracing::warn!(column = %key, "Skipping invalid filter column");
                    None
                }
            }));

        self
    }

    /// Builds and returns the final QueryParams.
    ///
    /// # Returns
    ///
    /// Returns the constructed `QueryParams<T>` with all the configured parameters.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use chrono::{DateTime, Utc};
    /// use sqlx_paginated::{QueryParamsBuilder, QuerySortDirection};
    /// use serde::{Serialize};
    ///
    /// #[derive(Serialize, Default)]
    /// struct UserExample {
    ///     name: String,
    ///     status: String,
    ///     email: String,
    ///     created_at: DateTime<Utc>
    /// }
    ///
    /// let params = QueryParamsBuilder::<UserExample>::new()
    ///     .with_pagination(1, 20)
    ///     .with_sort("created_at", QuerySortDirection::Descending)
    ///     .with_search("john", vec!["name", "email"])
    ///     .with_filter("status", Some("active"))
    ///     .build();
    /// ```
    pub fn build(self) -> QueryParams<'q, T> {
        self.query
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paginated_query_as::internal::{
        DEFAULT_SEARCH_COLUMN_NAMES, DEFAULT_SORT_COLUMN_NAME,
    };
    use crate::paginated_query_as::models::QuerySortDirection;
    use chrono::{DateTime, Utc};
    use std::collections::HashMap;

    #[derive(Debug, Default, Serialize)]
    struct TestModel {
        name: String,
        title: String,
        description: String,
        status: String,
        category: String,
        updated_at: DateTime<Utc>,
        created_at: DateTime<Utc>,
    }

    #[test]
    fn test_pagination_defaults() {
        let params = QueryParamsBuilder::<TestModel>::new().build();

        assert_eq!(
            params.pagination.page_size, DEFAULT_MIN_PAGE_SIZE,
            "Default page size should be {}",
            DEFAULT_MIN_PAGE_SIZE
        );
        assert_eq!(
            params.pagination.page, DEFAULT_PAGE,
            "Default page should be {}",
            DEFAULT_PAGE
        );

        // Test page size clamping
        let params = QueryParamsBuilder::<TestModel>::new()
            .with_pagination(1, DEFAULT_MAX_PAGE_SIZE + 10)
            .build();

        assert_eq!(
            params.pagination.page_size, DEFAULT_MAX_PAGE_SIZE,
            "Page size should be clamped to maximum {}",
            DEFAULT_MAX_PAGE_SIZE
        );

        let params = QueryParamsBuilder::<TestModel>::new()
            .with_pagination(1, DEFAULT_MIN_PAGE_SIZE - 5)
            .build();

        assert_eq!(
            params.pagination.page_size, DEFAULT_MIN_PAGE_SIZE,
            "Page size should be clamped to minimum {}",
            DEFAULT_MIN_PAGE_SIZE
        );
    }

    #[test]
    fn test_default_sort_column() {
        let params = QueryParamsBuilder::<TestModel>::new().build();

        assert_eq!(
            params.sort.sort_column, DEFAULT_SORT_COLUMN_NAME,
            "Default sort column should be '{}'",
            DEFAULT_SORT_COLUMN_NAME
        );
    }

    #[test]
    fn test_date_range_defaults() {
        let params = QueryParamsBuilder::<TestModel>::new().build();

        assert_eq!(
            params.date_range.date_column,
            Some(DEFAULT_DATE_RANGE_COLUMN_NAME.to_string()),
            "Default date range column should be '{}'",
            DEFAULT_DATE_RANGE_COLUMN_NAME
        );
        assert!(
            params.date_range.date_after.is_none(),
            "Default date_after should be None"
        );
        assert!(
            params.date_range.date_before.is_none(),
            "Default date_before should be None"
        );
    }

    #[test]
    fn test_search_defaults() {
        let params = QueryParamsBuilder::<TestModel>::new().build();

        // Check if default search columns are set
        assert_eq!(
            params.search.search_columns,
            Some(
                DEFAULT_SEARCH_COLUMN_NAMES
                    .iter()
                    .map(|&s| s.to_string())
                    .collect()
            ),
            "Default search columns should be {:?}",
            DEFAULT_SEARCH_COLUMN_NAMES
        );
        assert!(
            params.search.search.is_none(),
            "Default search term should be None"
        );
    }

    #[test]
    fn test_combined_defaults() {
        let params = QueryParamsBuilder::<TestModel>::new().build();

        // Test all defaults together
        assert_eq!(params.pagination.page, DEFAULT_PAGE);
        assert_eq!(params.pagination.page_size, DEFAULT_MIN_PAGE_SIZE);
        assert_eq!(params.sort.sort_column, DEFAULT_SORT_COLUMN_NAME);
        assert_eq!(params.sort.sort_direction, QuerySortDirection::Descending);
        assert_eq!(
            params.date_range.date_column,
            Some(DEFAULT_DATE_RANGE_COLUMN_NAME.to_string())
        );
        assert_eq!(
            params.search.search_columns,
            Some(
                DEFAULT_SEARCH_COLUMN_NAMES
                    .iter()
                    .map(|&s| s.to_string())
                    .collect()
            )
        );
        assert!(params.search.search.is_none());
        assert!(params.date_range.date_after.is_none());
        assert!(params.date_range.date_before.is_none());
    }

    #[test]
    fn test_empty_params() {
        let params = QueryParamsBuilder::<TestModel>::new().build();

        assert_eq!(params.pagination.page, 1);
        assert_eq!(params.pagination.page_size, 10);
        assert_eq!(params.sort.sort_column, "created_at");
        assert!(matches!(
            params.sort.sort_direction,
            QuerySortDirection::Descending
        ));
    }

    #[test]
    fn test_partial_params() {
        let params = QueryParamsBuilder::<TestModel>::new()
            .with_pagination(2, 10)
            .with_search("test".to_string(), vec!["name".to_string()])
            .build();

        assert_eq!(params.pagination.page, 2);
        assert_eq!(params.search.search, Some("test".to_string()));
        assert_eq!(params.pagination.page_size, 10);
        assert_eq!(params.sort.sort_column, "created_at");
        assert!(matches!(
            params.sort.sort_direction,
            QuerySortDirection::Descending
        ));
    }

    #[test]
    fn test_invalid_params() {
        // For builder pattern, invalid params would be handled at compile time
        // But we can test the defaults
        let params = QueryParamsBuilder::<TestModel>::new()
            .with_pagination(0, 0) // Should be clamped to minimum values
            .build();

        assert_eq!(params.pagination.page, 1);
        assert_eq!(params.pagination.page_size, 10);
    }

    #[test]
    fn test_filters() {
        let mut filters = HashMap::new();
        filters.insert("status".to_string(), Some("active".to_string()));
        filters.insert("category".to_string(), Some("test".to_string()));

        let params = QueryParamsBuilder::<TestModel>::new()
            .with_filters(filters)
            .build();

        assert!(params.filters.contains_key("status"));
        let status_filter = params.filters.get("status").unwrap();
        assert_eq!(status_filter.operator, QueryFilterOperator::Equal);
        assert_eq!(status_filter.value, Some("active".to_string()));

        assert!(params.filters.contains_key("category"));
        let category_filter = params.filters.get("category").unwrap();
        assert_eq!(category_filter.operator, QueryFilterOperator::Equal);
        assert_eq!(category_filter.value, Some("test".to_string()));
    }

    #[test]
    fn test_search_with_columns() {
        let params = QueryParamsBuilder::<TestModel>::new()
            .with_search(
                "test".to_string(),
                vec!["title".to_string(), "description".to_string()],
            )
            .build();

        assert_eq!(params.search.search, Some("test".to_string()));
        assert_eq!(
            params.search.search_columns,
            Some(vec!["title".to_string(), "description".to_string()])
        );
    }

    #[test]
    fn test_full_params() {
        let params = QueryParamsBuilder::<TestModel>::new()
            .with_pagination(2, 20)
            .with_sort("updated_at".to_string(), QuerySortDirection::Ascending)
            .with_search(
                "test".to_string(),
                vec!["title".to_string(), "description".to_string()],
            )
            .with_date_range(Some(Utc::now()), None, None::<String>)
            .build();

        assert_eq!(params.pagination.page, 2);
        assert_eq!(params.pagination.page_size, 20);
        assert_eq!(params.sort.sort_column, "updated_at");
        assert!(matches!(
            params.sort.sort_direction,
            QuerySortDirection::Ascending
        ));
        assert_eq!(params.search.search, Some("test".to_string()));
        assert_eq!(
            params.search.search_columns,
            Some(vec!["title".to_string(), "description".to_string()])
        );
        assert!(params.date_range.date_after.is_some());
        assert!(params.date_range.date_before.is_none());
    }

    #[test]
    fn test_filter_chain() {
        let params = QueryParamsBuilder::<TestModel>::new()
            .with_filter("status", Some("active"))
            .with_filter("category", Some("test"))
            .build();

        let status_filter = params.filters.get("status").unwrap();
        assert_eq!(status_filter.operator, QueryFilterOperator::Equal);
        assert_eq!(status_filter.value, Some("active".to_string()));

        let category_filter = params.filters.get("category").unwrap();
        assert_eq!(category_filter.operator, QueryFilterOperator::Equal);
        assert_eq!(category_filter.value, Some("test".to_string()));
    }

    #[test]
    fn test_mixed_pagination() {
        let params = QueryParamsBuilder::<TestModel>::new()
            .with_pagination(2, 10)
            .with_search("test".to_string(), vec!["title".to_string()])
            .with_filter("status", Some("active"))
            .build();

        assert_eq!(params.pagination.page, 2);
        assert_eq!(params.pagination.page_size, 10);
        assert_eq!(params.search.search, Some("test".to_string()));

        let status_filter = params.filters.get("status").unwrap();
        assert_eq!(status_filter.operator, QueryFilterOperator::Equal);
        assert_eq!(status_filter.value, Some("active".to_string()));
    }

    #[test]
    fn test_filter_operators() {
        let params = QueryParamsBuilder::<TestModel>::new()
            .with_filter_operator("title", QueryFilterOperator::Like, "%test%")
            .with_filter_operator("status", QueryFilterOperator::NotEqual, "deleted")
            .build();

        let title_filter = params.filters.get("title").unwrap();
        assert_eq!(title_filter.operator, QueryFilterOperator::Like);
        assert_eq!(title_filter.value, Some("%test%".to_string()));

        let status_filter = params.filters.get("status").unwrap();
        assert_eq!(status_filter.operator, QueryFilterOperator::NotEqual);
        assert_eq!(status_filter.value, Some("deleted".to_string()));
    }

    #[test]
    fn test_filter_null() {
        let params = QueryParamsBuilder::<TestModel>::new()
            .with_filter_null("description", true)
            .build();

        let filter = params.filters.get("description").unwrap();
        assert_eq!(filter.operator, QueryFilterOperator::IsNull);
        assert_eq!(filter.value, None);
    }

    #[test]
    fn test_filter_in() {
        let params = QueryParamsBuilder::<TestModel>::new()
            .with_filter_in("status", vec!["active", "pending", "approved"])
            .build();

        let filter = params.filters.get("status").unwrap();
        assert_eq!(filter.operator, QueryFilterOperator::In);
        assert_eq!(filter.value, Some("active,pending,approved".to_string()));

        let values = filter.split_values();
        assert_eq!(values, vec!["active", "pending", "approved"]);
    }
}
