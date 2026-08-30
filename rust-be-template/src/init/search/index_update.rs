use std::{collections::HashSet, path::Path};

use tracing::info;
use uuid::Uuid;

use super::PostSearchIndex;

impl PostSearchIndex {
    pub(crate) fn begin_rebuild(&self) -> anyhow::Result<()> {
        let writer = self
            .writer
            .write()
            .map_err(|e| anyhow::anyhow!("Writer lock poisoned: {e}"))?;
        writer.delete_all_documents()?;
        Ok(())
    }

    pub(crate) fn finish_rebuild(&self) -> anyhow::Result<()> {
        self.commit()
    }

    pub(crate) fn abort_rebuild(&self) -> anyhow::Result<()> {
        let mut writer = self
            .writer
            .write()
            .map_err(|e| anyhow::anyhow!("Writer lock poisoned: {e}"))?;
        writer.rollback()?;
        Ok(())
    }

    pub fn rebuild_index<'a, I>(&self, posts: I) -> anyhow::Result<usize>
    where
        I: Iterator<Item = (Uuid, &'a str, &'a [String])>,
    {
        self.begin_rebuild()?;
        let mut count = 0;
        for (post_id, title, tags) in posts {
            if let Err(e) = self.index_post(post_id, title, tags) {
                let _ = self.abort_rebuild();
                return Err(e);
            }
            count += 1;
        }
        self.finish_rebuild()?;
        info!(posts_indexed = count, "Search index rebuilt");
        Ok(count)
    }

    pub fn sync_with_posts<'a, I>(&self, posts: I) -> anyhow::Result<(usize, usize)>
    where
        I: Iterator<Item = (Uuid, &'a str, &'a [String])>,
    {
        let posts = posts.collect::<Vec<_>>();
        let expected_ids = posts.iter().map(|(id, _, _)| *id).collect::<HashSet<_>>();
        let (missing, extra) = self.check_coherence(&expected_ids)?;

        for post_id in &extra {
            self.remove_post(*post_id)?;
        }
        let missing_set = missing.iter().copied().collect::<HashSet<_>>();
        for (post_id, title, tags) in &posts {
            if missing_set.contains(post_id) {
                self.index_post(*post_id, title, tags)?;
            }
        }
        if !missing.is_empty() || !extra.is_empty() {
            self.commit()?;
            info!(added = missing.len(), removed = extra.len(), "Search index synchronized");
        }
        Ok((missing.len(), extra.len()))
    }

    pub fn update_post(&self, post_id: Uuid, title: &str, tags: &[String]) -> anyhow::Result<()> {
        self.remove_post(post_id)?;
        self.index_post(post_id, title, tags)?;
        self.commit()
    }

    pub fn add_post_and_commit(
        &self,
        post_id: Uuid,
        title: &str,
        tags: &[String],
    ) -> anyhow::Result<()> {
        self.index_post(post_id, title, tags)?;
        self.commit()
    }

    pub fn remove_post_and_commit(&self, post_id: Uuid) -> anyhow::Result<()> {
        self.remove_post(post_id)?;
        self.commit()
    }

    pub fn index_path(&self) -> Option<&Path> {
        self.index_path.as_deref()
    }

    pub fn num_docs(&self) -> u64 {
        self.reader.searcher().num_docs()
    }
}
