use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use crossbeam::channel::{Sender, Receiver, unbounded};
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use moka::future::Cache;
use anyhow::Result;
use reqwest::Client;
use thiserror::Error;

/// Monitoring and analytics errors
#[derive(Error, Debug)]
pub enum MonitoringError {
    #[error("Analytics service error: {0}")]
    AnalyticsService(String),
    #[error("Data collection failed: {0}")]
    DataCollection(String),
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Event processing failed: {0}")]
    EventProcessing(String),
}

/// User analytics event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserEvent {
    // Wallet events
    WalletCreated { wallet_address: String, network: String },
    WalletImported { wallet_address: String, method: String },
    WalletDeleted { wallet_address: String },
    
    // Transaction events
    TransactionInitiated { from: String, to: String, amount: u64, token: String },
    TransactionCompleted { signature: String, success: bool },
    TransactionFailed { error: String, amount: u64 },
    
    // Feature usage
    SwapExecuted { from_token: String, to_token: String, amount: f64 },
    NFTViewed { mint: String, collection: Option<String> },
    StakingAction { action: String, amount: u64, validator: String },
    
    // User interface
    PageViewed { page: String, duration_ms: u64 },
    FeatureUsed { feature: String, duration_ms: u64 },
    SettingsChanged { setting: String, old_value: String, new_value: String },
    
    // Security events
    SecurityAlert { alert_type: String, severity: String },
    LoginAttempt { method: String, success: bool },
    KeyRotation { key_type: String },
    
    // Performance
    PerformanceMetric { metric: String, value: f64, unit: String },
    ErrorOccurred { error_type: String, context: String },
}

/// Business metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessMetrics {
    pub daily_active_users: u64,
    pub monthly_active_users: u64,
    pub total_transactions: u64,
    pub total_volume: f64,
    pub average_transaction_amount: f64,
    pub feature_usage_stats: HashMap<String, u64>,
    pub error_rate: f64,
    pub performance_score: f64,
    pub user_retention_rate: f64,
    pub session_duration_avg: f64,
    pub timestamp: DateTime<Utc>,
}

/// System health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_latency: f64,
    pub active_connections: u32,
    pub error_rate: f64,
    pub uptime_seconds: u64,
    pub last_health_check: DateTime<Utc>,
}

/// Blockchain monitoring data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainMonitor {
    pub network_status: String,
    pub latest_slot: u64,
    pub block_time_avg: f64,
    pub network_tps: f64,
    pub gas_price_avg: f64,
    pub market_price_sol: f64,
    pub market_price_usd: f64,
    pub active_validators: u64,
    pub stake_concentration: f64,
    pub timestamp: DateTime<Utc>,
}

/// Analytics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    pub enabled: bool,
    pub batch_size: usize,
    pub flush_interval: Duration,
    pub retention_days: u32,
    pub anonymize_data: bool,
    pub error_tracking_enabled: bool,
    pub performance_monitoring: bool,
    pub user_analytics: bool,
    pub business_metrics: bool,
}

/// Monitoring and analytics manager
pub struct MonitoringManager {
    // Configuration
    config: AnalyticsConfig,
    
    // Event processing
    event_queue: Arc<DashMap<String, Vec<UserEvent>>>,
    event_sender: Sender<UserEvent>,
    event_receiver: Arc<RwLock<Receiver<UserEvent>>>,
    
    // Caches
    metrics_cache: Arc<Cache<String, f64>>,
    user_session_cache: Arc<Cache<String, UserSession>>,
    
    // Data storage
    metrics_store: Arc<MetricsStore>,
    user_data_store: Arc<UserDataStore>,
    
    // Monitoring
    health_monitor: Arc<HealthMonitor>,
    blockchain_monitor: Arc<BlockchainHealthMonitor>,
    
    // Analytics processors
    analytics_processors: Vec<Box<dyn AnalyticsProcessor + Send + Sync>>,
    
    // Reporting
    report_generators: Vec<Box<dyn ReportGenerator + Send + Sync>>,
}

/// User session tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub session_id: String,
    pub user_id: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub events: Vec<UserEvent>,
    pub page_views: HashMap<String, u64>,
    pub features_used: HashMap<String, u64>,
    pub performance_metrics: HashMap<String, f64>,
    pub location: Option<String>,
    pub device_info: Option<String>,
    pub version: String,
}

