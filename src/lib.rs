mod bool_kind;
mod error;
mod optional_num;
mod order;
mod select;
mod sql_value;
mod r#where;

use crate::bool_kind::BoolKind;
use crate::error::{QResult, QueryError};
use crate::optional_num::IntoOptional;
use crate::order::OrderDir;
use crate::r#where::Where;
use crate::select::IntoSelect;
use crate::sql_value::SQLValue;
use itertools::{EitherOrBoth, Itertools};
use sqlx::{Postgres, QueryBuilder};

#[derive(Clone, Default)]
pub struct Select {
    table: Option<TableType>,
    select: Vec<String>,
    where_: Vec<Where>,
    order_by: Option<(String, OrderDir)>,
    limit: Option<u64>,
    offset: Option<u64>,
}

#[derive(Clone)]
pub enum TableType {
    Simple(String),
    Complex(String, Vec<Select>),
}

impl From<&str> for TableType {
    fn from(value: &str) -> Self {
        TableType::Simple(value.to_string())
    }
}
impl From<String> for TableType {
    fn from(value: String) -> Self {
        TableType::Simple(value)
    }
}

impl Select {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from(table: impl Into<TableType>) -> Self {
        let q = Self::new();
        q.table(table)
    }

    pub fn table(mut self, table: impl Into<TableType>) -> Self {
        self.table = Some(table.into());
        self
    }

    pub fn where_<T>(mut self, where_: T) -> QResult<Self>
    where
        T: TryInto<Where, Error = QueryError>,
    {
        self.where_.push(where_.try_into()?);
        Ok(self)
    }

    pub fn or_where<T>(mut self, where_: T) -> QResult<Self>
    where
        T: TryInto<Where, Error = QueryError>,
    {
        let mut w = where_.try_into()?;
        w.kind(BoolKind::Or);
        self.where_.push(w);
        Ok(self)
    }

    pub fn select(mut self, column: impl IntoSelect) -> Self {
        self.select.append(&mut column.into_select());
        self
    }

    pub fn order_by(mut self, col: impl Into<String>, dir: OrderDir) -> Self {
        self.order_by = Some((col.into(), dir));
        self
    }

    pub fn limit(mut self, limit: impl IntoOptional<u64>) -> Self {
        self.limit = limit.into_optional();
        self
    }

    pub fn offset(mut self, offset: impl IntoOptional<u64>) -> Self {
        self.offset = offset.into_optional();
        self
    }

    fn parts(self) -> (String, Vec<SQLValue>) {
        let mut q = "select ".to_string();
        let mut vals: Vec<SQLValue> = vec![];

        // Select
        if self.select.is_empty() {
            q.push('*');
        } else {
            let l = self.select.len() - 1;
            for (last, s) in self.select.into_iter().enumerate().map(|x| (x.0 == l, x.1)) {
                q.push_str(s.as_str());
                if !last {
                    q.push_str(", ");
                }
            }
        }

        // Table
        q.push_str(" from ");
        match self.table {
            Some(TableType::Simple(s)) => q.push_str(s.as_str()),
            Some(TableType::Complex(s, _)) => todo!(),
            None => panic!("No table specified"),
        }

        // Where
        if !self.where_.is_empty() {
            q.push_str(" where ");
            let l = self.where_.len() - 1;
            for (last, clause) in self.where_.into_iter().enumerate().map(|x| (x.0 == l, x.1)) {
                match clause {
                    Where::Simple { expr, values, kind } => {
                        q.push_str(&expr);
                        vals.extend(values);
                        if !last {
                            q.push(' ');
                            q.push_str(kind.as_str());
                            q.push(' ');
                        } else {
                            q.push(' ');
                        }
                    }
                }
            }
        }

        // Order by
        if let Some((col, dir)) = self.order_by {
            q.push_str(" order by ");
            q.push_str(&col);
            q.push(' ');
            q.push_str(dir.as_str());
            q.push(' ');
        }

        // Limit
        if let Some(limit) = self.limit {
            q.push_str(" limit ");
            vals.push(limit.into());
            // vals.push(&mut SQLValue::U64(limit));
        }

        // Offset
        if let Some(offset) = self.offset {
            q.push_str(" offset ");
            vals.push(offset.into());
            // vals.push(&mut SQLValue::U64(offset));
        }

        (q, vals)
    }

