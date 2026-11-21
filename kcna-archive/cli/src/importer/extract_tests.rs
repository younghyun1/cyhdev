//! Tests for page extraction, kept beside the module to bound file size.

use crate::{
    error::Result,
    importer::extract::{legacy, legacy_index_titles, live},
};

#[test]
fn splits_historical_multi_article_pages() -> Result<()> {
    let source = "<body>\n<a name=\"1\"><h3>One</h3></a>\nFirst body<hr>\n<a name=\"2\"><h3>Two</h3></a>\nSecond body<center>back</center></body>";
    let articles = legacy(source)?;
    assert_eq!(articles.len(), 2);
    assert_eq!(articles[0].title, "One");
    assert!(articles[1].body.starts_with("Second body"));
    Ok(())
}

#[test]
fn splits_early_historical_h4_pages() -> Result<()> {
    let source = "<body>\n<a name=\"1\"><h4>One</h4></a>\nFirst body<hr>\n<a name=\"2\"><h4>Two</h4></a>\nSecond body</body>";
    let articles = legacy(source)?;
    assert_eq!(articles.len(), 2);
    assert_eq!(articles[1].title, "Two");
    Ok(())
}

#[test]
fn splits_historical_pages_with_reversed_closing_tags() -> Result<()> {
    let source = "<body>\n<a name=\"1\"><h4>One</a></h4>\nFirst body<hr>\n<a name=\"2\"><h4>Two</a></h4>\nSecond body</body>";
    let articles = legacy(source)?;
    assert_eq!(articles.len(), 2);
    assert_eq!(articles[0].title, "One");
    assert!(articles[1].body.starts_with("Second body"));
    Ok(())
}

#[test]
fn splits_historical_pages_with_body_on_heading_line() -> Result<()> {
    let source = "<body>\n<a name=\"1\"><h4>One</h4></a>First body<hr>\n<a name=\"2\"><h4>Two</h4></a>Second body</body>";
    let articles = legacy(source)?;
    assert_eq!(articles.len(), 2);
    assert_eq!(articles[1].title, "Two");
    assert!(articles[0].body.starts_with("First body"));
    Ok(())
}

#[test]
fn splits_historical_pages_with_unclosed_headings() -> Result<()> {
    let source = "<body>\n<a name=\"1\"><h4>One\nFirst body<hr>\n<a name=\"2\"><h4>Two</h4></a>\nSecond body</body>";
    let articles = legacy(source)?;
    assert_eq!(articles.len(), 2);
    assert_eq!(articles[1].title, "Two");
    Ok(())
}

#[test]
fn splits_historical_pages_with_headings_wrapping_anchors() -> Result<()> {
    let source = "<body>\n<H4><A NAME=\"1\">One\n</A></H4>\nFirst body<hr>\n<H4><A NAME=\"2\">Two</A>\n</H4>\nSecond body</body>";
    let articles = legacy(source)?;
    assert_eq!(articles.len(), 2);
    assert_eq!(articles[0].title, "One");
    assert!(articles[1].body.starts_with("Second body"));
    Ok(())
}

#[test]
fn preserves_empty_heading_positions_for_index_title_resolution() -> Result<()> {
    let source = "<body>\n<a name=\"1\"><h3></h3></a>First body<hr>\n<a name=\"2\"><h3>Two</h3></a>Second body</body>";
    let articles = legacy(source)?;
    assert_eq!(articles.len(), 2);
    assert!(articles[0].title.is_empty());
    assert_eq!(articles[1].title, "Two");
    Ok(())
}

#[test]
fn extracts_title_before_empty_heading() -> Result<()> {
    let source = "<body>\n<a name=\"1\">One<h3></h3></a>\nFirst body</body>";
    let articles = legacy(source)?;
    assert_eq!(articles[0].title, "One");
    Ok(())
}

#[test]
fn extracts_titles_from_historical_daily_index() -> Result<()> {
    let source = r#"<li><h3><a href="./news08/14.htm#1">One</a></h3><li><h3><a href="./news08/14.htm#2">Two</a></h3>"#;
    let titles = legacy_index_titles(source)?;
    assert_eq!(titles.get(&0).map(String::as_str), Some("One"));
    assert_eq!(titles.get(&1).map(String::as_str), Some("Two"));
    Ok(())
}

#[test]
fn replaces_spurious_empty_duplicate_anchor() -> Result<()> {
    let source =
        "<body>\n<a name=\"1\"><h3></h3></a>\n<a name=\"1\"><h3>One</h3></a>\nFirst body</body>";
    let articles = legacy(source)?;
    assert_eq!(articles.len(), 1);
    assert_eq!(articles[0].title, "One");
    assert!(articles[0].body.starts_with("First body"));
    Ok(())
}

#[test]
fn extracts_index_title_outside_empty_link() -> Result<()> {
    let source = r#"<li><h3><a href="./news08/18.htm#17"></a>Title outside link</h3>"#;
    let titles = legacy_index_titles(source)?;
    assert_eq!(
        titles.get(&16).map(String::as_str),
        Some("Title outside link")
    );
    Ok(())
}

