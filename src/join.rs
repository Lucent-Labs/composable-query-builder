use crate::error::QueryError;
use crate::util::placeholder_count;
use crate::Select;

#[derive(Debug, Clone)]
pub enum JoinKind {
    Left,
}

impl JoinKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            JoinKind::Left => "left",
        }
    }
}

#[derive(Debug, Clone)]
pub enum Join {
    Simple(String),
    SubQuery(String, Box<Select>),
}

impl TryFrom<String> for Join {
    type Error = QueryError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Join::Simple(value))
    }
}

impl TryFrom<&str> for Join {
    type Error = QueryError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Join::Simple(value.to_string()))
    }
}

impl<T: Into<String>> TryFrom<(T, Select)> for Join {
    type Error = QueryError;

    fn try_from((expr, select): (T, Select)) -> Result<Self, Self::Error> {
        let expr = expr.into();
        placeholder_count(&expr, 1)?;
        Ok(Join::SubQuery(expr, Box::new(select)))
    }
}
