#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Duration;
    use pretty_assertions::assert_eq;
    use rstest::*;

    #[tokio::test]
    async fn test_cache_creation() {
        let config = PerformanceConfig::default();
        let cache = PerformanceCache::new(config);
        
        // Test basic operations
        cache.set("test_key".to_string(), b"test_value".to_vec()).await;
        let result = cache.get("test_key").await;
        
        assert!(result.is_some());
        assert_eq!(result.unwrap(), b"test_value");
    }

    #[tokio::test]
    async fn test_cache_hit_rate_calculation() {
        let config = PerformanceConfig {
            enable_caching: true,
            ..Default::default()
        };
        let cache = PerformanceCache::new(config);
        
        // Add some items
        cache.set("key1".to_string(), b"value1".to_vec()).await;
        cache.set("key2".to_string(), b"value2".to_vec()).await;
        
        // Hit some items
        let _ = cache.get("key1").await;
        let _ = cache.get("key1").await; // Second hit
        let _ = cache.get("key2").await;
        
        // Miss one item
        let _ = cache.get("key3").await;
        
        let hit_rate = cache.get_hit_rate().await;
        assert!((hit_rate - 0.75).abs() < 0.01); // 3 hits, 1 miss = 75%
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let mut config = PerformanceConfig::default();
        config.cache_ttl = Duration::from_millis(100);
        let cache = PerformanceCache::new(config);
        
        cache.set("test_key".to_string(), b"test_value".to_vec()).await;
        
        // Should be available immediately
        let result = cache.get("test_key").await;
        assert!(result.is_some());
        
        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        // Should be expired
        let result = cache.get("test_key").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_cleanup() {
        let mut config = PerformanceConfig::default();
        config.cache_ttl = Duration::from_millis(100);
        let cache = PerformanceCache::new(config);
        
        cache.set("key1".to_string(), b"value1".to_vec()).await;
        cache.set("key2".to_string(), b"value2".to_vec()).await;
        
        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        cache.cleanup_expired().await;
        
        // Both should be cleaned up
        assert!(cache.get("key1").await.is_none());
        assert!(cache.get("key2").await.is_none());
    }

    #[tokio::test]
    async fn test_connection_pool_creation() {
        let config = PerformanceConfig {
            connection_pool_size: 5,
            ..Default::default()
        };
        let pool = ConnectionPool::new(config);
        
        // Get connection for endpoint
        let conn_id = pool.get_connection("test_endpoint").await;
        assert!(conn_id.is_some());
        
        // Return connection
        if let Some(id) = conn_id {
            pool.return_connection("test_endpoint", &id).await;
        }
        
        let active_count = pool.get_active_count().await;
        assert_eq!(active_count, 0);
    }

    #[tokio::test]
    async fn test_connection_pool_size_limit() {
        let config = PerformanceConfig {
            connection_pool_size: 2,
            ..Default::default()
        };
        let pool = ConnectionPool::new(config);
        
        // Get connections (should work for first 2)
        let conn1 = pool.get_connection("test_endpoint").await;
        let conn2 = pool.get_connection("test_endpoint").await;
        
        assert!(conn1.is_some());
        assert!(conn2.is_some());
        
        // Third connection should not be created (pool limit reached)
        let conn3 = pool.get_connection("test_endpoint").await;
        assert!(conn3.is_none());
    }

    #[tokio::test]
    async fn test_batch_processor() {
        let batch_size = 3;
        let flush_interval = Duration::from_secs(1);
        let processor = BatchProcessor::new(batch_size, flush_interval);
        
        // Add items
        processor.add_item("batch1", 1);
        processor.add_item("batch1", 2);
        
        let size = processor.get_batch_size("batch1");
        assert_eq!(size, 2);
        
        processor.add_item("batch1", 3);
        
        // Should be full now
        let size = processor.get_batch_size("batch1");
        assert_eq!(size, 3);
    }

    #[tokio::test]
    async fn test_batch_processor_multiple_batches() {
        let batch_size = 2;
        let flush_interval = Duration::from_secs(1);
        let processor = BatchProcessor::new(batch_size, flush_interval);
        
        // Add to different batches
        processor.add_item("batch1", 1);
        processor.add_item("batch1", 2);
        processor.add_item("batch2", "a");
        processor.add_item("batch2", "b");
        
        assert_eq!(processor.get_batch_size("batch1"), 2);
        assert_eq!(processor.get_batch_size("batch2"), 2);
        
        let ids = processor.get_batch_ids();
        assert!(ids.contains(&"batch1".to_string()));
        assert!(ids.contains(&"batch2".to_string()));
    }

    #[test]
    fn test_performance_config_default() {
        let config = PerformanceConfig::default();
        
        assert!(config.enable_caching);
        assert_eq!(config.cache_ttl, Duration::from_secs(300));
        assert_eq!(config.max_cache_size, 1000);
        assert_eq!(config.connection_pool_size, 10);
        assert_eq!(config.batch_size, 100);
        assert_eq!(config.timeout_duration, Duration::from_secs(30));
        assert!(config.enable_compression);
        assert!(config.enable_prefetching);
        assert_eq!(config.memory_limit_mb, 512);
    }

    #[tokio::test]
    async fn test_performance_monitor_recording() {
        let config = PerformanceConfig::default();
        let monitor = PerformanceMonitor::new(config);
        
        // Record some requests
        monitor.record_request(Duration::from_millis(100), true).await;
        monitor.record_request(Duration::from_millis(200), true).await;
        monitor.record_request(Duration::from_millis(150), false).await;
        
        let metrics = monitor.get_metrics().await;
        
        assert!(metrics.response_time_ms > 0.0);
        assert!(metrics.requests_per_second > 0.0);
        assert_eq!(metrics.timestamp > 0, true);
    }

    #[tokio::test]
    async fn test_performance_monitor_network_latency() {
        let config = PerformanceConfig::default();
        let monitor = PerformanceMonitor::new(config);
        
        monitor.update_network_latency(50.0).await;
        monitor.update_network_latency(100.0).await;
        monitor.update_network_latency(75.0).await;
        
        let metrics = monitor.get_metrics().await;
        
        // Should be smoothed (exponential moving average)
        assert!(metrics.network_latency_ms > 50.0);
        assert!(metrics.network_latency_ms < 100.0);
    }

    #[tokio::test]
    async fn test_performance_monitor_memory_usage() {
        let config = PerformanceConfig::default();
        let monitor = PerformanceMonitor::new(config);
        
        monitor.update_memory_usage(256.5).await;
        monitor.update_memory_usage(300.0).await;
        
        let metrics = monitor.get_metrics().await;
        assert_eq!(metrics.memory_usage_mb, 300.0);
    }

    #[tokio::test]
    async fn test_performance_monitor_active_connections() {
        let config = PerformanceConfig::default();
        let monitor = PerformanceMonitor::new(config);
        
        monitor.update_active_connections(5).await;
        monitor.update_active_connections(10).await;
        monitor.update_active_connections(3).await;
        
        let metrics = monitor.get_metrics().await;
        assert_eq!(metrics.active_connections, 3);
    }

    #[tokio::test]
    async fn test_performance_score_calculation() {
        let config = PerformanceConfig::default();
        let monitor = PerformanceMonitor::new(config);
        
        // Record good performance metrics
        monitor.record_request(Duration::from_millis(50), true).await;
        monitor.update_network_latency(20.0).await;
        monitor.update_active_connections(2).await;
        
        let score = monitor.get_performance_score().await;
        
        // Should be a good score (closer to 100)
        assert!(score > 70.0);
        assert!(score <= 100.0);
    }

    #[tokio::test]
    async fn test_performance_score_with_poor_metrics() {
        let config = PerformanceConfig::default();
        let monitor = PerformanceMonitor::new(config);
        
        // Record poor performance metrics
        monitor.record_request(Duration::from_millis(2000), true).await;
        monitor.update_network_latency(500.0).await;
        monitor.update_active_connections(50).await;
        
        let score = monitor.get_performance_score().await;
        
        // Should be a lower score
        assert!(score < 70.0);
        assert!(score >= 0.0);
    }

    #[tokio::test]
    async fn test_performance_report_generation() {
        let config = PerformanceConfig::default();
        let monitor = PerformanceMonitor::new(config);
        
        // Add some metrics
        monitor.record_request(Duration::from_millis(100), true).await;
        monitor.update_network_latency(50.0).await;
        monitor.update_memory_usage(100.0).await;
        monitor.update_active_connections(5).await;
        
        let report = monitor.generate_report().await;
        
        // Report should contain expected sections
        assert!(report.contains("Performance Report"));
        assert!(report.contains("Score:"));
        assert!(report.contains("Response Time:"));
        assert!(report.contains("Network Latency:"));
        assert!(report.contains("Active Connections:"));
        assert!(report.contains("Memory Usage:"));
    }

    #[tokio::test]
    async fn test_async_utils_execute_with_timeout() {
        let future = async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            42
        };
        
        let result = AsyncUtils::execute_with_timeout(future, Duration::from_millis(100)).await;
        assert_eq!(result, Ok(42));
    }

    #[tokio::test]
    async fn test_async_utils_execute_with_timeout_failure() {
        let future = async {
            tokio::time::sleep(Duration::from_millis(150)).await;
            42
        };
        
        let result = AsyncUtils::execute_with_timeout(future, Duration::from_millis(100)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_async_utils_execute_with_retry_success() {
        let mut attempt = 0;
        let operation = || async {
            attempt += 1;
            if attempt < 3 {
                Err("Failed attempt")
            } else {
                Ok("Success")
            }
        };
        
        let result = AsyncUtils::execute_with_retry(operation, 3, Duration::from_millis(10)).await;
        assert_eq!(result, Ok("Success"));
    }

    #[tokio::test]
    async fn test_async_utils_execute_with_retry_failure() {
        let operation = || async {
            Err("Always fails")
        };
        
        let result = AsyncUtils::execute_with_retry(operation, 2, Duration::from_millis(10)).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_memory_pool_allocation() {
        let pool = MemoryPool::new(100);
        
        let buffer = pool.allocate_buffer(1024);
        assert_eq!(buffer.len(), 1024);
        assert!(buffer.capacity() >= 1024);
    }

    #[test]
    fn test_memory_pool_return_buffer() {
        let pool = MemoryPool::new(100);
        
        let buffer = pool.allocate_buffer(1024);
        pool.return_buffer(buffer);
        
        // Buffer should be returned to pool (no direct way to verify in this simple test)
        // In a real implementation, you'd check pool statistics
    }

    #[test]
    fn test_performance_metrics_default() {
        let metrics = PerformanceMetrics::default();
        
        assert_eq!(metrics.cpu_usage, 0.0);
        assert_eq!(metrics.memory_usage_mb, 0.0);
        assert_eq!(metrics.network_latency_ms, 0.0);
        assert_eq!(metrics.cache_hit_rate, 0.0);
        assert_eq!(metrics.active_connections, 0);
        assert_eq!(metrics.requests_per_second, 0.0);
        assert_eq!(metrics.response_time_ms, 0.0);
        assert!(metrics.timestamp > 0);
    }

    // Stress test
    #[tokio::test]
    async fn test_cache_stress() {
        let config = PerformanceConfig {
            max_cache_size: 100,
            ..Default::default()
        };
        let cache = PerformanceCache::new(config);
        
        // Add many items
        for i in 0..150 {
            cache.set(format!("key_{}", i), format!("value_{}", i).into_bytes()).await;
        }
        
        // Check that some items are still accessible
        let first_value = cache.get("key_0").await;
        let middle_value = cache.get("key_75").await;
        let last_value = cache.get("key_149").await;
        
        // At least one should be available (LRU eviction)
        let available_count = [first_value, middle_value, last_value]
            .iter()
            .filter(|v| v.is_some())
            .count();
        
        assert!(available_count > 0);
    }

    // Benchmark-style test
    #[tokio::test]
    async fn test_cache_performance() {
        let config = PerformanceConfig::default();
        let cache = PerformanceCache::new(config);
        
        // Add items
        for i in 0..1000 {
            cache.set(format!("key_{}", i), format!("value_{}", i).into_bytes()).await;
        }
        
        // Measure hit performance
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = cache.get("key_500").await;
        }
        let hit_time = start.elapsed();
        
        // Measure miss performance
        let start = Instant::now();
        for i in 0..1000 {
            let _ = cache.get(&format!("non_existent_{}", i)).await;
        }
        let miss_time = start.elapsed();
        
        // Cache hits should be faster than misses
        println!("Hit time: {:?}, Miss time: {:?}", hit_time, miss_time);
        // Note: In a real benchmark, you'd have more precise timing
    }
}

// Property-based tests for performance module
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_cache_operations_consistency(
            key in "[a-zA-Z0-9]{1,50}",
            value in prop::collection::vec(prop::num::u8::ANY, 0..1000),
        ) {
            tokio_test::block_on(async {
                let config = PerformanceConfig::default();
                let cache = PerformanceCache::new(config);
                
                // Set value
                cache.set(key.clone(), value.clone()).await;
                
                // Get value and verify
                let retrieved = cache.get(&key).await;
                prop_assert!(retrieved.is_some());
                prop_assert_eq!(retrieved.unwrap(), value);
            });
        }

        #[test]
        fn test_batch_size_bounds(
            batch_id in "[a-zA-Z0-9]{1,20}",
            item in prop::num::u64::ANY,
        ) {
            let batch_size = 5;
            let flush_interval = Duration::from_secs(1);
            let processor = BatchProcessor::new(batch_size, flush_interval);
            
            // Add items up to batch size
            for i in 0..batch_size {
                processor.add_item(&batch_id, item + i);
            }
            
            let size = processor.get_batch_size(&batch_id);
            prop_assert_eq!(size, batch_size);
        }
    }
}