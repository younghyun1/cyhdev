//! Persistence-independent forum state values.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ForumContentState {
    Visible,
    Hidden,
    Deleted,
}

impl ForumContentState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Visible => "visible",
            Self::Hidden => "hidden",
            Self::Deleted => "deleted",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ForumTopicAccessState {
    Open,
    Locked,
}

impl ForumTopicAccessState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Locked => "locked",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ForumModerationAction {
    TopicHidden,
    TopicRestored,
    TopicLocked,
    TopicUnlocked,
    TopicPinned,
    TopicUnpinned,
    ReplyHidden,
    ReplyRestored,
}

impl ForumModerationAction {
    pub const fn as_str(self) -> &'static str {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ForumNotificationKind {
    TopicReply,
}

impl ForumNotificationKind {
    pub const fn as_str(self) -> &'static str {
        "topic_reply"
    }
}
