//! Memory optimization module
//!
//! Provides advanced memory management features:
//! - Object pooling for reduced allocations
//! - Compressed pointers for reduced memory footprint
//! - Smart prefetching based on usage patterns

pub mod compressed;
pub mod pool;
pub mod prefetch;

pub use compressed::{CompressedHeap, CompressedPtr};
pub use pool::{ObjectPool, PoolStats, PooledObject};
pub use prefetch::{PrefetchHint, PrefetchStrategy, Prefetcher};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Verify all exports are accessible
        let _pool: ObjectPool<i32> = ObjectPool::new(10);
    }
}
