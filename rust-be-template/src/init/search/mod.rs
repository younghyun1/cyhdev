use std::collections::HashSet;
use std::path::Path;
use std::sync::RwLock;

use tantivy::{
    Index, IndexReader, IndexSettings, IndexWriter, TantivyDocument,
    directory::MmapDirectory,
    schema::{
        Field, IndexRecordOption, STORED, STRING, Schema, TextFieldIndexing, TextOptions, Value,
    },
};
use tracing::{info, warn};
use uuid::Uuid;

mod query;
mod index_update;

/// Disk-persisted search index for blog posts using Tantivy.
/// Indexes post titles and tags for fast full-text search.
/// Maintains coherence with the database cache.
pub struct PostSearchIndex {
    index: Index,
    reader: IndexReader,
    writer: RwLock<IndexWriter>,
    index_path: Option<std::path::PathBuf>,
    // Schema fields
    post_id_field: Field,
    title_field: Field,
    tags_field: Field,
}

impl PostSearchIndex {
    /// Build the schema used by the index.
    fn build_schema() -> (Schema, Field, Field, Field) {
        let mut schema_builder = Schema::builder();

        // Post ID stored as string for retrieval
        let post_id_field = schema_builder.add_text_field("post_id", STRING | STORED);

        // Title field - indexed for full-text search
        let text_field_indexing = TextFieldIndexing::default()
            .set_tokenizer("default")
            .set_index_option(IndexRecordOption::WithFreqsAndPositions);
        let text_options = TextOptions::default()
            .set_indexing_options(text_field_indexing)
            .set_stored();
        let title_field = schema_builder.add_text_field("title", text_options);

        // Tags field - indexed as individual terms for exact matching
        let tags_field = schema_builder.add_text_field("tags", STRING | STORED);

        let schema = schema_builder.build();
        (schema, post_id_field, title_field, tags_field)
    }

    /// Create a new in-memory search index (no persistence).
    pub fn new_in_memory() -> anyhow::Result<Self> {
        let (schema, post_id_field, title_field, tags_field) = Self::build_schema();

        let index = Index::create_in_ram(schema);
        let writer = index.writer(50_000_000)?;
        let reader = index.reader()?;

        Ok(Self {
            index,
            reader,
            writer: RwLock::new(writer),
            index_path: None,
            post_id_field,
            title_field,
            tags_field,
        })
    }

    /// Open or create a disk-persisted search index.
    /// If the directory doesn't exist, it will be created.
    /// If the index exists but is corrupted, it will be recreated.
    pub fn open_or_create<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let (schema, post_id_field, title_field, tags_field) = Self::build_schema();

        // Ensure directory exists
        if !path.exists() {
            std::fs::create_dir_all(path)?;
            info!(path = %path.display(), "Created search index directory");
        }

        // Try to open existing index, create new if it doesn't exist or is corrupted
        let index = match MmapDirectory::open(path) {
            Ok(dir) => {
                match Index::open(dir) {
                    Ok(idx) => {
                        info!(path = %path.display(), "Opened existing search index");
                        idx
                    }
                    Err(e) => {
                        warn!(path = %path.display(), error = %e, "Failed to open index, creating new one");
                        // Clear the directory and create fresh
                        Self::clear_directory(path)?;
                        let dir = MmapDirectory::open(path)?;
                        Index::create(dir, schema.clone(), IndexSettings::default())?
                    }
                }
            }
            Err(e) => {
                warn!(path = %path.display(), error = %e, "Failed to open directory, creating new index");
                Self::clear_directory(path)?;
                let dir = MmapDirectory::open(path)?;
                Index::create(dir, schema.clone(), IndexSettings::default())?
            }
        };

        let writer = index.writer(50_000_000)?;
        let reader = index.reader()?;

        Ok(Self {
            index,
            reader,
            writer: RwLock::new(writer),
            index_path: Some(path.to_path_buf()),
            post_id_field,
            title_field,
            tags_field,
        })
    }

    /// Clear a directory of all files (used when recreating a corrupted index).
    fn clear_directory(path: &Path) -> anyhow::Result<()> {
        if path.exists() {
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    std::fs::remove_file(&path)?;
                }
            }
        }
        Ok(())
    }

    /// Get all post IDs currently in the index.
    pub fn get_indexed_post_ids(&self) -> anyhow::Result<HashSet<Uuid>> {
        let searcher = self.reader.searcher();
        let mut post_ids = HashSet::new();

        for segment_reader in searcher.segment_readers() {
            let store_reader = segment_reader.get_store_reader(1)?;
            for doc_id in segment_reader.doc_ids_alive() {
                if let Ok(doc) = store_reader.get::<TantivyDocument>(doc_id)
                    && let Some(post_id_value) = doc.get_first(self.post_id_field)
                    && let Some(post_id_str) = post_id_value.as_str()
                    && let Ok(uuid) = Uuid::parse_str(post_id_str)
                {
                    post_ids.insert(uuid);
                }
            }
        }

        Ok(post_ids)
    }

    /// Check if the index is coherent with a set of expected post IDs.
    /// Returns (missing_from_index, extra_in_index).
    pub fn check_coherence(
        &self,
        expected_post_ids: &HashSet<Uuid>,
    ) -> anyhow::Result<(Vec<Uuid>, Vec<Uuid>)> {
        let indexed_ids = self.get_indexed_post_ids()?;

        let missing: Vec<Uuid> = expected_post_ids
            .difference(&indexed_ids)
            .copied()
            .collect();

        let extra: Vec<Uuid> = indexed_ids.difference(expected_post_ids).copied().collect();

        Ok((missing, extra))
    }

    /// Index a single post. Call commit() after batch operations.
    pub fn index_post(&self, post_id: Uuid, title: &str, tags: &[String]) -> anyhow::Result<()> {
        let mut doc = TantivyDocument::new();
        doc.add_text(self.post_id_field, post_id.to_string());
        doc.add_text(self.title_field, title);

        // Add each tag as a separate field value for exact term matching
        for tag in tags {
            doc.add_text(self.tags_field, tag.to_lowercase());
        }

        let writer = self
            .writer
            .write()
            .map_err(|e| anyhow::anyhow!("Writer lock poisoned: {}", e))?;
        writer.add_document(doc)?;

        Ok(())
    }

    /// Remove a post from the index by its ID.
    pub fn remove_post(&self, post_id: Uuid) -> anyhow::Result<()> {
        let term = tantivy::Term::from_field_text(self.post_id_field, &post_id.to_string());
        let writer = self
            .writer
            .write()
            .map_err(|e| anyhow::anyhow!("Writer lock poisoned: {}", e))?;
        writer.delete_term(term);
        Ok(())
    }

    /// Commit pending changes to the index and persist to disk.
    pub fn commit(&self) -> anyhow::Result<()> {
        let mut writer = self
            .writer
            .write()
            .map_err(|e| anyhow::anyhow!("Writer lock poisoned: {}", e))?;
        writer.commit()?;
        // Reload reader to see committed changes
        self.reader.reload()?;
        Ok(())
    }

}
