/// Agent optimizations for daily workflows
///
/// This module contains performance optimizations and improvements
/// for agent execution in production environments.

use std::time::Duration;
use tokio::time::timeout;

/// Configuration for agent execution optimization
#[derive(Debug, Clone)]
pub struct AgentOptimizationConfig {
    /// Maximum time to wait for a tool execution (in seconds)
    pub tool_timeout: Duration,

    /// Maximum number of retries for failed tools
    pub max_retries: u32,

    /// Enable caching of tool results
    pub enable_caching: bool,

    /// Maximum history size to maintain in memory
    pub max_history_size: usize,

    /// Enable parallel tool execution when possible
    pub enable_parallel_tools: bool,
}

impl Default for AgentOptimizationConfig {
    fn default() -> Self {
        Self {
            tool_timeout: Duration::from_secs(30),
            max_retries: 3,
            enable_caching: true,
            max_history_size: 100,
            enable_parallel_tools: false, // Conservative default
        }
    }
}

/// Tool execution result with metadata for optimization
#[derive(Debug, Clone)]
pub struct ToolExecutionResult {
    pub success: bool,
    pub result: serde_json::Value,
    pub duration: Duration,
    pub cached: bool,
}

/// Simple in-memory cache for tool results
/// In production, consider using Redis or similar
pub struct ToolResultCache {
    cache: std::sync::Arc<dashmap::DashMap<String, (serde_json::Value, std::time::Instant)>>,
    ttl: Duration,
}

impl ToolResultCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: std::sync::Arc::new(dashmap::DashMap::new()),
            ttl,
        }
    }

    /// Generate cache key from tool name and arguments
    fn generate_key(tool_name: &str, args: &serde_json::Value) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(tool_name.as_bytes());
        hasher.update(args.to_string().as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Get cached result if available and not expired
    pub fn get(&self, tool_name: &str, args: &serde_json::Value) -> Option<serde_json::Value> {
        let key = Self::generate_key(tool_name, args);
        if let Some(entry) = self.cache.get(&key) {
            let (value, timestamp) = entry.value();
            if timestamp.elapsed() < self.ttl {
                return Some(value.clone());
            }
            // Expired, remove it
            drop(entry);
            self.cache.remove(&key);
        }
        None
    }

    /// Store result in cache
    pub fn set(&self, tool_name: &str, args: &serde_json::Value, result: serde_json::Value) {
        let key = Self::generate_key(tool_name, args);
        self.cache.insert(key, (result, std::time::Instant::now()));
    }

    /// Clear all cached results
    pub fn clear(&self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            size: self.cache.len(),
            capacity: self.cache.capacity(),
        }
    }
}

#[derive(Debug)]
pub struct CacheStats {
    pub size: usize,
    pub capacity: usize,
}

/// Execute tool with timeout and retry logic
pub async fn execute_tool_with_optimization<F, Fut>(
    tool_name: &str,
    args: &serde_json::Value,
    executor: F,
    config: &AgentOptimizationConfig,
    cache: Option<&ToolResultCache>,
) -> Result<ToolExecutionResult, String>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<serde_json::Value, String>>,
{
    let start = std::time::Instant::now();

    // Check cache first if enabled
    if let Some(cache) = cache {
        if let Some(cached_result) = cache.get(tool_name, args) {
            return Ok(ToolExecutionResult {
                success: true,
                result: cached_result,
                duration: start.elapsed(),
                cached: true,
            });
        }
    }

    // Execute with retries
    let mut last_error = None;
    for attempt in 0..=config.max_retries {
        if attempt > 0 {
            // Exponential backoff
            let delay = Duration::from_millis(100 * 2_u64.pow(attempt - 1));
            tokio::time::sleep(delay).await;
        }

        // Execute with timeout
        match timeout(config.tool_timeout, executor()).await {
            Ok(Ok(result)) => {
                // Cache successful result
                if let Some(cache) = cache {
                    cache.set(tool_name, args, result.clone());
                }

                return Ok(ToolExecutionResult {
                    success: true,
                    result,
                    duration: start.elapsed(),
                    cached: false,
                });
            }
            Ok(Err(e)) => {
                last_error = Some(e);
            }
            Err(_) => {
                last_error = Some("Tool execution timed out".to_string());
            }
        }
    }

    Err(last_error.unwrap_or_else(|| "Tool execution failed".to_string()))
}

/// Trim history to keep only recent messages
pub fn trim_history(
    history: &mut Vec<rig::completion::Message>,
    max_size: usize,
) {
    if history.len() > max_size {
        // Keep the most recent messages
        let drain_count = history.len() - max_size;
        history.drain(0..drain_count);
    }
}

/// Estimate token count (simplified approximation)
pub fn estimate_tokens(text: &str) -> usize {
    // Rough estimate: 1 token ≈ 4 characters
    text.len() / 4
}

