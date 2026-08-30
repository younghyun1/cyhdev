use diesel::deserialize::{FromSql, Result as DeserializeResult};
use diesel::pg::{Pg, PgValue};
use diesel::query_builder::QueryId;
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::{AsExpression, FromSqlRow};
use std::io::Write;

use crate::{
    features::photography::domain::photograph::PhotographContext,
    schema::sql_types::PhotographContext as PhotographContextSql,
};

impl QueryId for PhotographContextSql {
    type QueryId = PhotographContextSql;
    const HAS_STATIC_QUERY_ID: bool = true;
}

#[derive(Clone, Copy, Debug, AsExpression, FromSqlRow)]
#[diesel(sql_type = PhotographContextSql)]
pub enum DbPhotographContext {
    Photography,
    Post,
}

impl ToSql<PhotographContextSql, Pg> for DbPhotographContext {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
        out.write_all(match self {
            Self::Photography => b"photography",
            Self::Post => b"post",
        })?;
        Ok(IsNull::No)
    }
}
impl FromSql<PhotographContextSql, Pg> for DbPhotographContext {
    fn from_sql(bytes: PgValue<'_>) -> DeserializeResult<Self> {
        match bytes.as_bytes() {
            b"photography" => Ok(Self::Photography),
            b"post" => Ok(Self::Post),
            _ => Err("unrecognized photograph context".into()),
        }
    }
}
impl From<DbPhotographContext> for PhotographContext {
    fn from(value: DbPhotographContext) -> Self {
        match value {
            DbPhotographContext::Photography => Self::Photography,
            DbPhotographContext::Post => Self::Post,
        }
    }
}
impl From<PhotographContext> for DbPhotographContext {
    fn from(value: PhotographContext) -> Self {
        match value {
            PhotographContext::Photography => Self::Photography,
            PhotographContext::Post => Self::Post,
        }
    }
}
