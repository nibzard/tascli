//! Main NLP parser that coordinates the parsing process

use super::types::*;
use super::client::OpenAIClient;
use super::mapper::CommandMapper;
use super::validator::CommandValidator;
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct NLPParser {
    client: Arc<Mutex<OpenAIClient>>,
    cache: Arc<Mutex<HashMap<String, (NLPCommand, std::time::Instant)>>>,
    config: NLPConfig,
}

impl NLPParser {
    /// Create a new NLP parser with the given configuration
    pub fn new(config: NLPConfig) -> Self {
        let client = Arc::new(Mutex::new(OpenAIClient::new(config.clone())));
        let cache = Arc::new(Mutex::new(HashMap::new()));

        Self {
            client,
            cache,
            config,
        }
    }

    /// Parse natural language input and return a structured command
    pub async fn parse(&self, input: &str) -> NLPResult<NLPCommand> {
        // Check cache first if enabled
        if self.config.cache_commands {
            if let Some(cached) = self.get_cached_command(input).await {
                return Ok(cached);
            }
        }

        // Parse using OpenAI
        let mut client = self.client.lock().await;
        let mut command = client.parse_command(input).await?;

        // Validate the command
        CommandValidator::validate(&command)?;

        // Cache the result if enabled
        if self.config.cache_commands {
            self.cache_command(input, command.clone()).await;
        }

        Ok(command)
    }

    /// Convert natural language input to tascli arguments
    pub async fn parse_to_args(&self, input: &str) -> NLPResult<(Vec<String>, String)> {
        let command = self.parse(input).await?;
        let args = CommandMapper::to_tascli_args(&command);
        let description = CommandMapper::describe_command(&command);

        Ok((args, description))
    }

    /// Get cached command if available and not expired
    async fn get_cached_command(&self, input: &str) -> Option<NLPCommand> {
        let mut cache = self.cache.lock().await;
        let hash = self.hash_input(input);

        if let Some((command, timestamp)) = cache.get(&hash) {
            // Cache entries expire after 1 hour
            if timestamp.elapsed() < std::time::Duration::from_secs(3600) {
                return Some(command.clone());
            } else {
                cache.remove(&hash);
            }
        }

        None
    }

    /// Cache a parsed command
    async fn cache_command(&self, input: &str, command: NLPCommand) {
        let mut cache = self.cache.lock().await;
        let hash = self.hash_input(input);

        // Limit cache size to prevent memory bloat
        if cache.len() > 1000 {
            // Remove oldest entries
            let mut keys_to_remove = Vec::new();
            for (key, (_, timestamp)) in cache.iter() {
                if timestamp.elapsed() > std::time::Duration::from_secs(3600) {
                    keys_to_remove.push(key.clone());
                }
            }

            for key in keys_to_remove {
                cache.remove(&key);
            }
        }

        cache.insert(hash, (command, std::time::Instant::now()));
    }

    /// Create a hash for caching purposes
    fn hash_input(&self, input: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(input.trim().to_lowercase());
        format!("{:x}", hasher.finalize())
    }

    /// Clear the cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.lock().await;
        cache.clear();
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> (usize, usize) {
        let cache = self.cache.lock().await;
        let total = cache.len();
        let expired = cache.values()
            .filter(|(_, timestamp)| timestamp.elapsed() > std::time::Duration::from_secs(3600))
            .count();

        (total, expired)
    }

    /// Parse with fallback to traditional commands if NLP fails
    pub async fn parse_with_fallback(&self, input: &str) -> Result<(Vec<String>, String), String> {
        if !self.config.enabled {
            return Err("NLP is not enabled".to_string());
        }

        match self.parse_to_args(input).await {
            Ok(result) => Ok(result),
            Err(e) => {
                if self.config.fallback_to_traditional {
                    Err(format!("NLP parsing failed: {}. Please use traditional tascli commands.", e))
                } else {
                    Err(format!("NLP parsing failed: {}", e))
                }
            }
        }
    }

    /// Check if the parser is ready (has valid API key and is enabled)
    pub fn is_ready(&self) -> bool {
        self.config.enabled &&
        self.config.api_key.as_ref().map_or(false, |k| !k.is_empty())
    }

    /// Get configuration
    pub fn config(&self) -> &NLPConfig {
        &self.config
    }

