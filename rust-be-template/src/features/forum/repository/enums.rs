//! Diesel mappings for native forum enums.

use crate::{
    features::forum::domain::enums::{
        ForumContentState, ForumModerationAction, ForumNotificationKind, ForumTopicAccessState,
    },
    schema::sql_types::{
        ForumContentState as ForumContentStateSql,
        ForumModerationAction as ForumModerationActionSql,
        ForumNotificationKind as ForumNotificationKindSql,
        ForumTopicAccessState as ForumTopicAccessStateSql,
    },
};
use diesel::deserialize::{FromSql, Result as DeserializeResult};
use diesel::pg::{Pg, PgValue};
use diesel::query_builder::QueryId;
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::{AsExpression, FromSqlRow};
use std::io::Write;

impl QueryId for ForumContentStateSql {
    type QueryId = ForumContentStateSql;
    const HAS_STATIC_QUERY_ID: bool = true;
}
impl QueryId for ForumTopicAccessStateSql {
    type QueryId = ForumTopicAccessStateSql;
    const HAS_STATIC_QUERY_ID: bool = true;
}
impl QueryId for ForumModerationActionSql {
    type QueryId = ForumModerationActionSql;
    const HAS_STATIC_QUERY_ID: bool = true;
}
impl QueryId for ForumNotificationKindSql {
    type QueryId = ForumNotificationKindSql;
    const HAS_STATIC_QUERY_ID: bool = true;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, AsExpression, FromSqlRow)]
#[diesel(sql_type = ForumContentStateSql)]
pub enum DbForumContentState {
    Visible,
    Hidden,
    Deleted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, AsExpression, FromSqlRow)]
#[diesel(sql_type = ForumTopicAccessStateSql)]
pub enum DbForumTopicAccessState {
    Open,
    Locked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, AsExpression, FromSqlRow)]
#[diesel(sql_type = ForumModerationActionSql)]
pub enum DbForumModerationAction {
    TopicHidden,
    TopicRestored,
    TopicLocked,
    TopicUnlocked,
    TopicPinned,
    TopicUnpinned,
    ReplyHidden,
    ReplyRestored,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, AsExpression, FromSqlRow)]
#[diesel(sql_type = ForumNotificationKindSql)]
pub enum DbForumNotificationKind {
    TopicReply,
}

macro_rules! pg_enum {
    ($rust:ty, $sql:ty, {$($bytes:literal => $variant:path),+ $(,)?}) => {
        impl ToSql<$sql, Pg> for $rust {
            fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
                out.write_all(self.as_str().as_bytes())?; Ok(IsNull::No)
            }
        }
        impl FromSql<$sql, Pg> for $rust {
            fn from_sql(bytes: PgValue<'_>) -> DeserializeResult<Self> {
                match bytes.as_bytes() { $($bytes => Ok($variant),)+ _ => Err("unrecognized forum enum value".into()) }
            }
        }
    };
}

impl DbForumContentState {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Visible => "visible",
            Self::Hidden => "hidden",
            Self::Deleted => "deleted",
        }
    }
}
impl DbForumTopicAccessState {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Locked => "locked",
        }
    }
}
impl DbForumModerationAction {
    const fn as_str(self) -> &'static str {
        match self {
            Self::TopicHidden => "topic_hidden",
            Self::TopicRestored => "topic_restored",
            Self::TopicLocked => "topic_locked",
            Self::TopicUnlocked => "topic_unlocked",
            Self::TopicPinned => "topic_pinned",
            Self::TopicUnpinned => "topic_unpinned",
            Self::ReplyHidden => "reply_hidden",
            Self::ReplyRestored => "reply_restored",
        }
    }
}
impl DbForumNotificationKind {
    const fn as_str(self) -> &'static str {
        "topic_reply"
    }
}

pg_enum!(DbForumContentState, ForumContentStateSql, { b"visible" => Self::Visible, b"hidden" => Self::Hidden, b"deleted" => Self::Deleted });
pg_enum!(DbForumTopicAccessState, ForumTopicAccessStateSql, { b"open" => Self::Open, b"locked" => Self::Locked });
pg_enum!(DbForumModerationAction, ForumModerationActionSql, {
    b"topic_hidden" => Self::TopicHidden, b"topic_restored" => Self::TopicRestored, b"topic_locked" => Self::TopicLocked, b"topic_unlocked" => Self::TopicUnlocked,
    b"topic_pinned" => Self::TopicPinned, b"topic_unpinned" => Self::TopicUnpinned, b"reply_hidden" => Self::ReplyHidden, b"reply_restored" => Self::ReplyRestored,
});
pg_enum!(DbForumNotificationKind, ForumNotificationKindSql, { b"topic_reply" => Self::TopicReply });

impl From<DbForumContentState> for ForumContentState {
    fn from(value: DbForumContentState) -> Self {
        match value {
            DbForumContentState::Visible => Self::Visible,
            DbForumContentState::Hidden => Self::Hidden,
            DbForumContentState::Deleted => Self::Deleted,
        }
    }
}
impl From<DbForumTopicAccessState> for ForumTopicAccessState {
    fn from(value: DbForumTopicAccessState) -> Self {
        match value {
            DbForumTopicAccessState::Open => Self::Open,
            DbForumTopicAccessState::Locked => Self::Locked,
        }
    }
}
impl From<DbForumModerationAction> for ForumModerationAction {
    fn from(value: DbForumModerationAction) -> Self {
        match value {
            DbForumModerationAction::TopicHidden => Self::TopicHidden,
            DbForumModerationAction::TopicRestored => Self::TopicRestored,
            DbForumModerationAction::TopicLocked => Self::TopicLocked,
            DbForumModerationAction::TopicUnlocked => Self::TopicUnlocked,
            DbForumModerationAction::TopicPinned => Self::TopicPinned,
            DbForumModerationAction::TopicUnpinned => Self::TopicUnpinned,
            DbForumModerationAction::ReplyHidden => Self::ReplyHidden,
            DbForumModerationAction::ReplyRestored => Self::ReplyRestored,
        }
    }
}
impl From<ForumModerationAction> for DbForumModerationAction {
    fn from(value: ForumModerationAction) -> Self {
        match value {
            ForumModerationAction::TopicHidden => Self::TopicHidden,
            ForumModerationAction::TopicRestored => Self::TopicRestored,
            ForumModerationAction::TopicLocked => Self::TopicLocked,
            ForumModerationAction::TopicUnlocked => Self::TopicUnlocked,
            ForumModerationAction::TopicPinned => Self::TopicPinned,
            ForumModerationAction::TopicUnpinned => Self::TopicUnpinned,
            ForumModerationAction::ReplyHidden => Self::ReplyHidden,
            ForumModerationAction::ReplyRestored => Self::ReplyRestored,
        }
    }
}
impl From<DbForumNotificationKind> for ForumNotificationKind {
    fn from(_: DbForumNotificationKind) -> Self {
        Self::TopicReply
    }
}
