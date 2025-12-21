use crate::paginated_query_as::internal::{FieldType, QueryDialect};

pub struct SqliteDialect;

impl QueryDialect for SqliteDialect {
    fn quote_identifier(&self, ident: &str) -> String {
        format!("\"{}\"", ident.replace('"', "\"\""))
    }

    fn placeholder(&self, _position: usize) -> String {
        "?".to_string()
    }

    fn type_cast(&self, _field_type: &FieldType) -> String {
        String::new() // SQLite doesn't use type casts
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqlite_no_type_casts() {
        let dialect = SqliteDialect;
        // SQLite doesn't use type casts - all should return empty string
        assert_eq!(dialect.type_cast(&FieldType::Int), "");
        assert_eq!(dialect.type_cast(&FieldType::Float), "");
        assert_eq!(dialect.type_cast(&FieldType::DateTime), "");
        assert_eq!(dialect.type_cast(&FieldType::Date), "");
        assert_eq!(dialect.type_cast(&FieldType::Time), "");
        assert_eq!(dialect.type_cast(&FieldType::Uuid), "");
        assert_eq!(dialect.type_cast(&FieldType::Bool), "");
        assert_eq!(dialect.type_cast(&FieldType::String), "");
        assert_eq!(dialect.type_cast(&FieldType::Unknown), "");
    }

    #[test]
    fn test_sqlite_placeholder_is_question_mark() {
        let dialect = SqliteDialect;
        // SQLite uses ? for all placeholders regardless of position
        assert_eq!(dialect.placeholder(1), "?");
        assert_eq!(dialect.placeholder(5), "?");
        assert_eq!(dialect.placeholder(100), "?");
    }

    #[test]
    fn test_quote_identifier() {
        let dialect = SqliteDialect;
        assert_eq!(dialect.quote_identifier("column"), "\"column\"");
    }
}
