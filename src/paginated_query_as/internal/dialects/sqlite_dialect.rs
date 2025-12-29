use crate::paginated_query_as::internal::{get_sqlite_type_casting, QueryDialect};

pub struct SqliteDialect;

impl QueryDialect for SqliteDialect {
    fn quote_identifier(&self, ident: &str) -> String {
        format!("\"{}\"", ident.replace('"', "\"\""))
    }

    fn placeholder(&self, _position: usize) -> String {
        "?".to_string()
    }

    fn type_cast(&self, value: &str) -> String {
        get_sqlite_type_casting(value).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quote_identifier() {
        let dialect = SqliteDialect;

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
        let dialect = SqliteDialect;

        // SQLite uses ? for all positions (position is ignored)
        assert_eq!(dialect.placeholder(1), "?");
        assert_eq!(dialect.placeholder(2), "?");
        assert_eq!(dialect.placeholder(100), "?");
        assert_eq!(dialect.placeholder(0), "?");
    }

    #[test]
    fn test_type_cast() {
        let dialect = SqliteDialect;

        // SQLite doesn't use explicit type casting like Postgres
        // All should return empty string (letting SQLite handle types dynamically)
        assert_eq!(dialect.type_cast("42"), "");
        assert_eq!(dialect.type_cast("3.14"), "");
        assert_eq!(dialect.type_cast("true"), "");
        assert_eq!(dialect.type_cast("2024-01-01"), "");
        assert_eq!(dialect.type_cast("hello"), "");
    }
}
