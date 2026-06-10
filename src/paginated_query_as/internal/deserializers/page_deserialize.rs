use crate::paginated_query_as::internal::{extract_digits_from_strings, DEFAULT_PAGE};
use serde::{Deserialize, Deserializer};

pub fn page_deserialize<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    let value_with_fallbacks = match value {
        None => return Ok(DEFAULT_PAGE),
        Some(s) if s.trim().is_empty() || s.trim().starts_with('-') => return Ok(DEFAULT_PAGE),
        Some(s) => extract_digits_from_strings(s),
    };

    value_with_fallbacks
        .parse::<i64>()
        .map(|digit| {
            if digit < DEFAULT_PAGE {
                DEFAULT_PAGE
            } else {
                digit
            }
        })
        .or(Ok(DEFAULT_PAGE))
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
    fn test_page_deserialize() {
        // Default cases
        assert_eq!(
            deserialize_test(r#"null"#, page_deserialize).unwrap(),
            DEFAULT_PAGE
        );
        assert_eq!(
            deserialize_test(r#""""#, page_deserialize).unwrap(),
            DEFAULT_PAGE
        );

        // Valid cases
        assert_eq!(deserialize_test(r#""2""#, page_deserialize).unwrap(), 2);
        assert_eq!(deserialize_test(r#""10""#, page_deserialize).unwrap(), 10);

        // Invalid/Edge cases
        assert_eq!(
            deserialize_test(r#""0""#, page_deserialize).unwrap(),
            DEFAULT_PAGE
        );
        assert_eq!(
            deserialize_test(r#""-1""#, page_deserialize).unwrap(),
            DEFAULT_PAGE
        );
        assert_eq!(
            deserialize_test(r#""abc123""#, page_deserialize).unwrap(),
            123
        );
        assert_eq!(
            deserialize_test(r#""page 5""#, page_deserialize).unwrap(),
            5
        );
    }
}
