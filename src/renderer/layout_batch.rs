//! Layout batching for efficient updates
//!
//! Groups multiple layout changes together to minimize recalculations:
//! - Batch DOM mutations
//! - Coalesce style changes
//! - Defer layout until needed

use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::dirty_tracking::{DirtyTracker, LayoutChange, Rect};

/// Layout batch configuration
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Maximum batch size before forced flush
    pub max_batch_size: usize,
    /// Maximum time to wait before flush
    pub max_batch_delay: Duration,
    /// Enable automatic batching
    pub auto_batch: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 100,
            max_batch_delay: Duration::from_millis(16), // ~60fps
            auto_batch: true,
        }
    }
}

/// Layout batch manager
pub struct LayoutBatcher {
    config: BatchConfig,
    dirty_tracker: DirtyTracker,
    batch_start: Option<Instant>,
    layout_count: usize,
    stats: BatchStats,
}

/// Batching statistics
#[derive(Debug, Clone, Default)]
pub struct BatchStats {
    pub total_batches: usize,
    pub total_changes: usize,
    pub changes_per_batch: f32,
    pub layouts_saved: usize,
}

impl LayoutBatcher {
    /// Create a new layout batcher
    pub fn new(config: BatchConfig) -> Self {
        Self {
            config,
            dirty_tracker: DirtyTracker::new(),
            batch_start: None,
            layout_count: 0,
            stats: BatchStats::default(),
        }
    }

    /// Queue a layout change
    pub fn queue(&mut self, change: LayoutChange) {
        if self.batch_start.is_none() {
            self.batch_start = Some(Instant::now());
        }

        self.dirty_tracker.queue_change(change);
        self.layout_count += 1;

        // Check if we should auto-flush
        if self.config.auto_batch && self.should_flush() {
            self.flush();
        }
    }

    /// Check if batch should be flushed
    fn should_flush(&self) -> bool {
        // Flush if batch is too large
        if self.layout_count >= self.config.max_batch_size {
            return true;
        }

        // Flush if batch has been waiting too long
        if let Some(start) = self.batch_start {
            if start.elapsed() >= self.config.max_batch_delay {
                return true;
            }
        }

        false
    }

    /// Flush the current batch
    pub fn flush(&mut self) -> BatchResult {
        let changes = self.dirty_tracker.pending_changes().len();
        let dirty_nodes = self.dirty_tracker.dirty_count();
        let dirty_region = self.dirty_tracker.combined_dirty_region();
        let needs_full = self.dirty_tracker.needs_full_layout();

        // Update stats
        self.stats.total_batches += 1;
        self.stats.total_changes += changes;
        self.stats.changes_per_batch = 
            self.stats.total_changes as f32 / self.stats.total_batches as f32;
        
        // Layouts saved = changes - 1 (we do one layout instead of many)
        if changes > 1 {
            self.stats.layouts_saved += changes - 1;
        }

        let result = BatchResult {
            changes_processed: changes,
            dirty_nodes,
            dirty_region,
            needs_full_layout: needs_full,
        };

        // Clear state
        self.dirty_tracker.clear();
        self.batch_start = None;
        self.layout_count = 0;

        result
    }

    /// Get dirty tracker
    pub fn dirty_tracker(&self) -> &DirtyTracker {
        &self.dirty_tracker
    }

    /// Get mutable dirty tracker
    pub fn dirty_tracker_mut(&mut self) -> &mut DirtyTracker {
        &mut self.dirty_tracker
    }

    /// Get statistics
    pub fn stats(&self) -> &BatchStats {
        &self.stats
    }

    /// Get pending change count
    pub fn pending_count(&self) -> usize {
        self.layout_count
    }

    /// Check if batch is empty
    pub fn is_empty(&self) -> bool {
        self.layout_count == 0
    }
}

/// Result of flushing a batch
#[derive(Debug, Clone)]
pub struct BatchResult {
    pub changes_processed: usize,
    pub dirty_nodes: usize,
    pub dirty_region: Rect,
    pub needs_full_layout: bool,
}

impl Default for LayoutBatcher {
    fn default() -> Self {
        Self::new(BatchConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_config_default() {
        let config = BatchConfig::default();
        assert_eq!(config.max_batch_size, 100);
        assert!(config.auto_batch);
    }

    #[test]
    fn test_layout_batcher_creation() {
        let batcher = LayoutBatcher::default();
        assert!(batcher.is_empty());
        assert_eq!(batcher.pending_count(), 0);
    }

    #[test]
    fn test_layout_batcher_queue() {
        let mut batcher = LayoutBatcher::new(BatchConfig {
            auto_batch: false,
            ..Default::default()
        });

        batcher.queue(LayoutChange::ContentChange(1));
        batcher.queue(LayoutChange::StyleChange(2));

        assert_eq!(batcher.pending_count(), 2);
        assert!(!batcher.is_empty());
    }

    #[test]
    fn test_layout_batcher_flush() {
        let mut batcher = LayoutBatcher::new(BatchConfig {
            auto_batch: false,
            ..Default::default()
        });

        batcher.queue(LayoutChange::ContentChange(1));
        batcher.queue(LayoutChange::ContentChange(2));
        batcher.queue(LayoutChange::ContentChange(3));

        let result = batcher.flush();

        assert_eq!(result.changes_processed, 3);
        assert!(batcher.is_empty());
    }

    #[test]
    fn test_layout_batcher_stats() {
        let mut batcher = LayoutBatcher::new(BatchConfig {
            auto_batch: false,
            ..Default::default()
        });

        batcher.queue(LayoutChange::ContentChange(1));
        batcher.queue(LayoutChange::ContentChange(2));
        batcher.flush();

        let stats = batcher.stats();
        assert_eq!(stats.total_batches, 1);
        assert_eq!(stats.total_changes, 2);
        assert_eq!(stats.layouts_saved, 1);
    }

    #[test]
    fn test_layout_batcher_auto_flush_size() {
        let mut batcher = LayoutBatcher::new(BatchConfig {
            max_batch_size: 3,
            auto_batch: true,
            ..Default::default()
        });

        batcher.queue(LayoutChange::ContentChange(1));
        batcher.queue(LayoutChange::ContentChange(2));
        // Third change should trigger auto-flush
        batcher.queue(LayoutChange::ContentChange(3));

        // After auto-flush, should be empty
        assert!(batcher.is_empty());
    }
}

