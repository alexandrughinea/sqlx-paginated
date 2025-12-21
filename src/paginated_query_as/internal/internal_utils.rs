use crate::paginated_query_as::internal::{
    DEFAULT_MIN_PAGE_SIZE, DEFAULT_PAGE, DEFAULT_SEARCH_COLUMN_NAMES, DEFAULT_SORT_COLUMN_NAME,
};
use crate::QuerySortDirection;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

/// Represents the inferred type of a struct field based on its default value.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    String,
    Uuid,
    Int,
    Float,
    Bool,
    DateTime,
    Date,
    Time,
    Unknown,
}

/// Returns a map of field names to their inferred types by inspecting
/// the JSON-serialized default values of the struct.
///
/// This is used to determine the correct SQL type cast based on the struct's
/// field types rather than inferring from filter values.
pub fn get_struct_field_meta<T>() -> HashMap<String, FieldType>
where
    T: Default + Serialize,
{
    let default_value = T::default();
    let json_value = serde_json::to_value(default_value).unwrap();

    let mut result = HashMap::new();
    if let Value::Object(map) = json_value {
        for (key, value) in map {
            let field_type = match &value {
                Value::Number(n) => {
                    if n.is_f64() && n.as_f64().map(|f| f.fract() != 0.0).unwrap_or(false) {
                        FieldType::Float
                    } else {
                        FieldType::Int
                    }
                }
                Value::Bool(_) => FieldType::Bool,
                Value::String(s) => {
                    if Uuid::parse_str(s).is_ok() {
                        FieldType::Uuid
                    } else {
                        FieldType::String
                    }
                }
                _ => FieldType::Unknown,
            };
            result.insert(key, field_type);
        }
    }
    result
}

pub fn default_page() -> i64 {
    DEFAULT_PAGE
}

pub fn default_page_size() -> i64 {
    DEFAULT_MIN_PAGE_SIZE
}

pub fn default_search_columns() -> Option<Vec<String>> {
    Some(
        DEFAULT_SEARCH_COLUMN_NAMES
            .iter()
            .map(|&s| s.to_string())
            .collect(),
    )
}

pub fn default_sort_column() -> String {
    DEFAULT_SORT_COLUMN_NAME.to_string()
}

pub fn default_sort_direction() -> QuerySortDirection {
    QuerySortDirection::Descending
}

pub fn quote_identifier(identifier: &str) -> String {
    identifier
        .split('.')
        .collect::<Vec<&str>>()
        .iter()
        .map(|part| format!("\"{}\"", part.replace("\"", "\"\"")))
        .collect::<Vec<_>>()
        .join(".")
}

