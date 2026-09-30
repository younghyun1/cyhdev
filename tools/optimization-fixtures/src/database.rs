//! Snapshot cloning confines destructive reset to the campaign's disposable database.

use diesel_async::{AsyncConnection, AsyncPgConnection};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::Path, process::Command};

use crate::{files, seed};

#[derive(Serialize, Deserialize)]
struct Fixture {
    database: String,
    snapshot: String,
    snapshot_sha256: String,
    parameters: serde_json::Value,
    inputs: BTreeMap<String, String>,
    inputs_sha256: String,
}

fn settings() -> anyhow::Result<reqwest::Url> {
    parse_url(&std::env::var("DB_URL")?)
}

fn parse_url(input: &str) -> anyhow::Result<reqwest::Url> {
    let url = reqwest::Url::parse(input)?;
    anyhow::ensure!(
        url.scheme() == "postgres"
            && url.host_str() == Some("127.0.0.1")
            && url.port() == Some(35432)
            && url.username() == "optimization_fixture"
            && url
                .password()
                .is_some_and(|p| p.starts_with("optimization-fixture-"))
            && url.query().is_none()
            && url.fragment().is_none(),
        "fixture database connection is outside the dedicated namespace"
    );
    let name = url.path().trim_start_matches('/');
    anyhow::ensure!(
        name.starts_with("cyhdev_optimization_")
            && name.len() <= 48
            && name
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'),
        "invalid fixture database name"
    );
    Ok(url)
}

fn pg(program: &str, url: &reqwest::Url) -> Command {
    let mut command = Command::new(program);
    command
        .args([
            "--host",
            "127.0.0.1",
            "--port",
            "35432",
            "--username",
            "optimization_fixture",
        ])
        .env("PGPASSWORD", url.password().unwrap_or(""));
    command
}

fn snapshot_digest(url: &reqwest::Url, database: &str, runtime: &Path) -> anyhow::Result<String> {
    let output = runtime.join("snapshot-dump.tmp");
    files::command(
        pg("pg_dump", url)
            .args([
                "--no-owner",
                "--no-privileges",
                "--restrict-key=0123456789abcdef0123456789abcdef",
                "--file",
            ])
            .arg(&output)
            .arg(database),
    )?;
    anyhow::ensure!(
        output.metadata()?.len() <= 128 * 1024 * 1024,
        "fixture database dump exceeded 128 MiB"
    );
    let digest = files::digest(&fs::read(&output)?);
    fs::remove_file(output)?;
    Ok(digest)
}

pub async fn prepare(runtime: &Path) -> anyhow::Result<()> {
    let url = settings()?;
    let database = url.path().trim_start_matches('/').to_owned();
    let snapshot = format!("{database}_snapshot");
    files::command(pg("createdb", &url).arg(&snapshot))?;
    let mut snapshot_url = url.clone();
    snapshot_url.set_path(&snapshot);
    rust_be_template::init::db_migrations::run_pending_migrations(snapshot_url.to_string()).await?;
    let mut connection = AsyncPgConnection::establish(snapshot_url.as_str()).await?;
    let parameters = connection
        .transaction::<serde_json::Value, anyhow::Error, _>(async |connection| {
            seed::seed(connection).await
        })
        .await?;
    drop(connection);
    let inputs = files::inputs(runtime)?;
    let inputs_sha256 = files::digest(&serde_json::to_vec(&inputs)?);
    let fixture = Fixture {
        database,
        snapshot: snapshot.clone(),
        snapshot_sha256: snapshot_digest(&url, &snapshot, runtime)?,
        parameters,
        inputs,
        inputs_sha256,
    };
    fs::write(
        runtime.join("fixture.json"),
        serde_json::to_vec_pretty(&fixture)?,
    )?;
    Ok(())
}

pub async fn reset(runtime: &Path) -> anyhow::Result<()> {
    let url = settings()?;
    let fixture: Fixture = serde_json::from_slice(&fs::read(runtime.join("fixture.json"))?)?;
    let inputs = files::inputs(runtime)?;
    anyhow::ensure!(
        inputs == fixture.inputs
            && files::digest(&serde_json::to_vec(&inputs)?) == fixture.inputs_sha256,
        "fixture assets, keys or Geo-IP inputs changed"
    );
    anyhow::ensure!(
        url.path().trim_start_matches('/') == fixture.database
            && fixture.snapshot == format!("{}_snapshot", fixture.database),
        "fixture receipt does not match database target"
    );
    anyhow::ensure!(
        snapshot_digest(&url, &fixture.snapshot, runtime)? == fixture.snapshot_sha256,
        "golden fixture snapshot changed"
    );
    files::command(pg("dropdb", &url).args(["--if-exists", "--force", &fixture.database]))?;
    files::command(
        pg("createdb", &url)
            .arg(format!("--template={}", fixture.snapshot))
            .arg(&fixture.database),
    )?;
    anyhow::ensure!(
        snapshot_digest(&url, &fixture.database, runtime)? == fixture.snapshot_sha256,
        "reset clone differs from golden snapshot"
    );
    let search = runtime.join("data/search_index");
    anyhow::ensure!(
        !search.symlink_metadata()?.file_type().is_symlink(),
        "fixture search index cannot be a symlink"
    );
    fs::remove_dir_all(&search)?;
    fs::create_dir(&search)?;
    let response = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?
        .post(format!(
            "http://127.0.0.1:{}/__fixture/reset",
            files::HTTP_PORT
        ))
        .send()
        .await?;
    anyhow::ensure!(
        response.status().is_success(),
        "fixture provider reset failed"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn database_admission_rejects_other_services_and_namespaces() {
        let valid = "postgres://optimization_fixture:optimization-fixture-postgres@127.0.0.1:35432/cyhdev_optimization_test";
        assert!(super::parse_url(valid).is_ok());
        for changed in [
            valid.replace("127.0.0.1", "example.test"),
            valid.replace("35432", "5432"),
            valid.replace("cyhdev_optimization_test", "production"),
            valid.replace("optimization_fixture:", "postgres:"),
            format!("{valid}?host=/tmp"),
            format!("{valid}#override"),
            valid.replace("optimization-fixture-postgres", "real-secret"),
        ] {
            assert!(super::parse_url(&changed).is_err());
        }
    }
}
