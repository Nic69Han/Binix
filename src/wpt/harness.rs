//! WPT Test Harness - Defines test structure and execution

use super::TestCategory;
use std::collections::HashMap;

/// Type of WPT test
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestType {
    /// testharness.js based tests
    Testharness,
    /// Reference tests (visual comparison)
    Reftest,
    /// WebDriver-based tests
    Wdspec,
    /// Crash tests
    Crashtest,
    /// Manual tests
    Manual,
}

/// Expected test outcome
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestExpectation {
    /// Test should pass
    Pass,
    /// Test is expected to fail
    Fail,
    /// Test result is unstable
    Flaky,
    /// Test should be skipped
    Skip,
    /// Test times out
    Timeout,
}

/// A single WPT test case
#[derive(Debug, Clone)]
pub struct TestCase {
    /// Unique test identifier
    pub id: String,
    /// Test name
    pub name: String,
    /// Test category
    pub category: TestCategory,
    /// Type of test
    pub test_type: TestType,
    /// Expected outcome
    pub expectation: TestExpectation,
    /// Test script/content
    pub content: String,
    /// Subtests within this test
    pub subtests: Vec<SubTest>,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// A subtest within a test case
#[derive(Debug, Clone)]
pub struct SubTest {
    pub name: String,
    pub expectation: TestExpectation,
}

impl TestCase {
    pub fn new(id: impl Into<String>, name: impl Into<String>, category: TestCategory) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            category,
            test_type: TestType::Testharness,
            expectation: TestExpectation::Pass,
            content: String::new(),
            subtests: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_type(mut self, test_type: TestType) -> Self {
        self.test_type = test_type;
        self
    }

    pub fn with_expectation(mut self, expectation: TestExpectation) -> Self {
        self.expectation = expectation;
        self
    }

    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }

    pub fn add_subtest(&mut self, name: impl Into<String>, expectation: TestExpectation) {
        self.subtests.push(SubTest {
            name: name.into(),
            expectation,
        });
    }
}

/// Test harness for running WPT tests
pub struct TestHarness {
    tests: Vec<TestCase>,
    timeout_ms: u64,
}

impl TestHarness {
    pub fn new() -> Self {
        Self {
            tests: Vec::new(),
            timeout_ms: 60000,
        }
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    pub fn add_test(&mut self, test: TestCase) {
        self.tests.push(test);
    }

    pub fn tests(&self) -> &[TestCase] {
        &self.tests
    }

    pub fn tests_by_category(&self, category: TestCategory) -> Vec<&TestCase> {
        self.tests
            .iter()
            .filter(|t| t.category == category)
            .collect()
    }

    pub fn load_builtin_tests(&mut self) {
        // HTML tests
        self.add_html_tests();
        // CSS tests
        self.add_css_tests();
        // DOM tests
        self.add_dom_tests();
        // URL tests
        self.add_url_tests();
        // Security tests
        self.add_security_tests();
    }

    fn add_html_tests(&mut self) {
        self.add_test(
            TestCase::new("html/parsing/001", "HTML5 doctype", TestCategory::Html)
                .with_content("<!DOCTYPE html><html><head></head><body></body></html>"),
        );
        self.add_test(
            TestCase::new("html/parsing/002", "Void elements", TestCategory::Html)
                .with_content("<br><hr><img><input>"),
        );
        self.add_test(
            TestCase::new("html/parsing/003", "Nested elements", TestCategory::Html)
                .with_content("<div><p><span>text</span></p></div>"),
        );
        self.add_test(
            TestCase::new("html/parsing/004", "Attribute parsing", TestCategory::Html)
                .with_content("<div id=\"test\" class='foo bar' data-value=123></div>"),
        );
        self.add_test(
            TestCase::new("html/parsing/005", "Script element", TestCategory::Html)
                .with_content("<script>var x = 1;</script>"),
        );
    }

    fn add_css_tests(&mut self) {
        self.add_test(
            TestCase::new("css/selectors/001", "Type selector", TestCategory::Css)
                .with_content("div { color: red; }"),
        );
        self.add_test(
            TestCase::new("css/selectors/002", "Class selector", TestCategory::Css)
                .with_content(".foo { color: blue; }"),
        );
        self.add_test(
            TestCase::new("css/selectors/003", "ID selector", TestCategory::Css)
                .with_content("#bar { color: green; }"),
        );
        self.add_test(
            TestCase::new("css/box/001", "Box model", TestCategory::Css)
                .with_content("div { width: 100px; padding: 10px; margin: 5px; }"),
        );
    }

    fn add_dom_tests(&mut self) {
        self.add_test(
            TestCase::new("dom/nodes/001", "createElement", TestCategory::Dom)
                .with_content("document.createElement('div')"),
        );
        self.add_test(
            TestCase::new("dom/nodes/002", "appendChild", TestCategory::Dom)
                .with_content("parent.appendChild(child)"),
        );
        self.add_test(
            TestCase::new("dom/nodes/003", "getElementById", TestCategory::Dom)
                .with_content("document.getElementById('test')"),
        );
    }

    fn add_url_tests(&mut self) {
        self.add_test(
            TestCase::new("url/parsing/001", "Basic URL", TestCategory::Url)
                .with_content("https://example.com/path"),
        );
        self.add_test(
            TestCase::new("url/parsing/002", "URL with query", TestCategory::Url)
                .with_content("https://example.com?foo=bar"),
        );
        self.add_test(
            TestCase::new("url/parsing/003", "URL with fragment", TestCategory::Url)
                .with_content("https://example.com#section"),
        );
    }

    fn add_security_tests(&mut self) {
        self.add_test(
            TestCase::new(
                "security/csp/001",
                "CSP header parsing",
                TestCategory::Security,
            )
            .with_content("default-src 'self'"),
        );
        self.add_test(
            TestCase::new(
                "security/cors/001",
                "CORS preflight",
                TestCategory::Security,
            )
            .with_content("Access-Control-Allow-Origin: *"),
        );
    }
}

impl Default for TestHarness {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_case() {
        let test = TestCase::new("test/001", "Test Name", TestCategory::Html);
        assert_eq!(test.id, "test/001");
        assert_eq!(test.name, "Test Name");
        assert_eq!(test.category, TestCategory::Html);
    }

    #[test]
    fn test_harness_add_tests() {
        let mut harness = TestHarness::new();
        harness.add_test(TestCase::new("t1", "Test 1", TestCategory::Html));
        harness.add_test(TestCase::new("t2", "Test 2", TestCategory::Css));
        assert_eq!(harness.tests().len(), 2);
    }

    #[test]
    fn test_harness_filter_by_category() {
        let mut harness = TestHarness::new();
        harness.add_test(TestCase::new("t1", "Test 1", TestCategory::Html));
        harness.add_test(TestCase::new("t2", "Test 2", TestCategory::Css));
        harness.add_test(TestCase::new("t3", "Test 3", TestCategory::Html));

        let html_tests = harness.tests_by_category(TestCategory::Html);
        assert_eq!(html_tests.len(), 2);
    }

    #[test]
    fn test_load_builtin_tests() {
        let mut harness = TestHarness::new();
        harness.load_builtin_tests();
        assert!(harness.tests().len() > 10);
    }
}
