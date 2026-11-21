//! Decides which copy of an article is preferred and which stored links
//! differ from that decision. Pure logic; the database pass is in `copies`.

use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use time::Date;
use uuid::Uuid;

use crate::{
    database::types::{Language, Source},
    importer::legacy_record::UNAVAILABLE_TITLE,
};

#[derive(Clone, Debug)]
pub(super) struct Member {
    pub(super) id: Uuid,
    pub(super) source: Source,
    /// Normalized digest of title and body; `None` for a headline-only record.
    pub(super) content: Option<Vec<u8>>,
    pub(super) canonical: Option<Uuid>,
}

/// Same language, same publication date, same normalized title.
pub(super) type ArticleKey = (Language, Date, u64);

/// What a relinking run has to write.
#[derive(Debug, Default, Eq, PartialEq)]
pub(super) struct Plan {
    /// Articles held by more than one source.
    pub(super) articles: usize,
    /// Documents whose link changes, with the new target.
    pub(super) changes: Vec<(Uuid, Option<Uuid>)>,
    /// Preferred copies whose set of copies changed.
    pub(super) touched: Vec<Uuid>,
}

/// Decides the preferred copy of each group and what differs from the stored links.
pub(super) fn plan(groups: impl Iterator<Item = Vec<Member>>) -> Plan {
    let mut plan = Plan::default();
    for members in groups {
        for cluster in clusters(members) {
            let preferred = preferred(&cluster);
            if cluster.len() > 1 {
                plan.articles += 1;
            }
            let mut changed = false;
            for member in &cluster {
                let target = preferred.filter(|id| cluster.len() > 1 && *id != member.id);
                if member.canonical != target {
                    plan.changes.push((member.id, target));
                    changed = true;
                }
            }
            if changed
                && cluster.len() > 1
                && let Some(preferred) = preferred
            {
                plan.touched.push(preferred);
            }
        }
    }
    plan
}

/// Splits documents sharing a date and title into sets of true copies.
///
/// KCNA repeats headlines on one day (greeting notices from different
/// senders), so a shared title is not enough: copies must also have the same
/// normalized text. A headline-only record carries no text to compare; it
/// joins the article only when the title names exactly one, and otherwise
/// stays on its own.
fn clusters(members: Vec<Member>) -> Vec<Vec<Member>> {
    let mut by_content: HashMap<Vec<u8>, Vec<Member>> = HashMap::new();
    let mut headlines = Vec::new();
    for member in members {
        match member.content.clone() {
            Some(content) => by_content.entry(content).or_default().push(member),
            None => headlines.push(member),
        }
    }
    let mut clusters = by_content.into_values().collect::<Vec<_>>();
    match (clusters.len(), headlines.len()) {
        (_, 0) => {}
        (1, _) => clusters[0].extend(headlines),
        // Headline-only records alone are copies when each comes from a
        // different source.
        (0, _) if distinct_sources(&headlines) => clusters.push(headlines),
        _ => clusters.extend(headlines.into_iter().map(|member| vec![member])),
    }
    clusters
}

fn distinct_sources(members: &[Member]) -> bool {
    let mut seen = [false; 3];
    members
        .iter()
        .all(|member| !std::mem::replace(&mut seen[priority(member.source)], true))
}

/// The copy that represents a cluster: one with text before a headline-only
/// record, then by source priority, then the oldest identifier for stability.
fn preferred(cluster: &[Member]) -> Option<Uuid> {
    cluster
        .iter()
        .min_by_key(|member| (member.content.is_none(), priority(member.source), member.id))
        .map(|member| member.id)
}

/// Lower is preferred: the live site, then the historical mirror, then the
/// public dataset.
const fn priority(source: Source) -> usize {
    match source {
        Source::KcnaKp => 0,
        Source::KcnaCoJp => 1,
        Source::StilesArchive => 2,
    }
}

