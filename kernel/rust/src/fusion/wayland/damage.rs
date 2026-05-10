//! Damage region tracking and optimization
//!
//! Efficiently tracks and merges damage rectangles to minimize compositor work.
//! Damage rectangles are clipped to surface bounds and overlapping regions merged.

use alloc::vec::Vec;

/// Damage rectangle [x, y, width, height]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DamageRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl DamageRect {
    /// Create a new damage rectangle
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { x, y, width, height }
    }

    /// Full surface damage (damage everything)
    pub fn full(width: i32, height: i32) -> Self {
        Self { x: 0, y: 0, width, height }
    }

    /// Right edge (x + width)
    pub fn right(&self) -> i32 {
        self.x.saturating_add(self.width)
    }

    /// Bottom edge (y + height)
    pub fn bottom(&self) -> i32 {
        self.y.saturating_add(self.height)
    }

    /// Check if two rectangles overlap
    pub fn overlaps(&self, other: &DamageRect) -> bool {
        self.x < other.right()
            && other.x < self.right()
            && self.y < other.bottom()
            && other.y < self.bottom()
    }

    /// Check if two rectangles are adjacent (share an edge)
    pub fn adjacent(&self, other: &DamageRect) -> bool {
        // Horizontally adjacent
        if self.y == other.y && self.height == other.height {
            if self.right() == other.x || other.right() == self.x {
                return true;
            }
        }
        // Vertically adjacent
        if self.x == other.x && self.width == other.width {
            if self.bottom() == other.y || other.bottom() == self.y {
                return true;
            }
        }
        false
    }

    /// Compute bounding box of two rectangles
    pub fn bounding_box(&self, other: &DamageRect) -> DamageRect {
        let min_x = self.x.min(other.x);
        let min_y = self.y.min(other.y);
        let max_x = self.right().max(other.right());
        let max_y = self.bottom().max(other.bottom());

        DamageRect {
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
        }
    }

    /// Clip this rectangle to bounds
    pub fn clip(&self, bounds: &DamageRect) -> Option<DamageRect> {
        let x = self.x.max(bounds.x);
        let y = self.y.max(bounds.y);
        let right = self.right().min(bounds.right());
        let bottom = self.bottom().min(bounds.bottom());

        if x < right && y < bottom {
            Some(DamageRect {
                x,
                y,
                width: right - x,
                height: bottom - y,
            })
        } else {
            None
        }
    }
}

/// Damage tracker for efficient region updates
#[derive(Debug, Clone)]
pub struct DamageTracker {
    /// Accumulated damage rectangles
    regions: Vec<DamageRect>,
    /// Surface bounds (width, height)
    bounds: (i32, i32),
    /// Flag indicating entire surface is damaged
    full_damage: bool,
}

impl DamageTracker {
    /// Create a new damage tracker with surface bounds
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            regions: Vec::new(),
            bounds: (width, height),
            full_damage: false,
        }
    }

    /// Add damage rectangle
    pub fn add_damage(&mut self, rect: DamageRect) {
        if self.full_damage {
            return; // Already damaged everything
        }

        // Check if damage covers entire surface
        if rect.x <= 0 && rect.y <= 0 && rect.width >= self.bounds.0 && rect.height >= self.bounds.1 {
            self.full_damage = true;
            self.regions.clear();
            return;
        }

        // Clip to bounds
        let bounds_rect = DamageRect::full(self.bounds.0, self.bounds.1);
        if let Some(clipped) = rect.clip(&bounds_rect) {
            self.regions.push(clipped);
        }
    }

    /// Merge overlapping and adjacent rectangles
    pub fn optimize(&mut self) {
        if self.full_damage || self.regions.is_empty() {
            return;
        }

        let mut merged = Vec::new();
        let mut remaining: Vec<DamageRect> = self.regions.drain(..).collect();

        while let Some(current) = remaining.pop() {
            let mut merged_rect = current;
            let mut did_merge = true;

            // Try to merge with other rectangles
            while did_merge {
                did_merge = false;
                remaining.retain(|other| {
                    if merged_rect.overlaps(other) || merged_rect.adjacent(other) {
                        merged_rect = merged_rect.bounding_box(other);
                        did_merge = true;
                        false
                    } else {
                        true
                    }
                });
            }

            merged.push(merged_rect);
        }

        self.regions = merged;
    }

    /// Get damage regions
    pub fn regions(&self) -> &[DamageRect] {
        &self.regions
    }

    /// Check if entire surface is damaged
    pub fn is_full_damage(&self) -> bool {
        self.full_damage
    }

    /// Clear all damage
    pub fn clear(&mut self) {
        self.regions.clear();
        self.full_damage = false;
    }

    /// Reset surface bounds
    pub fn set_bounds(&mut self, width: i32, height: i32) {
        self.bounds = (width, height);
        // Re-clip all existing damage to new bounds
        if !self.full_damage && !self.regions.is_empty() {
            let bounds_rect = DamageRect::full(width, height);
            self.regions.retain_mut(|rect| {
                if let Some(clipped) = rect.clip(&bounds_rect) {
                    *rect = clipped;
                    true
                } else {
                    false
                }
            });
        }
    }

    /// Get total damage as a single bounding box
    pub fn bounding_box(&self) -> Option<DamageRect> {
        if self.full_damage {
            Some(DamageRect::full(self.bounds.0, self.bounds.1))
        } else if self.regions.is_empty() {
            None
        } else {
            let mut bbox = self.regions[0];
            for rect in &self.regions[1..] {
                bbox = bbox.bounding_box(rect);
            }
            Some(bbox)
        }
    }
}

