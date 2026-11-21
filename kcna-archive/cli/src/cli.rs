//! Command-line configuration.

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

use crate::database::types::Language;

/// Downloads public KCNA source archives into a resumable local corpus.
#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    /// Root directory for raw pages, indexes, and manifests.
    #[arg(long, default_value = "data")]
    pub data_dir: PathBuf,

    /// Directory of the snapshot the website serves; `<data-dir>/snapshot` when omitted.
    #[arg(long)]
    pub snapshot_dir: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Collect the live multilingual archive and the historical Korea News Service mirror.
    All(CommonArgs),
    /// Collect the current official multilingual archive from kcna.kp.
    Live(LiveArgs),
    /// Collect the historical Korean-first archive from kcna.co.jp.
    Legacy(LegacyArgs),
    /// Manage the PostgreSQL research corpus.
    Database(DatabaseArgs),
    /// Load the snapshot into memory as the website does and time requests against it.
    Verify,
}

#[derive(Debug, clap::Args)]
pub struct DatabaseArgs {
    #[command(subcommand)]
    pub command: DatabaseCommand,
}

#[derive(Debug, Subcommand)]
pub enum DatabaseCommand {
    /// Apply embedded PostgreSQL 18 migrations using DATABASE_URL.
    Migrate,
    /// Import full local source pages and provenance using DATABASE_URL.
    Import(DatabaseImportArgs),
    /// Bring the snapshot the website serves up to date with the database.
    Snapshot(DatabaseSnapshotArgs),
}

#[derive(Clone, Debug, clap::Args)]
pub struct DatabaseSnapshotArgs {
    /// Discard the snapshot and build it again from every stored document.
    #[arg(long)]
    pub rebuild: bool,
}

/// Language edition of the live kcna.kp site, named by its URL path segment.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, ValueEnum)]
pub enum LiveEdition {
    Kp,
    En,
    Cn,
    Ru,
    Es,
    Jp,
}

impl LiveEdition {
    /// URL and raw-directory segment for this edition.
    pub const fn path(self) -> &'static str {
        match self {
            Self::Kp => "kp",
            Self::En => "en",
            Self::Cn => "cn",
            Self::Ru => "ru",
            Self::Es => "es",
            Self::Jp => "jp",
        }
    }

    /// Document language published under this edition.
    pub const fn language(self) -> Language {
        match self {
            Self::Kp => Language::Korean,
            Self::En => Language::English,
            Self::Cn => Language::Chinese,
            Self::Ru => Language::Russian,
            Self::Es => Language::Spanish,
            Self::Jp => Language::Japanese,
        }
    }
}

#[derive(Clone, Debug, clap::Args)]
pub struct DatabaseImportArgs {
    /// Rows per PostgreSQL transaction.
    #[arg(long, default_value_t = 250, value_parser = import_batch_size)]
    pub batch_size: usize,

    /// Open Korean Historical Corpus JSONL metadata and URL inventory.
    #[arg(long)]
    pub seed_index: Option<PathBuf>,

    /// Do not import historical kcna.co.jp pages.
    #[arg(long)]
    pub skip_legacy: bool,

    /// Maximum historical metadata records to import after the skipped ones; omitted means all.
    #[arg(long)]
    pub max_legacy_records: Option<usize>,

    /// Number of leading historical metadata records to skip when resuming.
    #[arg(long, default_value_t = 0)]
    pub skip_legacy_records: usize,

    /// Do not import current kcna.kp pages.
    #[arg(long)]
    pub skip_live: bool,

    /// Do not import the recent public CC0 English cross-check.
    #[arg(long)]
    pub skip_stiles: bool,

    /// Comma-separated live language paths to import.
    #[arg(
        long,
        value_enum,
        value_delimiter = ',',
        default_value = "kp,en,cn,ru,es,jp"
    )]
    pub live_languages: Vec<LiveEdition>,
}

#[derive(Clone, Debug, clap::Args)]
pub struct CommonArgs {
    /// Comma-separated KCNA language paths, in priority order.
    #[arg(
        long,
        value_enum,
        value_delimiter = ',',
        default_value = "kp,en,cn,ru,es,jp"
    )]
    pub languages: Vec<LiveEdition>,

    /// Maximum live-site requests in flight.
    #[arg(long, default_value_t = 12, value_parser = live_concurrency)]
    pub live_concurrency: usize,

    /// Maximum historical-mirror requests in flight.
    #[arg(long, default_value_t = 32, value_parser = legacy_concurrency)]
    pub legacy_concurrency: usize,

    /// Optional Open Korean Historical Corpus JSONL URL inventory.
    #[arg(long)]
    pub seed_index: Option<PathBuf>,

    /// Ask again for historical pages a recent run found nonexistent.
    #[arg(long)]
    pub recheck_missing: bool,
}

#[derive(Clone, Debug, clap::Args)]
pub struct LiveArgs {
    /// Comma-separated KCNA language paths, in priority order.
    #[arg(
        long,
        value_enum,
        value_delimiter = ',',
        default_value = "kp,en,cn,ru,es,jp"
    )]
    pub languages: Vec<LiveEdition>,

    /// Maximum live-site requests in flight.
    #[arg(long, default_value_t = 12, value_parser = live_concurrency)]
    pub concurrency: usize,
}

#[derive(Clone, Debug, clap::Args)]
pub struct LegacyArgs {
    /// Maximum historical-mirror requests in flight.
    #[arg(long, default_value_t = 32, value_parser = legacy_concurrency)]
    pub concurrency: usize,

    /// Optional Open Korean Historical Corpus JSONL URL inventory.
    #[arg(long)]
    pub seed_index: Option<PathBuf>,

    /// Ask again for historical pages a recent run found nonexistent.
    #[arg(long)]
    pub recheck_missing: bool,
}

fn live_concurrency(value: &str) -> Result<usize, String> {
    bounded_usize(value, 64)
}

fn legacy_concurrency(value: &str) -> Result<usize, String> {
    bounded_usize(value, 128)
}

fn import_batch_size(value: &str) -> Result<usize, String> {
    bounded_usize(value, 1_000)
}

fn bounded_usize(value: &str, maximum: usize) -> Result<usize, String> {
    let parsed = value
        .parse::<usize>()
        .map_err(|error| format!("invalid number: {error}"))?;
    if (1..=maximum).contains(&parsed) {
        Ok(parsed)
    } else {
        Err(format!("value must be between 1 and {maximum}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounds_numeric_arguments() {
        assert_eq!(bounded_usize("32", 128), Ok(32));
        assert!(bounded_usize("0", 128).is_err());
        assert!(bounded_usize("129", 128).is_err());
        assert!(bounded_usize("many", 128).is_err());
    }

    #[test]
    fn rejects_unknown_live_editions() {
        let parsed = Cli::try_parse_from(["kcna-archive", "live", "--languages", "kp,xx"]);
        assert!(parsed.is_err());
        let parsed = Cli::try_parse_from(["kcna-archive", "live", "--languages", "kp,en"]);
        assert!(parsed.is_ok_and(|cli| matches!(
            cli.command,
            Command::Live(arguments) if arguments.languages == [LiveEdition::Kp, LiveEdition::En]
        )));
    }
}
