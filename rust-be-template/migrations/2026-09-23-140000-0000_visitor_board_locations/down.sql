-- The table only holds counts derived from visitation_data, which this rollback keeps;
-- reapplying the up migration rebuilds it exactly, so no rollback guard is needed.
DROP TABLE visitor_board_locations;
