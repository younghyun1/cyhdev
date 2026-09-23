//! Keyset pagination values for threaded comment lists.
//!
//! Comments are paged oldest first by `(created_at, id)`. A reply is always
//! created after its parent, so a parent appears on the same or an earlier
//! page than its replies and a client that accumulates pages can rebuild the
//! whole thread. Blog posts and photographs share this contract.

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Comments returned with a detail read and by default on later pages.
pub const DEFAULT_COMMENT_PAGE_SIZE: u16 = 50;
/// Upper bound for one comment page.
pub const MAX_COMMENT_PAGE_SIZE: u16 = 100;

/// Position after the last comment of the previous page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommentCursor {
    pub created_at: DateTime<Utc>,
    pub comment_id: Uuid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum CommentPageError {
    /// Only one of the two cursor fields was supplied.
    #[error("comment cursor fields must be supplied together")]
    IncompleteCursor,
    /// The page size was zero or above [`MAX_COMMENT_PAGE_SIZE`].
    #[error("comment page size must be between 1 and 100")]
    InvalidLimit,
}

/// A validated request for one comment page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommentPageRequest {
    after: Option<CommentCursor>,
    limit: u16,
}

impl CommentPageRequest {
    pub const fn first_page() -> Self {
        Self {
            after: None,
            limit: DEFAULT_COMMENT_PAGE_SIZE,
        }
    }

    pub fn parse(
        after_created_at: Option<DateTime<Utc>>,
        after_comment_id: Option<Uuid>,
        limit: Option<u16>,
    ) -> Result<Self, CommentPageError> {
        let after = match (after_created_at, after_comment_id) {
            (Some(created_at), Some(comment_id)) => Some(CommentCursor {
                created_at,
                comment_id,
            }),
            (None, None) => None,
            _ => return Err(CommentPageError::IncompleteCursor),
        };
        let limit = limit.unwrap_or(DEFAULT_COMMENT_PAGE_SIZE);
        if !(1..=MAX_COMMENT_PAGE_SIZE).contains(&limit) {
            return Err(CommentPageError::InvalidLimit);
        }
        Ok(Self { after, limit })
    }

    pub const fn after(&self) -> Option<CommentCursor> {
        self.after
    }

    pub const fn limit(&self) -> u16 {
        self.limit
    }

    /// Rows to fetch: one extra row proves whether another page exists.
    pub fn fetch_rows(&self) -> i64 {
        i64::from(self.limit) + 1
    }

    /// Trims the probe row and returns the cursor for the following page.
    pub fn split_page<T>(
        &self,
        rows: &mut Vec<T>,
        cursor_of: impl Fn(&T) -> CommentCursor,
    ) -> Option<CommentCursor> {
        if rows.len() <= usize::from(self.limit) {
            return None;
        }
        rows.truncate(usize::from(self.limit));
        rows.last().map(cursor_of)
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use super::{CommentCursor, CommentPageError, CommentPageRequest, MAX_COMMENT_PAGE_SIZE};

    #[test]
    fn cursor_fields_are_both_or_neither() {
        assert!(matches!(
            CommentPageRequest::parse(Some(Utc::now()), None, None),
            Err(CommentPageError::IncompleteCursor)
        ));
        assert!(matches!(
            CommentPageRequest::parse(None, Some(Uuid::now_v7()), None),
            Err(CommentPageError::IncompleteCursor)
        ));
        assert_eq!(
            CommentPageRequest::parse(None, None, None),
            Ok(CommentPageRequest::first_page())
        );
    }

    #[test]
    fn page_size_is_bounded() {
        assert!(CommentPageRequest::parse(None, None, Some(0)).is_err());
        assert!(CommentPageRequest::parse(None, None, Some(MAX_COMMENT_PAGE_SIZE + 1)).is_err());
        assert!(CommentPageRequest::parse(None, None, Some(MAX_COMMENT_PAGE_SIZE)).is_ok());
    }

    #[test]
    fn split_page_trims_the_probe_row_and_points_at_the_last_kept_row() {
        let request = CommentPageRequest::parse(None, None, Some(2))
            .unwrap_or_else(|_| CommentPageRequest::first_page());
        let now = Utc::now();
        let cursor = |id: &u128| CommentCursor {
            created_at: now,
            comment_id: Uuid::from_u128(*id),
        };
        let mut full = vec![1_u128, 2, 3];
        assert_eq!(request.split_page(&mut full, cursor), Some(cursor(&2)));
        assert_eq!(full, vec![1, 2]);
        let mut last = vec![1_u128, 2];
        assert_eq!(request.split_page(&mut last, cursor), None);
        assert_eq!(last.len(), 2);
    }
}
