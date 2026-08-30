//! File-backed staging port used by the photography batch service.

use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

pub fn batch_root_dir() -> PathBuf { std::env::temp_dir().join("cyhdev-batch") }
pub fn batch_temp_dir(batch_id: Uuid) -> PathBuf { batch_root_dir().join(batch_id.to_string()) }
pub fn batch_item_path(batch_id: Uuid, item_id: Uuid) -> PathBuf { batch_temp_dir(batch_id).join(format!("{item_id}.orig")) }
pub async fn open_staging_file(batch_id: Uuid, item_id: Uuid) -> std::io::Result<tokio::fs::File> {
    tokio::fs::create_dir_all(batch_temp_dir(batch_id)).await?;
    tokio::fs::File::create(batch_item_path(batch_id, item_id)).await
}
pub async fn append_chunk(file: &mut tokio::fs::File, chunk: &[u8]) -> std::io::Result<()> { file.write_all(chunk).await }