/// Analytics processor trait
pub trait AnalyticsProcessor: Send + Sync {
    fn process_event(&self, event: &UserEvent) -> Result<(), MonitoringError>;
    fn generate_insights(&self) -> Result<serde_json::Value, MonitoringError>;
    fn name(&self) -> &str;
}

/// Business metrics processor
struct BusinessMetricsProcessor {
    metrics: Arc<RwLock<BusinessMetrics>>,
}

impl BusinessMetricsProcessor {
    fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(BusinessMetrics {
                daily_active_users: 0,
                monthly_active_users: 0,
                total_transactions: 0,
                total_volume: 0.0,
                average_transaction_amount: 0.0,
                feature_usage_stats: HashMap::new(),
                error_rate: 0.0,
                performance_score: 0.0,
                user_retention_rate: 0.0,
                session_duration_avg: 0.0,
                timestamp: Utc::now(),
            })),
        }
    }
}

impl AnalyticsProcessor for BusinessMetricsProcessor {
    fn process_event(&self, event: &UserEvent) -> Result<(), MonitoringError> {
        let mut metrics = self.metrics.write().await;
        
        match event {
            UserEvent::TransactionCompleted { success: true, amount, .. } => {
                metrics.total_transactions += 1;
                metrics.total_volume += *amount as f64;
                metrics.average_transaction_amount = metrics.total_volume / metrics.total_transactions as f64;
            }
            UserEvent::FeatureUsed { feature, .. } => {
                *metrics.feature_usage_stats.entry(feature.clone()).or_insert(0) += 1;
            }
            _ => {}
        }
        
        metrics.timestamp = Utc::now();
        Ok(())
    }
    
    fn generate_insights(&self) -> Result<serde_json::Value, MonitoringError> {
        let metrics = self.metrics.read().await;
        
        Ok(serde_json::json!({
            "total_volume": metrics.total_volume,
            "average_transaction": metrics.average_transaction_amount,
            "top_features": metrics.feature_usage_stats
                .iter()
                .max_by_key(|(_, count)| *count)
                .map(|(feature, count)| (feature, count)),
            "timestamp": metrics.timestamp
        }))
    }
    
    fn name(&self) -> &str {
        "business_metrics"
    }
}

/// Performance monitoring processor
struct PerformanceProcessor {
    performance_tracker: Arc<DashMap<String, Vec<f64>>>,
}

impl PerformanceProcessor {
    fn new() -> Self {
        Self {
            performance_tracker: Arc::new(DashMap::new()),
        }
    }
}

impl AnalyticsProcessor for PerformanceProcessor {
    fn process_event(&self, event: &UserEvent) -> Result<(), MonitoringError> {
        if let UserEvent::PerformanceMetric { metric, value, .. } = event {
            let mut tracker = self.performance_tracker.entry(metric.clone()).or_insert_with(Vec::new);
            tracker.push(*value);
            
            // Keep only last 1000 values
            if tracker.len() > 1000 {
                tracker.drain(0..tracker.len() - 1000);
            }
        }
        Ok(())
    }
    
    fn generate_insights(&self) -> Result<serde_json::Value, MonitoringError> {
        let mut performance_summary = serde_json::Map::new();
        
        for entry in self.performance_tracker.iter() {
            let values = entry.value();
            if !values.is_empty() {
                let sum: f64 = values.iter().sum();
                let avg = sum / values.len() as f64;
                let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                
                performance_summary.insert(entry.key().clone(), serde_json::json!({
                    "average": avg,
                    "min": min,
                    "max": max,
                    "count": values.len()
                }));
            }
        }
        
        Ok(serde_json::Value::Object(performance_summary))
    }
    
    fn name(&self) -> &str {
        "performance"
    }
}

/// Report generator trait
pub trait ReportGenerator: Send + Sync {
    fn generate_report(&self, data: &serde_json::Value) -> Result<String, MonitoringError>;
    fn report_type(&self) -> &str;
}

/// Daily report generator
struct DailyReportGenerator;

