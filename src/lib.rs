mod bool_kind;
mod error;
mod group_by;
mod join;
mod optional_num;
mod order;
mod select;
mod sql_value;
mod util;
mod r#where;

use crate::bool_kind::BoolKind;
use crate::error::QResult;
use crate::join::{Join, JoinKind};
use crate::optional_num::IntoOptional;
pub use crate::order::OrderDir;
pub use crate::r#where::{IntoWhere, Where, WhereBuilder};
use crate::select::IntoSelect;
pub use crate::sql_value::SQLValue;
pub use error::QueryError;
use group_by::IntoGroupBy;
use itertools::{EitherOrBoth, Itertools};
use sqlx::{Postgres, QueryBuilder};

#[derive(Debug, Clone, Default)]
pub struct Select {
    table: Option<TableType>,
    select: Vec<String>,
    join: Vec<(JoinKind, Join)>,
    where_: Vec<Where>,
    order_by: Option<(String, OrderDir)>,
    group_by: Option<String>,
    limit: Option<u64>,
    offset: Option<u64>,
}

#[derive(Debug, Clone)]
pub enum TableType {
    Simple(String),
    Complex(String, Vec<Select>),
}

impl From<&str> for TableType {
    fn from(value: &str) -> Self {
        TableType::Simple(value.to_string())
    }
}
impl From<&String> for TableType {
    fn from(value: &String) -> Self {
        TableType::Simple(value.to_string())
    }
}
impl From<String> for TableType {
    fn from(value: String) -> Self {
        TableType::Simple(value)
    }
}
impl From<(&str, Select)> for TableType {
    fn from((stmt, value): (&str, Select)) -> Self {
        TableType::Complex(stmt.to_string(), vec![value])
    }
}
impl From<(String, Select)> for TableType {
    fn from((stmt, value): (String, Select)) -> Self {
        TableType::Complex(stmt, vec![value])
    }
}
impl From<(&str, Select, Select)> for TableType {
    fn from((stmt, value1, value2): (&str, Select, Select)) -> Self {
        TableType::Complex(stmt.to_string(), vec![value1, value2])
    }
}
impl From<(String, Select, Select)> for TableType {
    fn from((stmt, value1, value2): (String, Select, Select)) -> Self {
        TableType::Complex(stmt, vec![value1, value2])
    }
}

impl Select {
    pub fn new() -> Self {
        Self::default()
    }

    /// Shorthand for creating a new Select builder, then
    /// setting the table.
    ///
    /// Example:
    /// ```
    /// use composable_query_builder2::Select;
    /// Select::from("my_table");
    /// ```
    pub fn from(table: impl Into<TableType>) -> Self {
        let q = Self::new();
        q.table(table)
    }

    /// Just a helper. Shorthand for:
    /// ```
    /// use composable_query_builder2::Select;
    /// let a = Select::from("my_table");
    /// let b = Select::from("my_table");
    /// Select::from(("((?) union (?) as alias", a, b));
    /// ```
    pub fn union(a: Select, b: Select, alias: impl Into<String>) -> Self {
        let q = format!("((?) union (?)) as {}", alias.into());
        Self::from((q, a, b))
    }

    /// Example:
    /// ```
    /// use composable_query_builder2::Select;
    /// Select::new().table("my_table");
    /// ```
    ///
    /// You will probably wany to use [Select::from] in most cases
    pub fn table(mut self, table: impl Into<TableType>) -> Self {
        self.table = Some(table.into());
        self
    }

    /// The passed item should _not_ contain leading "left join" text.
    /// That is added automatically.
    pub fn left_join<T>(mut self, join: T) -> QResult<Self>
    where
        T: TryInto<Join, Error = QueryError>,
    {
        self.join.push((JoinKind::Left, join.try_into()?));
        Ok(self)
    }

