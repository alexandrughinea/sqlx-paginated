use crate::paginated_query_as::internal::DEFAULT_SEARCH_COLUMN_NAME_SEPARATOR_SYMBOL;
use serde::{Deserialize, Deserializer};

pub fn search_columns_deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;

    Ok(value.map(|s| {
        s.split(DEFAULT_SEARCH_COLUMN_NAME_SEPARATOR_SYMBOL)
            .filter_map(|s| {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            })
            .collect()
    }))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde_json::Value;
    fn deserialize_test<T, F>(json: &str, deserialize_fn: F) -> Result<T, serde_json::Error>
    where
        F: FnOnce(Value) -> Result<T, serde_json::Error>,
    {
        let value: Value = serde_json::from_str(json)?;
        deserialize_fn(value)
    }

    #[test]
    fn test_search_columns_deserialize() {
        // Default cases
        assert_eq!(
            deserialize_test(r#"null"#, search_columns_deserialize).unwrap(),
            None
        );
        assert_eq!(
            deserialize_test(r#""""#, search_columns_deserialize).unwrap(),
            Some(vec![])
        );

        // Valid cases
        assert_eq!(
            deserialize_test(r#""name,email""#, search_columns_deserialize).unwrap(),
            Some(vec!["name".to_string(), "email".to_string()])
        );

        // Whitespace handling
        assert_eq!(
            deserialize_test(r#""name , email , phone""#, search_columns_deserialize).unwrap(),
            Some(vec![
                "name".to_string(),
                "email".to_string(),
                "phone".to_string()
            ])
        );

        // Empty segments
        assert_eq!(
            deserialize_test(r#""name,,email""#, search_columns_deserialize).unwrap(),
            Some(vec!["name".to_string(), "email".to_string()])
        );
    }
}
