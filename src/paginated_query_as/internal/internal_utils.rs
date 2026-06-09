use crate::paginated_query_as::internal::{
    DEFAULT_DATE_RANGE_COLUMN_NAME, DEFAULT_MIN_PAGE_SIZE, DEFAULT_PAGE,
    DEFAULT_SEARCH_COLUMN_NAMES, DEFAULT_SORT_COLUMN_NAME,
};
use crate::QuerySortDirection;
use serde::Serialize;
use serde_json::Value;

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

pub fn default_date_range_column() -> Option<String> {
    Some(DEFAULT_DATE_RANGE_COLUMN_NAME.to_string())
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

pub fn get_struct_field_names<T>() -> Vec<String>
where
    T: Default + Serialize,
{
    let default_value = T::default();
        return vec![];
    };

    if let Value::Object(map) = json_value {
        map.keys().cloned().collect()
    } else {
        vec![]
    }
}

pub fn extract_digits_from_strings(val: impl Into<String>) -> String {
    val.into().chars().filter(|c| c.is_ascii_digit()).collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
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
    fn test_get_struct_field_names() {
        let fields = get_struct_field_names::<TestStruct>();

        assert!(fields.contains(&"id".to_string()));
        assert!(fields.contains(&"name".to_string()));
        assert!(fields.contains(&"email_address".to_string())); // renamed field
        assert!(!fields.contains(&"internal".to_string())); // skipped field
        assert_eq!(fields.len(), 3);
    }

    #[derive(Default, Serialize)]
    struct EmptyStruct {}

    #[test]
    fn test_get_struct_field_names_edge_cases() {
        // Empty struct
        assert!(get_struct_field_names::<EmptyStruct>().is_empty());

        // Unit struct
        #[derive(Default, Serialize)]
        struct UnitStruct;
        assert!(get_struct_field_names::<UnitStruct>().is_empty());
    }

    #[test]
    fn test_extract_digits_from_strings() {
        assert_eq!(extract_digits_from_strings("123abc456"), "123456");
        assert_eq!(extract_digits_from_strings("abc"), "");
        assert_eq!(extract_digits_from_strings("1a2b3c"), "123");
        assert_eq!(extract_digits_from_strings(String::from("12.34")), "1234");
        assert_eq!(extract_digits_from_strings("page=5"), "5");
    }
}
