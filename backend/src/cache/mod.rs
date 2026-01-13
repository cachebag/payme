use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct CacheEntry<V> {
    value: V,
    created_at: Instant,
    last_accessed: Instant,
    access_count: u64,
    ttl: Option<Duration>,
}

impl<V> CacheEntry<V> {
    fn new(value: V, ttl: Option<Duration>) -> Self {
        let now = Instant::now();
        Self {
            value,
            created_at: now,
            last_accessed: now,
            access_count: 0,
            ttl,
        }
    }

    fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            self.created_at.elapsed() > ttl
        } else {
            false
        }
    }

    fn access(&mut self) {
        self.last_accessed = Instant::now();
        self.access_count += 1;
    }
}

pub struct LruCache<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone,
{
    capacity: usize,
    cache: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    order: Arc<RwLock<VecDeque<K>>>,
    default_ttl: Option<Duration>,
}

impl<K, V> LruCache<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone,
{
    pub fn new(capacity: usize, default_ttl: Option<Duration>) -> Self {
        Self {
            capacity,
            cache: Arc::new(RwLock::new(HashMap::new())),
            order: Arc::new(RwLock::new(VecDeque::new())),
            default_ttl,
        }
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        let mut cache = self.cache.write().await;

        if let Some(entry) = cache.get_mut(key) {
            if entry.is_expired() {
                cache.remove(key);
                self.remove_from_order(key).await;
                return None;
            }

            entry.access();
            self.move_to_front(key).await;
            return Some(entry.value.clone());
        }

        None
    }

    pub async fn put(&self, key: K, value: V) {
        self.put_with_ttl(key, value, self.default_ttl).await;
    }

    pub async fn put_with_ttl(&self, key: K, value: V, ttl: Option<Duration>) {
        let mut cache = self.cache.write().await;

        if cache.contains_key(&key) {
            self.remove_from_order(&key).await;
        } else if cache.len() >= self.capacity {
            self.evict_lru().await;
        }

        cache.insert(key.clone(), CacheEntry::new(value, ttl));
        self.add_to_front(key).await;
    }

    pub async fn remove(&self, key: &K) {
        let mut cache = self.cache.write().await;
        cache.remove(key);
        self.remove_from_order(key).await;
    }

    async fn evict_lru(&self) {
        let mut order = self.order.write().await;
        if let Some(key) = order.pop_back() {
            drop(order);
            let mut cache = self.cache.write().await;
            cache.remove(&key);
        }
    }

    async fn move_to_front(&self, key: &K) {
        let mut order = self.order.write().await;
        order.retain(|k| k != key);
        order.push_front(key.clone());
    }

    async fn add_to_front(&self, key: K) {
        let mut order = self.order.write().await;
        order.push_front(key);
    }

    async fn remove_from_order(&self, key: &K) {
        let mut order = self.order.write().await;
        order.retain(|k| k != key);
    }

    pub async fn cleanup_expired(&self) {
        let cache = self.cache.read().await;
        let expired_keys: Vec<K> = cache
            .iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(k, _)| k.clone())
            .collect();
        drop(cache);

        for key in expired_keys {
            self.remove(&key).await;
        }
    }
}

impl<K, V> Clone for LruCache<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone,
{
    fn clone(&self) -> Self {
        Self {
            capacity: self.capacity,
            cache: Arc::clone(&self.cache),
            order: Arc::clone(&self.order),
            default_ttl: self.default_ttl,
        }
    }
}

pub struct CacheManager {
    response_cache: LruCache<String, Vec<u8>>,
    query_cache: LruCache<String, String>,
    cleanup_interval: Duration,
}

impl CacheManager {
    pub fn new(response_capacity: usize, query_capacity: usize, ttl: Duration) -> Self {
        let manager = Self {
            response_cache: LruCache::new(response_capacity, Some(ttl)),
            query_cache: LruCache::new(query_capacity, Some(ttl)),
            cleanup_interval: Duration::from_secs(300),
        };

        manager.start_cleanup_task();
        manager
    }

    fn start_cleanup_task(&self) {
        let response_cache = self.response_cache.clone();
        let query_cache = self.query_cache.clone();
        let interval = self.cleanup_interval;

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;
                response_cache.cleanup_expired().await;
                query_cache.cleanup_expired().await;
            }
        });
    }

    pub async fn get_response(&self, key: &str) -> Option<Vec<u8>> {
        self.response_cache.get(&key.to_string()).await
    }

    pub async fn put_response(&self, key: String, value: Vec<u8>) {
        self.response_cache.put(key, value).await;
    }
}

impl Clone for CacheManager {
    fn clone(&self) -> Self {
        Self {
            response_cache: self.response_cache.clone(),
            query_cache: self.query_cache.clone(),
            cleanup_interval: self.cleanup_interval,
        }
    }
}
