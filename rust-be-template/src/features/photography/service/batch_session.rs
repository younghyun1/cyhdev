//! Bounded process-owned state for one photograph batch.

use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering};

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::features::photography::domain::batch::{BatchItem, ProcessingStatus};

pub struct BatchSession {
    pub batch_id: Uuid,
    pub owner: Uuid,
    pub created_at: DateTime<Utc>,
    pub total: usize,
    items: scc::HashMap<Uuid, BatchItem>,
    completed: AtomicUsize,
    failed: AtomicUsize,
    last_activity: AtomicI64,
}

impl BatchSession {
    pub fn new(batch_id: Uuid, owner: Uuid, total: usize, now: DateTime<Utc>) -> Self {
        Self {
            batch_id,
            owner,
            created_at: now,
            total,
            items: scc::HashMap::new(),
            completed: AtomicUsize::new(0),
            failed: AtomicUsize::new(0),
            last_activity: AtomicI64::new(now.timestamp()),
        }
    }
    fn touch(&self, now: DateTime<Utc>) {
        self.last_activity.store(now.timestamp(), Ordering::SeqCst);
    }
    pub async fn register_item(&self, item: BatchItem) {
        let _ = self.items.insert_async(item.item_id, item).await;
    }
    pub async fn set_status(&self, item_id: Uuid, status: ProcessingStatus, now: DateTime<Utc>) {
        let _ = self
            .items
            .update_async(&item_id, |_, item| {
                if !item.status.is_terminal() {
                    item.status = status.clone();
                    item.updated_at = now;
                }
            })
            .await;
        self.touch(now);
    }
    pub async fn complete_item(
        &self,
        item_id: Uuid,
        photograph_id: Uuid,
        photograph_link: String,
        thumbnail_link: String,
        now: DateTime<Utc>,
    ) {
        let transitioned = self
            .items
            .update_async(&item_id, |_, item| {
                if item.status.is_terminal() {
                    false
                } else {
                    item.status = ProcessingStatus::Completed {
                        photograph_id,
                        photograph_link: photograph_link.clone(),
                        thumbnail_link: thumbnail_link.clone(),
                    };
                    item.updated_at = now;
                    true
                }
            })
            .await;
        if matches!(transitioned, Some(true)) {
            self.completed.fetch_add(1, Ordering::SeqCst);
        }
        self.touch(now);
    }
    pub async fn fail_item(&self, item_id: Uuid, reason: String, now: DateTime<Utc>) {
        let transitioned = self
            .items
            .update_async(&item_id, |_, item| {
                if item.status.is_terminal() {
                    false
                } else {
                    item.status = ProcessingStatus::Failed {
                        reason: reason.clone(),
                    };
                    item.updated_at = now;
                    true
                }
            })
            .await;
        if matches!(transitioned, Some(true)) {
            self.failed.fetch_add(1, Ordering::SeqCst);
        }
        self.touch(now);
    }
    pub fn completed_count(&self) -> usize {
        self.completed.load(Ordering::SeqCst)
    }
    pub fn failed_count(&self) -> usize {
        self.failed.load(Ordering::SeqCst)
    }
    pub fn pending_count(&self) -> usize {
        self.total
            .saturating_sub(self.completed_count() + self.failed_count())
    }
    pub fn is_done(&self) -> bool {
        self.completed_count() + self.failed_count() >= self.total
    }
    pub fn last_activity_unix(&self) -> i64 {
        self.last_activity.load(Ordering::SeqCst)
    }
    pub async fn snapshot_items(&self) -> Vec<BatchItem> {
        let mut items = Vec::with_capacity(self.total);
        self.items
            .iter_async(|_, item| {
                items.push(item.clone());
                true
            })
            .await;
        items
    }
}
