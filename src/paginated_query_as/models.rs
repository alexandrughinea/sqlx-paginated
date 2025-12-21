use crate::paginated_query_as::internal::{
    filters_deserialize, QueryPaginationParams, QuerySearchParams, QuerySortParams,
};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PaginatedResponse<T> {
    pub records: Vec<T>,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub pagination: Option<QueryPaginationParams>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_pages: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FlatQueryParams {
    #[serde(flatten)]
    pub pagination: Option<QueryPaginationParams>,
    #[serde(flatten)]
    pub sort: Option<QuerySortParams>,
    #[serde(flatten)]
    pub search: Option<QuerySearchParams>,
    #[serde(flatten, default, deserialize_with = "filters_deserialize")]
    pub filters: Option<Vec<Filter>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum FilterOperator {
    Eq,
    Ne,
    Gt,
    Lt,
    Gte,
    Lte,
    Like,
    ILike,
    In,
    NotIn,
    IsNull,
    IsNotNull,
    Between,
    Contains,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum FilterValue {
    String(String),
    Uuid(uuid::Uuid),
    Int(i64),
    Float(f64),
    Bool(bool),
    DateTime(String),
    Date(String),
    Time(String),
    Array(Vec<FilterValue>),
    Null,
}

impl FilterValue {
    pub fn to_bindable_string(&self) -> String {
        match self {
            FilterValue::String(s) => s.clone(),
            FilterValue::Int(i) => i.to_string(),
            FilterValue::Float(f) => f.to_string(),
            FilterValue::Bool(b) => b.to_string(),
            FilterValue::Uuid(uuid) => uuid.to_string(),
            FilterValue::DateTime(dt) => dt.clone(),
            FilterValue::Date(d) => d.clone(),
            FilterValue::Time(t) => t.clone(),
            FilterValue::Array(arr) => arr.first().map(|v| v.to_bindable_string()).unwrap_or_default(),
            FilterValue::Null => String::new(),
        }
    }

    pub fn to_bindable_strings(&self) -> Vec<String> {
        match self {
            FilterValue::Array(arr) => arr.iter().map(|v| v.to_bindable_string()).collect(),
            _ => vec![self.to_bindable_string()],
        }
    }

    pub fn to_sql_string(&self) -> String {
        match self {
            FilterValue::String(s) => format!("'{}'", s.replace('\'', "''")),
            FilterValue::Int(i) => i.to_string(),
            FilterValue::Float(f) => f.to_string(),
            FilterValue::Bool(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
            FilterValue::DateTime(dt) => format!("'{}'", dt),
            FilterValue::Date(d) => format!("'{}'", d),
            FilterValue::Time(t) => format!("'{}'", t),
            FilterValue::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| v.to_sql_string()).collect();
                format!("({})", items.join(", "))
            }
            FilterValue::Null => "NULL".to_string(),
            FilterValue::Uuid(uuid) => format!("'{}'", uuid.to_string()),
        }
    }

    /// Converts the filter value to a corresponding FieldType for type casting.
    /// This is used as a fallback when the struct field type cannot be inferred
    /// (e.g., Option<T> fields that default to None).
    pub fn to_field_type(&self) -> crate::paginated_query_as::internal::FieldType {
        use crate::paginated_query_as::internal::FieldType;
        match self {
            FilterValue::String(_) => FieldType::String,
            FilterValue::Int(_) => FieldType::Int,
            FilterValue::Float(_) => FieldType::Float,
            FilterValue::Bool(_) => FieldType::Bool,
            FilterValue::Uuid(_) => FieldType::Uuid,
            FilterValue::DateTime(_) => FieldType::DateTime,
            FilterValue::Date(_) => FieldType::Date,
            FilterValue::Time(_) => FieldType::Time,
            FilterValue::Array(arr) => arr.first().map(|v| v.to_field_type()).unwrap_or(FieldType::Unknown),
            FilterValue::Null => FieldType::Unknown,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Filter {
    pub field: String,
    pub operator: FilterOperator,
    pub value: FilterValue,
}

#[derive(Default, Clone)]
pub struct QueryParams<'q, T> {
    pub pagination: QueryPaginationParams,
    pub sort: QuerySortParams,
    pub search: QuerySearchParams,
    pub filters: Vec<Filter>,
    pub(crate) _phantom: PhantomData<&'q T>,
}

impl<'q, T> From<FlatQueryParams> for QueryParams<'q, T> {
    fn from(params: FlatQueryParams) -> Self {
        QueryParams {
            pagination: params.pagination.unwrap_or_default(),
            sort: params.sort.unwrap_or_default(),
            search: params.search.unwrap_or_default(),
            filters: params.filters.unwrap_or_default(),
            _phantom: PhantomData::<&'q T>,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum QuerySortDirection {
    Ascending,
    #[default]
    Descending,
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::paginated_query_as::internal::FieldType;

    #[test]
    fn test_to_field_type_int() {
        assert_eq!(FilterValue::Int(42).to_field_type(), FieldType::Int);
    }

    #[test]
    fn test_to_field_type_float() {
        assert_eq!(FilterValue::Float(3.14).to_field_type(), FieldType::Float);
    }

    #[test]
    fn test_to_field_type_bool() {
        assert_eq!(FilterValue::Bool(true).to_field_type(), FieldType::Bool);
    }

    #[test]
    fn test_to_field_type_string() {
        assert_eq!(FilterValue::String("test".to_string()).to_field_type(), FieldType::String);
    }

    #[test]
    fn test_to_field_type_uuid() {
        let uuid = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        assert_eq!(FilterValue::Uuid(uuid).to_field_type(), FieldType::Uuid);
    }

    #[test]
    fn test_to_field_type_datetime() {
        assert_eq!(
            FilterValue::DateTime("2025-12-02T10:30:00Z".to_string()).to_field_type(),
            FieldType::DateTime
        );
    }

    #[test]
    fn test_to_field_type_date() {
        assert_eq!(
            FilterValue::Date("2025-12-02".to_string()).to_field_type(),
            FieldType::Date
        );
    }

    #[test]
    fn test_to_field_type_time() {
        assert_eq!(
            FilterValue::Time("10:30:00".to_string()).to_field_type(),
            FieldType::Time
        );
    }

    #[test]
    fn test_to_field_type_array_uses_first_element() {
        let arr = FilterValue::Array(vec![FilterValue::Int(1), FilterValue::Int(2)]);
        assert_eq!(arr.to_field_type(), FieldType::Int);
    }

    #[test]
    fn test_to_field_type_empty_array_returns_unknown() {
        let arr = FilterValue::Array(vec![]);
        assert_eq!(arr.to_field_type(), FieldType::Unknown);
    }

    #[test]
    fn test_to_field_type_null_returns_unknown() {
        assert_eq!(FilterValue::Null.to_field_type(), FieldType::Unknown);
    }
}
