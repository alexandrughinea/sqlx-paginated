use crate::paginated_query_as::internal::{
    default_date_range_column, default_page, default_page_size, default_search_columns,
    default_sort_column, default_sort_direction, page_deserialize, page_size_deserialize,
    search_columns_deserialize, search_deserialize,
};

use crate::QuerySortDirection;
use chrono::{DateTime, Utc};
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

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct QueryDateRangeParams {
    pub date_after: Option<DateTime<Utc>>,
    pub date_before: Option<DateTime<Utc>>,
    #[serde(default = "default_date_range_column")]
    pub date_column: Option<String>,
}

impl Default for QueryDateRangeParams {
    fn default() -> Self {
        Self {
            date_after: None,
            date_before: None,
            date_column: default_date_range_column(),
        }
    }
}
