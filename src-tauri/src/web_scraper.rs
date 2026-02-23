// Web scraper — Fetch URL and extract readable text content
// Uses scraper crate for HTML parsing and text extraction

pub struct ScrapedContent {
    pub text: String,
    pub title: String,
}

/// Scrape a URL and extract text content
pub async fn scrape_url(url: &str) -> Result<ScrapedContent, String> {
    let client = reqwest::Client::new();

    let resp = client
        .get(url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (compatible; ClawX/1.0; Knowledge Base)",
        )
        .send()
        .await
        .map_err(|e| format!("Failed to fetch URL: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("HTTP error {}: {}", resp.status(), url));
    }

    let html = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    extract_content(&html, url)
}

/// Extract text content from HTML using scraper
fn extract_content(html: &str, url: &str) -> Result<ScrapedContent, String> {
    let document = scraper::Html::parse_document(html);

    // Extract title
    let title_selector = scraper::Selector::parse("title")
        .map_err(|e| format!("Failed to parse title selector: {:?}", e))?;
    let title = document
        .select(&title_selector)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| {
            // Fallback: use hostname from URL
            extract_hostname(url).unwrap_or_else(|| url.to_string())
        });

    // Try to find main content area
    let content_selectors = [
        "article",
        "main",
        "[role=main]",
        ".post-content",
        ".article-content",
        ".entry-content",
        "#content",
    ];

    let mut root_element = None;
    for selector_str in &content_selectors {
        if let Ok(selector) = scraper::Selector::parse(selector_str) {
            if let Some(el) = document.select(&selector).next() {
                root_element = Some(el);
                break;
            }
        }
    }

    // Fallback to body
    if root_element.is_none() {
        if let Ok(body_selector) = scraper::Selector::parse("body") {
            root_element = document.select(&body_selector).next();
        }
    }

    let root = root_element.ok_or("No content found in page")?;

    // Remove script and style elements by filtering text nodes
    let text = extract_text_from_element(root);

    if text.trim().is_empty() {
        return Err("No text content found in page".to_string());
    }

    Ok(ScrapedContent { text, title })
}

/// Extract hostname from a URL without the `url` crate
fn extract_hostname(url: &str) -> Option<String> {
    // Find the start after "://"
    let after_scheme = if let Some(idx) = url.find("://") {
        &url[idx + 3..]
    } else {
        url
    };
    // Take everything before the first '/' or '?' or '#'
    let host = after_scheme
        .split(&['/', '?', '#'][..])
        .next()
        .unwrap_or(after_scheme);
    // Remove port
    let host = host.split(':').next().unwrap_or(host);
    if host.is_empty() {
        None
    } else {
        Some(host.to_string())
    }
}

/// Extract text from an element, skipping script/style/nav/footer elements
fn extract_text_from_element(element: scraper::ElementRef) -> String {
    let skip_tags = ["script", "style", "nav", "footer", "header", "aside", "noscript"];
    let mut parts = Vec::new();
    let mut last_was_block = false;

    for child in element.children() {
        // Check if this is a text node
        if child.value().is_text() {
            let text_content = child
                .value()
                .as_text()
                .map(|t| t.trim())
                .unwrap_or("");
            if !text_content.is_empty() {
                parts.push(text_content.to_string());
                last_was_block = false;
            }
            continue;
        }

        // Check if this is an element node
        if let Some(child_el) = scraper::ElementRef::wrap(child) {
            let tag = child_el.value().name();

            // Skip unwanted elements
            if skip_tags.contains(&tag) {
                continue;
            }

            let child_text = extract_text_from_element(child_el);
            if !child_text.trim().is_empty() {
                // Add paragraph breaks for block-level elements
                let is_block = matches!(
                    tag,
                    "p" | "div" | "section" | "article" | "h1" | "h2" | "h3"
                        | "h4" | "h5" | "h6" | "li" | "blockquote" | "pre"
                        | "br" | "hr" | "table" | "tr"
                );

                if is_block && !last_was_block && !parts.is_empty() {
                    parts.push("\n\n".to_string());
                }

                parts.push(child_text);

                if is_block {
                    parts.push("\n".to_string());
                    last_was_block = true;
                } else {
                    last_was_block = false;
                }
            }
        }
    }

    // Clean up excessive whitespace
    let raw = parts.join(" ");
    let lines: Vec<&str> = raw.lines().map(|l| l.trim()).collect();
    let cleaned = lines.join("\n");

    // Collapse multiple blank lines
    use std::sync::OnceLock;
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        regex::Regex::new(r"\n{3,}").expect("static whitespace regex must compile")
    });
    re.replace_all(&cleaned, "\n\n").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_content_basic() {
        let html = r#"
        <html>
        <head><title>Test Page</title></head>
        <body>
            <article>
                <h1>Hello World</h1>
                <p>This is a test paragraph.</p>
                <p>Another paragraph here.</p>
            </article>
            <script>var x = 1;</script>
        </body>
        </html>
        "#;

        let result = extract_content(html, "https://example.com").unwrap();
        assert_eq!(result.title, "Test Page");
        assert!(result.text.contains("Hello World"));
        assert!(result.text.contains("test paragraph"));
        assert!(!result.text.contains("var x"));
    }

    #[test]
    fn test_extract_content_no_article() {
        let html = r#"
        <html>
        <head><title>Simple</title></head>
        <body>
            <p>Just body content</p>
        </body>
        </html>
        "#;

        let result = extract_content(html, "https://example.com").unwrap();
        assert!(result.text.contains("Just body content"));
    }
}