    /// Where expressions are constructed as either strings or tuples.
    /// The first value in the tuple is the query fragment, and the remaining
    /// are the values to pass in.
    ///
    /// Placeholders are `?` and _not_ the normal Postgres `$1, $2, $3` style.
    /// They will be converted to the Postgres `$1` style when the query is built.
    ///
    /// One, two, or three values can be passed in, in addition to
    /// the first string value.
    ///
    /// Example:
    /// ```
    /// use composable_query_builder2::Select;
    /// Select::from("my_table").where_("id > 0")?;
    /// Select::from("my_table").where_(("id > ?", 0))?;
    /// Select::from("my_table").where_(("id > ? and id < ?", 0, 10))?;
    /// Select::from("my_table").where_(("id > ? and id < ? and other_col > ?", 0, 10, 5))?;
    /// # Ok::<(), composable_query_builder2::QueryError>(())
    /// ```
    ///
    /// Where clauses can also be composed:
    /// ```
    /// use composable_query_builder2::{Where, Select};
    /// let sub: Where = ("id > ? and id < ?", 1, 10).try_into()?;
    /// Select::from("my_table").where_(("id = 20 or (?)", sub))?;
    /// # Ok::<(), composable_query_builder2::QueryError>(())
    /// ```
    pub fn where_<T, E>(mut self, where_: T) -> QResult<Self>
    where
        T: TryInto<Where, Error = E>,
        QueryError: From<E>,
    {
        self.where_.push(where_.try_into()?);
        Ok(self)
    }

    pub fn where_if<T, E>(mut self, cond: bool, callback: impl Fn() -> T) -> QResult<Self>
    where
        T: TryInto<Where, Error = E>,
        QueryError: From<E>,
    {
        if cond {
            self.where_.push(callback().try_into()?);
        }
        Ok(self)
    }

    pub fn where_in(mut self, col: impl Into<String>, values: Vec<i64>) -> Self {
        let expr = format!("{} = ANY(?)", col.into());
        self.where_.push(Where::Simple {
            expr,
            values: vec![values.into()],
            kind: BoolKind::And,
        });
        self
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

    /// Adds one or more columns to the select statement.
    ///
    /// See [`IntoSelect`] for details on what can be passed in.
    pub fn select(mut self, column: impl IntoSelect) -> Self {
        self.select.append(&mut column.into_select());
        self
    }

    pub fn group_by(mut self, group_by: impl IntoGroupBy) -> Self {
        self.group_by = Some(group_by.into_group_by());
        self
    }

    /// ## Danger: SQL injection
    ///
    /// The passed `col` is _not_ sanitized. If this is taking
    /// user input, it should be compared against an allow-list.
    pub fn order_by(mut self, col: impl Into<String>, dir: OrderDir) -> Self {
        self.order_by = Some((col.into(), dir));
        self
    }

    pub fn limit(mut self, limit: impl IntoOptional<u64>) -> Self {
        self.limit = limit.into_optional();
        self
    }

    /// An alias for [Select::limit]
    pub fn take(self, take: impl IntoOptional<u64>) -> Self {
        self.limit(take)
    }

    pub fn offset(mut self, offset: impl IntoOptional<u64>) -> Self {
        self.offset = offset.into_optional();
        self
    }

    /// An alais for [Select::offset]
    pub fn skip(self, skip: impl IntoOptional<u64>) -> Self {
        self.offset(skip)
    }

    pub fn parts(self) -> (String, Vec<SQLValue>) {
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
            Some(TableType::Complex(s, v)) => {
                // println!("q at the start is {}", q);
                // println!("vals at the start are {:?}", vals);

                let mut parts = s.split("?");
                if let Some(part) = parts.next() {
                    q.push_str(part);
                }
                for select in v.iter() {
                    let (sub_q, sub_vals) = select.clone().parts();
                    // println!("sub_q: {}", sub_q);
                    // println!("sub_vals: {:?}", sub_vals);
                    // q.push_str(" (");
                    q.push_str(sub_q.as_str());
                    // q.push(')');
                    // if i < v.len() - 1 {
                    //     q.push_str(", ");
                    // }
                    vals.extend(sub_vals);

                    if let Some(part) = parts.next() {
                        q.push_str(part);
                    }
                }

                for part in parts {
                    q.push_str(part);
                }
            }
            None => panic!("No table specified"),
        }

        // Joins
        for (kind, join) in self.join {
            match join {
                Join::Simple(s) => {
                    q.push(' ');
                    q.push_str(kind.as_str());
                    q.push_str(" join ");
                    q.push_str(&s);
                }
                Join::SubQuery(s, select) => {
                    q.push(' ');
                    q.push_str(kind.as_str());
                    q.push_str(" join ");
                    let (sub_q, sub_vals) = select.parts();

                    let mut parts = s.split('?');

                    // When creating the join we check to ensure we have
                    // at least one `?`, so this unwrap should be safe.
                    q.push_str(parts.next().unwrap());
                    q.push_str(sub_q.trim());
                    if let Some(part) = parts.next() {
                        q.push_str(part);
                    }

                    vals.extend(sub_vals);
                }
            }
        }

        // Where
        if !self.where_.is_empty() {
            q.push_str(" where ");
            let last_index = self.where_.len() - 1;
            for (index, clause) in self.where_.iter().enumerate() {
                match clause {
                    Where::Simple {
                        expr,
                        values,
                        kind: _,
                    } => {
                        q.push_str(expr);
                        vals.extend(values.clone());
                        if index != last_index {
                            // Get the next kind
                            let next_kind = self.where_.get(index + 1).unwrap().get_kind();
                            q.push(' ');
                            q.push_str(next_kind.as_str());
                            q.push(' ');
                        } else {
                            q.push(' ');
                        }
                    }
                }
            }
        }

        // Group by
        if let Some(group_by) = self.group_by {
            q.push_str(" group by ");
            q.push_str(&group_by);
            q.push(' ');
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
            q.push_str(" limit ?");
            vals.push(limit.into());
        }

        // Offset
        if let Some(offset) = self.offset {
            q.push_str(" offset ?");
            vals.push(offset.into());
        }

        // println!("at the end q is {:?}", q);

        (q, vals)
    }