    pub fn into_builder<'args>(self) -> QueryBuilder<'args, Postgres> {
        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new("");

        let (p, v) = self.parts();
        let mut parts = p.split('?').collect::<Vec<_>>();
        // The query should always have either exactly the same, or one more
        // placeholder than the number of query parts.
        //
        // In most cases, we'll have something like "select * from users where id = ? order by id desc",
        // giving 2 query parts and one placeholder.
        //
        // In cases where we have a trailing placeholder, the number of query parts and placeholders
        // will be equal
        // "select * from users limit $1"
        assert!(parts.len() == v.len() + 1 || parts.len() == v.len());

        for pair in parts.into_iter().zip_longest(v.into_iter()) {
            match pair {
                EitherOrBoth::Both(part, v) => {
                    qb.push(part);
                    v.push_bind(&mut qb);
                }
                EitherOrBoth::Left(part) => {
                    qb.push(part);
                }
                EitherOrBoth::Right(v) => {
                    v.push_bind(&mut qb);
                }
            }
        }

        qb
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_select_star() {
        let q = Select::from("users").into_builder();
        let sql = q.sql();
        assert_eq!("select * from users", sql);
    }

    #[test]
    fn basic_select_named() {
        let q = Select::from("users").select("id").into_builder();
        let sql = q.sql();
        assert_eq!("select id from users", sql);

        let q = Select::from("users")
            .select("id")
            .select("name")
            .into_builder();
        let sql = q.sql();
        assert_eq!("select id, name from users", sql);

        let q = Select::from("users")
            .select(vec!["id", "name"])
            .into_builder();
        let sql = q.sql();
        assert_eq!("select id, name from users", sql);

        let q = Select::from("users").select(("id", "name")).into_builder();
        let sql = q.sql();
        assert_eq!("select id, name from users", sql);

        let q = Select::from("users")
            .select(("id", "name", "email"))
            .into_builder();
        let sql = q.sql();
        assert_eq!("select id, name, email from users", sql);
    }

    #[test]
    fn basic_where() -> QResult<()> {
        let q = Select::from("users")
            .where_(("orders > ?", 1))?
            .into_builder();
        let query = q.sql();

        assert_eq!("select * from users where orders > $1 ", query);

        let q = Select::from("users")
            .where_(("orders > ?", 1))?
            .where_(("orders < ?", 10))?
            .into_builder();
        let query = q.sql();

        assert_eq!(
            "select * from users where orders > $1 and orders < $2 ",
            query
        );

        let q = Select::from("users")
            .where_(("orders > ? and orders < ?", 1, 10))?
            .into_builder();
        let query = q.sql();

        assert_eq!(
            "select * from users where orders > $1 and orders < $2 ",
            query
        );

        Ok(())
    }

    #[test]
    fn triple_where_different_types() -> QResult<()> {
        let q = Select::from("users")
            .where_(("(orders > ? and orders < ?) or sales > ?", 10, 100, 123.45))?
            .into_builder();
        let query = q.sql();

        assert_eq!(
            "select * from users where (orders > $1 and orders < $2) or sales > $3 ",
            query
        );
        Ok(())
    }

    #[test]
    fn or_where_works() -> QResult<()> {
        let q = Select::from("users")
            .or_where(("status_id = ?", 1))?
            .or_where(("status_id = ?", 2))?
            .or_where(("status_id = ?", 3))?
            .into_builder();
        let query = q.sql();

        assert_eq!(
            "select * from users where status_id = $1 or status_id = $2 or status_id = $3 ",
            query
        );
        Ok(())
    }

    #[test]
    fn order_by_works() {
        let q = Select::from("users")
            .order_by("email", OrderDir::Desc)
            .into_builder();
        let query = q.sql();

        assert_eq!("select * from users order by email desc ", query);

        let q = Select::from("users")
            .order_by("email", OrderDir::Asc)
            .into_builder();
        let query = q.sql();

        assert_eq!("select * from users order by email asc ", query);
    }

    #[test]
    fn limit() {
        let q = Select::from("users").limit(10).into_builder();
        let query = q.sql();

        assert_eq!("select * from users limit $1", query);

        let q = Select::from("users").limit(Some(10)).into_builder();
        let query = q.sql();

        assert_eq!("select * from users limit $1", query);
    }

    #[test]
    fn offset() {
        let q = Select::from("users").offset(10).into_builder();
        let query = q.sql();

        assert_eq!("select * from users offset $1", query);

        let q = Select::from("users").offset(Some(10)).into_builder();
        let query = q.sql();

        assert_eq!("select * from users offset $1", query);
    }
}
