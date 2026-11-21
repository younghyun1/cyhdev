//! Process shutdown signals shared by the collectors and the web server.

use tokio::signal::{
    self,
    unix::{SignalKind, signal as unix_signal},
};
use tracing::error;

/// Resolves on the first interrupt or termination signal.
///
/// A handler that cannot be installed is logged and treated as never firing,
/// so the caller keeps running instead of shutting down spuriously.
pub async fn signal() {
    let interrupt = async {
        if let Err(error) = signal::ctrl_c().await {
            error!(error = %error, "Failed to install interrupt handler");
            std::future::pending::<()>().await;
        }
    };
    let terminate = async {
        match unix_signal(SignalKind::terminate()) {
            Ok(mut stream) => {
                stream.recv().await;
            }
            Err(error) => {
                error!(error = %error, "Failed to install termination handler");
                std::future::pending::<()>().await;
            }
        }
    };
    tokio::select! {
        () = interrupt => {}
        () = terminate => {}
    }
}
