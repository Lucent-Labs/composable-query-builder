use crate::bool_kind::BoolKind;
use crate::error::{QResult, QueryError};
use crate::sql_value::SQLValue;
use crate::util::placeholder_count;

pub trait IntoWhere {
    fn into_where(self, expr: &mut String, vals: &mut Vec<SQLValue>) -> QResult<()>;
}

impl<T: Into<SQLValue>> IntoWhere for T {
    fn into_where(self, expr: &mut String, vals: &mut Vec<SQLValue>) -> QResult<()> {
        expr.push('?');
        vals.push(self.into());
        Ok(())
    }
}

impl IntoWhere for Where {
    fn into_where(self, expr: &mut String, vals: &mut Vec<SQLValue>) -> QResult<()> {
        match self {
            Where::Simple {
                expr: e,
                values,
                kind,
            } => {
                // expr.push_str(&format!(" {} ", kind.as_str()));
                expr.push_str(&e);
                vals.extend(values);
                Ok(())
            }
        }
    }
}

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
    V: IntoWhere,
{
    type Error = QueryError;

    fn try_from((input_expr, v1): (S, V)) -> Result<Self, Self::Error> {
        let input_expr: String = input_expr.into();
        placeholder_count(&input_expr, 1)?;

        let mut expr = String::new();
        let mut values = Vec::with_capacity(1);

        let mut parts = input_expr.split("?");
        expr.push_str(parts.next().unwrap());
        v1.into_where(&mut expr, &mut values)?;
        if let Some(part) = parts.next() {
            expr.push_str(part);
        }
        assert!(parts.next().is_none());

        Ok(Where::Simple {
            expr,
            values,
            kind: BoolKind::And,
        })
    }
}

impl<S, V1, V2> TryFrom<(S, V1, V2)> for Where
where
    S: Into<String>,
    V1: IntoWhere,
    V2: IntoWhere,
{
    type Error = QueryError;

    fn try_from((expr, v1, v2): (S, V1, V2)) -> Result<Self, Self::Error> {
        let mut input_expr: String = expr.into();
        placeholder_count(&input_expr, 2)?;

        let mut expr = String::new();
        let mut values = Vec::with_capacity(2);

        let mut parts = input_expr.split("?");
        expr.push_str(parts.next().unwrap());
        v1.into_where(&mut expr, &mut values)?;
        expr.push_str(parts.next().unwrap());
        v2.into_where(&mut expr, &mut values)?;
        if let Some(part) = parts.next() {
            expr.push_str(part);
        }
        assert!(parts.next().is_none());

        Ok(Where::Simple {
            expr,
            values,
            kind: BoolKind::And,
        })
    }
}

// impl<S, V1, V2> TryFrom<(S, V1, V2)> for Where
// where
//     S: Into<String>,
//     V1: Into<SQLValue>,
//     V2: Into<SQLValue>,
// {
//     type Error = QueryError;
//
//     fn try_from(value: (S, V1, V2)) -> Result<Self, Self::Error> {
//         let expr: String = value.0.into();
//         placeholder_count(&expr, 2)?;
//
//         Ok(Where::Simple {
//             expr,
//             values: vec![value.1.into(), value.2.into()],
//             kind: BoolKind::And,
//         })
//     }
// }

impl<S, V1, V2, V3> TryFrom<(S, V1, V2, V3)> for Where
where
    S: Into<String>,
    V1: IntoWhere,
    V2: IntoWhere,
    V3: IntoWhere,
{
    type Error = QueryError;

    fn try_from((input_expr, v1, v2, v3): (S, V1, V2, V3)) -> Result<Self, Self::Error> {
        let input_expr: String = input_expr.into();
        placeholder_count(&input_expr, 3)?;

        let mut expr = String::new();
        let mut values = Vec::with_capacity(3);

        let mut parts = input_expr.split("?");
        expr.push_str(parts.next().unwrap());
        v1.into_where(&mut expr, &mut values)?;
        expr.push_str(parts.next().unwrap());
        v2.into_where(&mut expr, &mut values)?;
        expr.push_str(parts.next().unwrap());
        v3.into_where(&mut expr, &mut values)?;
        if let Some(part) = parts.next() {
            expr.push_str(part);
        }
        assert!(parts.next().is_none());

        Ok(Where::Simple {
            expr,
            values,
            kind: BoolKind::And,
        })
    }
}