/// Hash of the title's letters and digits, ignoring case, spacing, and
/// punctuation, which differ between sources. `None` for a title that cannot
/// identify an article.
pub(super) fn title_key(title: &str) -> Option<u64> {
    if title == UNAVAILABLE_TITLE {
        return None;
    }
    let mut hasher = DefaultHasher::new();
    let mut any = false;
    for character in title
        .chars()
        .filter(|character| character.is_alphanumeric())
    {
        for lower in character.to_lowercase() {
            lower.hash(&mut hasher);
        }
        any = true;
    }
    any.then(|| hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn member(id: u128, source: Source, content: Option<u8>, canonical: Option<u128>) -> Member {
        Member {
            id: Uuid::from_u128(id),
            source,
            content: content.map(|byte| vec![byte; 32]),
            canonical: canonical.map(Uuid::from_u128),
        }
    }

    fn links(members: Vec<Member>) -> Vec<(u128, Option<u128>)> {
        let mut changes = plan([members].into_iter())
            .changes
            .into_iter()
            .map(|(id, target)| (id.as_u128(), target.map(|target| target.as_u128())))
            .collect::<Vec<_>>();
        changes.sort_unstable();
        changes
    }

    #[test]
    fn titles_compare_without_case_spacing_or_punctuation() {
        assert_eq!(
            title_key("Kim Jong Un Sends Greetings"),
            title_key("KIM JONG UN  sends greetings.")
        );
        assert_ne!(
            title_key("Greetings to Kim"),
            title_key("Greetings to Kim Jong Un")
        );
        assert_eq!(title_key(UNAVAILABLE_TITLE), None);
        assert_eq!(title_key("!!!"), None);
    }

    #[test]
    fn prefers_the_live_site_then_the_mirror() {
        let group = vec![
            member(1, Source::StilesArchive, Some(7), None),
            member(2, Source::KcnaCoJp, Some(7), None),
            member(3, Source::KcnaKp, Some(7), None),
        ];
        assert_eq!(links(group.clone()), [(1, Some(3)), (2, Some(3))]);
        assert_eq!(links(group[..2].to_vec()), [(1, Some(2))]);
    }

    #[test]
    fn repeated_headlines_with_different_text_stay_separate() {
        let group = vec![
            member(1, Source::KcnaKp, Some(1), None),
            member(2, Source::KcnaKp, Some(2), None),
            member(3, Source::KcnaCoJp, Some(1), None),
            member(4, Source::KcnaCoJp, Some(2), None),
            member(5, Source::StilesArchive, None, None),
        ];
        // Each mirror copy joins the live article with the same text; the
        // headline-only record cannot tell the two articles apart.
        assert_eq!(links(group), [(3, Some(1)), (4, Some(2))]);
    }

    #[test]
    fn a_headline_only_record_joins_the_single_article_of_its_title() {
        let group = vec![
            member(1, Source::StilesArchive, None, None),
            member(2, Source::KcnaCoJp, Some(4), None),
        ];
        assert_eq!(links(group), [(1, Some(2))]);
        let alone = vec![member(1, Source::StilesArchive, None, None)];
        assert!(links(alone).is_empty());
        let same_source = vec![
            member(1, Source::StilesArchive, None, None),
            member(2, Source::StilesArchive, None, None),
        ];
        assert!(links(same_source).is_empty());
    }

    #[test]
    fn plans_only_the_links_that_change() {
        let linked = vec![
            member(1, Source::KcnaKp, Some(1), None),
            member(2, Source::StilesArchive, Some(1), Some(1)),
        ];
        let fresh = vec![
            member(3, Source::KcnaCoJp, Some(2), None),
            member(4, Source::StilesArchive, Some(2), None),
        ];
        let stale = vec![member(5, Source::KcnaKp, Some(3), Some(9))];
        let planned = plan([linked, fresh, stale].into_iter());
        assert_eq!(planned.articles, 2);
        assert_eq!(
            planned.changes,
            [
                (Uuid::from_u128(4), Some(Uuid::from_u128(3))),
                (Uuid::from_u128(5), None)
            ]
        );
        assert_eq!(planned.touched, [Uuid::from_u128(3)]);
    }
}
