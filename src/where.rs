use crate::bool_kind::BoolKind;
use crate::error::QueryError;
use crate::sql_value::SQLValue;
use crate::util::placeholder_count;

#[derive(Clone)]
pub enum Where {
    Simple {
        expr: String,
        values: Vec<SQLValue>,
        kind: BoolKind,
    },
}

impl Where {
    pub fn kind(&mut self, kind: BoolKind) {
        match self {
            Where::Simple { kind: k, .. } => *k = kind,
        }
    }
}

impl<S, V> TryFrom<(S, V)> for Where
where
    S: Into<String>,
    V: Into<SQLValue>,
{
    type Error = QueryError;

    fn try_from(value: (S, V)) -> Result<Self, Self::Error> {
        let expr: String = value.0.into();
        placeholder_count(&expr, 1)?;

        Ok(Where::Simple {
            expr,
            values: vec![value.1.into()],
            kind: BoolKind::And,
        })
    }
}

impl<S, V1, V2> TryFrom<(S, V1, V2)> for Where
where
    S: Into<String>,
    V1: Into<SQLValue>,
    V2: Into<SQLValue>,
{
    type Error = QueryError;

    fn try_from(value: (S, V1, V2)) -> Result<Self, Self::Error> {
        let expr: String = value.0.into();
        placeholder_count(&expr, 2)?;

        Ok(Where::Simple {
            expr,
            values: vec![value.1.into(), value.2.into()],
            kind: BoolKind::And,
        })
    }
}

impl<S, V1, V2, V3> TryFrom<(S, V1, V2, V3)> for Where
where
    S: Into<String>,
    V1: Into<SQLValue>,
    V2: Into<SQLValue>,
    V3: Into<SQLValue>,
{
    type Error = QueryError;

    fn try_from(value: (S, V1, V2, V3)) -> Result<Self, Self::Error> {
        let expr: String = value.0.into();
        placeholder_count(&expr, 3)?;

        Ok(Where::Simple {
            expr,
            values: vec![value.1.into(), value.2.into(), value.3.into()],
            kind: BoolKind::And,
        })
    }
}
