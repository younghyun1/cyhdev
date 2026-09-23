//! PostgreSQL evidence for the maintained visitor board and pooled session time bounds.

mod support;

use std::net::IpAddr;

use diesel::{Connection, QueryableByName, pg::PgConnection, sql_query};
use diesel_async::RunQueryDsl;

use rust_be_template::{
    features::visitor::{
        domain::visit::NewVisit, repository::visitor_repository::VisitorRepository,
    },
    init::{
        db_migrations::run_pending_migrations,
        db_pool::{DbPoolSettings, build_pool, with_session_options},
    },
};

use support::{
    database::{DatabaseTestFuture, TestDatabase, TestResult, require, run_database_test},
    migrations::rewind_to_migration,
};

/// The migration immediately before the visitor board projection.
const BEFORE_VISITOR_BOARD: &str = "202608300610000000";

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit safe TEST_DATABASE_URL and PostgreSQL 18"]
async fn visitor_board_backfill_and_flush_keep_exact_counts() -> TestResult {
    run_database_test(visitor_board_case).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit safe TEST_DATABASE_URL and PostgreSQL 18"]
async fn pooled_sessions_carry_time_bounds() -> TestResult {
    run_database_test(session_bounds_case).await
}

fn visit(latitude: f64, longitude: f64) -> NewVisit {
    NewVisit {
        latitude,
        longitude,
        ip_address: IpAddr::from([192, 0, 2, 44]),
        city: "Denver".to_owned(),
        country: "United States".to_owned(),
        visited_at: chrono::Utc::now(),
    }
}

fn count_at(board: &[(f64, f64, i64)], latitude: f64, longitude: f64) -> Option<i64> {
    board
        .iter()
        .find(|(lat, lon, _)| *lat == latitude && *lon == longitude)
        .map(|(_, _, count)| *count)
}

fn visitor_board_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        // Seed history under the previous schema so the migration backfill is exercised.
        let database_url = database.database_url().to_owned();
        let seed_url = database_url.clone();
        tokio::task::spawn_blocking(move || -> TestResult {
            let mut connection = PgConnection::establish(&seed_url)?;
            rewind_to_migration(&mut connection, BEFORE_VISITOR_BOARD)?;
            // A numeric literal -0.0 would cast to positive zero; the text form keeps the sign.
            diesel::RunQueryDsl::execute(
                sql_query(
                    "INSERT INTO visitation_data (latitude, longitude, ip_address, city, country) VALUES \
                     (39.7, -104.9, '192.0.2.1', 'Denver', 'US'), \
                     (39.7, -104.9, '192.0.2.2', 'Denver', 'US'), \
                     ('-0'::float8, 10.0, '192.0.2.3', 'Libreville', 'GA'), \
                     (0.0, 10.0, '192.0.2.4', 'Libreville', 'GA'), \
                     ('NaN', 5.0, '192.0.2.5', 'Unknown', 'XX'), \
                     (95.0, 5.0, '192.0.2.6', 'Unknown', 'XX')",
                ),
                &mut connection,
            )?;
            Ok(())
        })
        .await??;
        run_pending_migrations(database_url).await?;

        let repository = VisitorRepository::new(database.pool()?);
        let board = repository.board_aggregates(10).await?;
        require(board.len() == 2, "backfill must skip invalid coordinates")?;
        require(
            count_at(&board, 39.7, -104.9) == Some(2),
            "backfill must count Denver twice",
        )?;
        // Rust and PostgreSQL both compare -0.0 equal to 0.0.
        require(
            count_at(&board, 0.0, 10.0) == Some(2),
            "backfill must merge negative and positive zero",
        )?;

        // A flush adds raw rows and increments existing and new locations together.
        let inserted = repository
            .insert_visits(vec![
                visit(39.7, -104.9),
                visit(-0.0, 10.0),
                visit(51.5, -0.1),
                visit(95.0, 5.0),
            ])
            .await?;
        require(inserted == 4, "every visit keeps its raw row")?;
        let board = repository.board_aggregates(10).await?;
        require(board.len() == 3, "flush must add one new location")?;
        require(
            count_at(&board, 39.7, -104.9) == Some(3),
            "flush must increment Denver",
        )?;
        require(
            count_at(&board, 0.0, 10.0) == Some(3),
            "flush must fold -0.0",
        )?;
        require(
            board.last() == Some(&(51.5, -0.1, 1)),
            "flush must insert London last",
        )?;
        let top_two = board.iter().take(2).all(|(_, _, count)| *count == 3);
        require(top_two, "board must be ordered by visit count")?;

        #[derive(QueryableByName)]
        struct CountRow {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            visits: i64,
        }
        let pool = database.pool()?;
        let mut connection = pool.get().await?;
        let raw = sql_query("SELECT count(*) AS visits FROM visitation_data")
            .get_result::<CountRow>(&mut connection)
            .await?;
        require(
            raw.visits == 10,
            "raw visit rows are retained one per event",
        )?;
        Ok(())
    })
}

fn session_bounds_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        #[derive(QueryableByName)]
        struct SettingRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            setting: String,
        }
        let pool_url = with_session_options(database.database_url())?;
        let pool = build_pool(&pool_url, DbPoolSettings { max_size: 2 }).await?;
        let mut connection = pool.get().await?;
        for (name, expected) in [
            ("statement_timeout", "30s"),
            ("lock_timeout", "5s"),
            ("idle_in_transaction_session_timeout", "1min"),
        ] {
            let row = sql_query(format!("SELECT current_setting('{name}') AS setting"))
                .get_result::<SettingRow>(&mut connection)
                .await?;
            require(row.setting == expected, "pooled session setting mismatch")?;
        }
        drop(connection);

        // The migration connection keeps PostgreSQL's unbounded default.
        let migration_url = database.database_url().to_owned();
        let migration_setting =
            tokio::task::spawn_blocking(move || -> Result<String, support::database::BoxError> {
                let mut connection = PgConnection::establish(&migration_url)?;
                let row: SettingRow = diesel::RunQueryDsl::get_result(
                    sql_query("SELECT current_setting('statement_timeout') AS setting"),
                    &mut connection,
                )?;
                Ok(row.setting)
            })
            .await??;
        require(
            migration_setting == "0",
            "migration sessions must stay unbounded",
        )?;
        Ok(())
    })
}
