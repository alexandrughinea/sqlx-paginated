use crate::paginated_query_as::internal::{DEFAULT_MAX_FIELD_LENGTH, DEFAULT_SEPARATOR};
use serde::{Deserialize, Deserializer};

pub fn search_deserialize<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    let value_with_fallbacks = match value {
        None => return Ok(None), // No search if field is missing
        Some(s) if s.trim().is_empty() => return Ok(None), // No search if empty string
        Some(s) => s,
    };

    // Clean and normalize the search string
    let normalized_value = value_with_fallbacks
        .trim()
        .chars()
        .filter(|c| {
            if c.is_alphanumeric() || *c == ' ' || *c == '-' {
                true
            } else if c.is_ascii() {
                false
            } else {
                !c.is_control()
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(DEFAULT_SEPARATOR)
        .chars()
        .take(DEFAULT_MAX_FIELD_LENGTH as usize)
        .collect::<String>();

    if normalized_value.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(normalized_value))
    }
}

#[cfg(test)]
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
    fn test_search_deserialize() {
        // Default cases
        assert_eq!(
            deserialize_test(r#"null"#, search_deserialize).unwrap(),
            None
        );
        assert_eq!(deserialize_test(r#""""#, search_deserialize).unwrap(), None);
        assert_eq!(
            deserialize_test(r#""  ""#, search_deserialize).unwrap(),
            None
        );

        // Valid cases
        assert_eq!(
            deserialize_test(r#""test search""#, search_deserialize).unwrap(),
            Some("test search".to_string())
        );

        // Sanitization cases
        assert_eq!(
            deserialize_test(r#""test@#$%^&* search""#, search_deserialize).unwrap(),
            Some("test search".to_string())
        );

        // Length limit cases
        let long_search = "a".repeat((DEFAULT_MAX_FIELD_LENGTH + 10) as usize);
        assert_eq!(
            deserialize_test(&format!(r#""{}""#, long_search), search_deserialize)
                .unwrap()
                .unwrap()
                .len(),
            DEFAULT_MAX_FIELD_LENGTH as usize
        );

        assert_eq!(
            deserialize_test(r#""🥶""#, search_deserialize).unwrap(),
            Some("🥶".to_string())
        );
        assert_eq!(
            deserialize_test(r#""Hello 🦀 World""#, search_deserialize).unwrap(),
            Some("Hello 🦀 World".to_string())
        );
        assert_eq!(
            deserialize_test(r#""Café""#, search_deserialize).unwrap(),
            Some("Café".to_string())
        );
        assert_eq!(
            deserialize_test(r#""日本語""#, search_deserialize).unwrap(),
            Some("日本語".to_string())
        );
    }
}