#[test]
fn rejects_anchor_numbers_beyond_the_page_bound() {
    let source = "<body>\n<a name=\"1\"><h3>One</h3></a>\nBody\n<a name=\"20020101\"><h3>Dated anchor</h3></a>\nTail</body>";
    assert!(legacy(source).is_ok_and(|articles| articles.len() == 1 && articles[0].title == "One"));
    let index = r#"<li><h3><a href="./news08/18.htm#20020101">Dated</a></h3>"#;
    assert!(legacy_index_titles(index).is_ok_and(|titles| titles.is_empty()));
}

#[test]
fn strips_breadcrumb_title_and_footer_from_single_article_pages() -> Result<()> {
    let source = r#"<body><table width="610"><tr><td><a href="c.htm">과거기사 달력</a>&gt;&gt;<a href="d.html">2004년 2월 6일기사 목록</a></td></tr></table>
        <table width="610"><tr><td width="610"><div align="left"><b>조류독감 방역대책사업 활발<br></b></div></td></tr>
        <tr><td><div align="left"><p>(평양 2월 5일발 조선중앙통신)조선에서 사업이 벌어지고있다.</p><p>대책도 세워나가고있다.(끝)</p></div></td></tr></table>
        <table><tr><td><font size="1">Copyright (C) 2002 KOREA NEWS SERVICE(KNS) All Rights Reserved.</font></td></tr></table></body>"#;
    let articles = legacy(source)?;
    assert_eq!(articles.len(), 1);
    assert_eq!(articles[0].title, "조류독감 방역대책사업 활발");
    assert!(
        articles[0]
            .body
            .starts_with("(평양 2월 5일발 조선중앙통신)")
    );
    assert!(articles[0].body.ends_with("세워나가고있다.(끝)"));
    Ok(())
}

#[test]
fn keeps_a_headline_only_page_with_an_empty_body() -> Result<()> {
    let source = r#"<body><div align="left"><b>Headline only</b></div>
        <font>Copyright (C) 2002-2011 KOREA NEWS SERVICE(KNS) All Rights Reserved.</font></body>"#;
    let articles = legacy(source)?;
    assert_eq!(articles[0].title, "Headline only");
    assert!(articles[0].body.is_empty());
    Ok(())
}

#[test]
fn keeps_paragraph_breaks_and_skips_scripts() -> Result<()> {
    let source = "<body>\n<a name=\"1\"><h3>One</h3></a>\n첫 문단이다.<br>\n<br>둘째 문단<b>굵게</b>이다.<p>셋째 문단.</p><script>var x = 1;</script></body>";
    let articles = legacy(source)?;
    assert_eq!(
        articles[0].body,
        "첫 문단이다.\n\n둘째 문단굵게이다.\n\n셋째 문단."
    );
    Ok(())
}

#[test]
fn extracts_current_semantic_article() -> Result<()> {
    let source = "<article><h1>Title</h1><p>First.</p><p>Second.</p></article>";
    let article = live(source)?;
    assert_eq!(article.title, "Title");
    assert_eq!(article.body, "First.\n\nSecond.");
    Ok(())
}

#[test]
fn recovers_historical_title_outside_bold_tag() -> Result<()> {
    let source = r#"<body><div align="left">Recovered title<b><br></b></div><div align="left">Body text</div></body>"#;
    let articles = legacy(source)?;
    assert_eq!(articles[0].title, "Recovered title");
    assert!(articles[0].body.contains("Body text"));
    Ok(())
}

#[test]
fn takes_the_headline_instead_of_the_bold_breadcrumb() -> Result<()> {
    let source = r##"<body><table width="610"><tr><td><b><font size="3"><a href="../../calendar-2025e.html">calendar</a></font><font size="3" color="#ee0000">&gt;&gt;</font></b><a href="../2025-06-19ee.html"><b>June</b><font size="3"><b> 19. 2025</b></font></a></td></tr></table>
        <table width="610"><tr><td width="610"><div align="left"><b>Woljong Temple Repaired in South Hwanghae Province</b></div></td></tr>
        <tr><td width="610">Pyongyang, June 19 (KCNA) -- The temple is under repair.<br><p> The temple has the Manse Pavilion.<br></p></td></tr></table>
        <table><tr><td><font size="1">Copyright (C) KOREA NEWS SERVICE(KNS) All Rights Reserved.</font></td></tr></table></body>"##;
    let articles = legacy(source)?;
    assert_eq!(
        articles[0].title,
        "Woljong Temple Repaired in South Hwanghae Province"
    );
    assert_eq!(
        articles[0].body,
        "Pyongyang, June 19 (KCNA) -- The temple is under repair.\n\nThe temple has the Manse Pavilion."
    );
    Ok(())
}

#[test]
fn skips_navigation_when_the_title_block_is_absent() -> Result<()> {
    let source = r#"<body><b><a href="c.html">calendar</a> &gt;&gt;</b><a href="d.html"><b>June 19</b></a>
        <b>Plain headline</b><p>Body text.</p></body>"#;
    let articles = legacy(source)?;
    assert_eq!(articles[0].title, "Plain headline");
    assert_eq!(articles[0].body, "Body text.");
    Ok(())
}
