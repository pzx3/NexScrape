//! HTML parser with CSS selector and XPath support.

use crate::{NexError, Result};
use scraper::{Html, Selector, ElementRef};
use url::Url;

/// HTML parser with CSS selector support.
///
/// Wraps the `scraper` crate and adds convenience methods for
/// common extraction patterns.
pub struct HtmlParser {
    document: Html,
    base_url: Url,
}

impl HtmlParser {
    /// Create a new HTML parser from raw HTML and a base URL.
    pub fn new(html: &str, base_url: &str) -> Self {
        let document = Html::parse_document(html);
        let base_url = Url::parse(base_url).unwrap_or_else(|_| {
            Url::parse("http://localhost").unwrap()
        });

        Self { document, base_url }
    }

    /// Select elements matching a CSS selector.
    pub fn select(&self, selector: &str) -> Result<Vec<HtmlElement<'_>>> {
        let sel = Selector::parse(selector).map_err(|e| {
            NexError::SelectorError(format!("Invalid CSS selector '{}': {:?}", selector, e))
        })?;

        let elements: Vec<HtmlElement> = self
            .document
            .select(&sel)
            .map(|el| HtmlElement::new(el, &self.base_url))
            .collect();

        Ok(elements)
    }

    /// Select the first element matching a CSS selector.
    pub fn select_one(&self, selector: &str) -> Result<Option<HtmlElement<'_>>> {
        let sel = Selector::parse(selector).map_err(|e| {
            NexError::SelectorError(format!("Invalid CSS selector '{}': {:?}", selector, e))
        })?;

        Ok(self
            .document
            .select(&sel)
            .next()
            .map(|el| HtmlElement::new(el, &self.base_url)))
    }

    /// Select text content from the first element matching a CSS selector.
    pub fn select_text(&self, selector: &str) -> Result<String> {
        match self.select_one(selector)? {
            Some(el) => Ok(el.text()),
            None => Err(NexError::SelectorError(format!(
                "No element found for selector '{}'",
                selector
            ))),
        }
    }

    /// Select all text content from elements matching a CSS selector.
    pub fn select_all_text(&self, selector: &str) -> Result<Vec<String>> {
        let elements = self.select(selector)?;
        Ok(elements.iter().map(|el| el.text()).collect())
    }

    /// Extract structured data using a field mapping.
    ///
    /// # Example
    /// ```no_run
    /// use std::collections::HashMap;
    /// let mut fields = HashMap::new();
    /// fields.insert("title".to_string(), "h1::text".to_string());
    /// fields.insert("links".to_string(), "a::attr(href)".to_string());
    /// ```
    pub fn extract_map(
        &self,
        selectors: &std::collections::HashMap<String, String>,
    ) -> Result<std::collections::HashMap<String, String>> {
        let mut result = std::collections::HashMap::new();

        for (field, selector) in selectors {
            // Support pseudo-selectors like ::text and ::attr()
            if selector.ends_with("::text") {
                let css = selector.trim_end_matches("::text");
                if let Ok(text) = self.select_text(css) {
                    result.insert(field.clone(), text);
                }
            } else if selector.contains("::attr(") {
                if let Some((css, attr)) = parse_attr_selector(selector) {
                    if let Ok(Some(el)) = self.select_one(&css) {
                        if let Some(val) = el.attr(&attr) {
                            result.insert(field.clone(), val.to_string());
                        }
                    }
                }
            } else {
                if let Ok(text) = self.select_text(selector) {
                    result.insert(field.clone(), text);
                }
            }
        }

        Ok(result)
    }

    /// Get the page title.
    pub fn title(&self) -> Option<String> {
        self.select_text("title").ok()
    }

    /// Get all links on the page.
    pub fn links(&self) -> Result<Vec<String>> {
        let elements = self.select("a[href]")?;
        Ok(elements
            .iter()
            .filter_map(|el| el.abs_url("href"))
            .collect())
    }

    /// Get all image URLs on the page.
    pub fn images(&self) -> Result<Vec<String>> {
        let elements = self.select("img[src]")?;
        Ok(elements
            .iter()
            .filter_map(|el| el.abs_url("src"))
            .collect())
    }

    /// Get all meta tags as key-value pairs.
    pub fn meta_tags(&self) -> Result<std::collections::HashMap<String, String>> {
        let mut meta = std::collections::HashMap::new();
        let elements = self.select("meta[name], meta[property]")?;

        for el in &elements {
            let key = el
                .attr("name")
                .or_else(|| el.attr("property"))
                .unwrap_or_default()
                .to_string();
            let value = el.attr("content").unwrap_or_default().to_string();
            if !key.is_empty() {
                meta.insert(key, value);
            }
        }

        Ok(meta)
    }
}

/// Wrapper around an HTML element with convenience methods.
pub struct HtmlElement<'a> {
    inner: ElementRef<'a>,
    base_url: Url,
}

