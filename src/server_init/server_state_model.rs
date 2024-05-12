use std::sync::Arc;

use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use tokio::sync::RwLock;

use crate::handlers::healthcheck_handlers::quotes::QUOTE_NUMBER;

pub struct ServerState {
    pub pool: Arc<Pool>,
    pub server_start_time: DateTime<Utc>,
    pub quote_shuffle_bag: Arc<RwLock<ShuffleBag>>,
}

pub struct ShuffleBag {
    pub count: usize,
    pub shuffle_bag: Vec<u8>,
}

impl ShuffleBag {
    pub fn new() -> Arc<RwLock<ShuffleBag>> {
        let mut bag: Vec<u8> = Vec::with_capacity(QUOTE_NUMBER);
        // If you want to initialize the vector with a sequence of numbers:
        for i in 0..QUOTE_NUMBER {
            bag.push(i as u8);
        }

        bag.shuffle(&mut thread_rng());
        return Arc::new(RwLock::new(ShuffleBag {
            count: 0usize,
            shuffle_bag: bag,
        }));
    }
    pub fn shuffle_bag(&mut self) {
        self.shuffle_bag.shuffle(&mut thread_rng());
    }
}