pub fn extract_digits_from_strings(val: impl Into<String>) -> String {
    val.into().chars().filter(|c| c.is_ascii_digit()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paginated_query_as::internal::DEFAULT_MIN_PAGE_SIZE;
    use crate::paginated_query_as::models::QuerySortDirection;
    use serde::Serialize;

    #[test]
    fn test_default_page() {
        assert_eq!(default_page(), DEFAULT_PAGE);
        assert_eq!(default_page(), 1);
    }

    #[test]
    fn test_default_page_size() {
        assert_eq!(default_page_size(), DEFAULT_MIN_PAGE_SIZE);
        assert_eq!(default_page_size(), 10);
    }

    #[test]
    fn test_default_search_columns() {
        let columns = default_search_columns();
        assert!(columns.is_some());

        let columns = columns.unwrap();
        assert!(columns.contains(&"name".to_string()));
        assert!(columns.contains(&"description".to_string()));
        assert_eq!(columns.len(), 2);
    }

    #[test]
    fn test_default_sort_field() {
        assert_eq!(default_sort_column(), "created_at");
    }

    #[test]
    fn test_default_sort_direction() {
        assert!(matches!(
            default_sort_direction(),
            QuerySortDirection::Descending
        ));
    }

    #[test]
    fn test_quote_identifier_simple() {
        // Simple cases
        assert_eq!(quote_identifier("column"), "\"column\"");
        assert_eq!(quote_identifier("user_id"), "\"user_id\"");
        assert_eq!(quote_identifier("email"), "\"email\"");
    }

    #[test]
    fn test_quote_identifier_schema() {
        // Schema qualified identifiers
        assert_eq!(quote_identifier("schema.table"), "\"schema\".\"table\"");
        assert_eq!(
            quote_identifier("public.users.id"),
            "\"public\".\"users\".\"id\""
        );
        assert_eq!(
            quote_identifier("my_schema.my_table"),
            "\"my_schema\".\"my_table\""
        );
    }

    #[test]
    fn test_quote_identifier_escaping() {
        // Quote escaping - each quote becomes two quotes
        assert_eq!(quote_identifier("user\"name"), "\"user\"\"name\"");
        assert_eq!(quote_identifier("table\""), "\"table\"\"\"");
        assert_eq!(quote_identifier("\"column"), "\"\"\"column\"");
        assert_eq!(quote_identifier("weird\"\"name"), "\"weird\"\"\"\"name\"");
    }

    #[test]
    fn test_quote_identifier_sql_injection() {
        // SQL injection attempts
        assert_eq!(
            quote_identifier("table\"; DROP TABLE users; --"),
            "\"table\"\"; DROP TABLE users; --\""
        );
        assert_eq!(
            quote_identifier("name); DELETE FROM users; --"),
            "\"name); DELETE FROM users; --\""
        );
    }

    #[test]
    fn test_quote_identifier_dots() {
        // Empty parts get quoted as empty strings
        assert_eq!(quote_identifier("."), "\"\".\"\"");
        assert_eq!(quote_identifier("a.b.c"), "\"a\".\"b\".\"c\"");
        assert_eq!(quote_identifier("a..c"), "\"a\".\"\".\"c\"");
    }

    #[test]
    fn test_quote_identifier_empty() {
        // Empty string gets quoted
        assert_eq!(quote_identifier(""), "\"\"");
    }

    #[test]
    fn test_quote_identifier_special_cases() {
        // Special characters (other than quotes and dots)
        assert_eq!(quote_identifier("table$name"), "\"table$name\"");
        assert_eq!(quote_identifier("column@db"), "\"column@db\"");
        assert_eq!(quote_identifier("user#1"), "\"user#1\"");
    }

    #[derive(Default, Serialize)]
    struct TestStruct {
        id: i32,
        name: String,
        #[serde(rename = "email_address")]
        email: String,
        #[serde(skip)]
        #[allow(dead_code)]
        internal: bool,
    }

    #[test]
    fn test_get_struct_field_meta() {
        let meta = get_struct_field_meta::<TestStruct>();

        assert!(meta.contains_key("id"));
        assert!(meta.contains_key("name"));
        assert!(meta.contains_key("email_address")); // renamed field
        assert!(!meta.contains_key("internal")); // skipped field
        assert_eq!(meta.len(), 3);

        // Check field types
        assert_eq!(meta.get("id"), Some(&FieldType::Int));
        assert_eq!(meta.get("name"), Some(&FieldType::String));
        assert_eq!(meta.get("email_address"), Some(&FieldType::String));
    }

    #[derive(Default, Serialize)]
    struct TypedStruct {
        uuid_field: uuid::Uuid,
        string_field: String,
        int_field: i64,
        float_field: f64,
        bool_field: bool,
    }

    #[test]
    fn test_get_struct_field_meta_types() {
        let meta = get_struct_field_meta::<TypedStruct>();

        assert_eq!(meta.get("uuid_field"), Some(&FieldType::Uuid));
        assert_eq!(meta.get("string_field"), Some(&FieldType::String));
        assert_eq!(meta.get("int_field"), Some(&FieldType::Int));
        // Note: f64::default() is 0.0, which has no fractional part, so it's Int
        // This is a limitation of JSON-based type inference
        assert_eq!(meta.get("bool_field"), Some(&FieldType::Bool));
    }

    #[derive(Default, Serialize)]
    struct EmptyStruct {}

    #[test]
    fn test_get_struct_field_meta_edge_cases() {
        // Empty struct
        assert!(get_struct_field_meta::<EmptyStruct>().is_empty());

        // Unit struct
        #[derive(Default, Serialize)]
        struct UnitStruct;
        assert!(get_struct_field_meta::<UnitStruct>().is_empty());
    }

    #[test]
    fn test_extract_digits_from_strings() {
        assert_eq!(extract_digits_from_strings("123abc456"), "123456");
        assert_eq!(extract_digits_from_strings("abc"), "");
        assert_eq!(extract_digits_from_strings("1a2b3c"), "123");
        assert_eq!(extract_digits_from_strings(String::from("12.34")), "1234");
        assert_eq!(extract_digits_from_strings("page=5"), "5");
    }

    #[derive(Default, Serialize)]
    struct ModelWithOptions {
        required_id: i64,
        optional_amount: Option<f64>,
        optional_name: Option<String>,
        optional_uuid: Option<uuid::Uuid>,
    }

    #[test]
    fn test_option_fields_return_unknown_type() {
        // Option<T> fields default to None, which serializes as JSON null
        // This results in FieldType::Unknown (a known limitation)
        let meta = get_struct_field_meta::<ModelWithOptions>();

        // Required field should be correctly inferred
        assert_eq!(meta.get("required_id"), Some(&FieldType::Int));

        // Option fields serialize as null, resulting in Unknown type
        // This is expected behavior - the fallback mechanism in query_builder
        // will use FilterValue::to_field_type() instead
        assert_eq!(meta.get("optional_amount"), Some(&FieldType::Unknown));
        assert_eq!(meta.get("optional_name"), Some(&FieldType::Unknown));
        assert_eq!(meta.get("optional_uuid"), Some(&FieldType::Unknown));
    }
}
