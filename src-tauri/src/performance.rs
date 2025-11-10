use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use moka::future::Cache;
use instant::Instant as IInstant;
use dashmap::DashMap;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use anyhow::Result;

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub enable_caching: bool,
    pub cache_ttl: Duration,
    pub max_cache_size: usize,
    pub connection_pool_size: usize,
    pub batch_size: usize,
    pub timeout_duration: Duration,
    pub enable_compression: bool,
    pub enable_prefetching: bool,
    pub memory_limit_mb: usize,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub cpu_usage: f64,
    pub memory_usage_mb: f64,
    pub network_latency_ms: f64,
    pub cache_hit_rate: f64,
    pub active_connections: u32,
    pub requests_per_second: f64,
    pub response_time_ms: f64,
    pub timestamp: i64,
}

/// Cache entry with metadata
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    data: T,
    created_at: Instant,
    last_accessed: Instant,
    access_count: u64,
    size_bytes: usize,
}

/// Enhanced cache manager
pub struct PerformanceCache {
    // Memory cache
    memory_cache: Arc<DashMap<String, CacheEntry<Vec<u8>>>>,
    
    // Persistent cache
    disk_cache: Option<Cache<String, Vec<u8>>>,
    
    // Metrics
    hit_count: Arc<RwLock<u64>>,
    miss_count: Arc<RwLock<u64>>,
    
    // Configuration
    config: PerformanceConfig,
}

impl PerformanceCache {
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            memory_cache: Arc::new(DashMap::new()),
            disk_cache: if config.enable_caching {
                Some(Cache::builder()
                    .max_capacity(config.max_cache_size)
                    .time_to_live(config.cache_ttl)
                    .build())
            } else {
                None
            },
            hit_count: Arc::new(RwLock::new(0)),
            miss_count: Arc::new(RwLock::new(0)),
            config,
        }
    }

    /// Get item from cache
    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        // Check memory cache first
        if let Some(entry) = self.memory_cache.get(key) {
            let mut entry = entry.clone();
            entry.last_accessed = Instant::now();
            entry.access_count += 1;
            
            // Update hit count
            let mut hit_count = self.hit_count.write().await;
            *hit_count += 1;
            
            return Some(entry.data);
        }

        // Check disk cache if available
        if let Some(ref disk_cache) = self.disk_cache {
            if let Some(data) = disk_cache.get(key).await {
                let mut hit_count = self.hit_count.write().await;
                *hit_count += 1;
                return Some(data);
            }
        }

        // Update miss count
        let mut miss_count = self.miss_count.write().await;
        *miss_count += 1;
        
        None
    }

    /// Set item in cache
    pub async fn set(&self, key: String, data: Vec<u8>) {
        let now = Instant::now();
        
        // Store in memory cache
        let entry = CacheEntry {
            data: data.clone(),
            created_at: now,
            last_accessed: now,
            access_count: 1,
            size_bytes: data.len(),
        };
        
        self.memory_cache.insert(key.clone(), entry);
        
        // Store in disk cache if available
        if let Some(ref disk_cache) = self.disk_cache {
            let _ = disk_cache.insert(key, data).await;
        }
    }

    /// Remove item from cache
    pub async fn remove(&self, key: &str) {
        self.memory_cache.remove(key);
        if let Some(ref disk_cache) = self.disk_cache {
            let _ = disk_cache.invalidate(key).await;
        }
    }

    /// Clear all caches
    pub async fn clear(&self) {
        self.memory_cache.clear();
        if let Some(ref disk_cache) = self.disk_cache {
            let _ = disk_cache.invalidate_all().await;
        }
    }

    /// Get cache hit rate
    pub async fn get_hit_rate(&self) -> f64 {
        let hit_count = *self.hit_count.read().await;
        let miss_count = *self.miss_count.read().await;
        let total = hit_count + miss_count;
        
        if total > 0 {
            hit_count as f64 / total as f64
        } else {
            0.0
        }
    }

    /// Clean up expired entries
    pub async fn cleanup_expired(&self) {
        let now = Instant::now();
        let ttl = self.config.cache_ttl;
        
        // Clean memory cache
        let keys_to_remove: Vec<String> = self.memory_cache
            .iter()
            .filter(|entry| now.duration_since(entry.value().created_at) > ttl)
            .map(|entry| entry.key().clone())
            .collect();
        
        for key in keys_to_remove {
            self.memory_cache.remove(&key);
        }
    }
}

/// Connection pool for RPC connections
pub struct ConnectionPool {
    connections: Arc<DashMap<String, Vec<Connection>>>,
    config: PerformanceConfig,
    active_connections: Arc<RwLock<u32>>,
}

struct Connection {
    id: String,
    created_at: Instant,
    last_used: Instant,
    is_busy: bool,
}

