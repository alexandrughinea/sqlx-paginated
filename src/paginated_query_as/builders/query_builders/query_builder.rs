use crate::paginated_query_as::internal::{ColumnProtection, QueryDialect};
use crate::paginated_query_as::models::{QueryFilterCondition, QueryFilterOperator};
use crate::QueryParams;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{Arguments, Database, Encode, Type};
use std::marker::PhantomData;

pub struct QueryBuilder<'q, T, DB: Database> {
    pub conditions: Vec<String>,
    pub arguments: DB::Arguments<'q>,
    pub(crate) valid_columns: Vec<String>,
    pub(crate) protection: Option<ColumnProtection>,
    pub(crate) protection_enabled: bool,
    pub(crate) dialect: Box<dyn QueryDialect>,
    pub(crate) _phantom: PhantomData<&'q T>,
}

impl<'q, T, DB> QueryBuilder<'q, T, DB>
where
    T: Default + Serialize,
    DB: Database,
    String: for<'a> Encode<'a, DB> + Type<DB>,
{
    /// Checks if a column exists in the list of valid columns for T struct.
    ///
    /// # Arguments
    ///
    /// * `column` - The name of the column to check
    ///
    /// # Returns
    ///
    /// Returns `true` if the column exists in the valid columns list, `false` otherwise.
    pub(crate) fn has_column(&self, column: &str) -> bool {
        self.valid_columns.contains(&column.to_string())
    }

    fn is_column_safe(&self, column: &str) -> bool {
        let column_exists = self.has_column(column);

        if !self.protection_enabled {
            return column_exists;
        }

        match &self.protection {
            Some(protection) => column_exists && protection.is_safe(column),
            None => column_exists,
        }
    }

    /// Adds search functionality to the query by creating LIKE conditions for specified columns.
    ///
    /// # Arguments
    ///
    /// * `params` - Query parameters containing search text and columns to search in
    ///
    /// # Details
    ///
    /// - Only searches in columns that are both specified and considered safe
    /// - Creates case-insensitive LIKE conditions with wildcards
    /// - Multiple search columns are combined with OR operators
    /// - Empty search text or no valid columns results in no conditions being added
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    ///
    /// # Example
    ///
    /// ```rust
    /// use sqlx::Postgres;
    /// use serde::{Serialize};
    /// use sqlx_paginated::{QueryBuilder, QueryParamsBuilder};
    ///
    /// #[derive(Serialize, Default)]
    /// struct UserExample {
    ///     name: String
    /// }
    ///
    /// let initial_params = QueryParamsBuilder::<UserExample>::new()
    ///         .with_search("john", vec!["name", "email"])
    ///         .build();
    /// let query_builder = QueryBuilder::<UserExample, Postgres>::new()
    ///     .with_search(&initial_params)
    ///     .build();
    /// ```
    pub fn with_search(mut self, params: &QueryParams<T>) -> Self {
        if let Some(search) = &params.search.search {
            if let Some(columns) = &params.search.search_columns {
                let valid_search_columns: Vec<&String> = columns
                    .iter()
                    .filter(|column| self.is_column_safe(column))
                    .collect();

                if !valid_search_columns.is_empty() && !search.trim().is_empty() {
                    let pattern = format!("%{}%", search);
                    let use_lower = search.is_ascii();

                    let search_conditions: Vec<String> = valid_search_columns
                        .iter()
                        .enumerate()
                        .map(|(idx, column)| {
                            let table_column = self.dialect.quote_identifier(column);
                            let placeholder =
                                self.dialect.placeholder(self.arguments.len() + idx + 1);
                            if use_lower {
                                format!("LOWER({}) LIKE LOWER({})", table_column, placeholder)
                            } else {
                                format!("{} LIKE {}", table_column, placeholder)
                            }
                        })
                        .collect();

                    if !search_conditions.is_empty() {
                        self.conditions
                            .push(format!("({})", search_conditions.join(" OR ")));
                        for _ in 0..valid_search_columns.len() {
                            self.arguments.add(pattern.clone()).unwrap_or_default();
                        }
                    }
                }
            }
        }
        self
    }

    /// Adds filter conditions to the query with support for various operators.
    ///
    /// # Arguments
    ///
    /// * `params` - Query parameters containing filters with operators
    ///
    /// # Details
    ///
    /// - Only applies filters for columns that exist and are considered safe
    /// - Supports multiple operators: =, !=, >, >=, <, <=, IN, NOT IN, IS NULL, IS NOT NULL, LIKE, NOT LIKE
    /// - Automatically handles type casting based on the database dialect
    /// - Skips invalid columns with a warning when tracing is enabled
    /// - For IN/NOT IN operators, comma-separated values are split into multiple parameters
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    ///
    /// # Example
    ///
    /// ```rust
    /// use sqlx::Postgres;
    /// use serde::{Serialize};
    /// use sqlx_paginated::{QueryBuilder, QueryParamsBuilder, QueryFilterOperator};
    ///
    /// #[derive(Serialize, Default)]
    /// struct Product {
    ///     name: String,
    ///     price: f64,
    ///     stock: i32,
    /// }
    ///
    /// let initial_params = QueryParamsBuilder::<Product>::new()
    ///         .with_filter_operator("price", QueryFilterOperator::GreaterThan, "10.00")
    ///         .with_filter_operator("stock", QueryFilterOperator::LessOrEqual, "100")
    ///         .build();
    ///
    /// let query_builder = QueryBuilder::<Product, Postgres>::new()
    ///     .with_filters(&initial_params)
    ///     .build();
    /// ```
    pub fn with_filters(mut self, params: &'q QueryParams<T>) -> Self {
        for (key, condition) in &params.filters {
            if self.is_column_safe(key) {
                self = self.apply_filter_condition(key, condition);
            } else {
                #[cfg(feature = "tracing")]
                tracing::warn!(column = %key, "Skipping invalid filter column");
            }
        }
        self
    }

    /// Applies a single filter condition to the query.
    ///
    /// This is a helper method that handles the SQL generation for different operators.
    fn apply_filter_condition(mut self, column: &str, condition: &'q QueryFilterCondition) -> Self {
        let table_column = self.dialect.quote_identifier(column);

        match &condition.operator {
            QueryFilterOperator::IsNull => {
                self.conditions.push(format!("{} IS NULL", table_column));
            }
            QueryFilterOperator::IsNotNull => {
                self.conditions
                    .push(format!("{} IS NOT NULL", table_column));
            }
            QueryFilterOperator::In | QueryFilterOperator::NotIn => {
                if let Some(_value) = &condition.value {
                    let values = condition.split_values();
                    if !values.is_empty() {
                        let mut placeholders = Vec::new();
                        for val in values {
                            let next_argument = self.arguments.len() + 1;
                            let placeholder = self.dialect.placeholder(next_argument);
                            let type_cast = self.dialect.type_cast(&val);
                            placeholders.push(format!("{}{}", placeholder, type_cast));
                            self.arguments.add(val).unwrap_or_default();
                        }

                        let operator = condition.operator.to_sql();
                        self.conditions.push(format!(
                            "{} {} ({})",
                            table_column,
                            operator,
                            placeholders.join(", ")
                        ));
                    }
                }
            }
            QueryFilterOperator::Like | QueryFilterOperator::NotLike => {
                if let Some(value) = &condition.value {
                    let next_argument = self.arguments.len() + 1;
                    let placeholder = self.dialect.placeholder(next_argument);
                    let operator = condition.operator.to_sql();

                    self.conditions.push(format!(
                        "LOWER({}) {} LOWER({})",
                        table_column, operator, placeholder
                    ));
                    self.arguments.add(value).unwrap_or_default();
                }
            }
            _ => {
                // Handle all comparison operators: =, !=, >, >=, <, <=
                if let Some(value) = &condition.value {
                    let next_argument = self.arguments.len() + 1;
                    let placeholder = self.dialect.placeholder(next_argument);
                    let type_cast = self.dialect.type_cast(value);
                    let operator = condition.operator.to_sql();

                    self.conditions.push(format!(
                        "{} {} {}{}",
                        table_column, operator, placeholder, type_cast
                    ));
                    self.arguments.add(value).unwrap_or_default();
                }
            }
        }

        self
    }

    /// Adds date range conditions to the query for a specified date column.
    ///
    /// # Arguments
    ///
    /// * `params` - Query parameters containing date range information
    ///
    /// # Type Parameters
    ///
    /// Requires `DateTime<Utc>` to be encodable for the target database
    ///
    /// # Details
    ///
    /// - Adds >= condition for date_after if specified
    /// - Adds <= condition for date_before if specified
    /// - Only applies to columns that exist and are considered safe
    /// - Skips invalid date columns with a warning when tracing is enabled
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    ///
    /// # Example
    ///
    /// ```rust
    /// use sqlx::Postgres;
    /// use serde::{Serialize};
    /// use chrono::{DateTime};
    /// use sqlx_paginated::{QueryBuilder, QueryParamsBuilder, QueryParams};
    ///
    /// #[derive(Serialize, Default)]
    /// struct UserExample {
    ///     name: String
    /// }
    ///
    /// let initial_params = QueryParamsBuilder::<UserExample>::new()
    ///         .with_date_range(None, Some(DateTime::parse_from_rfc3339("2024-12-31T23:59:59Z").unwrap().into()), Some("deleted_at"))
    ///         .build();
    /// let query_builder = QueryBuilder::<UserExample, Postgres>::new()
    ///     .with_date_range(&initial_params)
    ///     .build();
    /// ```
    pub fn with_date_range(mut self, params: &'q QueryParams<T>) -> Self
    where
        DateTime<Utc>: for<'a> Encode<'a, DB> + Type<DB>,
    {
        if let Some(date_column) = &params.date_range.date_column {
            if self.is_column_safe(date_column) {
                if let Some(after) = params.date_range.date_after {
                    let next_argument = self.arguments.len() + 1;
                    let table_column = self.dialect.quote_identifier(date_column);
                    let placeholder = self.dialect.placeholder(next_argument);
                    self.conditions
                        .push(format!("{} >= {}", table_column, placeholder));
                    self.arguments.add(after).unwrap_or_default();
                }

                if let Some(before) = params.date_range.date_before {
                    let next_argument = self.arguments.len() + 1;
                    let table_column = self.dialect.quote_identifier(date_column);
                    let placeholder = self.dialect.placeholder(next_argument);
                    self.conditions
                        .push(format!("{} <= {}", table_column, placeholder));
                    self.arguments.add(before).unwrap_or_default();
                }
            } else {
                #[cfg(feature = "tracing")]
                tracing::warn!(column = %date_column, "Skipping invalid date column");
            }
        }

        self
    }

    /// Adds a custom condition for a specific column with a provided operator and value.
    ///
    /// # Arguments
    ///
    /// * `column` - The column name to apply the condition to
    /// * `condition` - The operator or condition to use (e.g., ">", "LIKE", etc.)
    /// * `value` - The value to compare against
    ///
    /// # Details
    ///
    /// - Only applies to columns that exist and are considered safe
    /// - Automatically handles parameter binding
    /// - Skips invalid columns with a warning when tracing is enabled
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    ///
    /// # Example
    ///
    /// ```rust
    /// use sqlx::Postgres;
    /// use serde::{Serialize};
    /// use sqlx_paginated::{QueryBuilder};
    ///
    /// #[derive(Serialize, Default)]
    /// struct UserExample {
    ///     name: String
    /// }
    ///
    /// let query_builder = QueryBuilder::<UserExample, Postgres>::new()
    ///     .with_condition("age", ">", "18".to_string())
    ///     .build();
    /// ```
    pub fn with_condition(
        mut self,
        column: &str,
        condition: impl Into<String>,
        value: String,
    ) -> Self {
        if self.is_column_safe(column) {
            let next_argument = self.arguments.len() + 1;
            let table_column = self.dialect.quote_identifier(column);
            let placeholder = self.dialect.placeholder(next_argument);
            self.conditions.push(format!(
                "{} {} {}",
                table_column,
                condition.into(),
                placeholder
            ));
            let _ = self.arguments.add(value);
        } else {
            #[cfg(feature = "tracing")]
            tracing::warn!(column = %column, "Skipping invalid condition column");
        }
        self
    }

    /// Adds a raw SQL condition to the query without any safety checks.
    ///
    /// # Arguments
    ///
    /// * `condition` - Raw SQL condition to add to the query
    ///
    /// # Safety
    ///
    /// This method bypasses column safety checks. Use with caution to prevent SQL injection.
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    ///
    /// # Example
    ///
    /// ```rust
    /// use sqlx::Postgres;
    /// use serde::{Serialize};
    /// use sqlx_paginated::{QueryBuilder};
    ///
    /// #[derive(Serialize, Default)]
    /// struct UserExample {
    ///     name: String
    /// }
    ///
    /// let query_builder = QueryBuilder::<UserExample, Postgres>::new()
    ///     .with_raw_condition("status != 'deleted'")
    ///     .build();
    /// ```
    pub fn with_raw_condition(mut self, condition: impl Into<String>) -> Self {
        self.conditions.push(condition.into());
        self
    }

    /// Allows adding multiple conditions using a closure.
    ///
    /// # Arguments
    ///
    /// * `f` - Closure that takes a mutable reference to the QueryBuilder
    ///
    /// # Details
    ///
    /// Useful for grouping multiple conditions that are logically related
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    ///
    /// # Example
    ///
    /// ```rust
    /// use sqlx::Postgres;
    /// use serde::{Serialize};
    /// use sqlx_paginated::{QueryBuilder};
    ///
    /// #[derive(Serialize, Default)]
    /// struct UserExample {
    ///     name: String
    /// }
    /// let query_builder = QueryBuilder::<UserExample, Postgres>::new()
    ///     .with_combined_conditions(|builder| {
    ///         builder.conditions.push("status = 'active'".to_string());
    ///         builder.conditions.push("age >= 18".to_string());
    ///     })
    ///     .build();
    /// ```
    pub fn with_combined_conditions<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut QueryBuilder<T, DB>),
    {
        f(&mut self);
        self
    }

    /// Disables column protection for this query builder instance.
    ///
    /// # Safety
    ///
    /// This removes all column safety checks. Use with caution as it may expose
    /// the application to SQL injection if used with untrusted input.
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    ///
    /// # Example
    ///
    /// ```rust
    /// use sqlx::Postgres;
    /// use serde::{Serialize};
    /// use sqlx_paginated::{QueryBuilder};
    ///
    /// #[derive(Serialize, Default)]
    /// struct UserExample {
    ///     name: String
    /// }
    ///
    /// let query_builder = QueryBuilder::<UserExample, Postgres>::new()
    ///     .disable_protection()
    ///     .with_raw_condition("custom_column = 'value'")
    ///     .build();
    /// ```
    pub fn disable_protection(mut self) -> Self {
        self.protection_enabled = false;
        self
    }

    /// Builds the final query conditions and arguments.
    ///
    /// # Returns
    ///
    /// Returns a tuple containing:
    /// - Vec<String>: List of SQL conditions
    /// - DB::Arguments: Database-specific arguments for parameter binding
    ///
    /// # Example
    ///
    /// ```rust
    /// use sqlx::Postgres;
    /// use serde::{Serialize};
    /// use sqlx_paginated::{QueryBuilder, QueryParamsBuilder};
    ///
    /// #[derive(Serialize, Default)]
    /// struct UserExample {
    ///     name: String
    /// }
    ///
    /// let initial_params = QueryParamsBuilder::<UserExample>::new()
    ///         .with_search("john", vec!["name", "email"])
    ///         .build();
    /// let (conditions, arguments) = QueryBuilder::<UserExample, Postgres>::new()
    ///     .with_search(&initial_params)
    ///     .build();
    /// ```
    pub fn build(self) -> (Vec<String>, DB::Arguments<'q>) {
        (self.conditions, self.arguments)
    }
}
