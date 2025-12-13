//! Compressed Pointers - Reduce memory footprint on 64-bit systems
//!
//! On 64-bit systems, pointers take 8 bytes but often only use 48 bits.
//! Compressed pointers use 32-bit offsets from a base address, saving 50% memory.

use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};

/// A compressed pointer that stores a 32-bit offset from a base address
#[derive(Debug)]
pub struct CompressedPtr<T> {
    offset: u32,
    _marker: PhantomData<*const T>,
}

impl<T> Clone for CompressedPtr<T> {
    fn clone(&self) -> Self {
        Self {
            offset: self.offset,
            _marker: PhantomData,
        }
    }
}

impl<T> Copy for CompressedPtr<T> {}

impl<T> CompressedPtr<T> {
    /// Create a null compressed pointer
    pub const fn null() -> Self {
        Self {
            offset: u32::MAX,
            _marker: PhantomData,
        }
    }

    /// Check if the pointer is null
    pub fn is_null(&self) -> bool {
        self.offset == u32::MAX
    }

    /// Get the raw offset
    pub fn offset(&self) -> u32 {
        self.offset
    }
}

/// A heap that uses compressed pointers
pub struct CompressedHeap<T> {
    base: *mut T,
    storage: Vec<T>,
    capacity: usize,
    next_free: AtomicU64,
}

// Safety: CompressedHeap manages its own memory safely
unsafe impl<T: Send> Send for CompressedHeap<T> {}
unsafe impl<T: Sync> Sync for CompressedHeap<T> {}

impl<T> CompressedHeap<T> {
    /// Create a new compressed heap with the given capacity
    pub fn new(capacity: usize) -> Self {
        // Ensure capacity fits in 32-bit offset
        let capacity = capacity.min(u32::MAX as usize - 1);
        let mut storage = Vec::with_capacity(capacity);
        let base = storage.as_mut_ptr();

        Self {
            base,
            storage,
            capacity,
            next_free: AtomicU64::new(0),
        }
    }

    /// Allocate a value and return a compressed pointer
    pub fn alloc(&mut self, value: T) -> Option<CompressedPtr<T>> {
        if self.storage.len() >= self.capacity {
            return None;
        }

        let offset = self.storage.len() as u32;
        self.storage.push(value);
        self.base = self.storage.as_mut_ptr(); // Update base after potential realloc

        Some(CompressedPtr {
            offset,
            _marker: PhantomData,
        })
    }

    /// Get a reference to the value at the compressed pointer
    pub fn get(&self, ptr: CompressedPtr<T>) -> Option<&T> {
        if ptr.is_null() {
            return None;
        }
        self.storage.get(ptr.offset as usize)
    }

    /// Get a mutable reference to the value at the compressed pointer
    pub fn get_mut(&mut self, ptr: CompressedPtr<T>) -> Option<&mut T> {
        if ptr.is_null() {
            return None;
        }
        self.storage.get_mut(ptr.offset as usize)
    }

    /// Number of allocated objects
    pub fn len(&self) -> usize {
        self.storage.len()
    }

    /// Check if heap is empty
    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }

    /// Capacity of the heap
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Memory saved compared to regular pointers (bytes)
    pub fn memory_saved(&self) -> usize {
        // Each compressed pointer is 4 bytes vs 8 bytes for regular pointer
        self.storage.len() * 4
    }
}

impl<T> Default for CompressedHeap<T> {
    fn default() -> Self {
        Self::new(1024)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compressed_ptr_null() {
        let ptr: CompressedPtr<i32> = CompressedPtr::null();
        assert!(ptr.is_null());
    }

    #[test]
    fn test_compressed_heap_alloc() {
        let mut heap: CompressedHeap<i32> = CompressedHeap::new(100);
        let ptr = heap.alloc(42).unwrap();
        assert!(!ptr.is_null());
        assert_eq!(*heap.get(ptr).unwrap(), 42);
    }

    #[test]
    fn test_compressed_heap_multiple() {
        let mut heap: CompressedHeap<String> = CompressedHeap::new(100);
        let p1 = heap.alloc("hello".to_string()).unwrap();
        let p2 = heap.alloc("world".to_string()).unwrap();

        assert_eq!(heap.get(p1).unwrap(), "hello");
        assert_eq!(heap.get(p2).unwrap(), "world");
        assert_eq!(heap.len(), 2);
    }

    #[test]
    fn test_compressed_heap_mut() {
        let mut heap: CompressedHeap<i32> = CompressedHeap::new(100);
        let ptr = heap.alloc(10).unwrap();

        *heap.get_mut(ptr).unwrap() = 20;
        assert_eq!(*heap.get(ptr).unwrap(), 20);
    }

    #[test]
    fn test_memory_savings() {
        let mut heap: CompressedHeap<u64> = CompressedHeap::new(1000);
        for i in 0..100 {
            heap.alloc(i).unwrap();
        }
        // 100 objects * 4 bytes saved per pointer = 400 bytes
        assert_eq!(heap.memory_saved(), 400);
    }

    #[test]
    fn test_compressed_ptr_size() {
        // Verify compressed pointer is indeed smaller
        assert_eq!(std::mem::size_of::<CompressedPtr<u64>>(), 4);
        assert_eq!(std::mem::size_of::<*const u64>(), 8);
    }
}
