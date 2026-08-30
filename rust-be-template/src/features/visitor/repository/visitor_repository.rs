//! Diesel persistence for visitor-board aggregates and buffered visits.

use diesel::{ExpressionMethods, QueryDsl, dsl::count_star};
use diesel_async::{AsyncPgConnection, RunQueryDsl, pooled_connection::bb8::Pool};

use crate::{features::visitor::domain::visit::NewVisit, schema::visitation_data};

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

pub struct VisitorRepository {
    pool: Pool<AsyncPgConnection>,
}

impl VisitorRepository {
    pub fn new(pool: Pool<AsyncPgConnection>) -> Self {
        Self { pool }
    }

    pub async fn board_aggregates(&self, limit: i64) -> anyhow::Result<Vec<(f64, f64, i64)>> {
        let mut connection = self.pool.get().await?;
        Ok(visitation_data::table
            .group_by((visitation_data::latitude, visitation_data::longitude))
            .select((visitation_data::latitude, visitation_data::longitude, count_star()))
            .order((
                count_star().desc(),
                visitation_data::latitude.asc(),
                visitation_data::longitude.asc(),
            ))
            .limit(limit)
            .load(&mut connection)
            .await?)
    }

    pub async fn insert_visits(&self, visits: Vec<NewVisit>) -> anyhow::Result<usize> {
        if visits.is_empty() {
            return Ok(0);
        }
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
        Ok(diesel::insert_into(visitation_data::table)
            .values(&records)
            .execute(&mut connection)
            .await?)
    }
}
