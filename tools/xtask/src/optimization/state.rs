//! Campaign state persisted only after successful stage transitions.

use std::path::PathBuf;

use serde_json::Value;

use super::config::Config;

pub(super) struct State {
    pub root: PathBuf,
    pub directory: PathBuf,
    pub config: Config,
    pub config_path: PathBuf,
    pub config_digest: String,
    pub campaign_digest: String,
    pub revision: String,
    pub image: String,
    pub receipt: Value,
}
