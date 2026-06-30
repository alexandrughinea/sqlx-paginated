mod paginated_query_as;

pub use crate::paginated_query_as::{
    paginated_query_as, DatabaseQueryDefaults, FieldEnum, FlatQueryParams, PaginatedInfo,
    PaginatedQueryBuilder, PaginatedResponse, QueryBuilder, QueryFilterCondition,
    QueryFilterOperator, QueryParams, QueryParamsBuilder, QuerySortDirection,
};

pub use sqlx_paginated_derive::{Fields, Paginated};

pub mod prelude {
    pub use super::{
        paginated_query_as, DatabaseQueryDefaults, FieldEnum, FlatQueryParams, PaginatedInfo,
        PaginatedQueryBuilder, PaginatedResponse, QueryBuilder, QueryFilterCondition,
        QueryFilterOperator, QueryParams, QueryParamsBuilder, QuerySortDirection,
    };

}
