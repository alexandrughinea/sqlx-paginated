use crate::paginated_query_as::internal::{
    QueryDateRangeParams, QueryPaginationParams, QuerySearchParams, QuerySortParams,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FlatQueryParams {
    #[serde(flatten)]
    pub pagination: Option<QueryPaginationParams>,
    #[serde(flatten)]
    pub sort: Option<QuerySortParams>,
    #[serde(flatten)]
    pub search: Option<QuerySearchParams>,
    #[serde(flatten)]
    pub date_range: Option<QueryDateRangeParams>,
    #[serde(flatten)]
    pub filters: Option<HashMap<String, Option<String>>>,
}
