use jiff::Timestamp;
use serde_derive::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ResponseMeta<T: serde::Serialize> {
    time_to_process: String,
    timestamp: Timestamp,
    metadata: T,
}

impl<T: serde::Serialize> ResponseMeta<T> {
    pub fn get_metadata(self) -> T {
        self.metadata
    }
}

impl<T: serde::Serialize> ResponseMeta<T> {
    pub fn from(start: tokio::time::Instant, metadata: T) -> Self {
        ResponseMeta {
            time_to_process: format!("{:?}", start.elapsed()),
            timestamp: Timestamp::now(),
            metadata,
        }
    }

    pub fn redacted(metadata: T) -> Self {
        ResponseMeta {
            time_to_process: "redacted".to_owned(),
            timestamp: Timestamp::now(),
            metadata,
        }
    }
}
