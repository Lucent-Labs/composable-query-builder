pub trait IntoOptional<T> {
    fn into_optional(self) -> Option<T>;
}

impl<T> IntoOptional<T> for T {
    fn into_optional(self) -> Option<T> {
        Some(self)
    }
}

impl<T> IntoOptional<T> for Option<T> {
    fn into_optional(self) -> Option<T> {
        self
    }
}