    pub fn into_builder<'args>(self) -> QueryBuilder<'args, Postgres> {
        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new("");

        let (p, v) = self.parts();
        let parts = p.split('?').collect::<Vec<_>>();
        assert_query_part_and_placeholder_lengths_correct(&parts, v.len());

        for pair in parts.into_iter().zip_longest(v.into_iter()) {
            use EitherOrBoth::*;
            match pair {
                Both(part, v) => {
                    qb.push(part);
                    v.push_bind(&mut qb);
                }
                Left(part) => {
                    qb.push(part);
                }
                Right(v) => {
                    v.push_bind(&mut qb);
                }
            }
        }

        qb
    }
}

fn assert_query_part_and_placeholder_lengths_correct(query_parts: &[&str], placeholders: usize) {
    assert!(
        query_parts.len() == placeholders + 1 || query_parts.len() == placeholders,
        "Query part count and placeholder count mismatch.

The query should always have either exactly the same, or one
more placeholder than the number of query parts.

In most cases, we'll have something like \"select * from users where id = ? order by id desc\",
giving 2 query parts and one placeholder.

In cases where we have a trailing placeholder, the number of query parts and placeholders
will be equal: \"select * from users limit $1\"

    {} Query parts: {:?}
Placeholder count: {}
",
        query_parts.len(),
        query_parts,
        placeholders,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::r#where::WhereBuilder;

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

        // Multiple selects append
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
    fn basic_where_in() {
        let q = Select::from("users")
            .where_in("id", vec![1, 2, 3])
            .into_builder();
        let sql = q.sql();
        assert_eq!("select * from users where id = ANY($1) ", sql);
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
    fn mixed_and_or_where() -> QResult<()> {
        let q = Select::from("users")
            .where_(("status_id = ?", 1))?
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

    #[test]
    fn simple_join() -> QResult<()> {
        let q = Select::from("users")
            .left_join("posts on users.id = posts.user_id")?
            .into_builder();
        let query = q.sql();

        assert_eq!(
            "select * from users left join posts on users.id = posts.user_id",
            query
        );
        Ok(())
    }

    #[test]
    fn nested_join() -> QResult<()> {
        let sub = Select::from("posts")
            .select(("id", "user_id"))
            .where_(("posts.id = ?", 1))?;

        let q = Select::from("users")
            .left_join(("(?) as sub on users.id = sub.user_id", sub))?
            .into_builder();
        let query = q.sql();

        assert_eq!(
            "select * from users left join (select id, user_id from posts where posts.id = $1) as sub on users.id = sub.user_id",
            query
        );
        Ok(())
    }

    #[test]
    fn nested_where() -> QResult<()> {
        let w: Where = ("(orders > ? and orders < ?)", 1, 10).try_into()?;
        let q = Select::from("users")
            .where_(("id = ? or ?", 1, w))?
            .into_builder();
        let query = q.sql();

        assert_eq!(
            "select * from users where id = $1 or (orders > $2 and orders < $3) ",
            query
        );
        Ok(())
    }

    #[test]
    fn where_builder() -> QResult<()> {
        let w = WhereBuilder::new()
            .where_(("name ilike %?%", "test"))?
            .where_(("email ilike %?%", "test"))?
            .where_(("business ilike %?%", "test"))?
            .build();

        let q = Select::from("users").where_(("(?)", w))?.into_builder();
        let query = q.sql();

        assert_eq!(
            "select * from users where (name ilike %$1% and email ilike %$2% and business ilike %$3%) ",
            query
        );
        Ok(())
    }

    #[test]
    fn union() -> QResult<()> {
        let a = Select::from("users").select("id").where_(("id = ?", 1))?;
        let b = Select::from("users").select("id").where_(("id = ?", 2))?;
        let u = Select::from(("(?) as a", a))
            .left_join(("(?) as b on a.id = b.id", b))?
            .into_builder();
        println!("{}", u.sql());

        let exp = "select * from (select id from users where id = $1 ) as a left join (select id from users where id = $2) as b on a.id = b.id";
        assert_eq!(u.sql(), exp);
        Ok(())
    }

    #[test]
    fn multiple_string() -> QResult<()> {
        let w = WhereBuilder::new()
            .or_where("id = 1")?
            .or_where("id = 2")?
            .or_where("id = 3")?
            .build();

        let q = Select::from("users").where_(("?", w))?.into_builder();
        let query = q.sql();
        assert_eq!(
            "select * from users where id = 1 or id = 2 or id = 3 ",
            query
        );
        Ok(())
    }

    #[test]
    fn conditional_where() -> QResult<()> {
        let q = Select::from("users").where_if(true, || ("id > ?", 5))?;
        assert_eq!("select * from users where id > $1 ", q.into_builder().sql());
        let q = Select::from("users").where_if(false, || ("id > ?", 5))?;
        assert_eq!("select * from users", q.into_builder().sql());
        Ok(())
    }

    #[test]
    fn can_select_from_slices_and_arrays() -> QResult<()> {
        let q = Select::from("users").select(["id", "email"].as_slice());
        assert_eq!("select id, email from users", q.into_builder().sql());

        let q = Select::from("users").select(["id", "email"]);
        assert_eq!("select id, email from users", q.into_builder().sql());

        let q = Select::from("users").select(&["id", "email"]);
        assert_eq!("select id, email from users", q.into_builder().sql());
        Ok(())
    }

    #[test]
    fn multi_group_by_works() -> QResult<()> {
        let q = Select::from("my_table").group_by(("a", "b"));
        assert_eq!(
            "select * from my_table group by a, b ",
            q.into_builder().sql()
        );

        let q = Select::from("my_table").group_by(["a", "b"]);
        assert_eq!(
            "select * from my_table group by a, b ",
            q.into_builder().sql()
        );

        let q = Select::from("my_table").group_by(vec!["a", "b"]);
        assert_eq!(
            "select * from my_table group by a, b ",
            q.into_builder().sql()
        );

        Ok(())
    }

    #[test]
    fn it_can_union() -> QResult<()> {
        let a = Select::from("users").where_(("id > ?", 5))?;
        let b = Select::from("users").where_(("id < ?", 3))?;
        let q = Select::from(("((?) union (?)) as t", a, b));

        assert_eq!(
            "select * from ((select * from users where id > $1 ) union (select * from users where id < $2 )) as t",
            q.into_builder().sql()
        );

        let a = Select::from("users").where_(("id > ?", 5))?;
        let b = Select::from("users").where_(("id < ?", 3))?;
        let q = Select::union(a, b, "t");

        assert_eq!(
            "select * from ((select * from users where id > $1 ) union (select * from users where id < $2 )) as t",
            q.into_builder().sql()
        );
        Ok(())
    }
}
