//! Resource prioritization for optimal loading
//!
//! Implements priority hints similar to Chromium's resource scheduling.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// Resource type for prioritization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    /// Main HTML document
    Document,
    /// CSS stylesheet (render blocking)
    Stylesheet,
    /// JavaScript (potentially blocking)
    Script,
    /// Web font
    Font,
    /// Image
    Image,
    /// Video
    Video,
    /// Audio
    Audio,
    /// XMLHttpRequest/Fetch
    Xhr,
    /// Other resources
    Other,
}

impl ResourceType {
    /// Get default priority for resource type
    pub fn default_priority(&self) -> Priority {
        match self {
            Self::Document => Priority::VeryHigh,
            Self::Stylesheet => Priority::VeryHigh,
            Self::Font => Priority::High,
            Self::Script => Priority::High,
            Self::Xhr => Priority::High,
            Self::Image => Priority::Low,
            Self::Video => Priority::VeryLow,
            Self::Audio => Priority::VeryLow,
            Self::Other => Priority::Low,
        }
    }

    /// Check if this resource is render blocking
    pub fn is_render_blocking(&self) -> bool {
        matches!(self, Self::Document | Self::Stylesheet)
    }

    /// Parse resource type from content-type header
    pub fn from_content_type(content_type: &str) -> Self {
        let ct = content_type.to_lowercase();

        if ct.contains("html") {
            Self::Document
        } else if ct.contains("css") {
            Self::Stylesheet
        } else if ct.contains("javascript") || ct.contains("ecmascript") {
            Self::Script
        } else if ct.contains("font") || ct.contains("woff") || ct.contains("ttf") {
            Self::Font
        } else if ct.contains("image") {
            Self::Image
        } else if ct.contains("video") {
            Self::Video
        } else if ct.contains("audio") {
            Self::Audio
        } else {
            Self::Other
        }
    }

    /// Parse from URL extension
    pub fn from_url(url: &str) -> Self {
        let lower = url.to_lowercase();

        if lower.ends_with(".html") || lower.ends_with(".htm") {
            Self::Document
        } else if lower.ends_with(".css") {
            Self::Stylesheet
        } else if lower.ends_with(".js") || lower.ends_with(".mjs") {
            Self::Script
        } else if lower.ends_with(".woff2") || lower.ends_with(".woff") ||
                  lower.ends_with(".ttf") || lower.ends_with(".otf") {
            Self::Font
        } else if lower.ends_with(".png") || lower.ends_with(".jpg") ||
                  lower.ends_with(".jpeg") || lower.ends_with(".gif") ||
                  lower.ends_with(".webp") || lower.ends_with(".svg") {
            Self::Image
        } else if lower.ends_with(".mp4") || lower.ends_with(".webm") {
            Self::Video
        } else if lower.ends_with(".mp3") || lower.ends_with(".ogg") {
            Self::Audio
        } else {
            Self::Other
        }
    }
}

/// Resource priority levels (similar to Chrome's)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    /// Highest priority (main document)
    VeryHigh = 0,
    /// High priority (CSS, fonts)
    High = 1,
    /// Medium priority (sync scripts)
    Medium = 2,
    /// Low priority (images)
    Low = 3,
    /// Very low (video, audio)
    VeryLow = 4,
    /// Idle (prefetch)
    Idle = 5,
}

/// A prioritized resource request
#[derive(Debug, Clone)]
pub struct PrioritizedRequest {
    /// URL to fetch
    pub url: String,
    /// Resource type
    pub resource_type: ResourceType,
    /// Priority level
    pub priority: Priority,
    /// Whether this is async (non-blocking)
    pub is_async: bool,
    /// Whether this is deferred
    pub is_deferred: bool,
    /// Importance hint from HTML
    pub importance_hint: Option<ImportanceHint>,
}

/// Importance hint from fetchpriority attribute
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportanceHint {
    High,
    Low,
    Auto,
}

impl PrioritizedRequest {
    /// Create a new request with automatic priority
    pub fn new(url: &str) -> Self {
        let resource_type = ResourceType::from_url(url);
        Self {
            url: url.to_string(),
            resource_type,
            priority: resource_type.default_priority(),
            is_async: false,
            is_deferred: false,
            importance_hint: None,
        }
    }

