use std::sync::atomic::Ordering;

use scc::hash_map::Entry;

use super::visitor_service::{VISITOR_BOARD_MAX_ENTRIES, VisitorService, rejection, try_reserve};

impl VisitorService {
    pub async fn synchronize_board(&self) -> anyhow::Result<usize> {
        let limit = i64::try_from(VISITOR_BOARD_MAX_ENTRIES)?;
        let aggregates = self.repository.board_aggregates(limit).await?;
        self.board.clear_async().await;
        let mut cached = 0usize;
        for (latitude, longitude, count) in aggregates {
            let count = match u64::try_from(count) {
                Ok(count) => count,
                Err(error) => {
                    tracing::warn!(error = %error, count, "Skipped invalid visitor aggregate");
                    continue;
                }
            };
            let _ = self
                .board
                .insert_async((latitude.to_be_bytes(), longitude.to_be_bytes()), count)
                .await;
            cached = cached.saturating_add(1);
        }
        self.board_entries.store(cached, Ordering::SeqCst);
        Ok(cached)
    }

    pub(super) async fn increment_board(&self, latitude: f64, longitude: f64) {
        let key = (latitude.to_be_bytes(), longitude.to_be_bytes());
        match self.board.entry_async(key).await {
            Entry::Occupied(mut occupied) => {
                let count = occupied.get_mut();
                *count = count.saturating_add(1);
            }
            Entry::Vacant(vacant) => {
                if try_reserve(&self.board_entries, VISITOR_BOARD_MAX_ENTRIES) {
                    vacant.insert_entry(1);
                } else {
                    rejection(&self.board_rejections, "visitor_board");
                }
            }
        }
    }

    pub async fn board_entries(&self) -> Vec<((f64, f64), u64)> {
        let mut result = Vec::with_capacity(self.board_entries.load(Ordering::Relaxed));
        self.board
            .iter_async(|&(latitude, longitude), &count| {
                let latitude = f64::from_be_bytes(latitude);
                let longitude = f64::from_be_bytes(longitude);
                if latitude.is_finite() && longitude.is_finite() {
                    result.push(((latitude, longitude), count));
                }
                true
            })
            .await;
        result
    }
}