impl<'a> HtmlElement<'a> {
    fn new(inner: ElementRef<'a>, base_url: &Url) -> Self {
        Self {
            inner,
            base_url: base_url.clone(),
        }
    }

    /// Get the text content of this element.
    pub fn text(&self) -> String {
        self.inner.text().collect::<Vec<_>>().join(" ").trim().to_string()
    }

    /// Get the inner HTML of this element.
    pub fn inner_html(&self) -> String {
        self.inner.inner_html()
    }

    /// Get the outer HTML of this element.
    pub fn outer_html(&self) -> String {
        self.inner.html()
    }

    /// Get an attribute value.
    pub fn attr(&self, name: &str) -> Option<&str> {
        self.inner.value().attr(name)
    }

    /// Get an attribute value as an absolute URL.
    pub fn abs_url(&self, attr: &str) -> Option<String> {
        self.inner.value().attr(attr).and_then(|href| {
            self.base_url.join(href).ok().map(|u| u.to_string())
        })
    }

    /// Select child elements matching a CSS selector.
    pub fn select(&self, selector: &str) -> Result<Vec<HtmlElement<'a>>> {
        let sel = Selector::parse(selector).map_err(|e| {
            NexError::SelectorError(format!("Invalid selector '{}': {:?}", selector, e))
        })?;

        Ok(self
            .inner
            .select(&sel)
            .map(|el| HtmlElement::new(el, &self.base_url))
            .collect())
    }

    /// Check if this element has a specific class.
    pub fn has_class(&self, class: &str) -> bool {
        self.inner
            .value()
            .attr("class")
            .map_or(false, |classes| {
                classes.split_whitespace().any(|c| c == class)
            })
    }

    /// Get the tag name of this element.
    pub fn tag_name(&self) -> &str {
        self.inner.value().name()
    }
}

/// Parse a selector like "div.class::attr(href)" into ("div.class", "href").
fn parse_attr_selector(selector: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = selector.splitn(2, "::attr(").collect();
    if parts.len() == 2 {
        let css = parts[0].to_string();
        let attr = parts[1].trim_end_matches(')').to_string();
        Some((css, attr))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_HTML: &str = r#"
    <!DOCTYPE html>
    <html>
    <head><title>Test Page</title></head>
    <body>
        <h1 class="title">Hello, NexScrape!</h1>
        <div class="products">
            <div class="product">
                <h2>Product A</h2>
                <span class="price">$10.99</span>
                <a href="/product/a">Details</a>
            </div>
            <div class="product">
                <h2>Product B</h2>
                <span class="price">$24.99</span>
                <a href="/product/b">Details</a>
            </div>
        </div>
        <meta name="description" content="Test page for NexScrape">
        <img src="/images/logo.png" alt="Logo">
    </body>
    </html>
    "#;

    #[test]
    fn test_select_text() {
        let parser = HtmlParser::new(TEST_HTML, "https://example.com");
        let title = parser.select_text("h1").unwrap();
        assert_eq!(title, "Hello, NexScrape!");
    }

    #[test]
    fn test_select_all_text() {
        let parser = HtmlParser::new(TEST_HTML, "https://example.com");
        let prices = parser.select_all_text(".price").unwrap();
        assert_eq!(prices, vec!["$10.99", "$24.99"]);
    }

    #[test]
    fn test_title() {
        let parser = HtmlParser::new(TEST_HTML, "https://example.com");
        assert_eq!(parser.title().unwrap(), "Test Page");
    }

    #[test]
    fn test_links() {
        let parser = HtmlParser::new(TEST_HTML, "https://example.com");
        let links = parser.links().unwrap();
        assert_eq!(links.len(), 2);
        assert_eq!(links[0], "https://example.com/product/a");
        assert_eq!(links[1], "https://example.com/product/b");
    }

    #[test]
    fn test_images() {
        let parser = HtmlParser::new(TEST_HTML, "https://example.com");
        let imgs = parser.images().unwrap();
        assert_eq!(imgs.len(), 1);
        assert_eq!(imgs[0], "https://example.com/images/logo.png");
    }

    #[test]
    fn test_meta_tags() {
        let parser = HtmlParser::new(TEST_HTML, "https://example.com");
        let meta = parser.meta_tags().unwrap();
        assert_eq!(
            meta.get("description").unwrap(),
            "Test page for NexScrape"
        );
    }

    #[test]
    fn test_element_has_class() {
        let parser = HtmlParser::new(TEST_HTML, "https://example.com");
        let elements = parser.select("h1").unwrap();
        assert!(elements[0].has_class("title"));
    }

    #[test]
    fn test_nested_select() {
        let parser = HtmlParser::new(TEST_HTML, "https://example.com");
        let products = parser.select(".product").unwrap();
        assert_eq!(products.len(), 2);

        let first_title = products[0].select("h2").unwrap();
        assert_eq!(first_title[0].text(), "Product A");
    }
}
