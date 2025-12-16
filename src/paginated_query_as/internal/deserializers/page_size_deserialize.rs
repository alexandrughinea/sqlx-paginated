use crate::paginated_query_as::internal::{
    extract_digits_from_strings, DEFAULT_MAX_PAGE_SIZE, DEFAULT_MIN_PAGE_SIZE,
};
use serde::{Deserialize, Deserializer};

pub fn page_size_deserialize<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    let value_with_fallbacks = match value {
        None => return Ok(DEFAULT_MIN_PAGE_SIZE),
        Some(s) if s.trim().is_empty() || s.trim().starts_with('-') => {
            return Ok(DEFAULT_MIN_PAGE_SIZE)
        }
        Some(s) => extract_digits_from_strings(s),
    };

    value_with_fallbacks
        .parse::<i64>()
        .or(Ok(DEFAULT_MIN_PAGE_SIZE))
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
    fn test_page_size_deserialize() {
        // Default cases
        assert_eq!(
            deserialize_test(r#"null"#, page_size_deserialize).unwrap(),
            DEFAULT_MIN_PAGE_SIZE
        );
        assert_eq!(
            deserialize_test(r#""""#, page_size_deserialize).unwrap(),
            DEFAULT_MIN_PAGE_SIZE
        );

        // Valid cases
        assert_eq!(
            deserialize_test(r#""20""#, page_size_deserialize).unwrap(),
            20
        );

        // Clamping cases
        assert_eq!(
            deserialize_test(r#""5""#, page_size_deserialize).unwrap(),
            DEFAULT_MIN_PAGE_SIZE
        );
        assert_eq!(
            deserialize_test(r#""100""#, page_size_deserialize).unwrap(),
            DEFAULT_MAX_PAGE_SIZE
        );

        // Invalid/Edge cases
        assert_eq!(
            deserialize_test(r#""size25""#, page_size_deserialize).unwrap(),
            25
        );
        assert_eq!(
            deserialize_test(r#""-50""#, page_size_deserialize).unwrap(),
            DEFAULT_MIN_PAGE_SIZE
        );
    }
}
