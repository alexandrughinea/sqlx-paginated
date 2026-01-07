use crate::paginated_query_as::internal::{get_postgres_type_casting, QueryDialect};

pub struct PostgresDialect;

impl QueryDialect for PostgresDialect {
    fn quote_identifier(&self, ident: &str) -> String {
        format!("\"{}\"", ident.replace('"', "\"\""))
    }

    fn placeholder(&self, position: usize) -> String {
        format!("${}", position)
    }

    fn type_cast(&self, value: &str) -> String {
        get_postgres_type_casting(value).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quote_identifier() {
        let dialect = PostgresDialect;

        // Simple identifier
        assert_eq!(dialect.quote_identifier("column_name"), "\"column_name\"");
        assert_eq!(dialect.quote_identifier("table"), "\"table\"");

        // Identifier with spaces
        assert_eq!(dialect.quote_identifier("my column"), "\"my column\"");

        // Identifier with special characters
        assert_eq!(dialect.quote_identifier("user-name"), "\"user-name\"");
        assert_eq!(dialect.quote_identifier("table.column"), "\"table.column\"");

        // Identifier with quotes (should be escaped)
        assert_eq!(
            dialect.quote_identifier("column\"name"),
            "\"column\"\"name\""
        );
        assert_eq!(
            dialect.quote_identifier("my\"table\"name"),
            "\"my\"\"table\"\"name\""
        );

        // Empty string
        assert_eq!(dialect.quote_identifier(""), "\"\"");
    }

    #[test]
    fn test_placeholder() {
        let dialect = PostgresDialect;

        // Postgres uses $1, $2, $3... format
        assert_eq!(dialect.placeholder(1), "$1");
        assert_eq!(dialect.placeholder(2), "$2");
        assert_eq!(dialect.placeholder(10), "$10");
        assert_eq!(dialect.placeholder(100), "$100");

        // Edge case: position 0 (shouldn't typically be used but should work)
        assert_eq!(dialect.placeholder(0), "$0");
    }

    #[test]
    fn test_type_cast() {
        let dialect = PostgresDialect;

        // Postgres uses explicit type casting with ::type syntax
        // Integers
        assert_eq!(dialect.type_cast("42"), "::smallint");
        assert_eq!(dialect.type_cast("2147483647"), "::integer");
        assert_eq!(dialect.type_cast("9223372036854775807"), "::bigint");

        // Floats
        assert_eq!(dialect.type_cast("3.14"), "::real");

        // Booleans
        assert_eq!(dialect.type_cast("true"), "::boolean");
        assert_eq!(dialect.type_cast("false"), "::boolean");

        // Dates
        assert_eq!(dialect.type_cast("2024-01-01"), "::date");

        // JSON
        assert_eq!(dialect.type_cast("{}"), "::jsonb");
        assert_eq!(dialect.type_cast("[]"), "::jsonb");

        // UUIDs
        assert_eq!(
            dialect.type_cast("550e8400-e29b-41d4-a716-446655440000"),
            "::uuid"
        );

        // IP addresses
        assert_eq!(dialect.type_cast("192.168.1.1"), "::inet");

        // Special values and invalid inputs return empty string
        assert_eq!(dialect.type_cast("NULL"), "");
        assert_eq!(dialect.type_cast("invalid"), "");
    }
}
