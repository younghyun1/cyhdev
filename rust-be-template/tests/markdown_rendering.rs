//! Preserve blog rendering when unused Comrak CLI and syntax-highlighter features are disabled.

#[test]
fn library_renderer_preserves_headings_links_and_fenced_code() {
    let rendered = comrak::markdown_to_html(
        "# Heading\n\n**Strong** and [link](https://example.test).\n\n```rust\nlet value = 1;\n```\n",
        &comrak::Options::default(),
    );
    assert_eq!(
        rendered,
        "<h1>Heading</h1>\n<p><strong>Strong</strong> and <a href=\"https://example.test\">link</a>.</p>\n<pre><code class=\"language-rust\">let value = 1;\n</code></pre>\n"
    );
}

#[test]
fn library_renderer_does_not_enable_unsafe_html() {
    let rendered = comrak::markdown_to_html(
        "<script>alert('fixture')</script>\n\n[unsafe](javascript:alert%281%29)",
        &comrak::Options::default(),
    );
    assert!(!rendered.contains("<script>"));
    assert!(!rendered.contains("javascript:"));
}

#[test]
fn explicitly_enabled_shortcodes_remain_available() {
    let mut options = comrak::Options::default();
    options.extension.shortcodes = true;
    assert_eq!(comrak::markdown_to_html(":smile:", &options), "<p>😄</p>\n");
}
