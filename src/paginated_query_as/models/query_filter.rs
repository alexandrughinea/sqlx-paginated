use serde::{Deserialize, Serialize};

/// Represents SQL comparison operators for filtering.
///
/// These operators provide type-safe filtering capabilities beyond simple equality.
/// All operators are validated and protected against SQL injection.
///
/// # Security
///
/// - All operators use parameterized queries
/// - Column names are validated against the model struct
/// - Input values are bound as parameters, never concatenated
///
/// # Examples
///
/// ```rust
/// use sqlx_paginated::{QueryFilterOperator, QueryParamsBuilder};
/// use serde::Serialize;
///
/// #[derive(Serialize, Default)]
/// struct Product {
///     price: f64,
///     stock: i32,
///     status: String,
/// }
///
/// let params = QueryParamsBuilder::<Product>::new()
///     .with_filter_operator("price", QueryFilterOperator::GreaterThan, "10.00")
///     .with_filter_operator("stock", QueryFilterOperator::LessOrEqual, "100")
///     .with_filter_operator("status", QueryFilterOperator::NotEqual, "deleted")
///     .build();
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum QueryFilterOperator {
    /// Equal to (`=`)
    ///
    /// Example: `age = 25`
    #[serde(alias = "eq")]
    #[default]
    Equal,

    /// Not equal to (`!=` or `<>`)
    ///
    /// Example: `status != 'deleted'`
    #[serde(alias = "ne", alias = "neq")]
    NotEqual,

    /// Greater than (`>`)
    ///
    /// Example: `price > 10.00`
    #[serde(alias = "gt")]
    GreaterThan,

    /// Greater than or equal to (`>=`)
    ///
    /// Example: `age >= 18`
    #[serde(alias = "gte")]
    GreaterOrEqual,

    /// Less than (`<`)
    ///
    /// Example: `stock < 10`
    #[serde(alias = "lt")]
    LessThan,

    /// Less than or equal to (`<=`)
    ///
    /// Example: `price <= 100.00`
    #[serde(alias = "lte")]
    LessOrEqual,

    /// IN clause - value in a list
    ///
    /// Values should be comma-separated strings.
    /// Example: `status IN ('active', 'pending')`
    #[serde(alias = "in")]
    In,

    /// NOT IN clause - value not in a list
    ///
    /// Values should be comma-separated strings.
    /// Example: `role NOT IN ('admin', 'moderator')`
    #[serde(alias = "nin", alias = "not_in")]
    NotIn,

    /// IS NULL check
    ///
    /// Example: `deleted_at IS NULL`
    #[serde(alias = "is_null")]
    IsNull,

    /// IS NOT NULL check
    ///
    /// Example: `email IS NOT NULL`
    #[serde(alias = "is_not_null", alias = "not_null")]
    IsNotNull,

    /// LIKE pattern matching (case-insensitive)
    ///
    /// Supports SQL wildcards: `%` (any characters) and `_` (single character)
    /// Example: `email LIKE '%@company.com'`
    #[serde(alias = "like")]
    Like,

    /// NOT LIKE pattern matching (case-insensitive)
    ///
    /// Example: `email NOT LIKE '%@spam.com'`
    #[serde(alias = "not_like", alias = "nlike")]
    NotLike,
}

