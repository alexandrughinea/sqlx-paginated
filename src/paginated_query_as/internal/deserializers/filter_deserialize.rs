use crate::paginated_query_as::models::{Filter, FilterOperator, FilterValue};
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;

/// Parses a string into a FilterOperator enum variant.
fn parse_operator(s: &str) -> Option<FilterOperator> {
    match s {
        "Eq" => Some(FilterOperator::Eq),
        "Ne" => Some(FilterOperator::Ne),
        "Gt" => Some(FilterOperator::Gt),
        "Lt" => Some(FilterOperator::Lt),
        "Gte" => Some(FilterOperator::Gte),
        "Lte" => Some(FilterOperator::Lte),
        "Like" => Some(FilterOperator::Like),
        "ILike" => Some(FilterOperator::ILike),
        "In" => Some(FilterOperator::In),
        "NotIn" => Some(FilterOperator::NotIn),
        "IsNull" => Some(FilterOperator::IsNull),
        "IsNotNull" => Some(FilterOperator::IsNotNull),
        "Between" => Some(FilterOperator::Between),
        "Contains" => Some(FilterOperator::Contains),
        _ => None,
    }
}

/// Parses a string value into a FilterValue with automatic type inference.
/// Type inference order: Bool -> Uuid -> DateTime -> Date -> Time -> Int -> Float -> String
fn parse_filter_value(s: &str) -> FilterValue {
    if s.is_empty() {
        return FilterValue::Null;
    }

    // Try boolean
    match s.to_lowercase().as_str() {
        "true" => return FilterValue::Bool(true),
        "false" => return FilterValue::Bool(false),
        _ => {}
    }

    // Try UUID
    if let Ok(uuid) = uuid::Uuid::parse_str(s) {
        return FilterValue::Uuid(uuid);
    }

    // Try DateTime (RFC 3339 / ISO 8601 with timezone)
    // Examples: 2025-12-02T10:30:00Z, 2025-12-02T10:30:00+00:00
    if DateTime::<FixedOffset>::parse_from_rfc3339(s).is_ok() {
        return FilterValue::DateTime(s.to_string());
    }

    // Try NaiveDateTime (ISO 8601 without timezone)
    // Examples: 2025-12-02T10:30:00, 2025-12-02 10:30:00
    if NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").is_ok()
        || NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").is_ok()
        || NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f").is_ok()
    {
        return FilterValue::DateTime(s.to_string());
    }

    // Try Date (YYYY-MM-DD)
    if NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok() {
        return FilterValue::Date(s.to_string());
    }

    // Try Time (HH:MM:SS)
    if NaiveTime::parse_from_str(s, "%H:%M:%S").is_ok()
        || NaiveTime::parse_from_str(s, "%H:%M").is_ok()
    {
        return FilterValue::Time(s.to_string());
    }

    // Try integer
    if let Ok(i) = s.parse::<i64>() {
        return FilterValue::Int(i);
    }

    // Try float
    if let Ok(f) = s.parse::<f64>() {
        return FilterValue::Float(f);
    }

    // Fallback to string
    FilterValue::String(s.to_string())
}

/// Parses comma-separated values into a Vec<FilterValue>.
fn parse_array_values(s: &str) -> Vec<FilterValue> {
    s.split(',')
        .map(|v| parse_filter_value(v.trim()))
        .collect()
}

