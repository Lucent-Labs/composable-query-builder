pub trait IntoOptional<T> {
    fn into_optional(self) -> Option<T>;
}

impl IntoOptional<u64> for u64 {
    fn into_optional(self) -> Option<u64> {
        Some(self)
    }
}

impl IntoOptional<u64> for Option<u64> {
    fn into_optional(self) -> Option<u64> {
        self
    }
}
