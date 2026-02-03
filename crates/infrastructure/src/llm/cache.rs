use std::collections::HashMap;
use std::hash::Hash;

pub struct ResponseCache<K, V> {
    cache: HashMap<K, V>,
    max_size: usize,
}

impl<K, V> ResponseCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::with_capacity(max_size),
            max_size,
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.cache.get(key)
    }

    pub fn insert(&mut self, key: K, value: V) {
        if self.cache.len() >= self.max_size && !self.cache.contains_key(&key) {
            if let Some(first_key) = self.cache.keys().next().cloned() {
                self.cache.remove(&first_key);
            }
        }
        self.cache.insert(key, value);
    }

    pub fn contains(&self, key: &K) -> bool {
        self.cache.contains_key(key)
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    pub fn max_size(&self) -> usize {
        self.max_size
    }
}

impl<K, V> Default for ResponseCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn default() -> Self {
        Self::new(100)
    }
}

pub type PromptCache = ResponseCache<String, String>;

#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
}

impl CacheStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    pub fn record_hit(&mut self) {
        self.hits += 1;
    }

    pub fn record_miss(&mut self) {
        self.misses += 1;
    }

    pub fn record_eviction(&mut self) {
        self.evictions += 1;
    }
}

pub struct CachedClient<C> {
    inner: C,
    cache: PromptCache,
    stats: CacheStats,
}

impl<C> CachedClient<C> {
    pub fn new(inner: C, cache_size: usize) -> Self {
        Self {
            inner,
            cache: ResponseCache::new(cache_size),
            stats: CacheStats::new(),
        }
    }

    pub fn inner(&self) -> &C {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut C {
        &mut self.inner
    }

    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    pub fn cache_mut(&mut self) -> &mut PromptCache {
        &mut self.cache
    }

    pub fn is_cached(&self, prompt: &str) -> bool {
        self.cache.contains(&prompt.to_string())
    }

    pub fn get_cached(&mut self, prompt: &str) -> Option<String> {
        if let Some(response) = self.cache.get(&prompt.to_string()) {
            self.stats.record_hit();
            Some(response.clone())
        } else {
            self.stats.record_miss();
            None
        }
    }

    pub fn cache_response(&mut self, prompt: String, response: String) {
        let was_full = self.cache.len() >= self.cache.max_size();
        self.cache.insert(prompt, response);
        if was_full {
            self.stats.record_eviction();
        }
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_cache() {
        let mut cache = ResponseCache::new(3);

        cache.insert("key1", "value1");
        cache.insert("key2", "value2");

        assert_eq!(cache.get(&"key1"), Some(&"value1"));
        assert_eq!(cache.get(&"key2"), Some(&"value2"));
        assert_eq!(cache.get(&"key3"), None);

        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_cache_eviction() {
        let mut cache = ResponseCache::new(2);

        cache.insert("key1", "value1");
        cache.insert("key2", "value2");
        cache.insert("key3", "value3");

        assert!(cache.len() <= 2);
    }

    #[test]
    fn test_cache_stats() {
        let mut stats = CacheStats::new();

        stats.record_hit();
        stats.record_hit();
        stats.record_miss();

        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert!((stats.hit_rate() - 0.666).abs() < 0.001);
    }

    #[test]
    fn test_cached_client() {
        struct MockClient;

        let client = CachedClient::new(MockClient, 10);
        assert_eq!(client.stats().hits, 0);
        assert!(!client.is_cached("test"));
    }
}
