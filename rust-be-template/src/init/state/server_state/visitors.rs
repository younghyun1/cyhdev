use std::collections::HashMap as StdHashMap;
use std::net::IpAddr;
use std::sync::atomic::Ordering;
use diesel::{ExpressionMethods, QueryDsl, dsl::count_star};
use diesel_async::RunQueryDsl;
use scc::hash_map::Entry;
use tracing::{info, warn};
use super::{ServerState, VisitorLogBatch, VisitorLogKey};
use super::visitor_cache_policy::{
    VISITOR_BOARD_MAX_ENTRIES, VISITOR_LOG_BUFFER_MAX_ENTRIES,
    VISITOR_LOG_BUFFER_MAX_EVENTS, record_rejection, reserve_up_to, try_reserve,
};
use crate::domain::geo::visitation_data::NewVisitationData;
use crate::util::time::now::tokio_now;
impl ServerState {
    pub async fn sync_visitor_board_data(&self) -> anyhow::Result<usize> {
        use crate::schema::visitation_data::dsl as vdsl;

        let start = tokio_now();
        let mut conn = self.get_conn().await?;
        let row_limit = i64::try_from(VISITOR_BOARD_MAX_ENTRIES)?;
        let visits: Vec<(f64, f64, i64)> = vdsl::visitation_data
            .group_by((vdsl::latitude, vdsl::longitude))
            .select((vdsl::latitude, vdsl::longitude, count_star()))
            .order((count_star().desc(), vdsl::latitude.asc(), vdsl::longitude.asc()))
            .limit(row_limit)
            .load::<(f64, f64, i64)>(&mut conn)
            .await?;
        drop(conn);

        self.visitor_board_map.clear_async().await;
        let mut cached = 0usize;
        for (latitude, longitude, count) in visits {
            let key = (latitude.to_be_bytes(), longitude.to_be_bytes());
            let count = match u64::try_from(count) {
                Ok(count) => count,
                Err(e) => {
                    warn!(error = %e, count, "Skipped invalid visitor-board aggregate");
                    continue;
                }
            };
            let _ = self.visitor_board_map.insert_async(key, count).await;
            cached = cached.saturating_add(1);
        }
        self.visitor_board_entry_count
            .store(cached, Ordering::SeqCst);

        info!(
            elapsed = ?start.elapsed(),
            entries_synchronized = cached,
            max_entries = VISITOR_BOARD_MAX_ENTRIES,
            rejected_entries = self.visitor_board_rejected_entries.load(Ordering::Relaxed),
            "Synchronized visitor board data."
        );
        Ok(cached)
    }

    pub async fn enqueue_visitor_log(&self, inp_ip: Option<IpAddr>) {
        let ip = match inp_ip {
            Some(ip) => ip,
            None => return,
        };

        let ip_info = match self.lookup_ip_location(ip) {
            Some(info) => info,
            None => {
                warn!(ip = %ip, "Failed to look up IP location for visitor log");
                return;
            }
        };

        let city_lat = ip_info.latitude;
        let city_lon = ip_info.longitude;

        if city_lat == 0.0 && city_lon == 0.0 {
            return;
        }

        let latitude_bytes = city_lat.to_be_bytes();
        let longitude_bytes = city_lon.to_be_bytes();
        let board_key = (latitude_bytes, longitude_bytes);

        match self.visitor_board_map.entry_async(board_key).await {
            Entry::Occupied(mut occ) => {
                *occ.get_mut() += 1;
            }
            Entry::Vacant(vac) => {
                if try_reserve(
                    &self.visitor_board_entry_count,
                    VISITOR_BOARD_MAX_ENTRIES,
                ) {
                    vac.insert_entry(1);
                } else {
                    record_rejection(&self.visitor_board_rejected_entries, "visitor_board");
                }
            }
        }

        let key = VisitorLogKey {
            latitude_bytes,
            longitude_bytes,
            ip_address: ip,
            city: ip_info.city,
            country: ip_info.country_name,
        };

        if !try_reserve(
            &self.visitor_log_pending_events,
            VISITOR_LOG_BUFFER_MAX_EVENTS,
        ) {
            record_rejection(
                &self.visitor_log_rejected_admissions,
                "visitor_log_buffer",
            );
            return;
        }

        match self.visitor_log_buffer.entry_async(key).await {
            Entry::Occupied(mut occ) => {
                let batch = occ.get_mut();
                batch.count = batch.count.saturating_add(1);
                batch.visited_at = chrono::Utc::now();
            }
            Entry::Vacant(vac) => {
                if try_reserve(
                    &self.visitor_log_entry_count,
                    VISITOR_LOG_BUFFER_MAX_ENTRIES,
                ) {
                    vac.insert_entry(VisitorLogBatch {
                        count: 1,
                        visited_at: chrono::Utc::now(),
                    });
                } else {
                    self.visitor_log_pending_events
                        .fetch_sub(1, Ordering::SeqCst);
                    record_rejection(
                        &self.visitor_log_rejected_admissions,
                        "visitor_log_buffer",
                    );
                }
            }
        }
    }

