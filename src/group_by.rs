use itertools::Itertools;

pub trait IntoGroupBy {
    fn into_group_by(self) -> String;
}

impl IntoGroupBy for &str {
    fn into_group_by(self) -> String {
        self.to_string()
    }
}

impl IntoGroupBy for String {
    fn into_group_by(self) -> String {
        self
    }
}

impl<T: Into<String>> IntoGroupBy for Vec<T> {
    fn into_group_by(self) -> String {
        self.into_iter().map(|x| x.into()).join(", ")
    }
}

impl<T: Into<String> + Clone> IntoGroupBy for &[T] {
    fn into_group_by(self) -> String {
        self.iter().cloned().map(|x| x.into()).join(", ")
    }
}

impl<T: Into<String>, const N: usize> IntoGroupBy for [T; N] {
    fn into_group_by(self) -> String {
        self.into_iter().map(|x| x.into()).join(", ")
    }
}

impl<T: Into<String> + Clone, const N: usize> IntoGroupBy for &[T; N] {
    fn into_group_by(self) -> String {
        self.iter().cloned().map(|x| x.into()).join(", ")
    }
}

impl<T: Into<String>> IntoGroupBy for (T, T) {
    fn into_group_by(self) -> String {
        format!("{}, {}", self.0.into(), self.1.into())
    }
}

impl<T: Into<String>> IntoGroupBy for (T, T, T) {
    fn into_group_by(self) -> String {
        format!("{}, {}, {}", self.0.into(), self.1.into(), self.2.into())
    }
}

impl<T: Into<String>> IntoGroupBy for (T, T, T, T) {
    fn into_group_by(self) -> String {
        format!(
            "{}, {}, {}, {}",
            self.0.into(),
            self.1.into(),
            self.2.into(),
            self.3.into()
        )
    }
}
