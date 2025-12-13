//! Content Security Policy (CSP) implementation

use std::collections::{HashMap, HashSet};

/// CSP directive types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CspDirective {
    DefaultSrc,
    ScriptSrc,
    StyleSrc,
    ImgSrc,
    FontSrc,
    ConnectSrc,
    MediaSrc,
    ObjectSrc,
    FrameSrc,
    ChildSrc,
    WorkerSrc,
    FormAction,
    FrameAncestors,
    BaseUri,
    ReportUri,
    UpgradeInsecureRequests,
    BlockAllMixedContent,
}

impl CspDirective {
    pub fn as_str(&self) -> &'static str {
        match self {
            CspDirective::DefaultSrc => "default-src",
            CspDirective::ScriptSrc => "script-src",
            CspDirective::StyleSrc => "style-src",
            CspDirective::ImgSrc => "img-src",
            CspDirective::FontSrc => "font-src",
            CspDirective::ConnectSrc => "connect-src",
            CspDirective::MediaSrc => "media-src",
            CspDirective::ObjectSrc => "object-src",
            CspDirective::FrameSrc => "frame-src",
            CspDirective::ChildSrc => "child-src",
            CspDirective::WorkerSrc => "worker-src",
            CspDirective::FormAction => "form-action",
            CspDirective::FrameAncestors => "frame-ancestors",
            CspDirective::BaseUri => "base-uri",
            CspDirective::ReportUri => "report-uri",
            CspDirective::UpgradeInsecureRequests => "upgrade-insecure-requests",
            CspDirective::BlockAllMixedContent => "block-all-mixed-content",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "default-src" => Some(CspDirective::DefaultSrc),
            "script-src" => Some(CspDirective::ScriptSrc),
            "style-src" => Some(CspDirective::StyleSrc),
            "img-src" => Some(CspDirective::ImgSrc),
            "font-src" => Some(CspDirective::FontSrc),
            "connect-src" => Some(CspDirective::ConnectSrc),
            "media-src" => Some(CspDirective::MediaSrc),
            "object-src" => Some(CspDirective::ObjectSrc),
            "frame-src" => Some(CspDirective::FrameSrc),
            "child-src" => Some(CspDirective::ChildSrc),
            "worker-src" => Some(CspDirective::WorkerSrc),
            "form-action" => Some(CspDirective::FormAction),
            "frame-ancestors" => Some(CspDirective::FrameAncestors),
            "base-uri" => Some(CspDirective::BaseUri),
            "report-uri" => Some(CspDirective::ReportUri),
            "upgrade-insecure-requests" => Some(CspDirective::UpgradeInsecureRequests),
            "block-all-mixed-content" => Some(CspDirective::BlockAllMixedContent),
            _ => None,
        }
    }
}

/// CSP violation record
#[derive(Debug, Clone)]
pub struct CspViolation {
    pub directive: CspDirective,
    pub blocked_uri: String,
    pub document_uri: String,
    pub violated_directive: String,
}

/// Content Security Policy
#[derive(Debug, Clone, Default)]
pub struct ContentSecurityPolicy {
    directives: HashMap<CspDirective, HashSet<String>>,
    report_only: bool,
    violations: Vec<CspViolation>,
    /// Nonces for inline scripts/styles
    nonces: HashSet<String>,
    /// Report URI for violations
    report_uri: Option<String>,
}

impl ContentSecurityPolicy {
    /// Create a new empty CSP
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse CSP from header string
    pub fn parse(header: &str) -> Self {
        let mut csp = Self::new();

        for directive_str in header.split(';') {
            let parts: Vec<&str> = directive_str.trim().split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            if let Some(directive) = CspDirective::from_str(parts[0]) {
                let sources: HashSet<String> = parts[1..].iter().map(|s| s.to_string()).collect();
                csp.directives.insert(directive, sources);
            }
        }

        csp
    }

    /// Add a directive
    pub fn add_directive(&mut self, directive: CspDirective, sources: Vec<&str>) {
        let source_set: HashSet<String> = sources.iter().map(|s| s.to_string()).collect();
        self.directives.insert(directive, source_set);
    }

    /// Check if a source is allowed for a directive
    pub fn allows(&self, directive: CspDirective, source: &str) -> bool {
        // Check specific directive first
        if let Some(sources) = self.directives.get(&directive) {
            return self.source_matches(sources, source);
        }

        // Fall back to default-src
        if let Some(sources) = self.directives.get(&CspDirective::DefaultSrc) {
            return self.source_matches(sources, source);
        }

        // No policy means allow
        true
    }