/// Calculate total tokens in history
pub fn calculate_history_tokens(history: &[rig::completion::Message]) -> usize {
    history.iter()
        .map(|msg| estimate_tokens(&msg.content))
        .sum()
}

/// Optimize history by removing older messages if token count is too high
pub fn optimize_history_by_tokens(
    history: &mut Vec<rig::completion::Message>,
    max_tokens: usize,
) {
    let mut total_tokens = calculate_history_tokens(history);

    while total_tokens > max_tokens && !history.is_empty() {
        // Remove oldest message
        let removed = history.remove(0);
        total_tokens -= estimate_tokens(&removed.content);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimization_config_default() {
        let config = AgentOptimizationConfig::default();
        assert_eq!(config.max_retries, 3);
        assert!(config.enable_caching);
        assert_eq!(config.max_history_size, 100);
    }

    #[test]
    fn test_tool_cache() {
        let cache = ToolResultCache::new(Duration::from_secs(60));
        let args = serde_json::json!({"path": "/"});

        // Cache miss
        assert!(cache.get("filesystem", &args).is_none());

        // Store result
        let result = serde_json::json!({"files": ["a.txt", "b.txt"]});
        cache.set("filesystem", &args, result.clone());

        // Cache hit
        let cached = cache.get("filesystem", &args);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap(), result);
    }

    #[test]
    fn test_cache_expiration() {
        let cache = ToolResultCache::new(Duration::from_millis(50));
        let args = serde_json::json!({"path": "/"});
        let result = serde_json::json!({"files": ["a.txt"]});

        cache.set("filesystem", &args, result.clone());
        assert!(cache.get("filesystem", &args).is_some());

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(60));
        assert!(cache.get("filesystem", &args).is_none());
    }

    #[test]
    fn test_trim_history() {
        let mut history = vec![
            rig::completion::Message { role: "user".to_string(), content: "msg1".to_string() },
            rig::completion::Message { role: "assistant".to_string(), content: "msg2".to_string() },
            rig::completion::Message { role: "user".to_string(), content: "msg3".to_string() },
            rig::completion::Message { role: "assistant".to_string(), content: "msg4".to_string() },
        ];

        trim_history(&mut history, 2);

        assert_eq!(history.len(), 2);
        assert_eq!(history[0].content, "msg3");
        assert_eq!(history[1].content, "msg4");
    }

    #[test]
    fn test_estimate_tokens() {
        let text = "Hello world";
        let tokens = estimate_tokens(text);
        assert!(tokens > 0);
        assert!(tokens < text.len());
    }

    #[test]
    fn test_optimize_history_by_tokens() {
        let mut history = vec![
            rig::completion::Message {
                role: "user".to_string(),
                content: "a".repeat(1000)
            },
            rig::completion::Message {
                role: "assistant".to_string(),
                content: "b".repeat(1000)
            },
            rig::completion::Message {
                role: "user".to_string(),
                content: "c".repeat(100)
            },
        ];

        let initial_len = history.len();
        optimize_history_by_tokens(&mut history, 300);

        assert!(history.len() < initial_len);
        // Should keep the most recent message
        assert_eq!(history.last().unwrap().content.chars().next().unwrap(), 'c');
    }

    #[tokio::test]
    async fn test_execute_with_timeout() {
        let config = AgentOptimizationConfig::default();

        // Fast execution
        let result = execute_tool_with_optimization(
            "test_tool",
            &serde_json::json!({}),
            || async { Ok(serde_json::json!({"status": "ok"})) },
            &config,
            None,
        ).await;

        assert!(result.is_ok());
        assert!(result.unwrap().success);
    }

    #[tokio::test]
    async fn test_execute_with_retry() {
        let config = AgentOptimizationConfig {
            max_retries: 2,
            tool_timeout: Duration::from_secs(5),
            ..Default::default()
        };

        let attempt_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let attempt_count_clone = attempt_count.clone();

        // Fails first time, succeeds second time
        let result = execute_tool_with_optimization(
            "test_tool",
            &serde_json::json!({}),
            move || {
                let count = attempt_count_clone.clone();
                async move {
                    let current = count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    if current == 0 {
                        Err("First attempt fails".to_string())
                    } else {
                        Ok(serde_json::json!({"status": "ok"}))
                    }
                }
            },
            &config,
            None,
        ).await;

        assert!(result.is_ok());
        assert_eq!(attempt_count.load(std::sync::atomic::Ordering::SeqCst), 2);
    }

    #[test]
    fn test_cache_stats() {
        let cache = ToolResultCache::new(Duration::from_secs(60));
        cache.set("tool1", &serde_json::json!({}), serde_json::json!({"a": 1}));
        cache.set("tool2", &serde_json::json!({}), serde_json::json!({"b": 2}));

        let stats = cache.stats();
        assert_eq!(stats.size, 2);
    }
}
