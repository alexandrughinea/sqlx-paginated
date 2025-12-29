mod paginated_query_as;

pub use crate::paginated_query_as::{
    paginated_query_as, FlatQueryParams, PaginatedQueryBuilder, PaginatedResponse, QueryBuilder,
    QueryParams, QueryParamsBuilder, QuerySortDirection,
};

#[cfg(feature = "sqlite")]
pub use crate::paginated_query_as::paginated_query_as_sqlite;

pub mod prelude {
    pub use super::{
        paginated_query_as, FlatQueryParams, PaginatedQueryBuilder, PaginatedResponse,
        QueryBuilder, QueryParams, QueryParamsBuilder, QuerySortDirection,
    };

    #[cfg(feature = "sqlite")]
    pub use super::paginated_query_as_sqlite;
}
