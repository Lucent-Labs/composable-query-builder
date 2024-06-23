use crate::bool_kind::BoolKind;
use crate::error::{QResult, QueryError};
use crate::sql_value::SQLValue;
use crate::util::placeholder_count;
use std::fmt::Debug;

#[derive(Default)]
pub struct WhereBuilder {
    expr: String,
    values: Vec<SQLValue>,
    count: usize,
    kind: BoolKind,
}

impl WhereBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn where_<T>(mut self, v: T) -> QResult<Self>
    where
        T: TryInto<Where, Error = QueryError>,
    {
        if self.count > 0 {
            self.expr.push_str(" and ");
        }

        let w: Where = v.try_into()?;
        w.into_where(&mut self.expr, &mut self.values)?;
        self.count += 1;

        Ok(self)
    }

    pub fn or_where<T>(mut self, v: T) -> QResult<Self>
    where
        T: TryInto<Where, Error = QueryError>,
    {
        if self.count > 0 {
            self.expr.push_str(" or ");
        }

        let w: Where = v.try_into()?;
        w.into_where(&mut self.expr, &mut self.values)?;
        self.count += 1;

        Ok(self)
    }

    pub fn kind(mut self, kind: BoolKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn build(self) -> Where {
        Where::Simple {
            expr: self.expr,
            values: self.values,
            kind: self.kind,
        }
    }
}

pub trait IntoWhere {
    fn into_where(self, expr: &mut String, vals: &mut Vec<SQLValue>) -> QResult<()>;
}

impl<T: Into<SQLValue>> IntoWhere for Option<T> {
    fn into_where(self, expr: &mut String, vals: &mut Vec<SQLValue>) -> QResult<()> {
        expr.push('?');
        match self {
            Some(v) => vals.push(v.into()),
            None => vals.push(SQLValue::Null),
        }
        Ok(())
    }
}

impl<T: Into<SQLValue>> IntoWhere for T {
    fn into_where(self, expr: &mut String, vals: &mut Vec<SQLValue>) -> QResult<()> {
        expr.push('?');
        vals.push(self.into());
        Ok(())
    }
}

impl IntoWhere for Where {
    fn into_where(self, expression: &mut String, vals: &mut Vec<SQLValue>) -> QResult<()> {
        match self {
            Where::Simple { expr, values, .. } => {
                expression.push_str(&expr);
                vals.extend(values);
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
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

impl TryFrom<&str> for Where {
    type Error = QueryError;

    fn try_from(input_expr: &str) -> Result<Self, Self::Error> {
        placeholder_count(input_expr, 0)?;

        Ok(Where::Simple {
            expr: input_expr.to_string(),
            values: vec![],
            kind: BoolKind::And,
        })
    }
}

impl TryFrom<String> for Where {
    type Error = QueryError;

    fn try_from(input_expr: String) -> Result<Self, Self::Error> {
        placeholder_count(&input_expr, 0)?;

        Ok(Where::Simple {
            expr: input_expr,
            values: vec![],
            kind: BoolKind::And,
        })
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
        let input_expr: String = expr.into();
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