impl ConnectionPool {
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            connections: Arc::new(DashMap::new()),
            config,
            active_connections: Arc::new(RwLock::new(0)),
        }
    }

    /// Get or create connection for endpoint
    pub async fn get_connection(&self, endpoint: &str) -> Option<String> {
        let mut connection_id = None;
        
        if let Some(mut pool) = self.connections.get_mut(endpoint) {
            // Find available connection
            for conn in pool.value_mut() {
                if !conn.is_busy {
                    conn.is_busy = true;
                    conn.last_used = Instant::now();
                    connection_id = Some(conn.id.clone());
                    break;
                }
            }
        }
        
        // Create new connection if needed
        if connection_id.is_none() {
            if let Some(mut pool) = self.connections.get_mut(endpoint) {
                if pool.len() < self.config.connection_pool_size {
                    let conn_id = format!("{}_{}", endpoint, pool.len());
                    let conn = Connection {
                        id: conn_id.clone(),
                        created_at: Instant::now(),
                        last_used: Instant::now(),
                        is_busy: true,
                    };
                    pool.push_back(conn);
                    connection_id = Some(conn_id);
                }
            }
        }
        
        if connection_id.is_some() {
            let mut active = self.active_connections.write().await;
            *active += 1;
        }
        
        connection_id
    }

    /// Return connection to pool
    pub async fn return_connection(&self, endpoint: &str, connection_id: &str) {
        if let Some(mut pool) = self.connections.get_mut(endpoint) {
            for conn in pool.value_mut() {
                if conn.id == connection_id {
                    conn.is_busy = false;
                    conn.last_used = Instant::now();
                    break;
                }
            }
        }
        
        let mut active = self.active_connections.write().await;
        if *active > 0 {
            *active -= 1;
        }
    }

    /// Get active connection count
    pub async fn get_active_count(&self) -> u32 {
        *self.active_connections.read().await
    }

    /// Clean up idle connections
    pub async fn cleanup_idle(&self) {
        let now = Instant::now();
        let idle_timeout = Duration::from_secs(300); // 5 minutes
        
        for mut pool in self.connections.iter_mut() {
            pool.retain(|conn| {
                if !conn.is_busy && now.duration_since(conn.last_used) > idle_timeout {
                    false
                } else {
                    true
                }
            });
        }
    }
}

/// Batch processor for bulk operations
pub struct BatchProcessor<T> {
    batch_queue: Arc<dashmap::DashMap<String, Vec<T>>>,
    batch_size: usize,
    flush_interval: Duration,
    is_flushing: Arc<RwLock<bool>>,
}

impl<T> BatchProcessor<T> {
    pub fn new(batch_size: usize, flush_interval: Duration) -> Self {
        Self {
            batch_queue: Arc::new(DashMap::new()),
            batch_size,
            flush_interval,
            is_flushing: Arc::new(RwLock::new(false)),
        }
    }

    /// Add item to batch
    pub fn add_item(&self, batch_id: &str, item: T) {
        if let Some(mut batch) = self.batch_queue.get_mut(batch_id) {
            batch.push(item);
            
            // Auto-flush if batch is full
            if batch.len() >= self.batch_size {
                // In a real implementation, you would trigger a flush here
            }
        } else {
            self.batch_queue.insert(batch_id.to_string(), vec![item]);
        }
    }

    /// Get current batch size
    pub fn get_batch_size(&self, batch_id: &str) -> usize {
        if let Some(batch) = self.batch_queue.get(batch_id) {
            batch.len()
        } else {
            0
        }
    }

    /// Get all batch IDs
    pub fn get_batch_ids(&self) -> Vec<String> {
        self.batch_queue.iter().map(|entry| entry.key().clone()).collect()
    }
}

