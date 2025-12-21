use crate::paginated_query_as::internal::{FieldType, QueryDialect};

pub struct PostgresDialect;

impl QueryDialect for PostgresDialect {
    fn quote_identifier(&self, ident: &str) -> String {
        format!("\"{}\"", ident.replace('"', "\"\""))
    }

    fn placeholder(&self, position: usize) -> String {
        format!("${}", position)
    }

    fn type_cast(&self, field_type: &FieldType) -> String {
        match field_type {
            FieldType::Uuid => "::uuid".to_string(),
            FieldType::Bool => "::boolean".to_string(),
            FieldType::Int => "::bigint".to_string(),
            FieldType::Float => "::float8".to_string(),
            FieldType::DateTime => "::timestamptz".to_string(),
            FieldType::Date => "::date".to_string(),
            FieldType::Time => "::time".to_string(),
            // String and Unknown don't need explicit casts
            _ => String::new(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_cast_int() {
        let dialect = PostgresDialect;
        assert_eq!(dialect.type_cast(&FieldType::Int), "::bigint");
    }

    #[test]
    fn test_type_cast_float() {
        let dialect = PostgresDialect;
        assert_eq!(dialect.type_cast(&FieldType::Float), "::float8");
    }

    #[test]
    fn test_type_cast_datetime() {
        let dialect = PostgresDialect;
        assert_eq!(dialect.type_cast(&FieldType::DateTime), "::timestamptz");
    }

    #[test]
    fn test_type_cast_date() {
        let dialect = PostgresDialect;
        assert_eq!(dialect.type_cast(&FieldType::Date), "::date");
    }

    #[test]
    fn test_type_cast_time() {
        let dialect = PostgresDialect;
        assert_eq!(dialect.type_cast(&FieldType::Time), "::time");
    }

    #[test]
    fn test_type_cast_uuid() {
        let dialect = PostgresDialect;
        assert_eq!(dialect.type_cast(&FieldType::Uuid), "::uuid");
    }

    #[test]
    fn test_type_cast_bool() {
        let dialect = PostgresDialect;
        assert_eq!(dialect.type_cast(&FieldType::Bool), "::boolean");
    }

    #[test]
    fn test_type_cast_string_no_cast() {
        let dialect = PostgresDialect;
        assert_eq!(dialect.type_cast(&FieldType::String), "");
    }

    #[test]
    fn test_type_cast_unknown_no_cast() {
        let dialect = PostgresDialect;
        assert_eq!(dialect.type_cast(&FieldType::Unknown), "");
    }

    #[test]
    fn test_quote_identifier() {
        let dialect = PostgresDialect;
        assert_eq!(dialect.quote_identifier("column"), "\"column\"");
        assert_eq!(dialect.quote_identifier("table.column"), "\"table.column\"");
    }

    #[test]
    fn test_placeholder() {
        let dialect = PostgresDialect;
        assert_eq!(dialect.placeholder(1), "$1");
        assert_eq!(dialect.placeholder(5), "$5");
    }
}
