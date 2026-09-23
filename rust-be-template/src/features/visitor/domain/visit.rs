//! Visitor aggregation and buffered persistence values.

use std::{collections::BTreeMap, net::IpAddr};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct VisitorLogKey {
    pub latitude_bytes: [u8; 8],
    pub longitude_bytes: [u8; 8],
    pub ip_address: IpAddr,
    pub city: String,
    pub country: String,
}

#[derive(Clone)]
pub struct VisitorLogBatch {
    pub count: u64,
    pub visited_at: chrono::DateTime<chrono::Utc>,
}

pub struct NewVisit {
    pub latitude: f64,
    pub longitude: f64,
    pub ip_address: IpAddr,
    pub city: String,
    pub country: String,
    pub visited_at: chrono::DateTime<chrono::Utc>,
}

/// Visits to add to one board location.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoardIncrement {
    pub latitude: f64,
    pub longitude: f64,
    pub visits: i64,
}

/// Whether a coordinate pair can appear on the board; mirrors the table's range checks.
///
/// Out-of-range pairs are still recorded as visits, but a single one on the board would
/// fail the whole flush transaction against the `CHECK` constraints.
pub fn is_board_coordinate(latitude: f64, longitude: f64) -> bool {
    (-90.0..=90.0).contains(&latitude) && (-180.0..=180.0).contains(&longitude)
}

/// Sums visits per board location in a stable order.
///
/// PostgreSQL compares `-0.0` equal to `0.0`, so both map to one key here; two rows
/// with the same conflict key in one `INSERT ... ON CONFLICT DO UPDATE` would abort
/// the statement. The stable order also keeps row-lock acquisition consistent.
pub fn board_increments(visits: &[NewVisit]) -> Vec<BoardIncrement> {
    let mut totals = BTreeMap::<(u64, u64), BoardIncrement>::new();
    for visit in visits {
        if !is_board_coordinate(visit.latitude, visit.longitude) {
            continue;
        }
        // Adding positive zero turns -0.0 into 0.0 and leaves every other value unchanged.
        let (latitude, longitude) = (visit.latitude + 0.0, visit.longitude + 0.0);
        totals
            .entry((latitude.to_bits(), longitude.to_bits()))
            .and_modify(|increment| increment.visits = increment.visits.saturating_add(1))
            .or_insert(BoardIncrement {
                latitude,
                longitude,
                visits: 1,
            });
    }
    totals.into_values().collect()
}

#[cfg(test)]
mod tests {
    use std::net::IpAddr;

    use super::{BoardIncrement, NewVisit, board_increments, is_board_coordinate};

    fn visit(latitude: f64, longitude: f64) -> NewVisit {
        NewVisit {
            latitude,
            longitude,
            ip_address: IpAddr::from([192, 0, 2, 1]),
            city: "Denver".to_owned(),
            country: "United States".to_owned(),
            visited_at: chrono::DateTime::<chrono::Utc>::UNIX_EPOCH,
        }
    }

    #[test]
    fn increments_sum_duplicates_and_fold_negative_zero() {
        let increments = board_increments(&[
            visit(39.7, -104.9),
            visit(-0.0, 10.0),
            visit(39.7, -104.9),
            visit(0.0, 10.0),
            visit(39.7, -104.9),
        ]);
        assert_eq!(increments.len(), 2);
        assert!(increments.contains(&BoardIncrement {
            latitude: 39.7,
            longitude: -104.9,
            visits: 3
        }));
        let zero = increments
            .iter()
            .find(|increment| increment.longitude == 10.0);
        assert_eq!(zero.map(|increment| increment.visits), Some(2));
        assert_eq!(
            zero.map(|increment| increment.latitude.is_sign_positive()),
            Some(true)
        );
    }

    #[test]
    fn invalid_coordinates_never_reach_the_board() {
        assert!(!is_board_coordinate(f64::NAN, 0.0));
        assert!(!is_board_coordinate(0.0, f64::INFINITY));
        assert!(!is_board_coordinate(90.5, 0.0));
        assert!(!is_board_coordinate(0.0, -180.5));
        assert!(is_board_coordinate(-90.0, 180.0));
        assert!(board_increments(&[visit(95.0, 0.0), visit(f64::NAN, 1.0)]).is_empty());
    }
}