    /// Apply importance hint from HTML
    pub fn with_importance(mut self, hint: ImportanceHint) -> Self {
        self.importance_hint = Some(hint);
        self.priority = self.calculate_final_priority();
        self
    }

    /// Mark as async loading
    pub fn async_loading(mut self) -> Self {
        self.is_async = true;
        self.priority = self.calculate_final_priority();
        self
    }

    /// Mark as deferred loading
    pub fn deferred(mut self) -> Self {
        self.is_deferred = true;
        self.priority = self.calculate_final_priority();
        self
    }

    /// Calculate final priority based on all factors
    fn calculate_final_priority(&self) -> Priority {
        let base = self.resource_type.default_priority();

        // Apply importance hint
        let with_hint = match self.importance_hint {
            Some(ImportanceHint::High) => match base {
                Priority::Low | Priority::VeryLow => Priority::Medium,
                Priority::Medium => Priority::High,
                p => p,
            },
            Some(ImportanceHint::Low) => match base {
                Priority::High | Priority::VeryHigh => Priority::Medium,
                Priority::Medium => Priority::Low,
                p => p,
            },
            _ => base,
        };

        // Async/defer scripts are lower priority
        if self.is_async || self.is_deferred {
            match with_hint {
                Priority::VeryHigh => Priority::High,
                Priority::High => Priority::Medium,
                p => p,
            }
        } else {
            with_hint
        }
    }
}

impl PartialEq for PrioritizedRequest {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}

impl Eq for PrioritizedRequest {}

impl PartialOrd for PrioritizedRequest {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrioritizedRequest {
    fn cmp(&self, other: &Self) -> Ordering {
        // Lower priority value = higher priority in heap
        other.priority.cmp(&self.priority)
    }
}

/// Priority queue for resource loading
pub struct ResourceQueue {
    /// Priority heap of pending requests
    queue: BinaryHeap<PrioritizedRequest>,
    /// Maximum concurrent requests
    max_concurrent: usize,
    /// Currently in-flight requests
    in_flight: usize,
}

impl ResourceQueue {
    /// Create a new resource queue
    pub fn new() -> Self {
        Self {
            queue: BinaryHeap::new(),
            max_concurrent: 6, // Typical browser limit per domain
            in_flight: 0,
        }
    }

    /// Add a request to the queue
    pub fn push(&mut self, request: PrioritizedRequest) {
        self.queue.push(request);
    }

    /// Get next requests to execute (up to available slots)
    pub fn next_batch(&mut self) -> Vec<PrioritizedRequest> {
        let available = self.max_concurrent.saturating_sub(self.in_flight);
        let mut batch = Vec::with_capacity(available);

        for _ in 0..available {
            if let Some(req) = self.queue.pop() {
                batch.push(req);
            } else {
                break;
            }
        }

        self.in_flight += batch.len();
        batch
    }

    /// Mark a request as complete
    pub fn complete(&mut self) {
        self.in_flight = self.in_flight.saturating_sub(1);
    }

    /// Get queue length
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

impl Default for ResourceQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_type_from_url() {
        assert_eq!(ResourceType::from_url("style.css"), ResourceType::Stylesheet);
        assert_eq!(ResourceType::from_url("app.js"), ResourceType::Script);
        assert_eq!(ResourceType::from_url("logo.png"), ResourceType::Image);
    }

    #[test]
    fn test_priority_ordering() {
        let css = PrioritizedRequest::new("style.css");
        let img = PrioritizedRequest::new("logo.png");

        assert!(css.priority < img.priority); // Lower = higher priority
    }

    #[test]
    fn test_importance_hint() {
        let img = PrioritizedRequest::new("hero.jpg")
            .with_importance(ImportanceHint::High);

        assert_eq!(img.priority, Priority::Medium); // Boosted from Low
    }

    #[test]
    fn test_resource_queue() {
        let mut queue = ResourceQueue::new();

        queue.push(PrioritizedRequest::new("image.jpg"));
        queue.push(PrioritizedRequest::new("style.css"));
        queue.push(PrioritizedRequest::new("script.js"));

        let batch = queue.next_batch();

        // CSS should come first (highest priority)
        assert_eq!(batch[0].resource_type, ResourceType::Stylesheet);
    }
}