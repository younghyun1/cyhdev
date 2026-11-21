//! Tests for link filtering and scheduling in the historical crawl.

use std::collections::{HashSet, VecDeque};

use url::Url;

use super::legacy::{enqueue, is_archive_html};
use crate::error::Result;

#[test]
fn rejects_non_html_and_external_links() -> Result<()> {
    assert!(is_archive_html(&Url::parse(
        "http://www.kcna.co.jp/calendar/2002/a.htm"
    )?));
    assert!(!is_archive_html(&Url::parse(
        "http://www.kcna.co.jp/images/a.jpg"
    )?));
    assert!(!is_archive_html(&Url::parse("https://example.com/a.htm")?));
    assert!(is_archive_html(&Url::parse(
        "http://www.kcna.co.jp/calendar/2011/a.HTML"
    )?));
    assert!(!is_archive_html(&Url::parse(
        "http://www.kcna.co.jp/item2/2001/200104/.htm"
    )?));
    assert!(!is_archive_html(&Url::parse(
        "http://www.kcna.co.jp/item2/htm"
    )?));
    Ok(())
}

#[test]
fn deduplicates_urls_before_queueing() -> Result<()> {
    let url = Url::parse("https://kcna.co.jp/calendar/2002/a.htm#fragment")?;
    let mut queue = VecDeque::new();
    let mut scheduled = HashSet::new();
    enqueue(&mut queue, &mut scheduled, url.clone())?;
    enqueue(&mut queue, &mut scheduled, url)?;
    assert_eq!(queue.len(), 1);
    assert_eq!(scheduled.len(), 1);
    assert_eq!(
        queue[0].as_str(),
        "http://www.kcna.co.jp/calendar/2002/a.htm"
    );
    Ok(())
}

#[test]
fn never_schedules_external_hosts() -> Result<()> {
    let mut queue = VecDeque::new();
    let mut scheduled = HashSet::new();
    for external in [
        "https://example.com/a.htm",
        "http://www.kcna.kp/kp/a.html",
        "ftp://www.kcna.co.jp/a.htm",
        "mailto:editor@kcna.co.jp",
    ] {
        enqueue(&mut queue, &mut scheduled, Url::parse(external)?)?;
    }
    assert!(queue.is_empty());
    assert!(scheduled.is_empty());
    Ok(())
}
