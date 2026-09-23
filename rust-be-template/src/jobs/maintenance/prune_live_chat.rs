use std::sync::Arc;

use chrono::Utc;

use crate::init::state::ServerState;

/// Periodic prune of per-actor live-chat in-memory state.
///
/// `message_rate_by_key` and `typing_by_actor` otherwise grow one entry per
/// distinct guest IP / user for the process lifetime (an unbounded runtime cache).
/// The rate window is 1s wide and typing entries carry their own expiry, so any
/// entry older than the sweep interval holds no live signal and is recreated on
/// demand. Running this once per minute bounds both maps to recently-active actors.
pub async fn prune_live_chat_state(state: Arc<ServerState>) {
    let now = Utc::now();
    state.live_chat_service().prune_runtime(now).await;
    // Drop empty SFU rooms and close their dangling call rows, bounding the
    // `rtc_rooms` registry to rooms with live participants.
    state.rtc_service().prune_empty_rooms().await;
}

/// Every-second sweep of rate windows older than two seconds.
///
/// The rate table is capped at 16,384 keys and rejects new senders while full;
/// sweeping every second keeps that capacity for senders active right now
/// instead of anyone seen in the last minute.
pub async fn prune_live_chat_rate_windows(state: Arc<ServerState>) {
    state
        .live_chat_service()
        .prune_stale_rate_windows(Utc::now())
        .await;
}
