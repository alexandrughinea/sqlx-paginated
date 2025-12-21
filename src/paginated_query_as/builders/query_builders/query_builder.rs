use crate::paginated_query_as::internal::{
    ColumnProtection, ComputedProperty, ComputedPropertyBuilder, FieldType, QueryDialect,
};
use crate::paginated_query_as::models::FilterOperator;
use crate::QueryParams;
use serde::Serialize;
use sqlx::{Arguments, Database, Encode, Type};
use std::collections::HashMap;
use std::marker::PhantomData;

/// Result of building a query with conditions, arguments, and joins.
///
/// This struct is returned by `QueryBuilder::build()` and contains all the
/// information needed to construct the final SQL query.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryBuildResult<'q, DB: Database> {
    /// SQL conditions to be combined with AND in the WHERE clause
    pub conditions: Vec<String>,
    /// Database-specific arguments for parameter binding
    pub arguments: DB::Arguments<'q>,
    /// JOIN clauses that should be included in the query (in order)
    pub joins: Vec<String>,
}

pub struct QueryBuilder<'q, T, DB: Database> {
    pub conditions: Vec<String>,
    pub arguments: DB::Arguments<'q>,
    pub mappers: HashMap<String, Box<dyn Fn(&str, &str) -> (String, Option<String>)>>,
    pub(crate) valid_columns: Vec<String>,
    pub(crate) field_meta: HashMap<String, FieldType>,
    pub(crate) protection: Option<ColumnProtection>,
    pub(crate) protection_enabled: bool,
    pub(crate) column_validation_enabled: bool,
    pub(crate) dialect: Box<dyn QueryDialect>,
    pub(crate) _phantom: PhantomData<&'q T>,
    /// Computed properties for virtual columns (e.g., from joins)
    pub(crate) computed_properties: HashMap<String, ComputedProperty>,
    /// Active JOIN clauses (in order, no duplicates)
    pub(crate) active_joins: Vec<String>,
    /// Optional table prefix for column references (e.g., "base_query" for CTE contexts)
    pub(crate) table_prefix: Option<String>,
}

