use crate::paginated_query_as::internal::{
    default_page, default_page_size, default_search_columns, default_sort_column,
    default_sort_direction, page_deserialize, page_size_deserialize, search_columns_deserialize,
    search_deserialize,
};

use crate::QuerySortDirection;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct QueryPaginationParams {
    #[serde(deserialize_with = "page_deserialize", default = "default_page")]
    pub page: i64,
    #[serde(
        deserialize_with = "page_size_deserialize",
        default = "default_page_size"
    )]
    pub page_size: i64,
}

impl Default for QueryPaginationParams {
    fn default() -> Self {
        Self {
            page: default_page(),
            page_size: default_page_size(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct QuerySortParams {
    #[serde(default = "default_sort_direction")]
    pub sort_direction: QuerySortDirection,
    #[serde(default = "default_sort_column")]
    pub sort_column: String,
}

impl Default for QuerySortParams {
    fn default() -> Self {
        Self {
            sort_direction: default_sort_direction(),
            sort_column: default_sort_column(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct QuerySearchParams {
    #[serde(deserialize_with = "search_deserialize")]
    pub search: Option<String>,
    #[serde(
        deserialize_with = "search_columns_deserialize",
        default = "default_search_columns"
    )]
    pub search_columns: Option<Vec<String>>,
}

impl Default for QuerySearchParams {
    fn default() -> Self {
        Self {
            search: None,
            search_columns: default_search_columns(),
        }
    }
}


use crate::paginated_query_as::internal::internal_utils::FieldType;

/// Builder for configuring a computed property within the closure.
///
/// Used with `QueryBuilder::with_computed_property()` to define virtual columns
/// that map to SQL expressions, optionally with required JOIN clauses.
#[derive(Debug, Clone)]
pub struct ComputedPropertyBuilder {
    pub(crate) joins: Vec<String>,
    pub(crate) field_type: FieldType,
}

impl Default for ComputedPropertyBuilder {
    fn default() -> Self {
        Self {
            joins: Vec::new(),
            field_type: FieldType::String,
        }
    }
}

impl ComputedPropertyBuilder {
    /// Creates a new ComputedPropertyBuilder with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a JOIN clause required for this computed property.
    ///
    /// Multiple joins can be added by calling this method multiple times.
    /// Joins are only included in the final query if the computed property
    /// is actually used in search or filter operations.
    ///
    /// # Example
    /// ```rust,ignore
    /// // Used within QueryBuilder::with_computed_property closure:
    /// // IMPORTANT: When using with PaginatedQueryBuilder, reference "base_query" not the original table
    /// builder.with_computed_property("counterparty_name", |cp| {
    ///     cp.with_join("LEFT JOIN counterparty ON counterparty.id = base_query.counterparty_id");
    ///     "counterparty.legal_name"
    /// })
    /// ```
    pub fn with_join(&mut self, clause: impl Into<String>) -> &mut Self {
        self.joins.push(clause.into());
        self
    }

    /// Set the field type for proper type casting in filters.
    ///
    /// Defaults to `FieldType::String` if not specified.
    pub fn with_field_type(&mut self, field_type: FieldType) -> &mut Self {
        self.field_type = field_type;
        self
    }
}

/// Stored computed property definition.
///
/// Represents a virtual column that maps to a SQL expression,
/// with optional JOIN clauses that are activated when the property is used.
#[derive(Debug, Clone)]
pub struct ComputedProperty {
    /// The SQL expression (e.g., "counterparty.name" or "(amount_micros / 1000000)::money")
    pub expression: String,
    /// JOIN clauses needed for this computed property
    pub joins: Vec<String>,
    /// Field type for proper type casting in filters
    pub field_type: FieldType,
}
