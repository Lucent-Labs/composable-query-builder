use crate::error::{QResult, QueryError};

pub fn placeholder_count(s: &str, exp: usize) -> QResult<()> {
    if s.chars().into_iter().filter(|c| *c == '?').count() != exp {
        Err(QueryError::IncorrectPlaceholderCount(s.to_string(), exp))
    } else {
        Ok(())
    }
}