/// Performance monitor
pub struct PerformanceMonitor {
    start_time: Instant,
    metrics: Arc<RwLock<PerformanceMetrics>>,
    config: PerformanceConfig,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_caching: true,
            cache_ttl: Duration::from_secs(300), // 5 minutes
            max_cache_size: 1000,
            connection_pool_size: 10,
            batch_size: 100,
            timeout_duration: Duration::from_secs(30),
            enable_compression: true,
            enable_prefetching: true,
            memory_limit_mb: 512,
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage_mb: 0.0,
            network_latency_ms: 0.0,
            cache_hit_rate: 0.0,
            active_connections: 0,
            requests_per_second: 0.0,
            response_time_ms: 0.0,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

impl PerformanceMonitor {
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            start_time: Instant::now(),
            metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            config,
        }
    }

    /// Record request performance
    pub async fn record_request(&self, response_time: Duration, success: bool) {
        let mut metrics = self.metrics.write().await;
        metrics.timestamp = chrono::Utc::now().timestamp();
        
        // Update response time with exponential moving average
        let alpha = 0.1;
        metrics.response_time_ms = (1.0 - alpha) * metrics.response_time_ms + 
                                  alpha * response_time.as_millis() as f64;
        
        // Calculate requests per second
        let elapsed = self.start_time.elapsed();
        if elapsed.as_secs() > 0 {
            let total_requests = (metrics.requests_per_second * elapsed.as_secs() as f64) + 1.0;
            metrics.requests_per_second = total_requests / elapsed.as_secs() as f64;
        }
    }

    /// Update network latency
    pub async fn update_network_latency(&self, latency_ms: f64) {
        let mut metrics = self.metrics.write().await;
        let alpha = 0.1;
        metrics.network_latency_ms = (1.0 - alpha) * metrics.network_latency_ms + alpha * latency_ms;
    }

    /// Update memory usage
    pub async fn update_memory_usage(&self, memory_mb: f64) {
        let mut metrics = self.metrics.write().await;
        metrics.memory_usage_mb = memory_mb;
    }

    /// Update active connections
    pub async fn update_active_connections(&self, count: u32) {
        let mut metrics = self.metrics.write().await;
        metrics.active_connections = count;
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.read().await.clone()
    }

    /// Get performance score (0-100)
    pub async fn get_performance_score(&self) -> f64 {
        let metrics = self.metrics.read().await;
        
        let response_score = (1000.0 / (metrics.response_time_ms + 1.0)).min(100.0);
        let latency_score = (100.0 / (metrics.network_latency_ms / 10.0 + 1.0)).min(100.0);
        let cache_score = metrics.cache_hit_rate * 100.0;
        let connection_score = (100.0 - (metrics.active_connections as f64 * 2.0)).max(0.0);
        
        (response_score + latency_score + cache_score + connection_score) / 4.0
    }

    /// Generate performance report
    pub async fn generate_report(&self) -> String {
        let metrics = self.get_metrics().await;
        let score = self.get_performance_score().await;
        let uptime = self.start_time.elapsed();
        
        format!(
            "Performance Report (Uptime: {:.1}s):
                Score: {:.1}/100
                Response Time: {:.1}ms
                Network Latency: {:.1}ms
                Cache Hit Rate: {:.1}%
                Active Connections: {}
                Memory Usage: {:.1}MB
                Requests/Second: {:.1}",
            uptime.as_secs_f64(),
            score,
            metrics.response_time_ms,
            metrics.network_latency_ms,
            metrics.cache_hit_rate * 100.0,
            metrics.active_connections,
            metrics.memory_usage_mb,
            metrics.requests_per_second
        )
    }
}

/// Async utility functions
pub struct AsyncUtils;

impl AsyncUtils {
    /// Execute with timeout
    pub async fn execute_with_timeout<F, T>(
        future: F,
        timeout_duration: Duration,
    ) -> Result<T, String>
    where
        F: Future<Output = T> + Send,
    {
        tokio::time::timeout(timeout_duration, future)
            .await
            .map_err(|_| "Operation timed out".to_string())
    }

    /// Execute with retry
    pub async fn execute_with_retry<F, T, E>(
        mut operation: F,
        max_retries: u32,
        delay: Duration,
    ) -> Result<T, E>
    where
        F: FnMut() -> F,
        F: Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        let mut last_error = None;
        
        for attempt in 0..=max_retries {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_retries {
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap())
    }

    /// Execute with circuit breaker
    pub async fn execute_with_circuit_breaker<F, T, E>(
        operation: F,
        failure_threshold: u32,
        reset_timeout: Duration,
    ) -> Result<T, E>
    where
        F: Future<Output = Result<T, E>>,
    {
        // This is a simplified circuit breaker
        // In a real implementation, you'd use a proper circuit breaker library
        operation().await
    }
}

/// Memory pool for efficient allocation
pub struct MemoryPool {
    pool: Arc<DashMap<usize, Vec<Vec<u8>>>>,
    max_pool_size: usize,
}

impl MemoryPool {
    pub fn new(max_pool_size: usize) -> Self {
        Self {
            pool: Arc::new(DashMap::new()),
            max_pool_size,
        }
    }

    /// Allocate buffer from pool
    pub fn allocate_buffer(&self, size: usize) -> Vec<u8> {
        // For simplicity, just allocate new buffer
        // In a real implementation, you'd implement proper object pooling
        vec![0; size]
    }

    /// Return buffer to pool
    pub fn return_buffer(&self, mut buffer: Vec<u8>) {
        if self.pool.len() < self.max_pool_size && buffer.len() <= 4096 {
            buffer.clear();
            let size = buffer.capacity();
            if let Some(mut pool) = self.pool.get_mut(&size) {
                if pool.len() < 100 { // Limit per size
                    pool.push_back(buffer);
                }
            } else {
                self.pool.insert(size, vec![buffer]);
            }
        }
    }
}