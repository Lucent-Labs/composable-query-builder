use thiserror::Error;

pub type QResult<T> = Result<T, QueryError>;

#[derive(Error, Debug)]
pub enum QueryError {
    #[error("incorrect placeholder count in query: {0} expected {1}")]
    IncorrectPlaceholderCount(String, usize),
}