    pub async fn flush_visitor_logs(&self) -> anyhow::Result<u64> {
        // Drain the per-actor buckets into a local map. retain_async returning false
        // removes each visited entry, so this atomically empties the buffer bucket by
        // bucket. A concurrent enqueue during the drain may land in the next window;
        // counts are approximate and requeue merges on DB failure, so that is fine.
        let mut pending: StdHashMap<VisitorLogKey, VisitorLogBatch> = StdHashMap::new();
        self.visitor_log_buffer
            .retain_async(|key, batch| {
                pending.insert(key.clone(), batch.clone());
                false
            })
            .await;
        let drained_entries = pending.len();
        let drained_events = pending.values().fold(0usize, |total, batch| {
            let count = match usize::try_from(batch.count) {
                Ok(count) => count,
                Err(_) => usize::MAX,
            };
            total.saturating_add(count)
        });
        self.visitor_log_entry_count
            .fetch_sub(drained_entries, Ordering::SeqCst);
        self.visitor_log_pending_events
            .fetch_sub(drained_events, Ordering::SeqCst);
        if pending.is_empty() {
            return Ok(0);
        }

        let mut conn = match self.get_conn().await {
            Ok(conn) => conn,
            Err(e) => {
                self.requeue_visitor_logs(pending).await;
                return Err(e);
            }
        };

        let mut rows: Vec<NewVisitationData> = Vec::new();
        let mut total_pending = 0u64;
        for (key, batch) in &pending {
            let latitude = f64::from_be_bytes(key.latitude_bytes);
            let longitude = f64::from_be_bytes(key.longitude_bytes);
            if latitude.is_nan() || longitude.is_nan() {
                continue;
            }

            for _ in 0..batch.count {
                rows.push(NewVisitationData {
                    latitude,
                    longitude,
                    ip_address: ipnet::IpNet::from(key.ip_address),
                    city: key.city.clone(),
                    country: key.country.clone(),
                    visited_at: batch.visited_at,
                });
                total_pending = total_pending.saturating_add(1);
            }
        }

        if rows.is_empty() {
            return Ok(0);
        }

        let insert_result = diesel::insert_into(crate::schema::visitation_data::table)
            .values(&rows)
            .execute(&mut conn)
            .await;

        match insert_result {
            Ok(inserted_rows) => {
                info!(
                    rows_flushed = inserted_rows,
                    visit_count = total_pending,
                    rejected_admissions = self
                        .visitor_log_rejected_admissions
                        .load(Ordering::Relaxed),
                    "Flushed buffered visitor logs"
                );
                Ok(total_pending)
            }
            Err(e) => {
                self.requeue_visitor_logs(pending).await;
                Err(e.into())
            }
        }
    }

    async fn requeue_visitor_logs(&self, pending: StdHashMap<VisitorLogKey, VisitorLogBatch>) {
        for (key, mut batch) in pending {
            let requested = match usize::try_from(batch.count) {
                Ok(requested) => requested,
                Err(_) => usize::MAX,
            };
            let admitted = reserve_up_to(
                &self.visitor_log_pending_events,
                requested,
                VISITOR_LOG_BUFFER_MAX_EVENTS,
            );
            if admitted == 0 {
                record_rejection(
                    &self.visitor_log_rejected_admissions,
                    "visitor_log_buffer",
                );
                continue;
            }
            batch.count = match u64::try_from(admitted) {
                Ok(count) => count,
                Err(e) => {
                    self.visitor_log_pending_events
                        .fetch_sub(admitted, Ordering::SeqCst);
                    warn!(error = %e, admitted, "Could not requeue visitor-log batch");
                    continue;
                }
            };
            match self.visitor_log_buffer.entry_async(key).await {
                Entry::Occupied(mut occ) => {
                    let existing = occ.get_mut();
                    existing.count = existing.count.saturating_add(batch.count);
                    if batch.visited_at > existing.visited_at {
                        existing.visited_at = batch.visited_at;
                    }
                }
                Entry::Vacant(vac) => {
                    if try_reserve(
                        &self.visitor_log_entry_count,
                        VISITOR_LOG_BUFFER_MAX_ENTRIES,
                    ) {
                        vac.insert_entry(batch);
                    } else {
                        self.visitor_log_pending_events
                            .fetch_sub(admitted, Ordering::SeqCst);
                        record_rejection(
                            &self.visitor_log_rejected_admissions,
                            "visitor_log_buffer",
                        );
                    }
                }
            }
        }
    }

    pub async fn get_visitor_board_entries(&self) -> Vec<((f64, f64), u64)> {
        let mut result = Vec::new();
        self.visitor_board_map
            .iter_async(|&(lat_bytes, long_bytes), &count| {
                let lat = f64::from_be_bytes(lat_bytes);
                let long = f64::from_be_bytes(long_bytes);
                if !lat.is_nan() && !long.is_nan() {
                    result.push(((lat, long), count));
                }
                true
            })
            .await;
        result
    }
}
