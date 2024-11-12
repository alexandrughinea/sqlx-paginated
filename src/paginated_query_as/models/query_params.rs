use crate::paginated_query_as::internal::{
    QueryDateRangeParams, QueryPaginationParams, QuerySearchParams, QuerySortParams,
};
use crate::FlatQueryParams;
use std::collections::HashMap;
use std::marker::PhantomData;

#[derive(Default, Clone)]
pub struct QueryParams<'q, T> {
    pub pagination: QueryPaginationParams,
    pub sort: QuerySortParams,
    pub search: QuerySearchParams,
    pub date_range: QueryDateRangeParams,
    pub filters: HashMap<String, Option<String>>,
    pub(crate) _phantom: PhantomData<&'q T>,
}

impl<'q, T> From<FlatQueryParams> for QueryParams<'q, T> {
    fn from(params: FlatQueryParams) -> Self {
        QueryParams {
            pagination: params.pagination.unwrap_or_default(),
            sort: params.sort.unwrap_or_default(),
            search: params.search.unwrap_or_default(),
            date_range: params.date_range.unwrap_or_default(),
            filters: params.filters.unwrap_or_default(),
            _phantom: PhantomData::<&'q T>,
        }
    }
}
