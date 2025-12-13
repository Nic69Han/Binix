//! Dirty region tracking for incremental layout
//!
//! Tracks which parts of the layout tree need recalculation:
//! - Dirty nodes: Nodes that have changed
//! - Dirty regions: Screen areas that need repaint
//! - Layout batching: Group multiple changes for efficiency

use std::collections::{HashSet, VecDeque};

/// Rectangle representing a region
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    /// Create a new rectangle
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Create an empty rectangle
    pub fn empty() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }

    /// Check if rectangle is empty
    pub fn is_empty(&self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }

    /// Check if rectangles intersect
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }

    /// Union of two rectangles
    pub fn union(&self, other: &Rect) -> Rect {
        if self.is_empty() {
            return *other;
        }
        if other.is_empty() {
            return *self;
        }

        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        let right = (self.x + self.width).max(other.x + other.width);
        let bottom = (self.y + self.height).max(other.y + other.height);

        Rect::new(x, y, right - x, bottom - y)
    }

    /// Area of the rectangle
    pub fn area(&self) -> f32 {
        self.width * self.height
    }
}

impl Default for Rect {
    fn default() -> Self {
        Self::empty()
    }
}

/// Dirty tracking for layout invalidation
pub struct DirtyTracker {
    /// Set of dirty node IDs
    dirty_nodes: HashSet<u64>,
    /// Dirty regions that need repaint
    dirty_regions: Vec<Rect>,
    /// Pending layout changes
    pending_changes: VecDeque<LayoutChange>,
    /// Whether full layout is needed
    needs_full_layout: bool,
}

/// Type of layout change
#[derive(Debug, Clone)]
pub enum LayoutChange {
    /// Node content changed
    ContentChange(u64),
    /// Node style changed
    StyleChange(u64),
    /// Node added
    NodeAdded(u64),
    /// Node removed
    NodeRemoved(u64),
    /// Node moved
    NodeMoved(u64),
    /// Viewport resized
    ViewportResize(f32, f32),
}

impl DirtyTracker {
    /// Create a new dirty tracker
    pub fn new() -> Self {
        Self {
            dirty_nodes: HashSet::new(),
            dirty_regions: Vec::new(),
            pending_changes: VecDeque::new(),
            needs_full_layout: false,
        }
    }

    /// Mark a node as dirty
    pub fn mark_dirty(&mut self, node_id: u64) {
        self.dirty_nodes.insert(node_id);
    }

    /// Mark a region as dirty
    pub fn mark_region_dirty(&mut self, region: Rect) {
        // Try to merge with existing regions
        for existing in &mut self.dirty_regions {
            if existing.intersects(&region) {
                *existing = existing.union(&region);
                return;
            }
        }
        self.dirty_regions.push(region);
    }

    /// Queue a layout change
    pub fn queue_change(&mut self, change: LayoutChange) {
        match &change {
            LayoutChange::ViewportResize(_, _) => {
                self.needs_full_layout = true;
            }
            LayoutChange::ContentChange(id)
            | LayoutChange::StyleChange(id)
            | LayoutChange::NodeAdded(id)
            | LayoutChange::NodeRemoved(id)
            | LayoutChange::NodeMoved(id) => {
                self.dirty_nodes.insert(*id);
            }
        }
        self.pending_changes.push_back(change);
    }

    /// Check if a node is dirty
    pub fn is_dirty(&self, node_id: u64) -> bool {
        self.dirty_nodes.contains(&node_id)
    }

    /// Check if any nodes are dirty
    pub fn has_dirty_nodes(&self) -> bool {
        !self.dirty_nodes.is_empty()
    }

    /// Check if full layout is needed
    pub fn needs_full_layout(&self) -> bool {
        self.needs_full_layout
    }

    /// Get dirty nodes
    pub fn dirty_nodes(&self) -> &HashSet<u64> {
        &self.dirty_nodes
    }

    /// Get dirty regions
    pub fn dirty_regions(&self) -> &[Rect] {
        &self.dirty_regions
    }

    /// Get pending changes
    pub fn pending_changes(&self) -> &VecDeque<LayoutChange> {
        &self.pending_changes
    }

    /// Clear all dirty state
    pub fn clear(&mut self) {
        self.dirty_nodes.clear();
        self.dirty_regions.clear();
        self.pending_changes.clear();
        self.needs_full_layout = false;
    }

    /// Get combined dirty region
    pub fn combined_dirty_region(&self) -> Rect {
        self.dirty_regions
            .iter()
            .fold(Rect::empty(), |acc, r| acc.union(r))
    }

    /// Dirty node count
    pub fn dirty_count(&self) -> usize {
        self.dirty_nodes.len()
    }
}

impl Default for DirtyTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_creation() {
        let rect = Rect::new(10.0, 20.0, 100.0, 50.0);
        assert_eq!(rect.x, 10.0);
        assert_eq!(rect.area(), 5000.0);
    }

    #[test]
    fn test_rect_intersects() {
        let r1 = Rect::new(0.0, 0.0, 100.0, 100.0);
        let r2 = Rect::new(50.0, 50.0, 100.0, 100.0);
        let r3 = Rect::new(200.0, 200.0, 50.0, 50.0);

        assert!(r1.intersects(&r2));
        assert!(!r1.intersects(&r3));
    }

    #[test]
    fn test_rect_union() {
        let r1 = Rect::new(0.0, 0.0, 50.0, 50.0);
        let r2 = Rect::new(25.0, 25.0, 50.0, 50.0);
        let union = r1.union(&r2);

        assert_eq!(union.x, 0.0);
        assert_eq!(union.y, 0.0);
        assert_eq!(union.width, 75.0);
        assert_eq!(union.height, 75.0);
    }

    #[test]
    fn test_dirty_tracker_mark_dirty() {
        let mut tracker = DirtyTracker::new();
        tracker.mark_dirty(1);
        tracker.mark_dirty(2);

        assert!(tracker.is_dirty(1));
        assert!(tracker.is_dirty(2));
        assert!(!tracker.is_dirty(3));
        assert_eq!(tracker.dirty_count(), 2);
    }

    #[test]
    fn test_dirty_tracker_queue_change() {
        let mut tracker = DirtyTracker::new();
        tracker.queue_change(LayoutChange::ContentChange(1));
        tracker.queue_change(LayoutChange::StyleChange(2));

        assert!(tracker.has_dirty_nodes());
        assert_eq!(tracker.pending_changes().len(), 2);
    }

    #[test]
    fn test_dirty_tracker_viewport_resize() {
        let mut tracker = DirtyTracker::new();
        tracker.queue_change(LayoutChange::ViewportResize(1920.0, 1080.0));

        assert!(tracker.needs_full_layout());
    }

    #[test]
    fn test_dirty_tracker_clear() {
        let mut tracker = DirtyTracker::new();
        tracker.mark_dirty(1);
        tracker.mark_region_dirty(Rect::new(0.0, 0.0, 100.0, 100.0));
        tracker.clear();

        assert!(!tracker.has_dirty_nodes());
        assert!(tracker.dirty_regions().is_empty());
    }
}