impl ReportGenerator for DailyReportGenerator {
    fn generate_report(&self, data: &serde_json::Value) -> Result<String, MonitoringError> {
        let report = format!(
            "Daily Analytics Report - {}\n\
             ===============================\n\
             \n\
             Volume: {:.2} SOL\n\
             Transactions: {}\n\
             Average Transaction: {:.4} SOL\n\
             Error Rate: {:.2}%\n\
             Performance Score: {:.1}/100\n\
             \n\
             Top Features:\n{}\n\
             \n\
             System Health: {}",
            Utc::now().format("%Y-%m-%d"),
            data.get("total_volume")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            data.get("total_transactions")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            data.get("average_transaction")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            data.get("error_rate")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) * 100.0,
            data.get("performance_score")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            "Feature analysis would go here",
            "System health metrics would go here"
        );
        
        Ok(report)
    }
    
    fn report_type(&self) -> &str {
        "daily"
    }
}

/// Metrics store for persistent storage
struct MetricsStore {
    database_path: String,
}

impl MetricsStore {
    fn new(database_path: String) -> Self {
        Self { database_path }
    }
    
    async fn store_metrics(&self, metrics: &BusinessMetrics) -> Result<(), MonitoringError> {
        // In a real implementation, you'd store to a database
        // For now, just log the metrics
        log::info!("Storing business metrics: {:?}", metrics);
        Ok(())
    }
    
    async fn retrieve_metrics(&self, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> 
        Result<Vec<BusinessMetrics>, MonitoringError> {
        // In a real implementation, you'd query the database
        Ok(vec![])
    }
}

/// User data store
struct UserDataStore {
    database_path: String,
}

impl UserDataStore {
    fn new(database_path: String) -> Self {
        Self { database_path }
    }
    
    async fn store_user_session(&self, session: &UserSession) -> Result<(), MonitoringError> {
        // Store user session data
        log::info!("Storing user session: {}", session.session_id);
        Ok(())
    }
}

/// Health monitor for system monitoring
struct HealthMonitor {
    health_data: Arc<RwLock<SystemHealth>>,
    last_check: Arc<RwLock<Instant>>,
}

impl HealthMonitor {
    fn new() -> Self {
        Self {
            health_data: Arc::new(RwLock::new(SystemHealth {
                cpu_usage: 0.0,
                memory_usage: 0.0,
                disk_usage: 0.0,
                network_latency: 0.0,
                active_connections: 0,
                error_rate: 0.0,
                uptime_seconds: 0,
                last_health_check: Utc::now(),
            })),
            last_check: Arc::new(RwLock::new(Instant::now())),
        }
    }
    
    async fn check_health(&self) -> Result<SystemHealth, MonitoringError> {
        let start_time = Instant::now();
        
        // Simulate health checks
        // In a real implementation, you'd check actual system metrics
        
        let cpu_usage = 25.0 + rand::random::<f64>() * 50.0;
        let memory_usage = 30.0 + rand::random::<f64>() * 40.0;
        let disk_usage = 45.0 + rand::random::<f64>() * 30.0;
        let network_latency = 10.0 + rand::random::<f64>() * 100.0;
        let active_connections = 5 + (rand::random::<u32>() % 20);
        let error_rate = rand::random::<f64>() * 0.05; // Max 5% error rate
        let uptime_seconds = start_time.elapsed().as_secs();
        
        let health = SystemHealth {
            cpu_usage,
            memory_usage,
            disk_usage,
            network_latency,
            active_connections,
            error_rate,
            uptime_seconds,
            last_health_check: Utc::now(),
        };
        
        *self.health_data.write().await = health.clone();
        *self.last_check.write().await = Instant::now();
        
        Ok(health)
    }
    
    async fn get_health(&self) -> SystemHealth {
        self.health_data.read().await.clone()
    }
}

/// Blockchain health monitor
struct BlockchainHealthMonitor {
    blockchain_data: Arc<RwLock<BlockchainMonitor>>,
}

impl BlockchainHealthMonitor {
    fn new() -> Self {
        Self {
            blockchain_data: Arc::new(RwLock::new(BlockchainMonitor {
                network_status: "unknown".to_string(),
                latest_slot: 0,
                block_time_avg: 0.0,
                network_tps: 0.0,
                gas_price_avg: 0.0,
                market_price_sol: 0.0,
                market_price_usd: 0.0,
                active_validators: 0,
                stake_concentration: 0.0,
                timestamp: Utc::now(),
            })),
        }
    }
    