impl QueryFilterOperator {
    /// Returns the SQL representation of the operator.
    pub fn to_sql(&self) -> &'static str {
        match self {
            QueryFilterOperator::Equal => "=",
            QueryFilterOperator::NotEqual => "!=",
            QueryFilterOperator::GreaterThan => ">",
            QueryFilterOperator::GreaterOrEqual => ">=",
            QueryFilterOperator::LessThan => "<",
            QueryFilterOperator::LessOrEqual => "<=",
            QueryFilterOperator::In => "IN",
            QueryFilterOperator::NotIn => "NOT IN",
            QueryFilterOperator::IsNull => "IS NULL",
            QueryFilterOperator::IsNotNull => "IS NOT NULL",
            QueryFilterOperator::Like => "LIKE",
            QueryFilterOperator::NotLike => "NOT LIKE",
        }
    }

    /// Returns true if the operator requires a value (excludes IS NULL/IS NOT NULL).
    pub fn requires_value(&self) -> bool {
        !matches!(
            self,
            QueryFilterOperator::IsNull | QueryFilterOperator::IsNotNull
        )
    }

    /// Returns true if the operator accepts multiple values (IN/NOT IN).
    pub fn accepts_multiple_values(&self) -> bool {
        matches!(self, QueryFilterOperator::In | QueryFilterOperator::NotIn)
    }

    /// Parses an operator from a string representation.
    ///
    /// # Arguments
    ///
    /// * `s` - String representation of the operator
    ///
    /// # Returns
    ///
    /// Returns the corresponding `QueryFilterOperator` or defaults to `Equal` if not recognized.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "eq" | "equal" => QueryFilterOperator::Equal,
            "ne" | "neq" | "not_equal" => QueryFilterOperator::NotEqual,
            "gt" | "greater_than" => QueryFilterOperator::GreaterThan,
            "gte" | "greater_or_equal" => QueryFilterOperator::GreaterOrEqual,
            "lt" | "less_than" => QueryFilterOperator::LessThan,
            "lte" | "less_or_equal" => QueryFilterOperator::LessOrEqual,
            "in" => QueryFilterOperator::In,
            "nin" | "not_in" => QueryFilterOperator::NotIn,
            "is_null" | "null" => QueryFilterOperator::IsNull,
            "is_not_null" | "not_null" => QueryFilterOperator::IsNotNull,
            "like" => QueryFilterOperator::Like,
            "not_like" | "nlike" => QueryFilterOperator::NotLike,
            _ => QueryFilterOperator::Equal,
        }
    }
}

/// Represents a complete filter condition with operator and value(s).
///
/// This structure encapsulates a filtering operation, including the operator
/// and the value(s) to compare against. It's used internally to build
/// type-safe SQL WHERE conditions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryFilterCondition {
    /// The comparison operator to use
    pub operator: QueryFilterOperator,

    /// The value(s) to compare against
    ///
    /// - For most operators: single value (e.g., "10", "active")
    /// - For IN/NOT IN: comma-separated values (e.g., "admin,moderator,user")
    /// - For IS NULL/IS NOT NULL: ignored (can be None)
    pub value: Option<String>,
}

impl QueryFilterCondition {
    /// Creates a new filter condition.
    ///
    /// # Arguments
    ///
    /// * `operator` - The comparison operator
    /// * `value` - The value to compare against (None for IS NULL/IS NOT NULL)
    pub fn new(operator: QueryFilterOperator, value: Option<impl Into<String>>) -> Self {
        Self {
            operator,
            value: value.map(Into::into),
        }
    }

    /// Creates an equality filter condition.
    pub fn equal(value: impl Into<String>) -> Self {
        Self::new(QueryFilterOperator::Equal, Some(value))
    }

    /// Creates a not equal filter condition.
    pub fn not_equal(value: impl Into<String>) -> Self {
        Self::new(QueryFilterOperator::NotEqual, Some(value))
    }

    /// Creates a greater than filter condition.
    pub fn greater_than(value: impl Into<String>) -> Self {
        Self::new(QueryFilterOperator::GreaterThan, Some(value))
    }

    /// Creates a greater or equal filter condition.
    pub fn greater_or_equal(value: impl Into<String>) -> Self {
        Self::new(QueryFilterOperator::GreaterOrEqual, Some(value))
    }

    /// Creates a less than filter condition.
    pub fn less_than(value: impl Into<String>) -> Self {
        Self::new(QueryFilterOperator::LessThan, Some(value))
    }

    /// Creates a less or equal filter condition.
    pub fn less_or_equal(value: impl Into<String>) -> Self {
        Self::new(QueryFilterOperator::LessOrEqual, Some(value))
    }

    /// Creates an IN filter condition.
    ///
    /// # Arguments
    ///
    /// * `values` - Vector of values to check against
    pub fn in_list(values: Vec<impl Into<String>>) -> Self {
        let value = values
            .into_iter()
            .map(|v| v.into())
            .collect::<Vec<String>>()
            .join(",");
        Self::new(QueryFilterOperator::In, Some(value))
    }

