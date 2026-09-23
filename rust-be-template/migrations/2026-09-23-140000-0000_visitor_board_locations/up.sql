-- Per-location visit counts for the public visitor board.
--
-- visitation_data remains the authoritative record: one retained row per visit.
-- This table is a derived projection so startup no longer groups the entire visit
-- history. The application increments it in the same transaction that inserts the
-- visitation_data rows, so counts stay exact. If they ever diverge, rebuild by
-- truncating this table and rerunning the INSERT ... SELECT below.
CREATE TABLE visitor_board_locations (
    visitor_board_location_latitude DOUBLE PRECISION NOT NULL,
    visitor_board_location_longitude DOUBLE PRECISION NOT NULL,
    visitor_board_location_visit_count BIGINT NOT NULL,
    CONSTRAINT visitor_board_locations_pkey
        PRIMARY KEY (visitor_board_location_latitude, visitor_board_location_longitude),
    CONSTRAINT visitor_board_locations_visit_count_positive
        CHECK (visitor_board_location_visit_count > 0),
    -- BETWEEN is false for NaN, so this also rejects non-finite coordinates.
    CONSTRAINT visitor_board_locations_latitude_range
        CHECK (visitor_board_location_latitude BETWEEN -90 AND 90),
    CONSTRAINT visitor_board_locations_longitude_range
        CHECK (visitor_board_location_longitude BETWEEN -180 AND 180)
);

COMMENT ON TABLE visitor_board_locations IS
    'Derived visit counts per coordinate pair; authoritative rows are in visitation_data.';

-- The board reads the most-visited locations once at startup. The table holds one
-- row per distinct Geo-IP location, so that sort needs no index, and leaving the
-- count unindexed keeps flush upserts eligible for heap-only tuple updates.
INSERT INTO visitor_board_locations (
    visitor_board_location_latitude,
    visitor_board_location_longitude,
    visitor_board_location_visit_count
)
SELECT latitude, longitude, count(*)
FROM visitation_data
WHERE latitude BETWEEN -90 AND 90
  AND longitude BETWEEN -180 AND 180
GROUP BY latitude, longitude;
