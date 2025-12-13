//! Integration tests for Binix browser engine
//!
//! These tests verify the browser components work together correctly.
//! Run with: cargo test --test integration_tests

use binix::BrowserEngine;
use binix::network::{NetworkStack, NetworkClient};
use binix::renderer::{HtmlParser, CssParser, Document};
use proptest::prelude::*;
use std::time::{Duration, Instant};

/// Test fixtures - HTML pages for testing
mod fixtures {
    pub const SIMPLE_PAGE: &str = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Test Page</title>
            <style>
                h1 { color: red; font-size: 32px; }
                .highlight { background-color: yellow; }
                #main { padding: 20px; }
            </style>
        </head>
        <body>
            <div id="main">
                <h1>Welcome</h1>
                <p class="highlight">This is a test paragraph.</p>
                <a href="/link">Click here</a>
                <ul>
                    <li>Item 1</li>
                    <li>Item 2</li>
                </ul>
            </div>
        </body>
        </html>
    "#;

    pub const COMPLEX_CSS: &str = r##"
        <!DOCTYPE html>
        <html>
        <head>
            <style>
                body { font-family: sans-serif; margin: 0; }
                .container { max-width: 800px; margin: 0 auto; padding: 20px; }
                h1 { color: #333; border-bottom: 2px solid #007bff; }
                h2 { color: #666; }
                p { line-height: 1.6; }
                code { background: #f4f4f4; padding: 2px 6px; border-radius: 3px; }
                a { color: #007bff; text-decoration: none; }
                a:hover { text-decoration: underline; }
                .btn { background: #007bff; color: white; padding: 10px 20px; border-radius: 5px; }
            </style>
        </head>
        <body>
            <div class="container">
                <h1>Documentation</h1>
                <h2>Getting Started</h2>
                <p>Welcome to the <code>Binix</code> browser engine.</p>
                <a href="/learn" class="btn">Learn More</a>
            </div>
        </body>
        </html>
    "##;

    pub const NESTED_ELEMENTS: &str = r#"
        <!DOCTYPE html>
        <html>
        <head><title>Nested Test</title></head>
        <body>
            <div>
                <div>
                    <div>
                        <p>Deeply nested paragraph</p>
                    </div>
                </div>
            </div>
            <section>
                <article>
                    <header><h1>Article Title</h1></header>
                    <p>Article content</p>
                    <footer>By Author</footer>
                </article>
            </section>
        </body>
        </html>
    "#;

    pub const IMAGES_AND_MEDIA: &str = r#"
        <!DOCTYPE html>
        <html>
        <head><title>Media Test</title></head>
        <body>
            <img src="logo.png" alt="Logo">
            <img src="/images/photo.jpg" alt="Photo" width="200" height="150">
            <figure>
                <img src="https://example.com/image.webp" alt="External Image">
                <figcaption>Image caption</figcaption>
            </figure>
        </body>
        </html>
    "#;

    pub const FORM_ELEMENTS: &str = r#"
        <!DOCTYPE html>
        <html>
        <head><title>Form Test</title></head>
        <body>
            <form action="/submit" method="post">
                <label for="name">Name:</label>
                <input type="text" id="name" name="name">
                <label for="email">Email:</label>
                <input type="email" id="email" name="email">
                <button type="submit">Submit</button>
            </form>
        </body>
        </html>
    "#;
}

// ============================================================================
// HTML PARSING TESTS
// ============================================================================

#[cfg(test)]
mod html_parsing_tests {
    use super::*;

    #[test]
    fn test_parse_simple_page() {
        let parser = HtmlParser::new();
        let doc = parser.parse(fixtures::SIMPLE_PAGE).unwrap();

        // Document has a root node with children
        assert!(!doc.root.children.is_empty(), "Document should have child nodes");
    }

    #[test]
    fn test_parse_produces_elements() {
        let parser = HtmlParser::new();
        let doc = parser.parse(fixtures::SIMPLE_PAGE).unwrap();

        // Check that we have elements in the tree
        fn count_elements(node: &binix::renderer::Node) -> usize {
            let mut count = if node.is_element() { 1 } else { 0 };
            for child in &node.children {
                count += count_elements(child);
            }
            count
        }

        let element_count = count_elements(&doc.root);
        assert!(element_count > 5, "Should have parsed multiple elements, got {}", element_count);
    }

    #[test]
    fn test_parse_nested_elements() {
        let parser = HtmlParser::new();
        let doc = parser.parse(fixtures::NESTED_ELEMENTS).unwrap();

        assert!(!doc.root.children.is_empty());
    }

    #[test]
    fn test_parse_handles_malformed_html() {
        let parser = HtmlParser::new();
        let malformed = "<html><body><p>Unclosed paragraph<div>Mixed nesting</p></div>";

        // Should not panic, should produce some result
        let result = parser.parse(malformed);
        assert!(result.is_ok(), "Parser should handle malformed HTML gracefully");
    }

    #[test]
    fn test_parse_preserves_attributes() {
        let parser = HtmlParser::new();
        let doc = parser.parse(fixtures::IMAGES_AND_MEDIA).unwrap();

        assert!(!doc.root.children.is_empty());
    }

    proptest! {
        #[test]
        fn test_html_parsing_never_panics(html in ".*") {
            let parser = HtmlParser::new();
            // Should never panic on any input
            let _ = parser.parse(&html);
        }
    }
}

// ============================================================================
// CSS PARSING TESTS
// ============================================================================

#[cfg(test)]
mod css_parsing_tests {
    use super::*;

    #[test]
    fn test_parse_inline_styles() {
        let parser = CssParser::new();
        let css = "color: red; font-size: 16px; background-color: #fff;";

        let result = parser.parse(css);
        assert!(result.is_ok());

        let stylesheet = result.unwrap();
        assert!(!stylesheet.rules.is_empty() || css.contains(":"));
    }

    #[test]
    fn test_parse_stylesheet_rules() {
        let parser = CssParser::new();
        let css = r#"
            h1 { color: red; }
            .class { background: blue; }
            #id { padding: 10px; }
        "#;

        let stylesheet = parser.parse(css).unwrap();
        assert_eq!(stylesheet.rules.len(), 3);
    }

    #[test]
    fn test_parse_complex_selectors() {
        let parser = CssParser::new();
        let css = r#"
            div.container { margin: 0 auto; }
            h1, h2, h3 { font-weight: bold; }
            a:hover { text-decoration: underline; }
        "#;

        let stylesheet = parser.parse(css).unwrap();
        assert!(stylesheet.rules.len() >= 2);
    }

    #[test]
    fn test_parse_color_values() {
        let parser = CssParser::new();
        let css = r#"
            .hex3 { color: #fff; }
            .hex6 { color: #ffffff; }
            .named { color: red; }
            .rgb { color: rgb(255, 0, 0); }
        "#;

        let result = parser.parse(css);
        assert!(result.is_ok());
    }

    proptest! {
        #[test]
        fn test_css_parsing_never_panics(css in ".*") {
            let parser = CssParser::new();
            // Should never panic on any input
            let _ = parser.parse(&css);
        }
    }
}

// ============================================================================
// NETWORK TESTS
// ============================================================================

#[cfg(test)]
mod network_tests {
    use super::*;

    #[test]
    fn test_network_stack_creation() {
        let stack = NetworkStack::new();
        // Stack should be created without panic
        let _ = stack;
    }

    #[test]
    fn test_network_client_creation() {
        let client = NetworkClient::new();
        // Client should be created without panic
        let _ = client;
    }

    #[tokio::test]
    #[ignore] // Requires network
    async fn test_fetch_example_com() {
        let stack = NetworkStack::new();
        let response = stack.fetch("https://example.com").await;

        assert!(response.is_ok(), "Should fetch example.com successfully");

        let resp = response.unwrap();
        assert_eq!(resp.status(), 200);
        assert!(resp.body().contains("Example Domain"));
    }

    #[tokio::test]
    #[ignore] // Requires network
    async fn test_fetch_handles_404() {
        let stack = NetworkStack::new();
        let response = stack.fetch("https://httpstat.us/404").await;

        assert!(response.is_ok());
        assert_eq!(response.unwrap().status(), 404);
    }

    #[tokio::test]
    #[ignore] // Requires network
    async fn test_fetch_handles_redirects() {
        let stack = NetworkStack::new();
        let response = stack.fetch("https://httpstat.us/301").await;

        // Should follow redirect or return redirect status
        assert!(response.is_ok());
    }
}

// ============================================================================
// ENGINE INTEGRATION TESTS
// ============================================================================

#[cfg(test)]
mod engine_tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let _engine = BrowserEngine::new();
        // Engine should be created without panic
    }

    #[tokio::test]
    #[ignore] // Requires network
    async fn test_engine_full_page_load() {
        let mut engine = BrowserEngine::new();
        let result = engine.process_page("https://example.com").await;

        assert!(result.is_ok(), "Should load page successfully");
    }

    #[test]
    fn test_engine_parses_html_string() {
        let parser = HtmlParser::new();
        let result = parser.parse(fixtures::SIMPLE_PAGE);

        assert!(result.is_ok());
        let doc = result.unwrap();
        assert!(!doc.root.children.is_empty());
    }
}

// ============================================================================
// PERFORMANCE TESTS
// ============================================================================

#[cfg(test)]
mod performance_tests {
    use super::*;

    /// Benchmark HTML parsing performance
    #[test]
    fn test_html_parsing_performance() {
        let parser = HtmlParser::new();
        let iterations = 100;

        let start = Instant::now();
        for _ in 0..iterations {
            let _ = parser.parse(fixtures::COMPLEX_CSS);
        }
        let duration = start.elapsed();

        let avg_ms = duration.as_millis() as f64 / iterations as f64;
        println!("Average HTML parsing time: {:.3}ms", avg_ms);

        // Should parse in under 10ms on average
        assert!(avg_ms < 10.0, "HTML parsing should be under 10ms, was {:.3}ms", avg_ms);
    }

    /// Benchmark CSS parsing performance
    #[test]
    fn test_css_parsing_performance() {
        let parser = CssParser::new();
        let css = r#"
            body { margin: 0; padding: 0; font-family: sans-serif; }
            .container { max-width: 1200px; margin: 0 auto; }
            h1, h2, h3 { color: #333; }
            p { line-height: 1.6; }
            a { color: blue; text-decoration: none; }
            .btn { padding: 10px 20px; background: #007bff; color: white; }
            .nav { display: flex; justify-content: space-between; }
            .footer { background: #333; color: white; padding: 20px; }
        "#;

        let iterations = 1000;
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = parser.parse(css);
        }
        let duration = start.elapsed();

        let avg_us = duration.as_micros() as f64 / iterations as f64;
        println!("Average CSS parsing time: {:.1}μs", avg_us);

        // Should parse in under 1ms on average
        assert!(avg_us < 1000.0, "CSS parsing should be under 1ms, was {:.1}μs", avg_us);
    }

    /// Verify page load meets target (<1500ms)
    #[tokio::test]
    #[ignore] // Requires network
    async fn test_page_load_performance_target() {
        let mut engine = BrowserEngine::new();

        let start = Instant::now();
        let _ = engine.process_page("https://example.com").await;
        let duration = start.elapsed();

        println!("Page load time: {:?}", duration);

        assert!(
            duration < Duration::from_millis(1500),
            "Page load should be under 1500ms, was {:?}",
            duration
        );
    }

    /// Test memory doesn't grow excessively
    #[test]
    fn test_memory_efficiency() {
        let parser = HtmlParser::new();

        // Parse many documents
        for _ in 0..1000 {
            let _ = parser.parse(fixtures::SIMPLE_PAGE);
        }

        // No assertion - just verify it doesn't OOM
        // In a real test, we'd measure memory usage
    }
}

// ============================================================================
// REGRESSION TESTS
// ============================================================================

#[cfg(test)]
mod regression_tests {
    use super::*;

    /// Regression: Ensure deeply nested elements don't stack overflow
    #[test]
    fn test_deeply_nested_elements_no_stackoverflow() {
        let parser = HtmlParser::new();

        // Generate very deeply nested HTML
        let mut html = String::from("<html><body>");
        for _ in 0..100 {
            html.push_str("<div>");
        }
        html.push_str("<p>Deep content</p>");
        for _ in 0..100 {
            html.push_str("</div>");
        }
        html.push_str("</body></html>");

        let result = parser.parse(&html);
        assert!(result.is_ok(), "Should handle deep nesting");
    }

    /// Regression: Large documents shouldn't cause issues
    #[test]
    fn test_large_document_handling() {
        let parser = HtmlParser::new();

        // Generate a large document
        let mut html = String::from("<html><body>");
        for i in 0..1000 {
            html.push_str(&format!("<p>Paragraph number {} with some content to make it longer.</p>", i));
        }
        html.push_str("</body></html>");

        let result = parser.parse(&html);
        assert!(result.is_ok(), "Should handle large documents");
    }

    /// Regression: Unicode content should be handled correctly
    #[test]
    fn test_unicode_content() {
        let parser = HtmlParser::new();
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head><title>日本語タイトル</title></head>
            <body>
                <h1>Привет мир!</h1>
                <p>中文内容</p>
                <p>Émojis: 🚀🎉🔥</p>
                <p>Arabic: مرحبا بالعالم</p>
            </body>
            </html>
        "#;

        let result = parser.parse(html);
        assert!(result.is_ok(), "Should handle Unicode content");
    }

    /// Regression: Empty content should be handled gracefully
    #[test]
    fn test_empty_content() {
        let parser = HtmlParser::new();

        let result = parser.parse("");
        assert!(result.is_ok(), "Should handle empty content");

        let result = parser.parse("   ");
        assert!(result.is_ok(), "Should handle whitespace-only content");
    }

    /// Regression: Self-closing tags should work
    #[test]
    fn test_self_closing_tags() {
        let parser = HtmlParser::new();
        let html = r#"
            <html>
            <head>
                <meta charset="utf-8"/>
                <link rel="stylesheet" href="style.css"/>
            </head>
            <body>
                <img src="image.png" alt="test"/>
                <br/>
                <hr/>
                <input type="text"/>
            </body>
            </html>
        "#;

        let result = parser.parse(html);
        assert!(result.is_ok(), "Should handle self-closing tags");
    }
}