    async fn check_blockchain_health(&self) -> Result<BlockchainMonitor, MonitoringError> {
        // In a real implementation, you'd check actual blockchain APIs
        let client = Client::new();
        
        // Mock blockchain data
        let monitor = BlockchainMonitor {
            network_status: "healthy".to_string(),
            latest_slot: 250_000_000 + rand::random::<u64>() % 1000,
            block_time_avg: 0.4 + rand::random::<f64>() * 0.2,
            network_tps: 1000.0 + rand::random::<f64>() * 500.0,
            gas_price_avg: 0.0001 + rand::random::<f64>() * 0.0005,
            market_price_sol: 150.0 + rand::random::<f64>() * 50.0,
            market_price_usd: 150.0 + rand::random::<f64>() * 50.0,
            active_validators: 1800 + (rand::random::<u64>() % 100),
            stake_concentration: 0.25 + rand::random::<f64>() * 0.3,
            timestamp: Utc::now(),
        };
        
        *self.blockchain_data.write().await = monitor.clone();
        Ok(monitor)
    }
    
    async fn get_blockchain_health(&self) -> BlockchainMonitor {
        self.blockchain_data.read().await.clone()
    }
}

impl MonitoringManager {
    /// Create new monitoring manager
    pub fn new(config: AnalyticsConfig) -> Self {
        let (event_sender, event_receiver) = unbounded();
        
        let metrics_cache = Cache::builder()
            .max_capacity(1000)
            .time_to_live(Duration::from_secs(300))
            .build();
            
        let user_session_cache = Cache::builder()
            .max_capacity(10000)
            .time_to_live(Duration::from_secs(3600))
            .build();
        
        let analytics_processors: Vec<Box<dyn AnalyticsProcessor + Send + Sync>> = vec![
            Box::new(BusinessMetricsProcessor::new()),
            Box::new(PerformanceProcessor::new()),
        ];
        
        let report_generators: Vec<Box<dyn ReportGenerator + Send + Sync>> = vec![
            Box::new(DailyReportGenerator),
        ];
        
        Self {
            config,
            event_queue: Arc::new(DashMap::new()),
            event_sender,
            event_receiver: Arc::new(RwLock::new(event_receiver)),
            metrics_cache: Arc::new(metrics_cache),
            user_session_cache: Arc::new(user_session_cache),
            metrics_store: Arc::new(MetricsStore::new("./analytics.db".to_string())),
            user_data_store: Arc::new(UserDataStore::new("./user_data.db".to_string())),
            health_monitor: Arc::new(HealthMonitor::new()),
            blockchain_monitor: Arc::new(BlockchainHealthMonitor::new()),
            analytics_processors,
            report_generators,
        }
    }
    
