use crate::paginated_query_as::internal::DEFAULT_EMPTY_VALUE;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime};
use sqlx::types::Uuid;

/// SQLite type casting utility
/// Note: SQLite uses dynamic typing with type affinities (NULL, INTEGER, REAL, TEXT, BLOB)
/// This function returns empty string for most cases as SQLite handles type conversion automatically
pub fn get_sqlite_type_casting(value: &str) -> &'static str {
    match value.trim().to_string().to_lowercase().as_str() {
        // Special values - no casting needed
        value
            if value.eq_ignore_ascii_case("null")
                || value.eq_ignore_ascii_case("nan")
                || value.eq_ignore_ascii_case("infinity")
                || value.eq_ignore_ascii_case("-infinity") =>
        {
            DEFAULT_EMPTY_VALUE
        }

        // Booleans - SQLite stores as INTEGER (0 or 1)
        value if value == "true" || value == "false" => DEFAULT_EMPTY_VALUE,
        value if value == "t" || value == "f" => DEFAULT_EMPTY_VALUE,
        value if value == "0" || value == "1" => DEFAULT_EMPTY_VALUE,

        // UUIDs - SQLite stores as TEXT or BLOB
        value if Uuid::parse_str(value).is_ok() => DEFAULT_EMPTY_VALUE,

        // Binary data - SQLite stores as BLOB
        // SQLite hex literal format: X'...' or x'...'
        value if (value.starts_with("x'") || value.starts_with("X'")) && value.ends_with('\'') => {
            DEFAULT_EMPTY_VALUE
        }

        // JSON - SQLite supports JSON via JSON1 extension (stored as TEXT)
        value if value.starts_with('{') || value.starts_with('[') => DEFAULT_EMPTY_VALUE,

        // Dates and Times - SQLite stores as TEXT, REAL, or INTEGER
        value if NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S").is_ok() => {
            DEFAULT_EMPTY_VALUE
        }
        value if NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok() => DEFAULT_EMPTY_VALUE,
        value if NaiveTime::parse_from_str(value, "%H:%M:%S").is_ok() => DEFAULT_EMPTY_VALUE,
        value if DateTime::parse_from_rfc3339(value).is_ok() => DEFAULT_EMPTY_VALUE,

        // Numbers - SQLite stores as INTEGER or REAL
        value if value.parse::<i64>().is_ok() => DEFAULT_EMPTY_VALUE,
        value if value.parse::<f64>().is_ok() => DEFAULT_EMPTY_VALUE,

        // Default - SQLite handles type conversion automatically
        _ => DEFAULT_EMPTY_VALUE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_special_values() {
        // NULL should return no cast
        assert_eq!(get_sqlite_type_casting("NULL"), "");
        assert_eq!(get_sqlite_type_casting("null"), "");
        assert_eq!(get_sqlite_type_casting("Null"), "");

        // Special numeric values
        assert_eq!(get_sqlite_type_casting("NaN"), "");
        assert_eq!(get_sqlite_type_casting("nan"), "");
        assert_eq!(get_sqlite_type_casting("Infinity"), "");
        assert_eq!(get_sqlite_type_casting("infinity"), "");
        assert_eq!(get_sqlite_type_casting("-Infinity"), "");
        assert_eq!(get_sqlite_type_casting("-infinity"), "");
    }

    #[test]
    fn test_boolean_types() {
        // SQLite stores booleans as INTEGER (0 or 1)
        // Standard boolean values
        assert_eq!(get_sqlite_type_casting("true"), "");
        assert_eq!(get_sqlite_type_casting("false"), "");
        assert_eq!(get_sqlite_type_casting("t"), "");
        assert_eq!(get_sqlite_type_casting("f"), "");

        // Integer representation
        assert_eq!(get_sqlite_type_casting("0"), "");
        assert_eq!(get_sqlite_type_casting("1"), "");

        // Case variations should NOT match (already lowercased in match)
        assert_eq!(get_sqlite_type_casting("TRUE"), "");
        assert_eq!(get_sqlite_type_casting("FALSE"), "");
        assert_eq!(get_sqlite_type_casting("True"), "");
        assert_eq!(get_sqlite_type_casting("False"), "");
    }

    #[test]
    fn test_numeric_types() {
        // SQLite stores numbers as INTEGER or REAL
        // Integers
        assert_eq!(get_sqlite_type_casting("42"), "");
        assert_eq!(get_sqlite_type_casting("-42"), "");
        assert_eq!(get_sqlite_type_casting("9223372036854775807"), ""); // i64 max
        assert_eq!(get_sqlite_type_casting("-9223372036854775808"), ""); // i64 min

        // Floating point
        assert_eq!(get_sqlite_type_casting("3.14"), "");
        assert_eq!(get_sqlite_type_casting("-3.14"), "");
        assert_eq!(get_sqlite_type_casting("1.23e-4"), "");
        assert_eq!(get_sqlite_type_casting("1.23E+4"), "");
        assert_eq!(get_sqlite_type_casting("0.0"), "");

        // Edge cases
        assert_eq!(get_sqlite_type_casting("0"), "");
        assert_eq!(get_sqlite_type_casting("-0"), "");
    }

    #[test]
    fn test_date_time_types() {
        // SQLite stores date/time as TEXT, REAL, or INTEGER
        // Date
        assert_eq!(get_sqlite_type_casting("2024-01-01"), "");
        assert_eq!(get_sqlite_type_casting("2024-12-31"), "");

        // Time
        assert_eq!(get_sqlite_type_casting("12:34:56"), "");
        assert_eq!(get_sqlite_type_casting("23:59:59"), "");
        assert_eq!(get_sqlite_type_casting("00:00:00"), "");

        // DateTime (without timezone)
        assert_eq!(get_sqlite_type_casting("2024-01-01 12:34:56"), "");
        assert_eq!(get_sqlite_type_casting("2024-12-31 23:59:59"), "");

        // DateTime with timezone (ISO 8601 / RFC 3339)
        assert_eq!(get_sqlite_type_casting("2024-01-01T12:34:56Z"), "");
        assert_eq!(get_sqlite_type_casting("2024-01-01T12:34:56+00:00"), "");
        assert_eq!(get_sqlite_type_casting("2024-01-01T12:34:56-05:00"), "");
    }

    #[test]
    fn test_json_types() {
        // SQLite supports JSON via JSON1 extension (stored as TEXT)
        // Objects
        assert_eq!(get_sqlite_type_casting("{}"), "");
        assert_eq!(get_sqlite_type_casting("{\"key\":\"value\"}"), "");
        assert_eq!(
            get_sqlite_type_casting("{\"nested\":{\"key\":\"value\"}}"),
            ""
        );

        // Arrays
        assert_eq!(get_sqlite_type_casting("[]"), "");
        assert_eq!(get_sqlite_type_casting("[1,2,3]"), "");
        assert_eq!(get_sqlite_type_casting("[{\"key\":\"value\"}]"), "");

        // Complex JSON
        assert_eq!(
            get_sqlite_type_casting("{\"array\":[1,2,3],\"object\":{\"key\":\"value\"}}"),
            ""
        );

        // Invalid JSON should still return empty (SQLite will handle as TEXT)
        assert_eq!(get_sqlite_type_casting("{invalid json}"), "");
    }

    #[test]
    fn test_binary_types() {
        // SQLite stores binary data as BLOB
        // SQLite hex literal format: X'...' or x'...'
        assert_eq!(get_sqlite_type_casting("X'0123456789ABCDEF'"), "");
        assert_eq!(get_sqlite_type_casting("x'0123456789abcdef'"), "");
        assert_eq!(get_sqlite_type_casting("X''"), ""); // Empty binary

        // Mixed case hex
        assert_eq!(get_sqlite_type_casting("X'DeAdBeEf'"), "");

        // Invalid hex should still return empty
        assert_eq!(get_sqlite_type_casting("X'GG'"), "");

        // Not SQLite hex format
        assert_eq!(get_sqlite_type_casting("0x1234"), ""); // C-style hex
    }

    #[test]
    fn test_uuid_types() {
        // SQLite stores UUIDs as TEXT (36 chars) or BLOB (16 bytes)
        // Standard UUID
        assert_eq!(
            get_sqlite_type_casting("550e8400-e29b-41d4-a716-446655440000"),
            ""
        );

        // UUID with uppercase
        assert_eq!(
            get_sqlite_type_casting("550E8400-E29B-41D4-A716-446655440000"),
            ""
        );

        // Mixed case
        assert_eq!(
            get_sqlite_type_casting("550e8400-E29B-41d4-A716-446655440000"),
            ""
        );

        // Invalid UUIDs should return empty string
        assert_eq!(get_sqlite_type_casting("550e8400-e29b-41d4-a716"), "");
        assert_eq!(
            get_sqlite_type_casting("550e8400-e29b-41d4-a716-44665544000G"),
            ""
        );
        assert_eq!(get_sqlite_type_casting("not-a-uuid"), "");
    }

    #[test]
    fn test_edge_cases() {
        // Empty string
        assert_eq!(get_sqlite_type_casting(""), "");

        // Whitespace
        assert_eq!(get_sqlite_type_casting("   "), "");
        assert_eq!(get_sqlite_type_casting("\t"), "");
        assert_eq!(get_sqlite_type_casting("\n"), "");

        // Mixed invalid types
        assert_eq!(get_sqlite_type_casting("123abc"), "");
        assert_eq!(get_sqlite_type_casting("true123"), "");
        assert_eq!(get_sqlite_type_casting("2024-13-01"), ""); // Invalid date

        // Almost valid values
        assert_eq!(get_sqlite_type_casting("trueish"), "");
        assert_eq!(get_sqlite_type_casting("192.168.1"), ""); // Incomplete (but still invalid)

        // Special characters
        assert_eq!(get_sqlite_type_casting("\\n\\t\\r"), "");
        assert_eq!(get_sqlite_type_casting("🦀"), ""); // Emoji
        assert_eq!(get_sqlite_type_casting("hello world"), "");

        // SQL keywords (should be treated as text)
        assert_eq!(get_sqlite_type_casting("SELECT"), "");
        assert_eq!(get_sqlite_type_casting("DROP TABLE"), "");

        // Very long strings
        let long_string = "a".repeat(10000);
        assert_eq!(get_sqlite_type_casting(&long_string), "");
    }

    #[test]
    fn test_type_precedence() {
        // Test that types are matched in the correct order
        // Since SQLite returns empty string for all valid types,
        // we're mainly ensuring the order doesn't cause issues

        // Simple numbers
        assert_eq!(get_sqlite_type_casting("0"), ""); // Could be bool or int
        assert_eq!(get_sqlite_type_casting("1"), ""); // Could be bool or int
        assert_eq!(get_sqlite_type_casting("42"), "");

        // Boolean-like values
        assert_eq!(get_sqlite_type_casting("true"), "");
        assert_eq!(get_sqlite_type_casting("false"), "");
        assert_eq!(get_sqlite_type_casting("t"), "");
        assert_eq!(get_sqlite_type_casting("f"), "");

        // Date vs string
        assert_eq!(get_sqlite_type_casting("2024-01-01"), "");

        // UUID vs string
        assert_eq!(
            get_sqlite_type_casting("550e8400-e29b-41d4-a716-446655440000"),
            ""
        );

        // JSON vs string
        assert_eq!(get_sqlite_type_casting("{}"), "");
        assert_eq!(get_sqlite_type_casting("[]"), "");

        // Binary vs string
        assert_eq!(get_sqlite_type_casting("X'DEADBEEF'"), "");
    }
}