impl Default for DamageTracker {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_damage_rect_overlap() {
        let r1 = DamageRect::new(0, 0, 100, 100);
        let r2 = DamageRect::new(50, 50, 100, 100);
        let r3 = DamageRect::new(200, 200, 100, 100);

        assert!(r1.overlaps(&r2));
        assert!(r2.overlaps(&r1));
        assert!(!r1.overlaps(&r3));
    }

    #[test]
    fn test_damage_rect_adjacent() {
        let r1 = DamageRect::new(0, 0, 100, 100);
        let r2 = DamageRect::new(100, 0, 100, 100);
        let r3 = DamageRect::new(200, 0, 100, 100);

        assert!(r1.adjacent(&r2));
        assert!(r2.adjacent(&r1));
        assert!(!r1.adjacent(&r3));
    }

    #[test]
    fn test_damage_rect_bounding_box() {
        let r1 = DamageRect::new(0, 0, 100, 100);
        let r2 = DamageRect::new(50, 50, 100, 100);
        let bbox = r1.bounding_box(&r2);

        assert_eq!(bbox.x, 0);
        assert_eq!(bbox.y, 0);
        assert_eq!(bbox.width, 150);
        assert_eq!(bbox.height, 150);
    }

    #[test]
    fn test_damage_rect_clip() {
        let rect = DamageRect::new(50, 50, 100, 100);
        let bounds = DamageRect::new(0, 0, 100, 100);
        let clipped = rect.clip(&bounds);

        assert!(clipped.is_some());
        let c = clipped.unwrap();
        assert_eq!(c.x, 50);
        assert_eq!(c.y, 50);
        assert_eq!(c.width, 50);
        assert_eq!(c.height, 50);
    }

    #[test]
    fn test_damage_rect_clip_outside_bounds() {
        let rect = DamageRect::new(200, 200, 100, 100);
        let bounds = DamageRect::new(0, 0, 100, 100);
        let clipped = rect.clip(&bounds);

        assert!(clipped.is_none());
    }

    #[test]
    fn test_damage_tracker_single_rect() {
        let mut tracker = DamageTracker::new(640, 480);
        tracker.add_damage(DamageRect::new(10, 10, 100, 100));

        assert!(!tracker.is_full_damage());
        assert_eq!(tracker.regions().len(), 1);
    }

    #[test]
    fn test_damage_tracker_full_damage() {
        let mut tracker = DamageTracker::new(640, 480);
        tracker.add_damage(DamageRect::new(0, 0, 640, 480));

        assert!(tracker.is_full_damage());
        assert_eq!(tracker.regions().len(), 0);
    }

    #[test]
    fn test_damage_tracker_clipping() {
        let mut tracker = DamageTracker::new(640, 480);
        // Add damage that extends beyond bounds
        tracker.add_damage(DamageRect::new(600, 450, 100, 100));

        assert!(!tracker.is_full_damage());
        let regions = tracker.regions();
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].width, 40);
        assert_eq!(regions[0].height, 30);
    }

    #[test]
    fn test_damage_tracker_optimize_overlapping() {
        let mut tracker = DamageTracker::new(640, 480);
        tracker.add_damage(DamageRect::new(0, 0, 100, 100));
        tracker.add_damage(DamageRect::new(50, 50, 100, 100));

        assert_eq!(tracker.regions().len(), 2);
        tracker.optimize();
        assert_eq!(tracker.regions().len(), 1);
    }

    #[test]
    fn test_damage_tracker_optimize_adjacent() {
        let mut tracker = DamageTracker::new(640, 480);
        tracker.add_damage(DamageRect::new(0, 0, 100, 100));
        tracker.add_damage(DamageRect::new(100, 0, 100, 100));

        assert_eq!(tracker.regions().len(), 2);
        tracker.optimize();
        assert_eq!(tracker.regions().len(), 1);
    }

    #[test]
    fn test_damage_tracker_bounding_box() {
        let mut tracker = DamageTracker::new(640, 480);
        tracker.add_damage(DamageRect::new(10, 10, 100, 100));
        tracker.add_damage(DamageRect::new(200, 200, 100, 100));

        let bbox = tracker.bounding_box();
        assert!(bbox.is_some());
        let b = bbox.unwrap();
        assert_eq!(b.x, 10);
        assert_eq!(b.y, 10);
        assert_eq!(b.width, 290);
        assert_eq!(b.height, 290);
    }

    #[test]
    fn test_damage_tracker_clear() {
        let mut tracker = DamageTracker::new(640, 480);
        tracker.add_damage(DamageRect::new(10, 10, 100, 100));
        assert!(!tracker.regions().is_empty());

        tracker.clear();
        assert!(tracker.regions().is_empty());
        assert!(!tracker.is_full_damage());
    }
}