    /// Track user event
    pub async fn track_event(&self, event: UserEvent) -> Result<(), MonitoringError> {
        if !self.config.enabled {
            return Ok(());
        }
        
        // Send event to queue
        if let Err(e) = self.event_sender.send(event.clone()) {
            return Err(MonitoringError::EventProcessing(e.to_string()));
        }
        
        // Process event immediately for critical events
        match &event {
            UserEvent::ErrorOccurred { .. } | UserEvent::SecurityAlert { .. } => {
                self.process_event(&event).await?;
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Process events from queue
    pub async fn process_events(&self) -> Result<(), MonitoringError> {
        let receiver = self.event_receiver.read().await;
        let mut processed_count = 0;
        
        // Process events in batch
        for _ in 0..self.config.batch_size {
            match receiver.try_recv() {
                Ok(event) => {
                    self.process_event(&event).await?;
                    processed_count += 1;
                }
                Err(_) => break, // No more events
            }
        }
        
        if processed_count > 0 {
            log::debug!("Processed {} events", processed_count);
        }
        
        Ok(())
    }
    
    /// Process a single event
    async fn process_event(&self, event: &UserEvent) -> Result<(), MonitoringError> {
        // Send to all analytics processors
        for processor in &self.analytics_processors {
            if let Err(e) = processor.process_event(event) {
                log::error!("Processor {} failed: {}", processor.name(), e);
            }
        }
        
        // Update session data
        if let Some(user_id) = self.get_user_id_from_event(event) {
            self.update_user_session(&user_id, event).await?;
        }
        
        Ok(())
    }
    
    /// Extract user ID from event (with privacy considerations)
    fn get_user_id_from_event(&self, event: &UserEvent) -> Option<String> {
        if self.config.anonymize_data {
            return None;
        }
        
        match event {
            UserEvent::WalletCreated { wallet_address, .. } => {
                Some(wallet_address.clone())
            }
            _ => None,
        }
    }
    
    /// Update user session
    async fn update_user_session(&self, user_id: &str, event: &UserEvent) -> Result<(), MonitoringError> {
        let mut session = self.user_session_cache.get(user_id)
            .await
            .unwrap_or_else(|| {
                UserSession {
                    session_id: format!("session_{}_{}", user_id, Utc::now().timestamp()),
                    user_id: Some(user_id.to_string()),
                    start_time: Utc::now(),
                    end_time: None,
                    events: Vec::new(),
                    page_views: HashMap::new(),
                    features_used: HashMap::new(),
                    performance_metrics: HashMap::new(),
                    location: None,
                    device_info: None,
                    version: "1.0.0".to_string(),
                }
            });
        
        session.events.push(event.clone());
        
        match event {
            UserEvent::PageViewed { page, duration_ms } => {
                *session.page_views.entry(page.clone()).or_insert(0) += 1;
            }
            UserEvent::FeatureUsed { feature, duration_ms } => {
                *session.features_used.entry(feature.clone()).or_insert(0) += 1;
            }
            UserEvent::PerformanceMetric { metric, value, unit } => {
                session.performance_metrics.insert(metric.clone(), *value);
            }
            _ => {}
        }
        
        self.user_session_cache.insert(user_id.to_string(), session).await;
        Ok(())
    }
    
    /// Generate analytics report
    pub async fn generate_report(&self, report_type: &str) -> Result<String, MonitoringError> {
        // Get data from processors
        let mut report_data = serde_json::Map::new();
        
        for processor in &self.analytics_processors {
            let insights = processor.generate_insights()?;
            report_data.insert(processor.name().to_string(), insights);
        }
        
        // Find appropriate generator
        for generator in &self.report_generators {
            if generator.report_type() == report_type {
                return generator.generate_report(&serde_json::Value::Object(report_data));
            }
        }
        
        Err(MonitoringError::AnalyticsService(
            format!("Report type '{}' not found", report_type)
        ))
    }
    
    /// Get current system health
    pub async fn get_system_health(&self) -> Result<SystemHealth, MonitoringError> {
        self.health_monitor.check_health().await
    }
    
    /// Get current blockchain health
    pub async fn get_blockchain_health(&self) -> Result<BlockchainMonitor, MonitoringError> {
        self.blockchain_monitor.check_blockchain_health().await
    }
    
    /// Start background monitoring tasks
    pub async fn start_background_monitoring(&self) -> Result<(), MonitoringError> {
        let monitoring_manager = Arc::new(self);
        
        // Event processing task
        let event_task = {
            let manager = monitoring_manager.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(1));
                loop {
                    interval.tick().await;
                    if let Err(e) = manager.process_events().await {
                        log::error!("Event processing error: {}", e);
                    }
                }
            })
        };
        
        // Health check task
        let health_task = {
            let manager = monitoring_manager.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(30));
                loop {
                    interval.tick().await;
                    if let Err(e) = manager.health_monitor.check_health().await {
                        log::error!("Health check error: {}", e);
                    }
                }
            })
        };
        
        // Blockchain monitoring task
        let blockchain_task = {
            let manager = monitoring_manager.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(60));
                loop {
                    interval.tick().await;
                    if let Err(e) = manager.blockchain_monitor.check_blockchain_health().await {
                        log::error!("Blockchain health check error: {}", e);
                    }
                }
            })
        };
        
        log::info!("Background monitoring tasks started");
        Ok(())
    }
}

/// Default configuration
impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            batch_size: 100,
            flush_interval: Duration::from_secs(30),
            retention_days: 90,
            anonymize_data: true,
            error_tracking_enabled: true,
            performance_monitoring: true,
            user_analytics: true,
            business_metrics: true,
        }
    }
}