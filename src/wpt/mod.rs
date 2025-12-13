//! Web Platform Tests (WPT) compliance testing framework
//!
//! Provides infrastructure for running standardized web platform tests
//! to ensure browser compliance with web standards.

mod harness;
mod results;
mod runner;

pub use harness::{TestCase, TestExpectation, TestHarness, TestType};
pub use results::{ComplianceScore, TestReport, TestResult, TestStatus};
pub use runner::{RunnerConfig, WptRunner};

/// WPT test categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TestCategory {
    /// HTML parsing and DOM tests
    Html,
    /// CSS parsing and rendering tests
    Css,
    /// JavaScript/ECMAScript tests
    JavaScript,
    /// DOM API tests
    Dom,
    /// Fetch and networking tests
    Fetch,
    /// WebAssembly tests
    Wasm,
    /// Security-related tests (CSP, CORS, etc.)
    Security,
    /// Performance API tests
    Performance,
    /// URL parsing tests
    Url,
    /// Encoding tests
    Encoding,
}

impl TestCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Html => "html",
            Self::Css => "css",
            Self::JavaScript => "javascript",
            Self::Dom => "dom",
            Self::Fetch => "fetch",
            Self::Wasm => "wasm",
            Self::Security => "security",
            Self::Performance => "performance",
            Self::Url => "url",
            Self::Encoding => "encoding",
        }
    }

    pub fn all() -> &'static [TestCategory] {
        &[
            Self::Html,
            Self::Css,
            Self::JavaScript,
            Self::Dom,
            Self::Fetch,
            Self::Wasm,
            Self::Security,
            Self::Performance,
            Self::Url,
            Self::Encoding,
        ]
    }
}

/// Web Platform Tests compliance checker
pub struct WptCompliance {
    runner: WptRunner,
    categories: Vec<TestCategory>,
}

impl WptCompliance {
    pub fn new() -> Self {
        Self {
            runner: WptRunner::new(RunnerConfig::default()),
            categories: TestCategory::all().to_vec(),
        }
    }

    pub fn with_categories(categories: Vec<TestCategory>) -> Self {
        Self {
            runner: WptRunner::new(RunnerConfig::default()),
            categories,
        }
    }

    /// Run all compliance tests
    pub fn run_all(&self) -> TestReport {
        self.runner.run_categories(&self.categories)
    }

    /// Run tests for a specific category
    pub fn run_category(&self, category: TestCategory) -> TestReport {
        self.runner.run_categories(&[category])
    }

    /// Get overall compliance score
    pub fn compliance_score(&self) -> ComplianceScore {
        let report = self.run_all();
        report.score()
    }
}

impl Default for WptCompliance {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_as_str() {
        assert_eq!(TestCategory::Html.as_str(), "html");
        assert_eq!(TestCategory::Css.as_str(), "css");
        assert_eq!(TestCategory::JavaScript.as_str(), "javascript");
    }

    #[test]
    fn test_all_categories() {
        let all = TestCategory::all();
        assert_eq!(all.len(), 10);
    }

    #[test]
    fn test_wpt_compliance_new() {
        let wpt = WptCompliance::new();
        assert_eq!(wpt.categories.len(), 10);
    }

    #[test]
    fn test_wpt_compliance_with_categories() {
        let wpt = WptCompliance::with_categories(vec![TestCategory::Html, TestCategory::Css]);
        assert_eq!(wpt.categories.len(), 2);
    }
}
