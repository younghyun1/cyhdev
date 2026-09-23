//! Diesel persistence for visitor-board aggregates and buffered visits.

use diesel::{ExpressionMethods, QueryDsl, upsert::excluded};
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl, pooled_connection::bb8::Pool};

use crate::{
    features::visitor::domain::visit::{NewVisit, board_increments},
    schema::{visitation_data, visitor_board_locations},
};

/// Rows per multi-row INSERT, keeping visit inserts (six binds per row) under
/// PostgreSQL's 65,535 bind-parameter limit whatever the buffer bound becomes.
const MAX_ROWS_PER_INSERT: usize = 8_192;

#[derive(diesel::Insertable)]
#[diesel(table_name = visitation_data)]
struct NewVisitRecord {
    latitude: f64,
    longitude: f64,
    ip_address: ipnet::IpNet,
    city: String,
    country: String,
    visited_at: chrono::DateTime<chrono::Utc>,
}

#[derive(diesel::Insertable)]
#[diesel(table_name = visitor_board_locations)]
struct BoardLocationRecord {
    visitor_board_location_latitude: f64,
    visitor_board_location_longitude: f64,
    visitor_board_location_visit_count: i64,
}

pub struct VisitorRepository {
    pool: Pool<AsyncPgConnection>,
}

impl VisitorRepository {
    pub fn new(pool: Pool<AsyncPgConnection>) -> Self {
        Self { pool }
    }

    /// Reads the most-visited locations from the maintained board table.
    pub async fn board_aggregates(&self, limit: i64) -> anyhow::Result<Vec<(f64, f64, i64)>> {
        let mut connection = self.pool.get().await?;
        Ok(visitor_board_locations::table
            .select((
                visitor_board_locations::visitor_board_location_latitude,
                visitor_board_locations::visitor_board_location_longitude,
                visitor_board_locations::visitor_board_location_visit_count,
            ))
            .order((
                visitor_board_locations::visitor_board_location_visit_count.desc(),
                visitor_board_locations::visitor_board_location_latitude.asc(),
                visitor_board_locations::visitor_board_location_longitude.asc(),
            ))
            .limit(limit)
            .load(&mut connection)
            .await?)
    }

    /// Inserts one raw row per visit and adds the same visits to the board counts.
    ///
    /// Both writes share one transaction, so the board never counts a visit whose row
    /// was not stored, and a failed flush leaves neither table changed for the retry.
    pub async fn insert_visits(&self, visits: Vec<NewVisit>) -> anyhow::Result<usize> {
        if visits.is_empty() {
            return Ok(0);
        }
        let increments = board_increments(&visits)
            .into_iter()
            .map(|increment| BoardLocationRecord {
                visitor_board_location_latitude: increment.latitude,
                visitor_board_location_longitude: increment.longitude,
                visitor_board_location_visit_count: increment.visits,
            })
            .collect::<Vec<_>>();
        let records = visits
            .into_iter()
            .map(|visit| NewVisitRecord {
                latitude: visit.latitude,
                longitude: visit.longitude,
                ip_address: ipnet::IpNet::from(visit.ip_address),
                city: visit.city,
                country: visit.country,
                visited_at: visit.visited_at,
            })
            .collect::<Vec<_>>();
        let mut connection = self.pool.get().await?;
        let inserted = connection
            .transaction::<usize, diesel::result::Error, _>(async move |connection| {
                let mut inserted = 0;
                for chunk in records.chunks(MAX_ROWS_PER_INSERT) {
                    inserted += diesel::insert_into(visitation_data::table)
                        .values(chunk)
                        .execute(&mut *connection)
                        .await?;
                }
                for chunk in increments.chunks(MAX_ROWS_PER_INSERT) {
                    diesel::insert_into(visitor_board_locations::table)
                        .values(chunk)
                        .on_conflict((
                            visitor_board_locations::visitor_board_location_latitude,
                            visitor_board_locations::visitor_board_location_longitude,
                        ))
                        .do_update()
                        .set(
                            visitor_board_locations::visitor_board_location_visit_count.eq(
                                visitor_board_locations::visitor_board_location_visit_count
                                    + excluded(
                                        visitor_board_locations::visitor_board_location_visit_count,
                                    ),
                            ),
                        )
                        .execute(&mut *connection)
                        .await?;
                }
                Ok(inserted)
            })
            .await?;
        Ok(inserted)
    }
}
