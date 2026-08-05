pub trait QueryDialect {
    fn quote_identifier(&self, ident: &str) -> String;
    fn placeholder(&self, position: usize) -> String;
    fn type_cast(&self, value: &str) -> String;
}
