//! Process sandboxing for security

use std::collections::HashSet;

/// Sandbox policy levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyLevel {
    /// No restrictions (for browser process)
    None,
    /// Minimal restrictions
    Relaxed,
    /// Standard restrictions
    Standard,
    /// Maximum restrictions
    Strict,
}

/// Sandbox policy configuration
#[derive(Debug, Clone)]
pub struct SandboxPolicy {
    pub level: PolicyLevel,
    pub allow_network: bool,
    pub allow_filesystem: bool,
    pub allow_ipc: bool,
    pub allow_clipboard: bool,
    pub allow_audio: bool,
    pub allow_video: bool,
    pub allow_webgl: bool,
    pub allowed_origins: HashSet<String>,
}

impl SandboxPolicy {
    /// Create a strict policy
    pub fn strict() -> Self {
        Self {
            level: PolicyLevel::Strict,
            allow_network: false,
            allow_filesystem: false,
            allow_ipc: true,
            allow_clipboard: false,
            allow_audio: false,
            allow_video: false,
            allow_webgl: true,
            allowed_origins: HashSet::new(),
        }
    }

    /// Create a standard policy
    pub fn standard() -> Self {
        Self {
            level: PolicyLevel::Standard,
            allow_network: true,
            allow_filesystem: false,
            allow_ipc: true,
            allow_clipboard: true,
            allow_audio: true,
            allow_video: true,
            allow_webgl: true,
            allowed_origins: HashSet::new(),
        }
    }

    /// Create a relaxed policy
    pub fn relaxed() -> Self {
        Self {
            level: PolicyLevel::Relaxed,
            allow_network: true,
            allow_filesystem: true,
            allow_ipc: true,
            allow_clipboard: true,
            allow_audio: true,
            allow_video: true,
            allow_webgl: true,
            allowed_origins: HashSet::new(),
        }
    }

    /// Allow a specific origin
    pub fn allow_origin(&mut self, origin: &str) {
        self.allowed_origins.insert(origin.to_string());
    }

    /// Check if origin is allowed
    pub fn is_origin_allowed(&self, origin: &str) -> bool {
        self.allowed_origins.is_empty() || self.allowed_origins.contains(origin)
    }
}

/// Sandbox for process isolation
pub struct Sandbox {
    policy: SandboxPolicy,
    active: bool,
    violations: Vec<SecurityViolation>,
}

/// Security violation record
#[derive(Debug, Clone)]
pub struct SecurityViolation {
    pub violation_type: ViolationType,
    pub description: String,
    pub blocked: bool,
}

/// Types of security violations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViolationType {
    NetworkAccess,
    FileSystemAccess,
    ClipboardAccess,
    CrossOriginAccess,
    UnsafeScript,
    PrivilegeEscalation,
}

impl Sandbox {
    /// Create a new sandbox
    pub fn new(policy: SandboxPolicy) -> Self {
        Self {
            policy,
            active: true,
            violations: Vec::new(),
        }
    }

    /// Check if network access is allowed
    pub fn check_network(&mut self, url: &str) -> bool {
        if !self.active || self.policy.allow_network {
            return true;
        }

        self.record_violation(
            ViolationType::NetworkAccess,
            format!("Network access to: {}", url),
        );
        false
    }

    /// Check if filesystem access is allowed
    pub fn check_filesystem(&mut self, path: &str) -> bool {
        if !self.active || self.policy.allow_filesystem {
            return true;
        }

        self.record_violation(
            ViolationType::FileSystemAccess,
            format!("Filesystem access to: {}", path),
        );
        false
    }

    /// Check cross-origin access
    pub fn check_cross_origin(&mut self, origin: &str) -> bool {
        if !self.active || self.policy.is_origin_allowed(origin) {
            return true;
        }

        self.record_violation(
            ViolationType::CrossOriginAccess,
            format!("Cross-origin access to: {}", origin),
        );
        false
    }

    /// Record a violation
    fn record_violation(&mut self, violation_type: ViolationType, description: String) {
        self.violations.push(SecurityViolation {
            violation_type,
            description,
            blocked: true,
        });
    }

    /// Get violations
    pub fn violations(&self) -> &[SecurityViolation] {
        &self.violations
    }

    /// Get policy
    pub fn policy(&self) -> &SandboxPolicy {
        &self.policy
    }

    /// Enable/disable sandbox
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    /// Check if sandbox is active
    pub fn is_active(&self) -> bool {
        self.active
    }
}

impl Default for Sandbox {
    fn default() -> Self {
        Self::new(SandboxPolicy::standard())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strict_policy() {
        let mut sandbox = Sandbox::new(SandboxPolicy::strict());
        assert!(!sandbox.check_network("https://example.com"));
        assert!(!sandbox.check_filesystem("/etc/passwd"));
        assert_eq!(sandbox.violations().len(), 2);
    }

    #[test]
    fn test_standard_policy() {
        let mut sandbox = Sandbox::new(SandboxPolicy::standard());
        assert!(sandbox.check_network("https://example.com"));
        assert!(!sandbox.check_filesystem("/etc/passwd"));
    }

    #[test]
    fn test_allowed_origins() {
        let mut policy = SandboxPolicy::strict();
        policy.allow_origin("https://trusted.com");
        let mut sandbox = Sandbox::new(policy);

        assert!(sandbox.check_cross_origin("https://trusted.com"));
        assert!(!sandbox.check_cross_origin("https://untrusted.com"));
    }

    #[test]
    fn test_sandbox_disable() {
        let mut sandbox = Sandbox::new(SandboxPolicy::strict());
        sandbox.set_active(false);
        assert!(sandbox.check_network("https://example.com"));
    }
}
