use crate::paginated_query_as::internal::FieldType;

pub trait QueryDialect {
    fn quote_identifier(&self, ident: &str) -> String;
    fn placeholder(&self, position: usize) -> String;
    fn type_cast(&self, field_type: &FieldType) -> String;
}