    /// Creates a NOT IN filter condition.
    pub fn not_in_list(values: Vec<impl Into<String>>) -> Self {
        let value = values
            .into_iter()
            .map(|v| v.into())
            .collect::<Vec<String>>()
            .join(",");
        Self::new(QueryFilterOperator::NotIn, Some(value))
    }

    /// Creates an IS NULL filter condition.
    pub fn is_null() -> Self {
        Self::new(QueryFilterOperator::IsNull, None::<String>)
    }

    /// Creates an IS NOT NULL filter condition.
    pub fn is_not_null() -> Self {
        Self::new(QueryFilterOperator::IsNotNull, None::<String>)
    }

    /// Creates a LIKE filter condition.
    pub fn like(pattern: impl Into<String>) -> Self {
        Self::new(QueryFilterOperator::Like, Some(pattern))
    }

    /// Creates a NOT LIKE filter condition.
    pub fn not_like(pattern: impl Into<String>) -> Self {
        Self::new(QueryFilterOperator::NotLike, Some(pattern))
    }

    /// Splits the value into a vector for IN/NOT IN operations.
    pub fn split_values(&self) -> Vec<String> {
        if let Some(ref value) = self.value {
            value
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            vec![]
        }
    }
}

// Backward compatibility: allow conversion from simple string to equal filter
impl From<String> for QueryFilterCondition {
    fn from(value: String) -> Self {
        QueryFilterCondition::equal(value)
    }
}

impl From<&str> for QueryFilterCondition {
    fn from(value: &str) -> Self {
        QueryFilterCondition::equal(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operator_to_sql() {
        assert_eq!(QueryFilterOperator::Equal.to_sql(), "=");
        assert_eq!(QueryFilterOperator::GreaterThan.to_sql(), ">");
        assert_eq!(QueryFilterOperator::In.to_sql(), "IN");
        assert_eq!(QueryFilterOperator::IsNull.to_sql(), "IS NULL");
    }

    #[test]
    fn test_operator_requires_value() {
        assert!(QueryFilterOperator::Equal.requires_value());
        assert!(QueryFilterOperator::GreaterThan.requires_value());
        assert!(!QueryFilterOperator::IsNull.requires_value());
        assert!(!QueryFilterOperator::IsNotNull.requires_value());
    }

    #[test]
    fn test_operator_accepts_multiple() {
        assert!(QueryFilterOperator::In.accepts_multiple_values());
        assert!(QueryFilterOperator::NotIn.accepts_multiple_values());
        assert!(!QueryFilterOperator::Equal.accepts_multiple_values());
    }

    #[test]
    fn test_operator_from_str() {
        assert_eq!(
            QueryFilterOperator::from_str("gt"),
            QueryFilterOperator::GreaterThan
        );
        assert_eq!(
            QueryFilterOperator::from_str("lte"),
            QueryFilterOperator::LessOrEqual
        );
        assert_eq!(
            QueryFilterOperator::from_str("invalid"),
            QueryFilterOperator::Equal
        );
    }

    #[test]
    fn test_filter_condition_constructors() {
        let cond = QueryFilterCondition::equal("test");
        assert_eq!(cond.operator, QueryFilterOperator::Equal);
        assert_eq!(cond.value, Some("test".to_string()));

        let cond = QueryFilterCondition::is_null();
        assert_eq!(cond.operator, QueryFilterOperator::IsNull);
        assert_eq!(cond.value, None);
    }

    #[test]
    fn test_filter_condition_in_list() {
        let cond = QueryFilterCondition::in_list(vec!["admin", "moderator", "user"]);
        assert_eq!(cond.operator, QueryFilterOperator::In);
        assert_eq!(cond.value, Some("admin,moderator,user".to_string()));

        let values = cond.split_values();
        assert_eq!(values, vec!["admin", "moderator", "user"]);
    }

    #[test]
    fn test_filter_condition_from_string() {
        let cond: QueryFilterCondition = "test_value".into();
        assert_eq!(cond.operator, QueryFilterOperator::Equal);
        assert_eq!(cond.value, Some("test_value".to_string()));
    }
}