/// Deserializes query parameters into a Vec<Filter>.
///
/// Expected format: `field=Operator:value`
/// Examples:
/// - `?status=Eq:Pending` -> Filter { field: "status", operator: Eq, value: String("Pending") }
/// - `?age=Gt:18` -> Filter { field: "age", operator: Gt, value: Int(18) }
/// - `?status=In:Active,Pending` -> Filter { field: "status", operator: In, value: Array([...]) }
/// - `?deleted_at=IsNull:` -> Filter { field: "deleted_at", operator: IsNull, value: Null }
pub fn filters_deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<Filter>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<HashMap<String, String>>::deserialize(deserializer)?;

    let map = match value {
        None => return Ok(None),
        Some(m) if m.is_empty() => return Ok(None),
        Some(m) => m,
    };

    let mut filters = Vec::new();

    for (field, raw_value) in map {
        if matches!(
            field.as_str(),
            "page" | "page_size" | "sort_column" | "sort_direction" | "search" | "search_columns"
        ) {
            continue;
        }

        let (operator_str, value_str) = match raw_value.split_once(':') {
            Some((op, val)) => (op, val),
            None => continue,
        };

        let operator = match parse_operator(operator_str) {
            Some(op) => op,
            None => continue,
        };

        let value = match operator {
            FilterOperator::IsNull | FilterOperator::IsNotNull => FilterValue::Null,
            FilterOperator::In | FilterOperator::NotIn | FilterOperator::Between => {
                FilterValue::Array(parse_array_values(value_str))
            }
            _ => parse_filter_value(value_str),
        };

        filters.push(Filter {
            field,
            operator,
            value,
        });
    }

    if filters.is_empty() {
        Ok(None)
    } else {
        Ok(Some(filters))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_operator() {
        assert_eq!(parse_operator("Eq"), Some(FilterOperator::Eq));
        assert_eq!(parse_operator("Ne"), Some(FilterOperator::Ne));
        assert_eq!(parse_operator("Gt"), Some(FilterOperator::Gt));
        assert_eq!(parse_operator("Lt"), Some(FilterOperator::Lt));
        assert_eq!(parse_operator("Gte"), Some(FilterOperator::Gte));
        assert_eq!(parse_operator("Lte"), Some(FilterOperator::Lte));
        assert_eq!(parse_operator("Like"), Some(FilterOperator::Like));
        assert_eq!(parse_operator("ILike"), Some(FilterOperator::ILike));
        assert_eq!(parse_operator("In"), Some(FilterOperator::In));
        assert_eq!(parse_operator("NotIn"), Some(FilterOperator::NotIn));
        assert_eq!(parse_operator("IsNull"), Some(FilterOperator::IsNull));
        assert_eq!(parse_operator("IsNotNull"), Some(FilterOperator::IsNotNull));
        assert_eq!(parse_operator("Between"), Some(FilterOperator::Between));
        assert_eq!(parse_operator("Contains"), Some(FilterOperator::Contains));
        assert_eq!(parse_operator("Invalid"), None);
    }

    #[test]
    fn test_parse_filter_value_bool() {
        assert_eq!(parse_filter_value("true"), FilterValue::Bool(true));
        assert_eq!(parse_filter_value("false"), FilterValue::Bool(false));
        assert_eq!(parse_filter_value("TRUE"), FilterValue::Bool(true));
        assert_eq!(parse_filter_value("False"), FilterValue::Bool(false));
    }

    #[test]
    fn test_parse_filter_value_int() {
        assert_eq!(parse_filter_value("123"), FilterValue::Int(123));
        assert_eq!(parse_filter_value("-456"), FilterValue::Int(-456));
        assert_eq!(parse_filter_value("0"), FilterValue::Int(0));
    }

    #[test]
    fn test_parse_filter_value_float() {
        assert_eq!(parse_filter_value("3.14"), FilterValue::Float(3.14));
        assert_eq!(parse_filter_value("-2.5"), FilterValue::Float(-2.5));
    }

    #[test]
    fn test_parse_filter_value_uuid() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let expected = uuid::Uuid::parse_str(uuid_str).unwrap();
        assert_eq!(parse_filter_value(uuid_str), FilterValue::Uuid(expected));
    }

    #[test]
    fn test_parse_filter_value_string() {
        assert_eq!(
            parse_filter_value("hello"),
            FilterValue::String("hello".to_string())
        );
        assert_eq!(
            parse_filter_value("Pending"),
            FilterValue::String("Pending".to_string())
        );
    }

    #[test]
    fn test_parse_filter_value_empty() {
        assert_eq!(parse_filter_value(""), FilterValue::Null);
    }

    #[test]
    fn test_parse_array_values() {
        let result = parse_array_values("Active,Pending,Done");
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], FilterValue::String("Active".to_string()));
        assert_eq!(result[1], FilterValue::String("Pending".to_string()));
        assert_eq!(result[2], FilterValue::String("Done".to_string()));
    }

    #[test]
    fn test_parse_array_values_mixed_types() {
        let result = parse_array_values("18,65");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], FilterValue::Int(18));
        assert_eq!(result[1], FilterValue::Int(65));
    }

    #[test]
    fn test_parse_filter_value_date() {
        assert_eq!(
            parse_filter_value("2025-12-02"),
            FilterValue::Date("2025-12-02".to_string())
        );
        assert_eq!(
            parse_filter_value("2024-01-15"),
            FilterValue::Date("2024-01-15".to_string())
        );
    }

    #[test]
    fn test_parse_filter_value_datetime() {
        // RFC 3339 with timezone
        assert_eq!(
            parse_filter_value("2025-12-02T10:30:00Z"),
            FilterValue::DateTime("2025-12-02T10:30:00Z".to_string())
        );
        assert_eq!(
            parse_filter_value("2025-12-02T10:30:00+00:00"),
            FilterValue::DateTime("2025-12-02T10:30:00+00:00".to_string())
        );
        // Naive datetime without timezone
        assert_eq!(
            parse_filter_value("2025-12-02T10:30:00"),
            FilterValue::DateTime("2025-12-02T10:30:00".to_string())
        );
        assert_eq!(
            parse_filter_value("2025-12-02 10:30:00"),
            FilterValue::DateTime("2025-12-02 10:30:00".to_string())
        );
    }

    #[test]
    fn test_parse_filter_value_time() {
        assert_eq!(
            parse_filter_value("10:30:00"),
            FilterValue::Time("10:30:00".to_string())
        );
        assert_eq!(
            parse_filter_value("14:45"),
            FilterValue::Time("14:45".to_string())
        );
    }
}
