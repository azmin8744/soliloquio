use dashmap::DashMap;
use pulldown_cmark::{html, Options, Parser};
use std::sync::Arc;
use uuid::Uuid;

/// Simple in-memory cache for rendered markdown
pub struct MarkdownCache {
    cache: Arc<DashMap<Uuid, String>>,
}

impl MarkdownCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
        }
    }

    pub fn get(&self, id: &Uuid) -> Option<String> {
        self.cache.get(id).map(|entry| entry.clone())
    }

    pub fn set(&self, id: Uuid, html: String) {
        self.cache.insert(id, html);
    }

    pub fn invalidate(&self, id: &Uuid) {
        self.cache.remove(id);
    }

    pub fn clear(&self) {
        self.cache.clear();
    }
}

impl Default for MarkdownCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Render markdown to HTML
pub fn render_markdown(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);

    let parser = Parser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

/// Render markdown with caching
pub fn render_markdown_cached(id: Uuid, markdown: &str, cache: &MarkdownCache) -> String {
    // Check cache first
    if let Some(cached_html) = cache.get(&id) {
        return cached_html;
    }

    // Render and cache
    let html = render_markdown(markdown);
    cache.set(id, html.clone());
    html
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_rendering() {
        let markdown = "# Hello\n\nThis is **bold** text.";
        let html = render_markdown(markdown);
        assert!(html.contains("<h1>Hello</h1>"));
        assert!(html.contains("<strong>bold</strong>"));
    }

    #[test]
    fn test_markdown_cache() {
        let cache = MarkdownCache::new();
        let id = Uuid::new_v4();
        let markdown = "# Test";

        // First render should cache
        let html1 = render_markdown_cached(id, markdown, &cache);

        // Second render should use cache
        let html2 = render_markdown_cached(id, markdown, &cache);

        assert_eq!(html1, html2);
        assert!(cache.get(&id).is_some());
    }
}
