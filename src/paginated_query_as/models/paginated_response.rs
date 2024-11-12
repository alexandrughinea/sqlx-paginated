use crate::paginated_query_as::internal::QueryPaginationParams;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PaginatedResponse<T> {
    pub records: Vec<T>,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub pagination: Option<QueryPaginationParams>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_pages: Option<i64>,
}
