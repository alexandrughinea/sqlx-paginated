use crate::paginated_query_as::internal::quote_identifier;
use crate::paginated_query_as::models::QuerySortDirection;
use crate::{FlatQueryParams, PaginatedResponse, QueryParams};
use serde::Serialize;
use sqlx::{query::QueryAs, Database, Execute, Executor, FromRow, IntoArguments, Pool};

type QueryBuilderFn<T, DB> = Box<
    dyn for<'p> Fn(&'p QueryParams<T>) -> (Vec<String>, <DB as Database>::Arguments<'p>)
        + Send
        + Sync,
>;

pub struct PaginatedQueryBuilder<'q, T, DB, A>
where
    DB: Database,
    T: for<'r> FromRow<'r, <DB as Database>::Row> + Send + Unpin,
{
    query: QueryAs<'q, DB, T, A>,
    params: QueryParams<'q, T>,
    totals_count_enabled: bool,
    build_query_fn: QueryBuilderFn<T, DB>,
}

/// A builder for constructing and executing paginated queries.
///
/// This builder provides a fluent interface for creating paginated queries.
/// For more examples explore `examples/paginated_query_builder_advanced_examples.rs`
///
/// # Type Parameters
///
/// * `'q`: The lifetime of the query and its arguments
/// * `T`: The model type that the query will return
/// * `A`: The type of the query arguments
///
/// # Generic Constraints
///
/// * `T`: Must be deserializable from Postgres rows (`FromRow`), `Send`, and `Unpin`
/// * `A`: Must be compatible with Postgres arguments and `Send`
///
impl<'q, T, DB, A> PaginatedQueryBuilder<'q, T, DB, A>
where
    DB: Database,
    T: for<'r> FromRow<'r, <DB as Database>::Row> + Send + Unpin + Serialize + Default,
    A: 'q + IntoArguments<'q, DB> + Send,
    DB::Arguments<'q>: IntoArguments<'q, DB>,
    for<'c> &'c Pool<DB>: Executor<'c, Database = DB>,
    usize: sqlx::ColumnIndex<<DB as Database>::Row>,
    i64: sqlx::Type<DB> + for<'r> sqlx::Decode<'r, DB> + Send + Unpin,
{
    /// Creates a new `PaginatedQueryBuilder` with default settings.
    ///
    /// # Arguments
    ///
    /// * `query` - The base query to paginate
    /// * `build_query_fn` - Function to build query conditions and arguments
    ///
    /// # Default Settings
    ///
    /// - Totals calculation is enabled
    /// - Uses default query parameters
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sqlx::{FromRow, Postgres};
    /// use serde::{Serialize};
    /// use sqlx_paginated::PaginatedQueryBuilder;
    ///
    /// #[derive(Serialize, FromRow, Default)]
    /// struct UserExample {
    ///     name: String
    /// }
    /// let base_query = sqlx::query_as::<Postgres, UserExample>("SELECT * FROM users");
    /// let builder = PaginatedQueryBuilder::new(base_query, |params| {
    ///     sqlx_paginated::QueryBuilder::<UserExample, Postgres>::new()
    ///         .with_search(params)
    ///         .with_filters(params)
    ///         .with_date_range(params)
    ///         .build()
    /// });
    /// ```
    pub fn new<F>(query: QueryAs<'q, DB, T, A>, build_query_fn: F) -> Self
    where
        F: for<'p> Fn(&'p QueryParams<T>) -> (Vec<String>, DB::Arguments<'p>)
            + Send
            + Sync
            + 'static,
    {
        Self {
            query,
            params: FlatQueryParams::default().into(),
            totals_count_enabled: true,
            build_query_fn: Box::new(build_query_fn),
        }
    }

    pub fn with_query_builder<F>(mut self, build_query_fn: F) -> Self
    where
        F: for<'p> Fn(&'p QueryParams<T>) -> (Vec<String>, DB::Arguments<'p>)
            + Send
            + Sync
            + 'static,
    {
        self.build_query_fn = Box::new(build_query_fn);
        self
    }

    pub fn with_params(mut self, params: impl Into<QueryParams<'q, T>>) -> Self {
        self.params = params.into();
        self
    }

    /// Disables the calculation of total record count.
    ///
    /// When disabled, the response will not include total count or total pages.
    /// This can improve query performance for large datasets where the total
    /// count is not needed.
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    pub fn disable_totals_count(mut self) -> Self {
        self.totals_count_enabled = false;
        self
    }

    /// Builds the base query with CTE (Common Table Expression).
    ///
    /// # Returns
    ///
    /// Returns the SQL string for the base query wrapped in a CTE
    fn build_base_query(&self) -> String {
        format!("WITH base_query AS ({})", self.query.sql())
    }

    /// Builds the WHERE clause from the provided conditions.
    ///
    /// # Arguments
    ///
    /// * `conditions` - Vector of condition strings to join with AND
    ///
    /// # Returns
    ///
    /// Returns the formatted WHERE clause or empty string if no conditions
    fn build_where_clause(&self, conditions: &[String]) -> String {
        if conditions.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", conditions.join(" AND "))
        }
    }

    /// Builds the ORDER BY clause based on sort parameters.
    ///
    /// # Returns
    ///
    /// Returns the formatted ORDER BY clause with proper column quoting
    fn build_order_clause(&self) -> String {
        let order = match self.params.sort.sort_direction {
            QuerySortDirection::Ascending => "ASC",
            QuerySortDirection::Descending => "DESC",
        };
        let column_name = quote_identifier(&self.params.sort.sort_column);

        format!(" ORDER BY {} {}", column_name, order)
    }

    fn build_limit_offset_clause(&self) -> String {
        let pagination = &self.params.pagination;
        let offset = (pagination.page - 1) * pagination.page_size;

        format!(" LIMIT {} OFFSET {}", pagination.page_size, offset)
    }
}

#[cfg(feature = "postgres")]
impl<'q, T, A> PaginatedQueryBuilder<'q, T, sqlx::Postgres, A>
where
    T: for<'r> FromRow<'r, <sqlx::Postgres as sqlx::Database>::Row>
        + Send
        + Unpin
        + Serialize
        + Default,
    A: 'q + IntoArguments<'q, sqlx::Postgres> + Send,
{
    /// Creates a new `PaginatedQueryBuilder` for PostgreSQL with default settings.
    ///
    /// # Arguments
    ///
    /// * `query` - The base query to paginate
    ///
    /// # Default Settings
    ///
    /// - Totals calculation is enabled
    /// - Uses default query parameters
    /// - Uses safe default query building function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sqlx::{FromRow, Postgres};
    /// use serde::{Serialize};
    /// use sqlx_paginated::PaginatedQueryBuilder;
    ///
    /// #[derive(Serialize, FromRow, Default)]
    /// struct UserExample {
    ///     name: String
    /// }
    /// let base_query = sqlx::query_as::<Postgres, UserExample>("SELECT * FROM users");
    /// let builder = PaginatedQueryBuilder::<UserExample, Postgres, _>::new_with_defaults(base_query);
    /// ```
    pub fn new_with_defaults(query: sqlx::query::QueryAs<'q, sqlx::Postgres, T, A>) -> Self {
        use crate::paginated_query_as::examples::postgres_examples::build_query_with_safe_defaults;
        Self::new(query, |params| {
            build_query_with_safe_defaults::<T, sqlx::Postgres>(params)
        })
    }

    /// Executes the paginated query and returns the results.
    ///
    /// # Arguments
    ///
    /// * `pool` - PostgreSQL database connection pool
    ///
    /// # Returns
    ///
    /// Returns a Result containing a `PaginatedResponse<T>` with:
    /// - Records for the requested page
    /// - Optional Pagination information (if enabled)
    /// - Optional total count and total pages (if enabled)
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the query execution fails
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sqlx::{FromRow, PgPool, Postgres};
    /// use serde::Serialize;
    /// use sqlx_paginated::{PaginatedQueryBuilder, QueryParamsBuilder};
    ///
    /// #[derive(Serialize, FromRow, Default)]
    /// struct User {
    ///     id: i32,
    ///     name: String,
    /// }
    ///
    /// # async fn example(pool: PgPool) -> Result<(), sqlx::Error> {
    /// let params = QueryParamsBuilder::<User>::new()
    ///     .with_pagination(1, 10)
    ///     .build();
    ///
    /// let result = PaginatedQueryBuilder::<User, Postgres, _>::new_with_defaults(
    ///     sqlx::query_as::<Postgres, User>("SELECT * FROM users")
    /// )
    /// .with_params(params)
    /// .fetch_paginated(&pool)
    /// .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn fetch_paginated(
        self,
        pool: &sqlx::PgPool,
    ) -> Result<PaginatedResponse<T>, sqlx::Error> {
        let base_sql = self.build_base_query();
        let params_ref = &self.params;
        let (conditions, main_arguments) = (self.build_query_fn)(params_ref);
        let where_clause = self.build_where_clause(&conditions);

        let count_sql = if self.totals_count_enabled {
            Some(format!(
                "{} SELECT COUNT(*) FROM base_query{}",
                base_sql, where_clause
            ))
        } else {
            None
        };

        let mut main_sql = format!("{} SELECT * FROM base_query{}", base_sql, where_clause);
        main_sql.push_str(&self.build_order_clause());
        main_sql.push_str(&self.build_limit_offset_clause());

        let (total, total_pages, pagination) = if self.totals_count_enabled {
            let (_, count_arguments) = (self.build_query_fn)(params_ref);
            let pagination_arguments = self.params.pagination.clone();
            let count_sql_str = count_sql.as_ref().unwrap();

            let count: i64 = sqlx::query_scalar_with(count_sql_str, count_arguments)
                .fetch_one(pool)
                .await?;

            let available_pages = match count {
                0 => 0,
                _ => (count + pagination_arguments.page_size - 1) / pagination_arguments.page_size,
            };

            (
                Some(count),
                Some(available_pages),
                Some(pagination_arguments),
            )
        } else {
            (None, None, None)
        };

        // For PostgreSQL, PgArguments doesn't have lifetime constraints
        let records = sqlx::query_as_with::<sqlx::Postgres, T, _>(&main_sql, main_arguments)
            .fetch_all(pool)
            .await?;

        Ok(PaginatedResponse {
            records,
            pagination,
            total,
            total_pages,
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'q, T, A> PaginatedQueryBuilder<'q, T, sqlx::Sqlite, A>
where
    T: for<'r> FromRow<'r, <sqlx::Sqlite as sqlx::Database>::Row>
        + Send
        + Unpin
        + Serialize
        + Default,
    A: 'q + IntoArguments<'q, sqlx::Sqlite> + Send,
{
    /// Creates a new `PaginatedQueryBuilder` for SQLite with default settings.
    ///
    /// # Arguments
    ///
    /// * `query` - The base query to paginate
    ///
    /// # Default Settings
    ///
    /// - Totals calculation is enabled
    /// - Uses default query parameters
    /// - Uses safe default query building function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sqlx::{FromRow, Sqlite};
    /// use serde::{Serialize};
    /// use sqlx_paginated::PaginatedQueryBuilder;
    ///
    /// #[derive(Serialize, FromRow, Default)]
    /// struct UserExample {
    ///     name: String
    /// }
    /// let base_query = sqlx::query_as::<Sqlite, UserExample>("SELECT * FROM users");
    /// let builder = PaginatedQueryBuilder::<UserExample, Sqlite, _>::new_with_defaults(base_query);
    /// ```
    pub fn new_with_defaults(query: sqlx::query::QueryAs<'q, sqlx::Sqlite, T, A>) -> Self {
        use crate::QueryBuilder;
        Self::new(query, |params| {
            QueryBuilder::<T, sqlx::Sqlite>::new()
                .with_search(params)
                .with_filters(params)
                .with_date_range(params)
                .build()
        })
    }

    /// Executes the paginated query and returns the results.
    ///
    /// # Arguments
    ///
    /// * `pool` - SQLite database connection pool
    ///
    /// # Returns
    ///
    /// Returns a Result containing a `PaginatedResponse<T>` with:
    /// - Records for the requested page
    /// - Optional Pagination information (if enabled)
    /// - Optional total count and total pages (if enabled)
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the query execution fails
    ///
    /// # Implementation Note
    ///
    /// This specialized implementation for SQLite handles lifetime requirements correctly.
    /// SQLite's `SqliteArguments<'q>` requires that SQL strings live long enough, so this
    /// implementation ensures all SQL strings are created and kept in scope before executing queries.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sqlx::{FromRow, SqlitePool, Sqlite};
    /// use serde::Serialize;
    /// use sqlx_paginated::{PaginatedQueryBuilder, QueryParamsBuilder};
    ///
    /// #[derive(Serialize, FromRow, Default)]
    /// struct User {
    ///     id: i32,
    ///     name: String,
    /// }
    ///
    /// # async fn example(pool: SqlitePool) -> Result<(), sqlx::Error> {
    /// let params = QueryParamsBuilder::<User>::new()
    ///     .with_pagination(1, 10)
    ///     .build();
    ///
    /// let result = PaginatedQueryBuilder::<User, Sqlite, _>::new_with_defaults(
    ///     sqlx::query_as::<Sqlite, User>("SELECT * FROM users")
    /// )
    /// .with_params(params)
    /// .fetch_paginated(&pool)
    /// .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn fetch_paginated(
        self,
        pool: &sqlx::SqlitePool,
    ) -> Result<PaginatedResponse<T>, sqlx::Error> {
        let base_sql = self.build_base_query();
        let params_ref = &self.params;
        let (conditions, main_arguments) = (self.build_query_fn)(params_ref);
        let where_clause = self.build_where_clause(&conditions);

        // Build all SQL strings first and keep them in scope
        // This ensures they live long enough for SqliteArguments<'q>
        let count_sql = if self.totals_count_enabled {
            Some(format!(
                "{} SELECT COUNT(*) FROM base_query{}",
                base_sql, where_clause
            ))
        } else {
            None
        };

        let mut main_sql = format!("{} SELECT * FROM base_query{}", base_sql, where_clause);
        main_sql.push_str(&self.build_order_clause());
        main_sql.push_str(&self.build_limit_offset_clause());

        // For SQLite, we need to execute queries in a way that ensures
        // the SQL strings and arguments have compatible lifetimes
        let (total, total_pages, pagination) = if self.totals_count_enabled {
            let (_, count_arguments) = (self.build_query_fn)(params_ref);
            let pagination_arguments = self.params.pagination.clone();
            let count_sql_str = count_sql.as_ref().unwrap();

            // Execute count query - SQL string and arguments are both in scope
            let count: i64 = sqlx::query_scalar_with(count_sql_str, count_arguments)
                .fetch_one(pool)
                .await?;

            let available_pages = match count {
                0 => 0,
                _ => (count + pagination_arguments.page_size - 1) / pagination_arguments.page_size,
            };

            (
                Some(count),
                Some(available_pages),
                Some(pagination_arguments),
            )
        } else {
            (None, None, None)
        };

        // Execute main query - both main_sql and main_arguments are in scope
        // The lifetime 'q from params_ref ensures compatibility
        let records = sqlx::query_as_with::<sqlx::Sqlite, T, _>(&main_sql, main_arguments)
            .fetch_all(pool)
            .await?;

        Ok(PaginatedResponse {
            records,
            pagination,
            total,
            total_pages,
        })
    }
}
