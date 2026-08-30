use serde::Serialize;
use utoipa::ToSchema;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, ToSchema)]
pub enum VoteState {
    Upvoted,
    Downvoted,
    DidNotVote,
}

impl Serialize for VoteState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(match self {
            Self::Upvoted => 0,
            Self::Downvoted => 1,
            Self::DidNotVote => 2,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct VoteCounts {
    pub upvotes: i64,
    pub downvotes: i64,
}
