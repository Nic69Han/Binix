use std::collections::HashMap;
use std::time::{SystemTime, Duration};

struct CacheEntry {
    data: Vec<u8>,
    expires: SystemTime,
}

pub struct HttpCache {
    store: HashMap<String, CacheEntry>,
}

impl HttpCache {
    pub fn new() -> Self {
        Self { store: HashMap::new() }
    }

    pub fn get(&self, url: &str) -> Option<&Vec<u8>> {
        self.store.get(url).filter(|e| e.expires > SystemTime::now()).map(|e| &e.data)
    }

    pub fn put(&mut self, url: String, data: Vec<u8>, ttl_secs: u64) {
        self.store.insert(url, CacheEntry {
            data,
            expires: SystemTime::now() + Duration::from_secs(ttl_secs),
        });
    }
}