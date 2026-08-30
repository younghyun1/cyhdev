use std::{collections::HashMap, net::IpAddr, sync::atomic::Ordering};

use scc::hash_map::Entry;

use crate::features::visitor::domain::visit::{NewVisit, VisitorLogBatch, VisitorLogKey};

use super::visitor_service::{
    VISITOR_LOG_BUFFER_MAX_ENTRIES, VISITOR_LOG_BUFFER_MAX_EVENTS, VisitorService, rejection,
    try_reserve,
};

impl VisitorService {
    pub async fn enqueue(&self, ip: Option<IpAddr>) {
        let ip = match ip {
            Some(ip) => ip,
            None => return,
        };
        let info = match self.geo.lookup(ip) {
            Some(info) if info.latitude != 0.0 || info.longitude != 0.0 => info,
            Some(_) | None => return,
        };
        self.increment_board(info.latitude, info.longitude).await;
        if !try_reserve(&self.pending_events, VISITOR_LOG_BUFFER_MAX_EVENTS) {
            rejection(&self.buffer_rejections, "visitor_log_buffer");
            return;
        }
        let key = VisitorLogKey {
            latitude_bytes: info.latitude.to_be_bytes(),
            longitude_bytes: info.longitude.to_be_bytes(),
            ip_address: ip,
            city: info.city,
            country: info.country_name,
        };
        match self.buffer.entry_async(key).await {
            Entry::Occupied(mut occupied) => {
                let batch = occupied.get_mut();
                batch.count = batch.count.saturating_add(1);
                batch.visited_at = chrono::Utc::now();
            }
            Entry::Vacant(vacant) => {
                if try_reserve(&self.buffer_entries, VISITOR_LOG_BUFFER_MAX_ENTRIES) {
                    vacant.insert_entry(VisitorLogBatch { count: 1, visited_at: chrono::Utc::now() });
                } else {
                    self.pending_events.fetch_sub(1, Ordering::SeqCst);
                    rejection(&self.buffer_rejections, "visitor_log_buffer");
                }
            }
        }
    }

    pub async fn flush(&self) -> anyhow::Result<u64> {
        let _flush = self.flush_gate.lock().await;
        let mut pending = HashMap::<VisitorLogKey, VisitorLogBatch>::new();
        self.buffer
            .retain_async(|key, batch| {
                pending.insert(key.clone(), batch.clone());
                false
            })
            .await;
        if pending.is_empty() {
            return Ok(0);
        }
        let drained_entries = pending.len();
        let drained_events = pending.values().fold(0usize, |total, batch| {
            total.saturating_add(usize::try_from(batch.count).unwrap_or(usize::MAX))
        });
        let mut visits = Vec::with_capacity(drained_events.min(VISITOR_LOG_BUFFER_MAX_EVENTS));
        for (key, batch) in &pending {
            let latitude = f64::from_be_bytes(key.latitude_bytes);
            let longitude = f64::from_be_bytes(key.longitude_bytes);
            if !latitude.is_finite() || !longitude.is_finite() {
                continue;
            }
            for _ in 0..batch.count {
                visits.push(NewVisit {
                    latitude,
                    longitude,
                    ip_address: key.ip_address,
                    city: key.city.clone(),
                    country: key.country.clone(),
                    visited_at: batch.visited_at,
                });
            }
        }
        let inserted = match self.repository.insert_visits(visits).await {
            Ok(inserted) => inserted,
            Err(error) => {
                self.requeue_losslessly(pending).await;
                return Err(error);
            }
        };
        self.buffer_entries.fetch_sub(drained_entries, Ordering::SeqCst);
        self.pending_events.fetch_sub(drained_events, Ordering::SeqCst);
        tracing::info!(
            rows_flushed = inserted,
            visit_count = drained_events,
            rejected_admissions = self.buffer_rejections.load(Ordering::Relaxed),
            "Flushed buffered visitor logs"
        );
        Ok(u64::try_from(drained_events).unwrap_or(u64::MAX))
    }

    async fn requeue_losslessly(&self, pending: HashMap<VisitorLogKey, VisitorLogBatch>) {
        for (key, batch) in pending {
            match self.buffer.entry_async(key).await {
                Entry::Occupied(mut occupied) => {
                    let existing = occupied.get_mut();
                    existing.count = existing.count.saturating_add(batch.count);
                    existing.visited_at = existing.visited_at.max(batch.visited_at);
                    self.buffer_entries.fetch_sub(1, Ordering::SeqCst);
                }
                Entry::Vacant(vacant) => {
                    vacant.insert_entry(batch);
                }
            }
        }
    }
}
