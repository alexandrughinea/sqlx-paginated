pub trait FieldEnum {
    fn as_str(&self) -> &'static str;

    fn contains<S: AsRef<str>>(s: S) -> bool;
}