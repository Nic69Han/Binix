//! WPT Test Results - Stores and analyzes test outcomes

use super::TestCategory;
use std::collections::HashMap;
use std::time::Duration;

/// Status of a single test
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestStatus {
    /// Test passed
    Pass,
    /// Test failed
    Fail,
    /// Test timed out
    Timeout,
    /// Test crashed
    Crash,
    /// Test was skipped
    Skip,
    /// Test is not implemented
    NotRun,
}

impl TestStatus {
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Pass | Self::Skip)
    }
}

/// Result of a single test execution
#[derive(Debug, Clone)]
pub struct TestResult {
    pub test_id: String,
    pub test_name: String,
    pub category: TestCategory,
    pub status: TestStatus,
    pub duration: Duration,
    pub message: Option<String>,
    pub subtest_results: Vec<SubTestResult>,
}

/// Result of a subtest
#[derive(Debug, Clone)]
pub struct SubTestResult {
    pub name: String,
    pub status: TestStatus,
    pub message: Option<String>,
}

/// Complete test report
#[derive(Debug)]
pub struct TestReport {
    results: Vec<TestResult>,
    duration: Duration,
    by_category: HashMap<TestCategory, CategoryStats>,
}

/// Statistics for a category
#[derive(Debug, Default, Clone)]
pub struct CategoryStats {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub timeout: usize,
}

impl CategoryStats {
    pub fn pass_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.passed as f64 / self.total as f64) * 100.0
        }
    }
}

impl TestReport {
    pub fn new(results: Vec<TestResult>, duration: Duration) -> Self {
        let mut by_category: HashMap<TestCategory, CategoryStats> = HashMap::new();

        for result in &results {
            let stats = by_category.entry(result.category).or_default();
            stats.total += 1;
            match result.status {
                TestStatus::Pass => stats.passed += 1,
                TestStatus::Fail | TestStatus::Crash => stats.failed += 1,
                TestStatus::Skip | TestStatus::NotRun => stats.skipped += 1,
                TestStatus::Timeout => stats.timeout += 1,
            }
        }

        Self {
            results,
            duration,
            by_category,
        }
    }

    pub fn total(&self) -> usize {
        self.results.len()
    }

    pub fn passed(&self) -> usize {
        self.results
            .iter()
            .filter(|r| r.status == TestStatus::Pass)
            .count()
    }

    pub fn failed(&self) -> usize {
        self.results
            .iter()
            .filter(|r| r.status == TestStatus::Fail)
            .count()
    }

    pub fn duration(&self) -> Duration {
        self.duration
    }

    pub fn results(&self) -> &[TestResult] {
        &self.results
    }

    pub fn category_stats(&self, category: TestCategory) -> Option<&CategoryStats> {
        self.by_category.get(&category)
    }

    pub fn score(&self) -> ComplianceScore {
        let total = self.total();
        let passed = self.passed();

        ComplianceScore {
            total_tests: total,
            passed_tests: passed,
            percentage: if total == 0 {
                0.0
            } else {
                (passed as f64 / total as f64) * 100.0
            },
            by_category: self.by_category.clone(),
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "WPT Results: {}/{} passed ({:.1}%) in {:?}",
            self.passed(),
            self.total(),
            self.score().percentage,
            self.duration
        )
    }
}

/// Overall compliance score
#[derive(Debug)]
pub struct ComplianceScore {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub percentage: f64,
    pub by_category: HashMap<TestCategory, CategoryStats>,
}

impl ComplianceScore {
    pub fn grade(&self) -> &'static str {
        match self.percentage as u32 {
            95..=100 => "A+",
            90..=94 => "A",
            85..=89 => "B+",
            80..=84 => "B",
            75..=79 => "C+",
            70..=74 => "C",
            60..=69 => "D",
            _ => "F",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_is_success() {
        assert!(TestStatus::Pass.is_success());
        assert!(TestStatus::Skip.is_success());
        assert!(!TestStatus::Fail.is_success());
    }

    #[test]
    fn test_report_empty() {
        let report = TestReport::new(Vec::new(), Duration::from_secs(0));
        assert_eq!(report.total(), 0);
        assert_eq!(report.passed(), 0);
    }

    #[test]
    fn test_report_with_results() {
        let results = vec![
            TestResult {
                test_id: "t1".into(),
                test_name: "Test 1".into(),
                category: TestCategory::Html,
                status: TestStatus::Pass,
                duration: Duration::from_millis(10),
                message: None,
                subtest_results: Vec::new(),
            },
            TestResult {
                test_id: "t2".into(),
                test_name: "Test 2".into(),
                category: TestCategory::Html,
                status: TestStatus::Fail,
                duration: Duration::from_millis(20),
                message: None,
                subtest_results: Vec::new(),
            },
        ];

        let report = TestReport::new(results, Duration::from_millis(30));
        assert_eq!(report.total(), 2);
        assert_eq!(report.passed(), 1);
        assert_eq!(report.failed(), 1);
    }

    #[test]
    fn test_compliance_score_grade() {
        let score = ComplianceScore {
            total_tests: 100,
            passed_tests: 95,
            percentage: 95.0,
            by_category: HashMap::new(),
        };
        assert_eq!(score.grade(), "A+");
    }
}
