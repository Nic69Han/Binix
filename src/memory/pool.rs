//! Object Pooling - Reuse allocated objects to reduce allocation overhead
//!
//! Object pooling is critical for high-frequency allocations like DOM nodes,
//! style objects, and layout boxes.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// Statistics for an object pool
#[derive(Debug, Default)]
pub struct PoolStats {
    /// Total objects created
    pub created: AtomicUsize,
    /// Objects currently in pool (available)
    pub available: AtomicUsize,
    /// Objects currently in use
    pub in_use: AtomicUsize,
    /// Number of times an object was reused
    pub reused: AtomicUsize,
    /// Number of times pool was empty and new object created
    pub misses: AtomicUsize,
}

impl PoolStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn hit_rate(&self) -> f64 {
        let reused = self.reused.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = reused + misses;
        if total == 0 {
            0.0
        } else {
            (reused as f64 / total as f64) * 100.0
        }
    }
}

/// A pooled object that returns to the pool when dropped
pub struct PooledObject<T> {
    value: Option<T>,
    pool: Arc<Mutex<VecDeque<T>>>,
    stats: Arc<PoolStats>,
}

impl<T> PooledObject<T> {
    fn new(value: T, pool: Arc<Mutex<VecDeque<T>>>, stats: Arc<PoolStats>) -> Self {
        stats.in_use.fetch_add(1, Ordering::Relaxed);
        Self {
            value: Some(value),
            pool,
            stats,
        }
    }
}

impl<T> std::ops::Deref for PooledObject<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.value.as_ref().unwrap()
    }
}

impl<T> std::ops::DerefMut for PooledObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value.as_mut().unwrap()
    }
}

impl<T> Drop for PooledObject<T> {
    fn drop(&mut self) {
        if let Some(value) = self.value.take() {
            self.stats.in_use.fetch_sub(1, Ordering::Relaxed);
            self.stats.available.fetch_add(1, Ordering::Relaxed);
            if let Ok(mut pool) = self.pool.lock() {
                pool.push_back(value);
            }
        }
    }
}

/// Object pool for reusing allocated objects
pub struct ObjectPool<T> {
    pool: Arc<Mutex<VecDeque<T>>>,
    stats: Arc<PoolStats>,
    max_size: usize,
    factory: Box<dyn Fn() -> T + Send + Sync>,
}

impl<T: Default + 'static> ObjectPool<T> {
    /// Create a new pool with default factory
    pub fn new(max_size: usize) -> Self {
        Self::with_factory(max_size, Box::new(T::default))
    }
}

impl<T: 'static> ObjectPool<T> {
    /// Create a pool with a custom factory function
    pub fn with_factory(max_size: usize, factory: Box<dyn Fn() -> T + Send + Sync>) -> Self {
        Self {
            pool: Arc::new(Mutex::new(VecDeque::with_capacity(max_size))),
            stats: Arc::new(PoolStats::new()),
            max_size,
            factory,
        }
    }

    /// Get an object from the pool, or create a new one
    pub fn get(&self) -> PooledObject<T> {
        let value = {
            let mut pool = self.pool.lock().unwrap();
            pool.pop_front()
        };

        match value {
            Some(v) => {
                self.stats.reused.fetch_add(1, Ordering::Relaxed);
                self.stats.available.fetch_sub(1, Ordering::Relaxed);
                PooledObject::new(v, self.pool.clone(), self.stats.clone())
            }
            None => {
                self.stats.misses.fetch_add(1, Ordering::Relaxed);
                self.stats.created.fetch_add(1, Ordering::Relaxed);
                let value = (self.factory)();
                PooledObject::new(value, self.pool.clone(), self.stats.clone())
            }
        }
    }

    /// Pre-populate the pool with objects
    pub fn warm(&self, count: usize) {
        let count = count.min(self.max_size);
        let mut pool = self.pool.lock().unwrap();
        for _ in 0..count {
            if pool.len() < self.max_size {
                pool.push_back((self.factory)());
                self.stats.created.fetch_add(1, Ordering::Relaxed);
                self.stats.available.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Get pool statistics
    pub fn stats(&self) -> &PoolStats {
        &self.stats
    }

    /// Current number of available objects
    pub fn available(&self) -> usize {
        self.stats.available.load(Ordering::Relaxed)
    }

    /// Clear the pool
    pub fn clear(&self) {
        let mut pool = self.pool.lock().unwrap();
        let count = pool.len();
        pool.clear();
        self.stats.available.fetch_sub(count, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_get_and_return() {
        let pool: ObjectPool<Vec<u8>> = ObjectPool::new(10);

        {
            let mut obj = pool.get();
            obj.push(1);
            obj.push(2);
            assert_eq!(obj.len(), 2);
        }

        // Object returned to pool
        assert_eq!(pool.available(), 1);
    }

    #[test]
    fn test_pool_reuse() {
        let pool: ObjectPool<i32> = ObjectPool::new(10);

        {
            let _obj = pool.get();
        }

        {
            let _obj = pool.get();
        }

        assert_eq!(pool.stats().reused.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_pool_warm() {
        let pool: ObjectPool<String> = ObjectPool::new(10);
        pool.warm(5);
        assert_eq!(pool.available(), 5);
    }

    #[test]
    fn test_pool_hit_rate() {
        let pool: ObjectPool<i32> = ObjectPool::new(10);
        pool.warm(5);

        for _ in 0..10 {
            let _obj = pool.get();
        }

        assert!(pool.stats().hit_rate() > 0.0);
    }

    #[test]
    fn test_pool_with_factory() {
        let pool = ObjectPool::with_factory(10, Box::new(|| vec![0u8; 1024]));
        let obj = pool.get();
        assert_eq!(obj.len(), 1024);
    }
}