    /// Update configuration
    pub async fn update_config(&mut self, new_config: NLPConfig) {
        // Clear cache when config changes
        self.clear_cache().await;

        // Create new client with updated config
        let client = Arc::new(Mutex::new(OpenAIClient::new(new_config.clone())));

        self.client = client;
        self.config = new_config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_input() {
        let parser = NLPParser::new(NLPConfig::default());
        let hash1 = parser.hash_input("Add task today");
        let hash2 = parser.hash_input("add task today"); // different case
        let hash3 = parser.hash_input("add task today  "); // extra space

        // Should be the same due to normalization
        assert_eq!(hash1, hash2);
        assert_eq!(hash2, hash3);

        // Different input should produce different hash
        let hash4 = parser.hash_input("Add task tomorrow");
        assert_ne!(hash1, hash4);
    }

    #[tokio::test]
    async fn test_cache_operations() {
        let config = NLPConfig {
            cache_commands: true,
            ..Default::default()
        };
        let parser = NLPParser::new(config);

        let command = NLPCommand {
            action: ActionType::Task,
            content: "test task".to_string(),
            ..Default::default()
        };

        // Test caching
        parser.cache_command("test input", command.clone()).await;
        let cached = parser.get_cached_command("test input").await;

        assert!(cached.is_some());
        assert_eq!(cached.unwrap().content, "test task");

        // Test cache stats
        let (total, expired) = parser.cache_stats().await;
        assert_eq!(total, 1);
        assert_eq!(expired, 0);

        // Test cache clearing
        parser.clear_cache().await;
        let (total, expired) = parser.cache_stats().await;
        assert_eq!(total, 0);
    }

    // === Hash Input Tests ===

    #[test]
    fn test_hash_input_empty() {
        let parser = NLPParser::new(NLPConfig::default());
        let hash = parser.hash_input("");
        // Empty string should produce a valid hash
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // SHA256 produces 64 hex characters
    }

    #[test]
    fn test_hash_input_unicode() {
        let parser = NLPParser::new(NLPConfig::default());
        let hash1 = parser.hash_input("Add task with emoji 🎉");
        let hash2 = parser.hash_input("add task with emoji 🎉");
        // Should be the same after normalization
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_input_special_chars() {
        let parser = NLPParser::new(NLPConfig::default());
        let hash1 = parser.hash_input("task!@#$%^&*()");
        let hash2 = parser.hash_input("task!@#$%^&*()");
        assert_eq!(hash1, hash2);
    }

    // === Cache Expiration Tests ===

    #[tokio::test]
    async fn test_cache_expiration() {
        let config = NLPConfig {
            cache_commands: true,
            ..Default::default()
        };
        let parser = NLPParser::new(config);

        let command = NLPCommand {
            action: ActionType::Task,
            content: "expires soon".to_string(),
            ..Default::default()
        };

        // Cache a command
        parser.cache_command("expire test", command).await;

        // Immediately retrieve should work
        let cached = parser.get_cached_command("expire test").await;
        assert!(cached.is_some());

        // Note: We can't easily test actual expiration in unit tests without
        // manipulating time or waiting, but we can test the stats reporting
        let (total, expired) = parser.cache_stats().await;
        assert_eq!(total, 1);
        // Fresh entries shouldn't be counted as expired
        assert_eq!(expired, 0);
    }

    #[tokio::test]
    async fn test_cache_with_different_inputs() {
        let config = NLPConfig {
            cache_commands: true,
            ..Default::default()
        };
        let parser = NLPParser::new(config);

        let command1 = NLPCommand {
            action: ActionType::Task,
            content: "first task".to_string(),
            ..Default::default()
        };
        let command2 = NLPCommand {
            action: ActionType::Record,
            content: "second record".to_string(),
            ..Default::default()
        };

        parser.cache_command("input 1", command1).await;
        parser.cache_command("input 2", command2).await;

        let cached1 = parser.get_cached_command("input 1").await;
        let cached2 = parser.get_cached_command("input 2").await;

        assert!(cached1.is_some());
        assert!(cached2.is_some());
        assert_eq!(cached1.unwrap().action, ActionType::Task);
        assert_eq!(cached2.unwrap().action, ActionType::Record);

        let (total, _) = parser.cache_stats().await;
        assert_eq!(total, 2);
    }

    #[tokio::test]
    async fn test_cache_case_insensitive() {
        let config = NLPConfig {
            cache_commands: true,
            ..Default::default()
        };
        let parser = NLPParser::new(config);

        let command = NLPCommand {
            action: ActionType::Task,
            content: "test".to_string(),
            ..Default::default()
        };

        parser.cache_command("ADD TASK Today", command).await;
        let cached = parser.get_cached_command("add task today").await;

        // Should find the cached entry due to case-insensitive hashing
        assert!(cached.is_some());
    }

    #[tokio::test]
    async fn test_cache_whitespace_normalization() {
        let config = NLPConfig {
            cache_commands: true,
            ..Default::default()
        };
        let parser = NLPParser::new(config);

        let command = NLPCommand {
            action: ActionType::Task,
            content: "test".to_string(),
            ..Default::default()
        };

        parser.cache_command("  add task  ", command).await;
        let cached = parser.get_cached_command("add task").await;

        // Should find the cached entry due to whitespace trimming
        assert!(cached.is_some());
    }

    #[tokio::test]
    async fn test_cache_disabled() {
        let config = NLPConfig {
            cache_commands: false,
            ..Default::default()
        };
        let parser = NLPParser::new(config);

        let command = NLPCommand {
            action: ActionType::Task,
            content: "test".to_string(),
            ..Default::default()
        };

        // Even though we call cache_command, when cache is disabled
        // we can still store, but get_cached_command won't return it
        // if we're checking the cache_commands flag
        parser.cache_command("test", command).await;

        // With cache disabled, get_cached_command still works as it's
        // a public method, but in practice parse() won't use it
        let cached = parser.get_cached_command("test").await;
        assert!(cached.is_some()); // It was cached
    }

    #[tokio::test]
    async fn test_clear_cache() {
        let config = NLPConfig {
            cache_commands: true,
            ..Default::default()
        };
        let parser = NLPParser::new(config);

        let command = NLPCommand {
            action: ActionType::Task,
            content: "test".to_string(),
            ..Default::default()
        };

        parser.cache_command("test1", command.clone()).await;
        parser.cache_command("test2", command.clone()).await;
        parser.cache_command("test3", command).await;

        let (total, _) = parser.cache_stats().await;
        assert_eq!(total, 3);

        parser.clear_cache().await;

        let (total, _) = parser.cache_stats().await;
        assert_eq!(total, 0);

        // Verify items are actually gone
        assert!(parser.get_cached_command("test1").await.is_none());
        assert!(parser.get_cached_command("test2").await.is_none());
        assert!(parser.get_cached_command("test3").await.is_none());
    }

    // === is_ready Tests ===

    #[test]
    fn test_is_ready_with_api_key() {
        let config = NLPConfig {
            enabled: true,
            api_key: Some("test-key-123".to_string()),
            ..Default::default()
        };
        let parser = NLPParser::new(config);
        assert!(parser.is_ready());
    }

    #[test]
    fn test_is_ready_disabled() {
        let config = NLPConfig {
            enabled: false,
            api_key: Some("test-key-123".to_string()),
            ..Default::default()
        };
        let parser = NLPParser::new(config);
        assert!(!parser.is_ready());
    }

    #[test]
    fn test_is_ready_no_api_key() {
        let config = NLPConfig {
            enabled: true,
            api_key: None,
            ..Default::default()
        };
        let parser = NLPParser::new(config);
        assert!(!parser.is_ready());
    }

    #[test]
    fn test_is_ready_empty_api_key() {
        let config = NLPConfig {
            enabled: true,
            api_key: Some("".to_string()),
            ..Default::default()
        };
        let parser = NLPParser::new(config);
        assert!(!parser.is_ready());
    }

    // === config getter Tests ===

    #[test]
    fn test_config_getter() {
        let config = NLPConfig {
            enabled: true,
            api_key: Some("test-key".to_string()),
            model: "custom-model".to_string(),
            cache_commands: false,
            ..Default::default()
        };
        let parser = NLPParser::new(config);
        let retrieved_config = parser.config();

        assert!(retrieved_config.enabled);
        assert_eq!(retrieved_config.api_key, Some("test-key".to_string()));
        assert_eq!(retrieved_config.model, "custom-model");
        assert!(!retrieved_config.cache_commands);
    }

    // === Default Config Tests ===

    #[test]
    fn test_parser_with_default_config() {
        let parser = NLPParser::new(NLPConfig::default());
        assert!(!parser.is_ready()); // Default has enabled=false, no api_key
        assert!(!parser.config().enabled);
        assert!(parser.config().api_key.is_none());
    }

    // === Cache Stats Edge Cases ===

    #[tokio::test]
    async fn test_cache_stats_empty() {
        let config = NLPConfig {
            cache_commands: true,
            ..Default::default()
        };
        let parser = NLPParser::new(config);

        let (total, expired) = parser.cache_stats().await;
        assert_eq!(total, 0);
        assert_eq!(expired, 0);
    }

    #[tokio::test]
    async fn test_cache_same_input_overwrite() {
        let config = NLPConfig {
            cache_commands: true,
            ..Default::default()
        };
        let parser = NLPParser::new(config);

        let command1 = NLPCommand {
            action: ActionType::Task,
            content: "original".to_string(),
            ..Default::default()
        };
        let command2 = NLPCommand {
            action: ActionType::Task,
            content: "updated".to_string(),
            ..Default::default()
        };

        parser.cache_command("same input", command1).await;
        parser.cache_command("same input", command2).await; // Should overwrite

        let cached = parser.get_cached_command("same input").await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().content, "updated");

        let (total, _) = parser.cache_stats().await;
        assert_eq!(total, 1); // Still only 1 entry
    }
}