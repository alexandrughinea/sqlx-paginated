pub trait FieldEnum {
    fn as_str(&self) -> &'static str;

    fn contains<S: AsRef<str>>(s: S) -> bool;
}

pub trait PaginatedInfo {
    type Fields: FieldEnum;

    fn is_known_field<S: AsRef<str>>(s: S) -> bool {
        Self::Fields::contains(s)
    }
}
