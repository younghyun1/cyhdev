//! Call-row reconciliation for process start and graceful shutdown.

use tracing::{error, info};

use super::{super::error::LiveChatError, live_chat_service::LiveChatService};

impl LiveChatService {
    /// Close every call and participant row without an end time.
    ///
    /// Call rows are only closed when a room empties, so a crash or restart
    /// leaves them open forever. Startup synchronization calls this because
    /// no room outlives the process; a graceful-shutdown hook should call it
    /// after connections stop so the final rows record the shutdown time.
    /// Idempotent: rows already closed are untouched.
    pub async fn close_open_calls(&self) -> Result<(usize, usize), LiveChatError> {
        match self.rtc.close_open_calls().await {
            Ok((calls_closed, participants_closed)) => {
                info!(
                    calls_closed,
                    participants_closed, "Closed open live chat call rows"
                );
                Ok((calls_closed, participants_closed))
            }
            Err(error_value) => {
                error!(error = %error_value, "Failed to close open live chat call rows");
                Err(error_value)
            }
        }
    }
}