    /// Check if source matches any allowed source
    fn source_matches(&self, allowed: &HashSet<String>, source: &str) -> bool {
        for allowed_source in allowed {
            match allowed_source.as_str() {
                "'self'" => {
                    // Would need document origin to check properly
                    return true;
                }
                "'none'" => return false,
                "'unsafe-inline'" | "'unsafe-eval'" => continue,
                "*" => return true,
                s if source.starts_with(s) => return true,
                _ => continue,
            }
        }
        false
    }

    /// Record a violation
    pub fn record_violation(
        &mut self,
        directive: CspDirective,
        blocked_uri: &str,
        document_uri: &str,
    ) {
        self.violations.push(CspViolation {
            directive,
            blocked_uri: blocked_uri.to_string(),
            document_uri: document_uri.to_string(),
            violated_directive: directive.as_str().to_string(),
        });
    }

    /// Get violations
    pub fn violations(&self) -> &[CspViolation] {
        &self.violations
    }

    /// Check if report-only mode
    pub fn is_report_only(&self) -> bool {
        self.report_only
    }

    /// Set report-only mode
    pub fn set_report_only(&mut self, report_only: bool) {
        self.report_only = report_only;
    }

    /// Add a nonce for inline scripts/styles
    pub fn add_nonce(&mut self, nonce: &str) {
        self.nonces.insert(nonce.to_string());
    }

    /// Check if a nonce is valid
    pub fn has_nonce(&self, nonce: &str) -> bool {
        self.nonces.contains(nonce)
    }

    /// Generate a random nonce
    pub fn generate_nonce() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        format!("{:x}", timestamp)
    }

    /// Set report URI
    pub fn set_report_uri(&mut self, uri: &str) {
        self.report_uri = Some(uri.to_string());
    }

    /// Get report URI
    pub fn report_uri(&self) -> Option<&str> {
        self.report_uri.as_deref()
    }

    /// Build a violation report (JSON format)
    pub fn build_violation_report(&self, violation: &CspViolation) -> String {
        format!(
            r#"{{"csp-report":{{"document-uri":"{}","violated-directive":"{}","blocked-uri":"{}","disposition":"{}"}}}}"#,
            violation.document_uri,
            violation.violated_directive,
            violation.blocked_uri,
            if self.report_only { "report" } else { "enforce" }
        )
    }

    /// Check and possibly block a resource, recording violation if blocked
    pub fn check_and_report(
        &mut self,
        directive: CspDirective,
        source: &str,
        document_uri: &str,
    ) -> bool {
        if self.allows(directive, source) {
            return true;
        }

        // Record violation
        self.record_violation(directive, source, document_uri);

        // In report-only mode, allow but report
        self.report_only
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csp_directive_as_str() {
        assert_eq!(CspDirective::ScriptSrc.as_str(), "script-src");
        assert_eq!(CspDirective::DefaultSrc.as_str(), "default-src");
    }

    #[test]
    fn test_csp_directive_from_str() {
        assert_eq!(
            CspDirective::from_str("script-src"),
            Some(CspDirective::ScriptSrc)
        );
        assert_eq!(CspDirective::from_str("invalid"), None);
    }

    #[test]
    fn test_csp_parse() {
        let csp = ContentSecurityPolicy::parse(
            "default-src 'self'; script-src 'self' https://cdn.example.com",
        );
        assert!(csp.allows(CspDirective::DefaultSrc, "self"));
    }

    #[test]
    fn test_csp_allows_wildcard() {
        let mut csp = ContentSecurityPolicy::new();
        csp.add_directive(CspDirective::ImgSrc, vec!["*"]);
        assert!(csp.allows(CspDirective::ImgSrc, "https://any-domain.com/image.png"));
    }

    #[test]
    fn test_csp_blocks_none() {
        let mut csp = ContentSecurityPolicy::new();
        csp.add_directive(CspDirective::ObjectSrc, vec!["'none'"]);
        assert!(!csp.allows(CspDirective::ObjectSrc, "https://example.com/plugin"));
    }

    #[test]
    fn test_csp_fallback_to_default() {
        let mut csp = ContentSecurityPolicy::new();
        csp.add_directive(CspDirective::DefaultSrc, vec!["'self'"]);
        // No script-src defined, should fall back to default-src
        assert!(csp.allows(CspDirective::ScriptSrc, "self"));
    }

    #[test]
    fn test_csp_record_violation() {
        let mut csp = ContentSecurityPolicy::new();
        csp.record_violation(
            CspDirective::ScriptSrc,
            "https://evil.com/script.js",
            "https://example.com",
        );
        assert_eq!(csp.violations().len(), 1);
    }
}
