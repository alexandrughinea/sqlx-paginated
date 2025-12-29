use crate::paginated_query_as::models::{QueryFilterCondition, QueryFilterOperator};
use serde::de::{MapAccess, Visitor};
use serde::Deserializer;
use std::collections::HashMap;
use std::fmt;

/// Deserializes an optional filter map from query parameters.
///
/// Supports the following formats:
/// - Simple equality: `field=value` → Equal operator
/// - With operator: `field[op]=value` → Specified operator
///
/// Operator abbreviations:
/// - eq: Equal (default)
/// - ne, neq: Not Equal
/// - gt: Greater Than
/// - gte: Greater or Equal
/// - lt: Less Than
/// - lte: Less or Equal
/// - in: In List
/// - nin, not_in: Not In List
/// - is_null, null: Is Null
/// - is_not_null, not_null: Is Not Null
/// - like: Like Pattern
/// - not_like, nlike: Not Like Pattern
///
/// # Examples
///
/// ```text
/// ?age=25                      → age = 25
/// ?price[gt]=10                → price > 10
/// ?status[ne]=deleted          → status != 'deleted'
/// ?role[in]=admin,moderator    → role IN ('admin', 'moderator')
/// ?deleted_at[is_null]=        → deleted_at IS NULL
/// ```
pub fn deserialize_filter_map<'de, D>(
    deserializer: D,
) -> Result<Option<HashMap<String, QueryFilterCondition>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct FilterMapVisitor;

    impl<'de> Visitor<'de> for FilterMapVisitor {
        type Value = Option<HashMap<String, QueryFilterCondition>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map of filter conditions")
        }

        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut filter_map = HashMap::new();

            while let Some((key, value)) = access.next_entry::<String, Option<String>>()? {
                // Check if the key contains an operator specification: field[op]
                if let Some(start_bracket) = key.find('[') {
                    if let Some(end_bracket) = key.find(']') {
                        if start_bracket < end_bracket {
                            let field = &key[..start_bracket];
                            let operator_str = &key[start_bracket + 1..end_bracket];
                            let operator = QueryFilterOperator::from_str(operator_str);

                            let condition = if operator.requires_value() {
                                QueryFilterCondition::new(operator, value)
                            } else {
                                // For IS NULL/IS NOT NULL, value is ignored
                                QueryFilterCondition::new(operator, None::<String>)
                            };

                            filter_map.insert(field.to_string(), condition);
                            continue;
                        }
                    }
                }

                // Simple format without operator → defaults to Equal
                if let Some(val) = value {
                    filter_map.insert(key, QueryFilterCondition::equal(val));
                }
            }

            if filter_map.is_empty() {
                Ok(None)
            } else {
                Ok(Some(filter_map))
            }
        }
    }

    deserializer.deserialize_map(FilterMapVisitor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    struct TestQuery {
        #[serde(flatten, deserialize_with = "deserialize_filter_map")]
        filters: Option<HashMap<String, QueryFilterCondition>>,
    }

    #[test]
    #[cfg(feature = "test-serde-urlencoded")]
    fn test_simple_equality() {
        let query = "status=active&role=admin";
        let parsed: TestQuery = serde_urlencoded::from_str(query).unwrap();

        let filters = parsed.filters.as_ref().unwrap();
        assert_eq!(filters.len(), 2);

        let status = filters.get("status").unwrap();
        assert_eq!(status.operator, QueryFilterOperator::Equal);
        assert_eq!(status.value, Some("active".to_string()));

        let role = filters.get("role").unwrap();
        assert_eq!(role.operator, QueryFilterOperator::Equal);
        assert_eq!(role.value, Some("admin".to_string()));
    }

    #[test]
    #[cfg(feature = "test-serde-urlencoded")]
    fn test_comparison_operators() {
        let query = "age[gt]=18&price[lte]=100&stock[gte]=10";
        let parsed: TestQuery = serde_urlencoded::from_str(query).unwrap();

        let filters = parsed.filters.as_ref().unwrap();
        let age = filters.get("age").unwrap();
        assert_eq!(age.operator, QueryFilterOperator::GreaterThan);
        assert_eq!(age.value, Some("18".to_string()));

        let price = filters.get("price").unwrap();
        assert_eq!(price.operator, QueryFilterOperator::LessOrEqual);
        assert_eq!(price.value, Some("100".to_string()));
    }

    #[test]
    #[cfg(feature = "test-serde-urlencoded")]
    fn test_in_operator() {
        let query = "role[in]=admin,moderator,user";
        let parsed: TestQuery = serde_urlencoded::from_str(query).unwrap();

        let filters = parsed.filters.as_ref().unwrap();
        let role = filters.get("role").unwrap();
        assert_eq!(role.operator, QueryFilterOperator::In);
        assert_eq!(role.value, Some("admin,moderator,user".to_string()));

        let values = role.split_values();
        assert_eq!(values, vec!["admin", "moderator", "user"]);
    }

    #[test]
    #[cfg(feature = "test-serde-urlencoded")]
    fn test_null_operators() {
        let query = "deleted_at[is_null]=&confirmed_at[is_not_null]=";
        let parsed: TestQuery = serde_urlencoded::from_str(query).unwrap();

        let filters = parsed.filters.as_ref().unwrap();
        let deleted = filters.get("deleted_at").unwrap();
        assert_eq!(deleted.operator, QueryFilterOperator::IsNull);
        assert_eq!(deleted.value, None);

        let confirmed = filters.get("confirmed_at").unwrap();
        assert_eq!(confirmed.operator, QueryFilterOperator::IsNotNull);
        assert_eq!(confirmed.value, None);
    }

    #[test]
    #[cfg(feature = "test-serde-urlencoded")]
    fn test_like_operator() {
        let query = "email[like]=%@example.com";
        let parsed: TestQuery = serde_urlencoded::from_str(query).unwrap();

        let filters = parsed.filters.as_ref().unwrap();
        let email = filters.get("email").unwrap();
        assert_eq!(email.operator, QueryFilterOperator::Like);
        assert_eq!(email.value, Some("%@example.com".to_string()));
    }

    #[test]
    #[cfg(feature = "test-serde-urlencoded")]
    fn test_mixed_formats() {
        let query = "status=active&age[gte]=18&role[in]=admin,user&deleted_at[is_null]=";
        let parsed: TestQuery = serde_urlencoded::from_str(query).unwrap();

        let filters = parsed.filters.as_ref().unwrap();
        assert_eq!(filters.len(), 4);

        let status = filters.get("status").unwrap();
        assert_eq!(status.operator, QueryFilterOperator::Equal);

        let age = filters.get("age").unwrap();
        assert_eq!(age.operator, QueryFilterOperator::GreaterOrEqual);

        let role = filters.get("role").unwrap();
        assert_eq!(role.operator, QueryFilterOperator::In);

        let deleted = parsed.filters.get("deleted_at").unwrap();
        assert_eq!(deleted.operator, QueryFilterOperator::IsNull);
    }
}