impl<'q, T, DB> QueryBuilder<'q, T, DB>
where
    T: Default + Serialize,
    DB: Database,
    String: for<'a> Encode<'a, DB> + Type<DB>,
{
    /// Checks if a column exists in the list of valid columns for T struct
    /// or is a registered computed property.
    ///
    /// # Arguments
    ///
    /// * `column` - The name of the column to check
    ///
    /// # Returns
    ///
    /// Returns `true` if the column exists in the valid columns list or is a computed property.
    pub(crate) fn has_column(&self, column: &str) -> bool {
        self.valid_columns.contains(&column.to_string())
            || self.computed_properties.contains_key(column)
    }

    fn is_column_safe(&self, column: &str) -> bool {
        // Computed properties bypass validation (developer-trusted)
        if self.computed_properties.contains_key(column) {
            return true;
        }

        let column_exists = if self.column_validation_enabled { 
            self.valid_columns.contains(&column.to_string())
        } else { 
            true 
        };

        if !self.protection_enabled {
            return column_exists;
        }

        match &self.protection {
            Some(protection) => column_exists && protection.is_safe(column),
            None => column_exists,
        }
    }

    /// Activates joins for a computed property (adds to active_joins if not already present).
    fn activate_joins(&mut self, property: &ComputedProperty) {
        for join in &property.joins {
            if !self.active_joins.contains(join) {
                self.active_joins.push(join.clone());
            }
        }
    }

    /// Returns the active JOIN clauses in the order they were added.
    pub fn get_active_joins(&self) -> Vec<String> {
        self.active_joins.clone()
    }

    /// Sets a table prefix for column references.
    ///
    /// When using `QueryBuilder` with `PaginatedQueryBuilder`, the query is wrapped in a CTE
    /// named `base_query`. If you have JOINs that introduce columns with the same names as
    /// your main table, you need to prefix columns to avoid ambiguity.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sqlx::Postgres;
    /// use serde::Serialize;
    /// use sqlx_paginated::QueryBuilder;
    ///
    /// #[derive(Serialize, Default)]
    /// struct Order {
    ///     id: i64,
    ///     organization_id: i64,
    /// }
    ///
    /// let result = QueryBuilder::<Order, Postgres>::new()
    ///     .with_table_prefix("base_query")  // Columns become "base_query"."column_name"
    ///     .build();
    /// ```
    pub fn with_table_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.table_prefix = Some(prefix.into());
        self
    }

    /// Formats a column name with the table prefix if set.
    /// Returns `"prefix"."column"` if prefix is set, otherwise just `"column"`.
    fn format_column(&self, column: &str) -> String {
        match &self.table_prefix {
            Some(prefix) => format!("{}.{}", self.dialect.quote_identifier(prefix), self.dialect.quote_identifier(column)),
            None => self.dialect.quote_identifier(column),
        }
    }

    /// Registers a computed property (virtual column) that can be used in search and filter operations.
    ///
    /// Computed properties allow you to search and filter by columns that don't exist directly
    /// in your struct, such as columns from joined tables or computed SQL expressions.
    ///
    /// The closure receives a `ComputedPropertyBuilder` that can be used to configure joins,
    /// and returns the SQL expression to use for this property.
    ///
    /// **IMPORTANT**: When using with `PaginatedQueryBuilder`, JOINs must reference `base_query`
    /// (not the original table name) because the query is wrapped in a CTE:
    /// ```sql
    /// WITH base_query AS (SELECT * FROM your_table WHERE ...) SELECT * FROM base_query LEFT JOIN ...
    /// ```
    ///
    /// # Arguments
    ///
    /// * `name` - The virtual column name to use in search_columns and filters
    /// * `f` - Closure that configures the property and returns the SQL expression
    ///
    /// # Example
    ///
    /// ```rust
    /// use sqlx::Postgres;
    /// use serde::Serialize;
    /// use sqlx_paginated::QueryBuilder;
    ///
    /// #[derive(Serialize, Default)]
    /// struct Order {
    ///     id: i64,
    ///     counterparty_id: i64,
    /// }
    ///
    /// // When using with PaginatedQueryBuilder, reference base_query in JOINs:
    /// let result = QueryBuilder::<Order, Postgres>::new()
    ///     .with_computed_property("counterparty_name", |cp| {
    ///         // Note: Use "base_query" not "orders" when used with PaginatedQueryBuilder
    ///         cp.with_join("LEFT JOIN counterparty ON counterparty.id = base_query.counterparty_id");
    ///         "counterparty.legal_name"
    ///     })
    ///     // For computed expressions without joins (no table reference needed)
    ///     .with_computed_property("amount_money", |_cp| {
    ///         "(amount_micros / 1000000)::money"
    ///     })
    ///     .build();
    /// ```
    pub fn with_computed_property<F>(mut self, name: impl Into<String>, f: F) -> Self
    where
        F: FnOnce(&mut ComputedPropertyBuilder) -> &str,
    {
        let mut builder = ComputedPropertyBuilder::new();
        let expression = f(&mut builder);

        self.computed_properties.insert(
            name.into(),
            ComputedProperty {
                expression: expression.to_string(),
                joins: builder.joins,
                field_type: builder.field_type,
            },
        );
        self
    }

    pub fn map_column<F>(mut self, column: &str, mapper: F) -> Self 
    where
        F: Fn(&str, &str) -> (String, Option<String>) + 'static,
    {
        self.mappers.insert(column.to_string(), Box::new(mapper));
        self
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
                if !columns.is_empty() && !search.trim().is_empty() {
                    let pattern = format!("%{}%", search);
                    let next_argument = self.arguments.len() + 1;

                    let mut joins_to_activate: Vec<ComputedProperty> = Vec::new();

                    let search_conditions: Vec<String> = columns
                        .iter()
                        .filter_map(|column| {
                            if let Some(prop) = self.computed_properties.get(column).cloned() {
                                joins_to_activate.push(prop.clone());
                                let placeholder = self.dialect.placeholder(next_argument);
                                return if prop.field_type == FieldType::String {
                                    Some(format!(
                                        "LOWER({}) LIKE LOWER({})",
                                        prop.expression, placeholder
                                    ))
                                } else {
                                    Some(format!(
                                        "({})::text LIKE {}",
                                        prop.expression, placeholder
                                    ))
                                };
                            }

                            let mapper = self.mappers.get(column);

                            if mapper.is_none() && !self.is_column_safe(column) {
                                return None;
                            }

                            let field_type = self.field_meta.get(column).cloned().unwrap_or(FieldType::Unknown);

                            let mapped_column = mapper.map(|mapper| mapper(column, search));

                            let table_column: String = mapped_column
                                .as_ref()
                                .map(|(tc, _)| tc.clone())
                                .unwrap_or_else(|| self.format_column(column));

                            let placeholder: String = mapped_column
                                .as_ref()
                                .and_then(|(_, p)| p.clone())
                                .unwrap_or_else(|| self.dialect.placeholder(next_argument));

                            if field_type == FieldType::String {
                                Some(format!("LOWER({}) LIKE LOWER({})", table_column, placeholder))
                            } else {
                                Some(format!("{}::text LIKE {}", table_column, placeholder))
                            }
                        })
                        .collect();

                    // Activate joins for used computed properties
                    for prop in joins_to_activate {
                        self.activate_joins(&prop);
                    }

                    if !search_conditions.is_empty() {
                        self.conditions
                            .push(format!("({})", search_conditions.join(" OR ")));
                        self.arguments.add(pattern).unwrap_or_default();
                    }
                }
            }
        }
        self
    }

    /// Adds filters to the query based on provided Filter structs.
    ///
    /// # Arguments
    ///
    /// * `params` - Query parameters containing filters
    ///
    /// # Details
    ///
    /// - Supports multiple operators: Eq, Ne, Gt, Lt, Gte, Lte, Like, ILike, In, NotIn, IsNull, IsNotNull, Between, Contains
    /// - Only applies filters for columns that exist and are considered safe
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
    ///
    /// let query_builder = QueryBuilder::<UserExample, Postgres>::new()
    ///     .with_filters(&initial_params)
    ///     .build();
    /// ```
    pub fn with_filters(mut self, params: &QueryParams<T>) -> Self {
        for filter in &params.filters {
            let field = &filter.field;

            // Check for computed property first
            let (table_column, field_type) =
                if let Some(prop) = self.computed_properties.get(field).cloned() {
                    self.activate_joins(&prop);
                    (prop.expression.clone(), prop.field_type.clone())
                } else {
                    if !self.is_column_safe(field) {
                        #[cfg(feature = "tracing")]
                        tracing::warn!(column = %field, "Skipping invalid filter column");
                        continue;
                    }
                    (
                        self.format_column(field),
                        self.field_meta.get(field).cloned().unwrap_or(FieldType::Unknown),
                    )
                };


            let effective_field_type = if field_type == FieldType::Unknown {
                filter.value.to_field_type()
            } else {
                field_type
            };


            let type_cast = self.dialect.type_cast(&effective_field_type);

            let condition = match filter.operator {
                FilterOperator::Eq => {
                    let value = filter.value.to_bindable_string();
                    let placeholder = self.dialect.placeholder(self.arguments.len() + 1);
                    self.arguments.add(value).unwrap_or_default();
                    format!("{} = {}{}", table_column, placeholder, type_cast)
                }
                FilterOperator::Ne => {
                    let value = filter.value.to_bindable_string();
                    let placeholder = self.dialect.placeholder(self.arguments.len() + 1);
                    self.arguments.add(value).unwrap_or_default();
                    format!("{} != {}{}", table_column, placeholder, type_cast)
                }
                FilterOperator::Gt => {
                    let value = filter.value.to_bindable_string();
                    let placeholder = self.dialect.placeholder(self.arguments.len() + 1);
                    self.arguments.add(value).unwrap_or_default();
                    format!("{} > {}{}", table_column, placeholder, type_cast)
                }
                FilterOperator::Lt => {
                    let value = filter.value.to_bindable_string();
                    let placeholder = self.dialect.placeholder(self.arguments.len() + 1);
                    self.arguments.add(value).unwrap_or_default();
                    format!("{} < {}{}", table_column, placeholder, type_cast)
                }
                FilterOperator::Gte => {
                    let value = filter.value.to_bindable_string();
                    let placeholder = self.dialect.placeholder(self.arguments.len() + 1);
                    self.arguments.add(value).unwrap_or_default();
                    format!("{} >= {}{}", table_column, placeholder, type_cast)
                }
                FilterOperator::Lte => {
                    let value = filter.value.to_bindable_string();
                    let placeholder = self.dialect.placeholder(self.arguments.len() + 1);
                    self.arguments.add(value).unwrap_or_default();
                    format!("{} <= {}{}", table_column, placeholder, type_cast)
                }
                FilterOperator::Like => {
                    let value = filter.value.to_bindable_string();
                    let placeholder = self.dialect.placeholder(self.arguments.len() + 1);
                    self.arguments.add(value).unwrap_or_default();
                    // Cast column to text for pattern matching on non-text types
                    if effective_field_type != FieldType::String && effective_field_type != FieldType::Unknown {
                        format!("{}::text LIKE {}", table_column, placeholder)
                    } else {
                        format!("{} LIKE {}", table_column, placeholder)
                    }
                }
                FilterOperator::ILike => {
                    let value = filter.value.to_bindable_string();
                    let placeholder = self.dialect.placeholder(self.arguments.len() + 1);
                    self.arguments.add(value).unwrap_or_default();
                    // Cast column to text for pattern matching on non-text types
                    if effective_field_type != FieldType::String && effective_field_type != FieldType::Unknown {
                        format!("{}::text ILIKE {}", table_column, placeholder)
                    } else {
                        format!("{} ILIKE {}", table_column, placeholder)
                    }
                }
                FilterOperator::In => {
                    let values = filter.value.to_bindable_strings();
                    let placeholders: Vec<String> = values
                        .iter()
                        .map(|v| {
                            let placeholder = self.dialect.placeholder(self.arguments.len() + 1);
                            self.arguments.add(v.clone()).unwrap_or_default();
                            format!("{}{}", placeholder, type_cast)
                        })
                        .collect();
                    format!("{} IN ({})", table_column, placeholders.join(", "))
                }
                FilterOperator::NotIn => {
                    let values = filter.value.to_bindable_strings();
                    let placeholders: Vec<String> = values
                        .iter()
                        .map(|v| {
                            let placeholder = self.dialect.placeholder(self.arguments.len() + 1);
                            self.arguments.add(v.clone()).unwrap_or_default();
                            format!("{}{}", placeholder, type_cast)
                        })
                        .collect();
                    format!("{} NOT IN ({})", table_column, placeholders.join(", "))
                }
                FilterOperator::IsNull => format!("{} IS NULL", table_column),
                FilterOperator::IsNotNull => format!("{} IS NOT NULL", table_column),
                FilterOperator::Between => {
                    let values = filter.value.to_bindable_strings();
                    if values.len() >= 2 {
                        let placeholder1 = self.dialect.placeholder(self.arguments.len() + 1);
                        self.arguments.add(values[0].clone()).unwrap_or_default();
                        let placeholder2 = self.dialect.placeholder(self.arguments.len() + 1);
                        self.arguments.add(values[1].clone()).unwrap_or_default();
                        format!("{} BETWEEN {}{} AND {}{}", table_column, placeholder1, type_cast, placeholder2, type_cast)
                    } else {
                        continue;
                    }
                }
                FilterOperator::Contains => {
                    let value = filter.value.to_bindable_string();
                    let placeholder = self.dialect.placeholder(self.arguments.len() + 1);
                    self.arguments.add(value).unwrap_or_default();
                    format!("{} @> {}{}", table_column, placeholder, type_cast)
                }
            };

            self.conditions.push(condition);
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
            self.conditions.push(format!(
                "{} {} {}",
                self.format_column(column),
                condition.into(),
                self.dialect.placeholder(next_argument)
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

    pub fn enable_protection(mut self) -> Self {
        self.protection_enabled = true;
        self
    }

    pub fn disable_column_validation(mut self) -> Self {
        self.column_validation_enabled = false;
        self
    }

    pub fn enable_column_validation(mut self) -> Self {
        self.column_validation_enabled = true;
        self
    }

    /// Builds the final query conditions, arguments, and joins.
    ///
    /// # Returns
    ///
    /// Returns a `QueryBuildResult` containing:
    /// - `conditions`: List of SQL conditions for the WHERE clause
    /// - `arguments`: Database-specific arguments for parameter binding
    /// - `joins`: JOIN clauses to include (only those needed by used computed properties)
    ///
    /// # Example
    ///
    /// ```rust
    /// use sqlx::Postgres;
    /// use serde::Serialize;
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
    /// let result = QueryBuilder::<UserExample, Postgres>::new()
    ///     .with_search(&initial_params)
    ///     .build();
    /// // Use result.conditions, result.arguments, result.joins
    /// ```
    pub fn build(self) -> QueryBuildResult<'q, DB> {
        QueryBuildResult {
            conditions: self.conditions,
            arguments: self.arguments,
            joins: self.active_joins,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paginated_query_as::models::{Filter, FilterOperator, FilterValue, QueryParams};
    use serde::Serialize;
    use sqlx::Postgres;

    #[derive(Default, Serialize)]
    struct TestModel {
        id: i64,
        name: String,
        amount: f64,
        is_active: bool,
        user_uuid: uuid::Uuid,
    }

    // Model with Option<T> fields to test fallback to filter value type inference
    // Option<T> fields serialize to null, resulting in FieldType::Unknown
    #[derive(Default, Serialize)]
    struct TestModelWithOptions {
        id: i64,
        name: String,
        // These Option fields will have FieldType::Unknown, triggering fallback
        optional_amount: Option<f64>,
        optional_datetime: Option<String>,
        optional_date: Option<String>,
        optional_time: Option<String>,
    }

    fn make_params_with_filter(filter: Filter) -> QueryParams<'static, TestModel> {
        QueryParams {
            filters: vec![filter],
            ..Default::default()
        }
    }

    fn make_option_params_with_filter(filter: Filter) -> QueryParams<'static, TestModelWithOptions> {
        QueryParams {
            filters: vec![filter],
            ..Default::default()
        }
    }

    // ========================================
    // Type Cast Tests for Comparison Operators
    // ========================================

    #[test]
    fn test_eq_filter_int_generates_bigint_cast() {
        let filter = Filter {
            field: "id".to_string(),
            operator: FilterOperator::Eq,
            value: FilterValue::Int(123),
        };
        let params = make_params_with_filter(filter);

        let result = QueryBuilder::<TestModel, Postgres>::new()
            .with_filters(&params)
            .build();

        assert_eq!(result.conditions.len(), 1);
        assert!(
            result.conditions[0].contains("::bigint"),
            "Expected ::bigint cast, got: {}",
            result.conditions[0]
        );
    }

    #[test]
    fn test_gt_filter_int_generates_bigint_cast() {
        let filter = Filter {
            field: "id".to_string(),
            operator: FilterOperator::Gt,
            value: FilterValue::Int(100),
        };
        let params = make_params_with_filter(filter);

        let result = QueryBuilder::<TestModel, Postgres>::new()
            .with_filters(&params)
            .build();

        assert!(
            result.conditions[0].contains("::bigint"),
            "Expected ::bigint cast for Gt operator, got: {}",
            result.conditions[0]
        );
    }

    #[test]
    fn test_lt_filter_float_generates_float8_cast() {
        // Use Option field to trigger fallback to filter value type inference
        let filter = Filter {
            field: "optional_amount".to_string(),
            operator: FilterOperator::Lt,
            value: FilterValue::Float(99.99),
        };
        let params = make_option_params_with_filter(filter);

        let result = QueryBuilder::<TestModelWithOptions, Postgres>::new()
            .with_filters(&params)
            .build();

        assert!(
            result.conditions[0].contains("::float8"),
            "Expected ::float8 cast for Float value on Option field, got: {}",
            result.conditions[0]
        );
    }

    #[test]
    fn test_eq_filter_bool_generates_boolean_cast() {
        let filter = Filter {
            field: "is_active".to_string(),
            operator: FilterOperator::Eq,
            value: FilterValue::Bool(true),
        };
        let params = make_params_with_filter(filter);

        let result = QueryBuilder::<TestModel, Postgres>::new()
            .with_filters(&params)
            .build();

        assert!(
            result.conditions[0].contains("::boolean"),
            "Expected ::boolean cast, got: {}",
            result.conditions[0]
        );
    }

    #[test]
    fn test_eq_filter_uuid_generates_uuid_cast() {
        let filter = Filter {
            field: "user_uuid".to_string(),
            operator: FilterOperator::Eq,
            value: FilterValue::Uuid(uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()),
        };
        let params = make_params_with_filter(filter);

        let result = QueryBuilder::<TestModel, Postgres>::new()
            .with_filters(&params)
            .build();

        assert!(
            result.conditions[0].contains("::uuid"),
            "Expected ::uuid cast, got: {}",
            result.conditions[0]
        );
    }

    #[test]
    fn test_eq_filter_string_no_cast() {
        let filter = Filter {
            field: "name".to_string(),
            operator: FilterOperator::Eq,
            value: FilterValue::String("John".to_string()),
        };
        let params = make_params_with_filter(filter);

        let result = QueryBuilder::<TestModel, Postgres>::new()
            .with_filters(&params)
            .build();

        // String values should not have type cast
        assert!(
            !result.conditions[0].contains("::"),
            "String filter should not have type cast, got: {}",
            result.conditions[0]
        );
    }

    // ========================================
    // DateTime/Date/Time Type Cast Tests
    // ========================================

    #[test]
    fn test_gt_filter_datetime_generates_timestamptz_cast() {
        // Use Option field to trigger fallback to filter value type inference
        let filter = Filter {
            field: "optional_datetime".to_string(),
            operator: FilterOperator::Gt,
            value: FilterValue::DateTime("2025-12-02T10:30:00Z".to_string()),
        };
        let params = make_option_params_with_filter(filter);

        let result = QueryBuilder::<TestModelWithOptions, Postgres>::new()
            .with_filters(&params)
            .build();

        assert!(
            result.conditions[0].contains("::timestamptz"),
            "Expected ::timestamptz cast for DateTime value on Option field, got: {}",
            result.conditions[0]
        );
    }

    #[test]
    fn test_gte_filter_date_generates_date_cast() {
        // Use Option field to trigger fallback to filter value type inference
        let filter = Filter {
            field: "optional_date".to_string(),
            operator: FilterOperator::Gte,
            value: FilterValue::Date("2025-12-02".to_string()),
        };
        let params = make_option_params_with_filter(filter);

        let result = QueryBuilder::<TestModelWithOptions, Postgres>::new()
            .with_filters(&params)
            .build();

        assert!(
            result.conditions[0].contains("::date"),
            "Expected ::date cast for Date value on Option field, got: {}",
            result.conditions[0]
        );
    }

    #[test]
    fn test_eq_filter_time_generates_time_cast() {
        // Use Option field to trigger fallback to filter value type inference
        let filter = Filter {
            field: "optional_time".to_string(),
            operator: FilterOperator::Eq,
            value: FilterValue::Time("10:30:00".to_string()),
        };
        let params = make_option_params_with_filter(filter);

        let result = QueryBuilder::<TestModelWithOptions, Postgres>::new()
            .with_filters(&params)
            .build();

        assert!(
            result.conditions[0].contains("::time"),
            "Expected ::time cast for Time value on Option field, got: {}",
            result.conditions[0]
        );
    }

    // ========================================
    // In/NotIn/Between Operator Tests
    // ========================================

    #[test]
    fn test_in_filter_generates_cast_per_value() {
        let filter = Filter {
            field: "id".to_string(),
            operator: FilterOperator::In,
            value: FilterValue::Array(vec![
                FilterValue::Int(1),
                FilterValue::Int(2),
                FilterValue::Int(3),
            ]),
        };
        let params = make_params_with_filter(filter);

        let result = QueryBuilder::<TestModel, Postgres>::new()
            .with_filters(&params)
            .build();

        // Each value in IN clause should have ::bigint cast
        let condition = &result.conditions[0];
        let bigint_count = condition.matches("::bigint").count();
        assert_eq!(
            bigint_count, 3,
            "Expected 3 ::bigint casts in IN clause, got {} in: {}",
            bigint_count, condition
        );
    }

    #[test]
    fn test_between_filter_generates_two_casts() {
        let filter = Filter {
            field: "optional_amount".to_string(),
            operator: FilterOperator::Between,
            value: FilterValue::Array(vec![
                FilterValue::Float(10.0),
                FilterValue::Float(100.0),
            ]),
        };
        let params: QueryParams<TestModelWithOptions> = QueryParams {
            filters: vec![filter],
            ..Default::default()
        };

        let result = QueryBuilder::<TestModelWithOptions, Postgres>::new()
            .with_filters(&params)
            .build();

        let condition = &result.conditions[0];
        let float8_count = condition.matches("::float8").count();
        assert_eq!(
            float8_count, 2,
            "Expected 2 ::float8 casts in BETWEEN clause, got {} in: {}",
            float8_count, condition
        );
    }

    // ========================================
    // Like/ILike Operator Tests
    // ========================================

    #[test]
    fn test_like_on_int_field_casts_column_to_text() {
        let filter = Filter {
            field: "id".to_string(),
            operator: FilterOperator::Like,
            value: FilterValue::String("%123%".to_string()),
        };
        let params = make_params_with_filter(filter);

        let result = QueryBuilder::<TestModel, Postgres>::new()
            .with_filters(&params)
            .build();

        // When using LIKE on non-string field, column should be cast to text
        assert!(
            result.conditions[0].contains("::text LIKE"),
            "Expected column::text LIKE for non-string field, got: {}",
            result.conditions[0]
        );
    }

    #[test]
    fn test_like_on_string_field_no_column_cast() {
        let filter = Filter {
            field: "name".to_string(),
            operator: FilterOperator::Like,
            value: FilterValue::String("%John%".to_string()),
        };
        let params = make_params_with_filter(filter);

        let result = QueryBuilder::<TestModel, Postgres>::new()
            .with_filters(&params)
            .build();

        // String field should not have column cast, just LIKE
        assert!(
            !result.conditions[0].contains("::text LIKE"),
            "String field should not have ::text cast, got: {}",
            result.conditions[0]
        );
        assert!(
            result.conditions[0].contains("LIKE"),
            "Should contain LIKE operator, got: {}",
            result.conditions[0]
        );
    }

    #[test]
    fn test_ilike_on_bool_field_casts_column_to_text() {
        let filter = Filter {
            field: "is_active".to_string(),
            operator: FilterOperator::ILike,
            value: FilterValue::String("%true%".to_string()),
        };
        let params = make_params_with_filter(filter);

        let result = QueryBuilder::<TestModel, Postgres>::new()
            .with_filters(&params)
            .build();

        assert!(
            result.conditions[0].contains("::text ILIKE"),
            "Expected column::text ILIKE for non-string field, got: {}",
            result.conditions[0]
        );
    }
}
