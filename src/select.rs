/// Converts one or more values into a list of column names.
///
/// Accepts:
///   - &str
///   - String
///   - Vec<Into<String>>
///   - Tuple of 2, 3, or 4 Into<String>
pub trait IntoSelect {
    fn into_select(self) -> Vec<String>;
}

impl IntoSelect for &str {
    fn into_select(self) -> Vec<String> {
        vec![self.into()]
    }
}

impl IntoSelect for String {
    fn into_select(self) -> Vec<String> {
        vec![self]
    }
}

impl<T: Into<String>> IntoSelect for Vec<T> {
    fn into_select(self) -> Vec<String> {
        self.into_iter().map(|x| x.into()).collect()
    }
}

impl<T: Into<String>> IntoSelect for (T, T) {
    fn into_select(self) -> Vec<String> {
        vec![self.0.into(), self.1.into()]
    }
}

impl<T: Into<String>> IntoSelect for (T, T, T) {
    fn into_select(self) -> Vec<String> {
        vec![self.0.into(), self.1.into(), self.2.into()]
    }
}

impl<T: Into<String>> IntoSelect for (T, T, T, T) {
    fn into_select(self) -> Vec<String> {
        vec![self.0.into(), self.1.into(), self.2.into(), self.3.into()]
    }
}
