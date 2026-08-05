use crate::paginated_query_as::internal::DEFAULT_EMPTY_VALUE;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime};
use sqlx::types::Uuid;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

pub fn get_postgres_type_casting(value: &str) -> &'static str {
    match value.trim().to_string().to_lowercase().as_str() {
        // Special values
        value
            if value.eq_ignore_ascii_case("null")
                || value.eq_ignore_ascii_case("nan")
                || value.eq_ignore_ascii_case("infinity")
                || value.eq_ignore_ascii_case("-infinity") =>
        {
            DEFAULT_EMPTY_VALUE
        }

        // Binary data
        value if value.starts_with("\\x") && value[2..].chars().all(|c| c.is_ascii_hexdigit()) => {
            "::bytea"
        }

        // JSON
        value if value.starts_with('{') || value.starts_with('[') => {
            match serde_json::from_str::<serde_json::Value>(value) {
                Ok(_) => "::jsonb", // Fully valid, parseable JSON gets binary format
                Err(_) => DEFAULT_EMPTY_VALUE,
            }
        }

        // XML
        value if value.starts_with("<?xml") || (value.starts_with('<') && value.ends_with('>')) => {
            "::xml"
        }

        // Network types
        value if value.parse::<IpAddr>().is_ok() => "::inet", // IPv4 and IPv6 networks
        value if value.parse::<Ipv4Addr>().is_ok() => "::inet", // IPv4 address
        value if value.parse::<Ipv6Addr>().is_ok() => "::inet", // IPv6 address

        // UUIDs
        value if Uuid::parse_str(value).is_ok() => "::uuid",

        // Dates and Times
        value if NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S").is_ok() => {
            "::timestamp without time zone"
        }
        value if NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok() => "::date",
        value if NaiveTime::parse_from_str(value, "%H:%M:%S").is_ok() => "::time",
        value if DateTime::parse_from_rfc3339(value).is_ok() => "::timestamp with time zone",

        // Booleans
        value if value == "t" || value == "f" => "::boolean",
        value if value == "true" || value == "false" => "::boolean",

        // Numbers
        value if value.parse::<i16>().is_ok() => "::smallint", // 2-byte integer
        value if value.parse::<i32>().is_ok() => "::integer",  // 4-byte integer
        value if value.parse::<i64>().is_ok() => "::bigint",   // 8-byte integer
        value if value.parse::<f32>().is_ok() => "::real",     // 4-byte floating point
        value if value.parse::<f64>().is_ok() => "::double precision", // 8-byte floating point

        // Default - no type cast
        _ => DEFAULT_EMPTY_VALUE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boolean_types() {
        // Standard boolean values
        assert_eq!(get_postgres_type_casting("true"), "::boolean");
        assert_eq!(get_postgres_type_casting("false"), "::boolean");
        assert_eq!(get_postgres_type_casting("t"), "::boolean");
        assert_eq!(get_postgres_type_casting("f"), "::boolean");

        // Case variations
        assert_eq!(get_postgres_type_casting("TRUE"), "::boolean");
        assert_eq!(get_postgres_type_casting("FALSE"), "::boolean");
        assert_eq!(get_postgres_type_casting("True"), "::boolean");
        assert_eq!(get_postgres_type_casting("False"), "::boolean");
    }

    #[test]
    fn test_numeric_types() {
        // Integers
        assert_eq!(get_postgres_type_casting("32767"), "::smallint"); // i16 max
        assert_eq!(get_postgres_type_casting("-32768"), "::smallint"); // i16 min
        assert_eq!(get_postgres_type_casting("2147483647"), "::integer"); // i32 max
        assert_eq!(get_postgres_type_casting("-2147483648"), "::integer"); // i32 min
        assert_eq!(get_postgres_type_casting("9223372036854775807"), "::bigint"); // i64 max
        assert_eq!(
            get_postgres_type_casting("-9223372036854775808"),
            "::bigint"
        ); // i64 min

        // Floating point
        assert_eq!(get_postgres_type_casting("3.14"), "::real");
        assert_eq!(get_postgres_type_casting("-3.14"), "::real");
        assert_eq!(get_postgres_type_casting("1.23e-4"), "::real");
        assert_eq!(get_postgres_type_casting("1.23E+4"), "::real");
        assert_eq!(
            get_postgres_type_casting("1.7976931348623157e+308"),
            "::real"
        );
    }

    #[test]
    fn test_network_types() {
        // IPv4
        assert_eq!(get_postgres_type_casting("192.168.0.1"), "::inet");
        assert_eq!(get_postgres_type_casting("0.0.0.0"), "::inet");
        assert_eq!(get_postgres_type_casting("255.255.255.255"), "::inet");

        // IPv6
        assert_eq!(get_postgres_type_casting("2001:db8::1"), "::inet");
        assert_eq!(get_postgres_type_casting("::1"), "::inet");
        assert_eq!(get_postgres_type_casting("fe80::1"), "::inet");
    }

    #[test]
    fn test_json_types() {
        // Objects
        assert_eq!(get_postgres_type_casting("{}"), "::jsonb");
        assert_eq!(get_postgres_type_casting("{\"key\":\"value\"}"), "::jsonb");
        assert_eq!(
            get_postgres_type_casting("{\"nested\":{\"key\":\"value\"}}"),
            "::jsonb"
        );

        // Arrays
        assert_eq!(get_postgres_type_casting("[]"), "::jsonb");
        assert_eq!(get_postgres_type_casting("[1,2,3]"), "::jsonb");
        assert_eq!(
            get_postgres_type_casting("[{\"key\":\"value\"}]"),
            "::jsonb"
        );

        // Complex JSON
        assert_eq!(
            get_postgres_type_casting("{\"array\":[1,2,3],\"object\":{\"key\":\"value\"}}"),
            "::jsonb"
        );
    }

    #[test]
    fn test_binary_types() {
        // Valid hexadecimal
        assert_eq!(get_postgres_type_casting("\\x0123456789ABCDEF"), "::bytea");
        assert_eq!(get_postgres_type_casting("\\x"), "::bytea"); // Empty binary

        // Invalid hex should return no cast
        assert_eq!(get_postgres_type_casting("\\xGG"), "");
    }

    #[test]
    fn test_date_time_types() {
        // Date
        assert_eq!(get_postgres_type_casting("2024-01-01"), "::date");
        assert_eq!(get_postgres_type_casting("2024-12-31"), "::date");

        // Time
        assert_eq!(get_postgres_type_casting("12:34:56"), "::time");
        assert_eq!(get_postgres_type_casting("23:59:59"), "::time");

        // Timestamp without timezone
        assert_eq!(
            get_postgres_type_casting("2024-01-01 12:34:56"),
            "::timestamp without time zone"
        );

        // Timestamp with timezone
        assert_eq!(
            get_postgres_type_casting("2024-01-01T12:34:56Z"),
            "::timestamp with time zone"
        );
        assert_eq!(
            get_postgres_type_casting("2024-01-01T12:34:56+00:00"),
            "::timestamp with time zone"
        );
    }

    #[test]
    fn test_uuid_types() {
        // Standard UUID
        assert_eq!(
            get_postgres_type_casting("550e8400-e29b-41d4-a716-446655440000"),
            "::uuid"
        );

        // UUID with uppercase
        assert_eq!(
            get_postgres_type_casting("550E8400-E29B-41D4-A716-446655440000"),
            "::uuid"
        );

        // Invalid UUIDs should return no cast
        assert_eq!(get_postgres_type_casting("550e8400-e29b-41d4-a716"), "");
        assert_eq!(
            get_postgres_type_casting("550e8400-e29b-41d4-a716-44665544000G"),
            ""
        );
    }

    #[test]
    fn test_xml_types() {
        // XML declaration
        assert_eq!(
            get_postgres_type_casting("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"),
            "::xml"
        );

        // Simple XML
        assert_eq!(
            get_postgres_type_casting("<root><child>value</child></root>"),
            "::xml"
        );

        // Self-closing tag
        assert_eq!(get_postgres_type_casting("<element/>"), "::xml");
    }

    #[test]
    fn test_edge_cases() {
        // Empty string
        assert_eq!(get_postgres_type_casting(""), "");

        // Whitespace
        assert_eq!(get_postgres_type_casting("   "), "");

        // Mixed invalid types
        assert_eq!(get_postgres_type_casting("123abc"), "");
        assert_eq!(get_postgres_type_casting("true123"), "");
        assert_eq!(get_postgres_type_casting("2024-13-01"), ""); // Invalid date

        // Almost valid values
        assert_eq!(get_postgres_type_casting("trueish"), "");
        assert_eq!(get_postgres_type_casting("192.168.1"), ""); // Incomplete IP
        assert_eq!(get_postgres_type_casting("{invalid json}"), "");

        // Special characters
        assert_eq!(get_postgres_type_casting("\\n\\t\\r"), "");
        assert_eq!(get_postgres_type_casting("🦀"), "");
    }

    #[test]
    fn test_type_precedence() {
        // String that could be multiple types should match first in match order
        assert_eq!(get_postgres_type_casting("123"), "::smallint"); // Should match smallint before integer/bigint
        assert_eq!(get_postgres_type_casting("t"), "::boolean"); // Should match boolean before char
    }

    #[test]
    fn test_special_values() {
        // NULL should return no cast
        assert_eq!(get_postgres_type_casting("NULL"), "");
        assert_eq!(get_postgres_type_casting("null"), "");

        // Special numeric values
        //  assert_eq!(get_postgres_type_casting("NaN"), "");
        assert_eq!(get_postgres_type_casting("Infinity"), "");
        assert_eq!(get_postgres_type_casting("-Infinity"), "");
    }
}
