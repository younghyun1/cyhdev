//! Shared bounded-import counters.

use crate::database::store::Upserted;

#[derive(Clone, Copy, Debug, Default)]
pub struct ImportStats {
    /// Documents present in the database after the import.
    pub stored: usize,
    /// Of those, the ones this import wrote because they were new or changed.
    pub changed: usize,
    pub failed: usize,
}

impl ImportStats {
    /// Adds the outcome of one stored batch.
    pub fn record(&mut self, batch: Upserted) {
        self.stored = self.stored.saturating_add(batch.stored);
        self.changed = self.changed.saturating_add(batch.changed);
    }
}

impl std::ops::Add for ImportStats {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            stored: self.stored.saturating_add(rhs.stored),
            changed: self.changed.saturating_add(rhs.changed),
            failed: self.failed.saturating_add(rhs.failed),
        }
    }
}
