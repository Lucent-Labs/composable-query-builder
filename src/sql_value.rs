use chrono::{NaiveDate, NaiveDateTime};
use serde_json::Value;
use sqlx::{Postgres, QueryBuilder};

/// SQLValue is an enum wrapper around the various types that can be bound to a query.
///
/// This allows us to do some fairly magic looking things with the query builder, in
/// particular with the where clause. For example, the same [where_clause](ComposableQueryBuilder::where_clause)
/// can be used for both a string and int column.
///
/// ```rust
/// use composable_query_builder2::Select;
/// let query = Select::new()
///     .table("users")
///     .where_(("status_id = ?", 2)) // an int
///     .unwrap()
///     .where_(("email = ?", "test@example")) // a string
///     .unwrap()
///     .into_builder();
///
/// let sql = query.sql();
/// assert_eq!("select * from users where status_id = $1 and email = $2", sql.trim());
/// ```
#[derive(Debug, Clone)]
pub enum SQLValue {
    I16(i16),
    I32(i32),
    I64(i64),
    U64(u64),
    F64(f64),
    DateTime(NaiveDateTime),
    Date(NaiveDate),
    VecI64(Vec<i64>),
    String(String),
    Bool(bool),
    Json(Value),
    Null,
}

impl SQLValue {
    pub fn push_bind(&self, qb: &mut QueryBuilder<Postgres>) {
        match self {
            SQLValue::I16(v) => qb.push_bind(*v),
            SQLValue::I32(v) => qb.push_bind(*v),
            SQLValue::I64(v) => qb.push_bind(*v),
            SQLValue::U64(v) => qb.push_bind(*v as i64),
            SQLValue::F64(v) => qb.push_bind(*v),
            SQLValue::DateTime(v) => qb.push_bind(*v),
            SQLValue::Date(v) => qb.push_bind(*v),
            SQLValue::VecI64(v) => qb.push_bind(v.clone()),
            SQLValue::String(v) => qb.push_bind(v.clone()),
            SQLValue::Bool(v) => qb.push_bind(*v),
            SQLValue::Json(v) => qb.push_bind(v.clone()),
            SQLValue::Null => qb.push_bind("null"),
        };
    }

    /// This method isn't actually used, but is here to enable a compile time check
    /// that we have a From<T> implementation for every type that we want to use.
    #[allow(dead_code)]
    fn dummy(&self) -> SQLValue {
        match self.clone() {
            SQLValue::I16(v) => v.into(),
            SQLValue::I32(v) => v.into(),
            SQLValue::I64(v) => v.into(),
            SQLValue::U64(v) => v.into(),
            SQLValue::F64(v) => v.into(),
            SQLValue::DateTime(v) => v.into(),
            SQLValue::Date(v) => v.into(),
            SQLValue::VecI64(v) => v.into(),
            SQLValue::String(v) => v.into(),
            SQLValue::Bool(v) => v.into(),
            SQLValue::Json(v) => v.into(),
            SQLValue::Null => ().into(),
        }
    }
}

impl From<i16> for SQLValue {
    fn from(v: i16) -> Self {
        SQLValue::I16(v)
    }
}

impl From<i32> for SQLValue {
    fn from(v: i32) -> Self {
        SQLValue::I32(v)
    }
}

impl From<i64> for SQLValue {
    fn from(v: i64) -> Self {
        SQLValue::I64(v)
    }
}

impl From<NaiveDateTime> for SQLValue {
    fn from(v: NaiveDateTime) -> Self {
        SQLValue::DateTime(v)
    }
}

impl From<NaiveDate> for SQLValue {
    fn from(v: NaiveDate) -> Self {
        SQLValue::Date(v)
    }
}

impl From<Vec<i64>> for SQLValue {
    fn from(v: Vec<i64>) -> Self {
        SQLValue::VecI64(v)
    }
}

impl From<u64> for SQLValue {
    fn from(v: u64) -> Self {
        SQLValue::U64(v)
    }
}

impl From<f64> for SQLValue {
    fn from(v: f64) -> Self {
        SQLValue::F64(v)
    }
}

impl From<String> for SQLValue {
    fn from(v: String) -> Self {
        SQLValue::String(v)
    }
}

impl From<&String> for SQLValue {
    fn from(v: &String) -> Self {
        SQLValue::String(v.to_string())
    }
}

impl From<&str> for SQLValue {
    fn from(v: &str) -> Self {
        SQLValue::String(v.to_string())
    }
}

impl From<bool> for SQLValue {
    fn from(v: bool) -> Self {
        SQLValue::Bool(v)
    }
}

impl From<Value> for SQLValue {
    fn from(v: Value) -> Self {
        SQLValue::Json(v)
    }
}

impl From<()> for SQLValue {
    fn from(_: ()) -> Self {
        SQLValue::Null
    }
}
