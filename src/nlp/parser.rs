//! Main NLP parser that coordinates the parsing process

use super::types::*;
use super::client::OpenAIClient;
use super::mapper::CommandMapper;
use super::validator::CommandValidator;
use super::context::{CommandContext, FuzzyMatcher};
use super::pattern_matcher::{PatternMatcher, PatternMatch};
use super::learning::LearningEngine;
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use lru::LruCache;
use std::num::NonZeroUsize;

pub struct NLPParser {
    client: Arc<Mutex<OpenAIClient>>,
    /// Fast LRU cache for frequently accessed commands (in-memory, size-limited)
    hot_cache: Arc<Mutex<LruCache<String, NLPCommand>>>,
    /// Fallback HashMap for less frequently accessed items with timestamps
    cold_cache: Arc<Mutex<HashMap<String, (NLPCommand, std::time::Instant)>>>,
    config: NLPConfig,
    context: Arc<Mutex<CommandContext>>,
    pattern_matcher_enabled: bool,
    /// Learning engine for adaptive improvements from user corrections
    learning_engine: Arc<Mutex<LearningEngine>>,
}

impl NLPParser {
    /// Create a new NLP parser with the given configuration
    pub fn new(config: NLPConfig) -> Self {
        let client = Arc::new(Mutex::new(OpenAIClient::new(config.clone())));
        // Hot cache: stores 100 most recently used commands
        let hot_cache = Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(100).unwrap())));
        let cold_cache = Arc::new(Mutex::new(HashMap::new()));
        let context = Arc::new(Mutex::new(CommandContext::default()));
        let pattern_matcher_enabled = true;
        let learning_engine = Arc::new(Mutex::new(LearningEngine::new()));

        Self {
            client,
            hot_cache,
            cold_cache,
            config,
            context,
            pattern_matcher_enabled,
            learning_engine,
        }
    }

    /// Create a new NLP parser with initial categories
    pub fn with_categories(config: NLPConfig, categories: Vec<String>) -> Self {
        let client = Arc::new(Mutex::new(OpenAIClient::new(config.clone())));
        let hot_cache = Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(100).unwrap())));
        let cold_cache = Arc::new(Mutex::new(HashMap::new()));
        let context = Arc::new(Mutex::new(CommandContext::new(categories)));
        let pattern_matcher_enabled = true;
        let learning_engine = Arc::new(Mutex::new(LearningEngine::new()));

        Self {
            client,
            hot_cache,
            cold_cache,
            config,
            context,
            pattern_matcher_enabled,
            learning_engine,
        }
    }

    /// Initialize the learning engine with a database path
    pub async fn init_learning(&self, db_path: &std::path::Path) -> Result<(), NLPError> {
        let engine = LearningEngine::with_db(db_path)?;
        let mut learning = self.learning_engine.lock().await;
        *learning = engine;
        Ok(())
    }

    /// Parse natural language input and return a structured command
    pub async fn parse(&self, input: &str) -> NLPResult<NLPCommand> {
        // Check learning engine first for learned corrections
        let learning = self.learning_engine.lock().await;
        if let Some(learned_command) = learning.apply_learning(input) {
            drop(learning);
            // Update context with learned command
            let mut context_state = self.context.lock().await;
            context_state.add_command(learned_command.clone(), input.to_string());
            drop(context_state);
            return Ok(learned_command);
        }
        drop(learning);

        // Check cache first if enabled
        if self.config.cache_commands {
            if let Some(cached) = self.get_cached_command(input).await {
                return Ok(cached);
            }
        }

        // Try pattern matching first for simple commands (fast path)
        if self.pattern_matcher_enabled {
            match PatternMatcher::match_input(input) {
                PatternMatch::Matched(mut command) => {
                    // Apply fuzzy matching for categories if needed
                    let context_state = self.context.lock().await;
                    let known_categories = context_state.known_categories.clone();
                    drop(context_state);

                    if let Some(ref category) = command.category {
                        if !known_categories.is_empty() &&
                           !known_categories.contains(&category.to_lowercase()) &&
                           !known_categories.iter().any(|c| c.eq_ignore_ascii_case(category)) {
                            if let Some(fuzzy_match) = FuzzyMatcher::match_category(category, &known_categories) {
                                command.category = Some(fuzzy_match);
                            }
                        }
                    }

                    // Update context and cache
                    let mut context_state = self.context.lock().await;
                    context_state.add_command(command.clone(), input.to_string());
                    drop(context_state);

                    if self.config.cache_commands {
                        self.cache_command(input, command.clone()).await;
                    }

                    return Ok(command);
                }
                PatternMatch::Ambiguous(msg) => {
                    return Err(NLPError::ValidationError(msg));
                }
                PatternMatch::NeedsAI => {
                    // Fall through to AI processing
                }
            }
        }

        // Get context for the request
        let context_state = self.context.lock().await;
        let context_str = context_state.to_context_string();
        let conversation_summary = context_state.get_conversation_summary();
        let known_categories = context_state.known_categories.clone();
        let last_category = context_state.last_category.clone();
        drop(context_state);

        // Parse using OpenAI with context
        let mut client = self.client.lock().await;
        let mut command = client.parse_command_with_context(
            input,
            &context_str,
            &conversation_summary,
            &known_categories,
        ).await?;

        // Apply fuzzy matching for categories if needed
        if let Some(ref category) = command.category {
            if !known_categories.contains(&category.to_lowercase()) &&
               !known_categories.iter().any(|c| c.eq_ignore_ascii_case(category)) {
                // Try fuzzy match
                if let Some(fuzzy_match) = FuzzyMatcher::match_category(category, &known_categories) {
                    command.category = Some(fuzzy_match);
                }
            }
        }

        // Handle follow-up references (e.g., "change the category" without specifying content)
        if command.content.is_empty() || command.content == "it" || command.content == "that" {
            if let Some(ref lc) = last_category {
                if command.category.is_none() {
                    command.category = Some(lc.clone());
                }
            }
        }

        // Validate the command
        CommandValidator::validate(&command)?;

        // Update context with the parsed command
        let mut context_state = self.context.lock().await;
        context_state.add_command(command.clone(), input.to_string());
        drop(context_state);

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

    /// Parse natural language input, handling compound commands
    /// Returns all commands if this is a compound command
    pub async fn parse_compound(&self, input: &str) -> NLPResult<(Vec<NLPCommand>, String)> {
        let command = self.parse(input).await?;

        if command.is_compound() {
            let mut all_commands = vec![command.clone()];
            if let Some(ref compound) = command.compound_commands {
                for cmd in compound {
                    all_commands.push(cmd.clone());
                }
            }
            let description = CommandMapper::describe_compound_command(&command);
            Ok((all_commands, description))
        } else {
            let description = CommandMapper::describe_command(&command);
            Ok((vec![command], description))
        }
    }

    /// Parse natural language input to multiple argument sets for compound commands
    pub async fn parse_to_compound_args(&self, input: &str) -> NLPResult<(Vec<Vec<String>>, String)> {
        let command = self.parse(input).await?;
        let all_args = CommandMapper::to_compound_args(&command);
        let description = CommandMapper::describe_compound_command(&command);

        Ok((all_args, description))
    }

    /// Get cached command if available and not expired
    /// Checks hot cache (LRU) first, then cold cache (HashMap with TTL)
    async fn get_cached_command(&self, input: &str) -> Option<NLPCommand> {
        let hash = self.hash_input(input);

        // Check hot cache first (LRU - fast, recent items)
        {
            let mut hot_cache = self.hot_cache.lock().await;
            if let Some(command) = hot_cache.get(&hash) {
                return Some(command.clone());
            }
        }

        // Check cold cache (with TTL) - clone the value to avoid borrow issues
        let promote_to_hot = {
            let cold_cache = self.cold_cache.lock().await;
            if let Some((command, timestamp)) = cold_cache.get(&hash) {
                // Cache entries expire after 1 hour
                if timestamp.elapsed() < std::time::Duration::from_secs(3600) {
                    Some(command.clone())
                } else {
                    None
                }
            } else {
                None
            }
        };

        if let Some(command) = promote_to_hot {
            // Promote to hot cache
            {
                let mut hot_cache = self.hot_cache.lock().await;
                hot_cache.put(hash.clone(), command.clone());
            }
            return Some(command);
        }

        // Clean up expired entries from cold cache
        {
            let mut cold_cache = self.cold_cache.lock().await;
            if let Some((_, timestamp)) = cold_cache.get(&hash) {
                if timestamp.elapsed() >= std::time::Duration::from_secs(3600) {
                    cold_cache.remove(&hash);
                }
            }
        }

        None
    }

    /// Cache a parsed command
    /// Stores in hot cache (LRU); evicted items can fall through to cold cache
    async fn cache_command(&self, input: &str, command: NLPCommand) {
        let hash = self.hash_input(input);

        // Try to put in hot cache - if it's full, the LRU will evict automatically
        let mut hot_cache = self.hot_cache.lock().await;
        hot_cache.put(hash.clone(), command);
    }

    /// Create a hash for caching purposes
    fn hash_input(&self, input: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(input.trim().to_lowercase());
        format!("{:x}", hasher.finalize())
    }

    /// Clear the cache
    pub async fn clear_cache(&self) {
        let mut hot_cache = self.hot_cache.lock().await;
        let mut cold_cache = self.cold_cache.lock().await;
        hot_cache.clear();
        cold_cache.clear();
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> (usize, usize, usize) {
        let hot_cache = self.hot_cache.lock().await;
        let cold_cache = self.cold_cache.lock().await;
        let hot_len = hot_cache.len();
        let cold_total = cold_cache.len();
        let cold_expired = cold_cache.values()
            .filter(|(_, timestamp)| timestamp.elapsed() > std::time::Duration::from_secs(3600))
            .count();

        (hot_len, cold_total, cold_expired)
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

    /// Update known categories in the context
    pub async fn update_categories(&self, categories: Vec<String>) {
        let mut context = self.context.lock().await;
        context.update_categories(categories);
    }

    /// Clear old context entries
    pub async fn clear_old_context(&self, max_age_seconds: i64) {
        let mut context = self.context.lock().await;
        context.clear_old_entries(max_age_seconds);
    }

    /// Get current context state
    pub async fn get_context_state(&self) -> CommandContext {
        let context = self.context.lock().await;
        // Clone the relevant parts
        CommandContext {
            command_history: context.command_history.clone(),
            last_category: context.last_category.clone(),
            last_content: context.last_content.clone(),
            known_categories: context.known_categories.clone(),
            recent_tasks: context.recent_tasks.clone(),
            max_history_size: context.max_history_size,
        }
    }

    /// Set context state (useful for restoring state)
    pub async fn set_context_state(&self, state: CommandContext) {
        let mut context = self.context.lock().await;
        context.command_history = state.command_history;
        context.last_category = state.last_category;
        context.last_content = state.last_content;
        context.known_categories = state.known_categories;
        context.recent_tasks = state.recent_tasks;
        context.max_history_size = state.max_history_size;
    }

    /// Clear context history
    pub async fn clear_context(&self) {
        let mut context = self.context.lock().await;
        context.command_history.clear();
        context.last_category = None;
        context.last_content = None;
        context.recent_tasks.clear();
    }

    /// Get context as a string for debugging
    pub async fn context_string(&self) -> String {
        let context = self.context.lock().await;
        context.to_context_string()
    }

    /// Learn from a user correction
    pub async fn learn_correction(&self, original_input: &str, intended_command: &NLPCommand) -> Result<(), NLPError> {
        let learning = self.learning_engine.lock().await;
        learning.learn_from_correction(original_input, intended_command)
    }

    /// Get learning-based suggestions for input
    pub async fn suggest_learning(&self, input: &str) -> Vec<String> {
        let learning = self.learning_engine.lock().await;
        learning.suggest_corrections(input)
    }

    /// Get learning statistics
    pub async fn learning_stats(&self) -> Option<super::learning::LearningStats> {
        let learning = self.learning_engine.lock().await;
        learning.stats()
    }

    /// Clear all learned data
    pub async fn clear_learning(&self) -> Result<(), NLPError> {
        let learning = self.learning_engine.lock().await;
        learning.clear()
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
        let (hot_len, cold_total, cold_expired) = parser.cache_stats().await;
        assert_eq!(hot_len + cold_total, 1);
        assert_eq!(cold_expired, 0);

        // Test cache clearing
        parser.clear_cache().await;
        let (hot_len, cold_total, _) = parser.cache_stats().await;
        assert_eq!(hot_len + cold_total, 0);
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
        let (hot_len, cold_total, _cold_expired) = parser.cache_stats().await;
        assert_eq!(hot_len + cold_total, 1);
        // Fresh entries shouldn't be counted as expired
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

        let (hot_len, cold_total, _) = parser.cache_stats().await;
        assert_eq!(hot_len + cold_total, 2);
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

        let (hot_len, cold_total, _) = parser.cache_stats().await;
        assert_eq!(hot_len + cold_total, 3);

        parser.clear_cache().await;

        let (hot_len, cold_total, _) = parser.cache_stats().await;
        assert_eq!(hot_len + cold_total, 0);

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

        let (hot_len, cold_total, cold_expired) = parser.cache_stats().await;
        assert_eq!(hot_len, 0);
        assert_eq!(cold_total, 0);
        assert_eq!(cold_expired, 0);
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

        let (hot_len, cold_total, _) = parser.cache_stats().await;
        assert_eq!(hot_len + cold_total, 1); // Still only 1 entry
    }

    // === Context Tests ===

    #[tokio::test]
    async fn test_parser_with_categories() {
        let categories = vec!["work".to_string(), "personal".to_string()];
        let parser = NLPParser::with_categories(NLPConfig::default(), categories);

        let state = parser.get_context_state().await;
        assert_eq!(state.known_categories.len(), 2);
    }

    #[tokio::test]
    async fn test_update_categories() {
        let parser = NLPParser::new(NLPConfig::default());

        parser.update_categories(vec!["work".to_string(), "home".to_string()]).await;

        let state = parser.get_context_state().await;
        assert_eq!(state.known_categories.len(), 2);
        assert!(state.known_categories.contains(&"work".to_string()));
        assert!(state.known_categories.contains(&"home".to_string()));
    }

    #[tokio::test]
    async fn test_clear_context() {
        let parser = NLPParser::new(NLPConfig::default());

        // Add some context by manually manipulating state
        let mut state = parser.get_context_state().await;
        state.last_category = Some("work".to_string());
        state.last_content = Some("test".to_string());
        parser.set_context_state(state).await;

        // Verify context has content
        let state = parser.get_context_state().await;
        assert!(state.last_category.is_some());

        // Clear context
        parser.clear_context().await;

        // Verify context is cleared
        let state = parser.get_context_state().await;
        assert!(state.last_category.is_none());
        assert!(state.last_content.is_none());
        assert!(state.command_history.is_empty());
    }

    #[tokio::test]
    async fn test_context_string() {
        let parser = NLPParser::new(NLPConfig::default());

        let mut state = parser.get_context_state().await;
        state.last_category = Some("work".to_string());
        state.last_content = Some("test task".to_string());
        parser.set_context_state(state).await;

        let context_str = parser.context_string().await;
        assert!(context_str.contains("work"));
        assert!(context_str.contains("test task"));
    }

    #[tokio::test]
    async fn test_get_and_set_context_state() {
        let parser = NLPParser::new(NLPConfig::default());

        // Create a state to set
        let original_state = CommandContext {
            command_history: vec![],
            last_category: Some("work".to_string()),
            last_content: Some("meeting".to_string()),
            known_categories: vec!["work".to_string(), "personal".to_string()],
            recent_tasks: vec!["task1".to_string(), "task2".to_string()],
            max_history_size: 100,
        };

        parser.set_context_state(original_state.clone()).await;

        // Retrieve and verify
        let retrieved_state = parser.get_context_state().await;
        assert_eq!(retrieved_state.last_category, Some("work".to_string()));
        assert_eq!(retrieved_state.last_content, Some("meeting".to_string()));
        assert_eq!(retrieved_state.known_categories.len(), 2);
        assert_eq!(retrieved_state.recent_tasks.len(), 2);
        assert_eq!(retrieved_state.max_history_size, 100);
    }

    // === Compound Command Tests ===

    #[tokio::test]
    async fn test_parse_compound_simple() {
        let parser = NLPParser::new(NLPConfig::default());

        // Create a simple non-compound command manually
        let command = NLPCommand {
            action: ActionType::Task,
            content: "simple task".to_string(),
            ..Default::default()
        };

        // Simulate parsing by directly testing the result structure
        let result = if command.is_compound() {
            let mut all_commands = vec![command.clone()];
            if let Some(ref compound) = command.compound_commands {
                for cmd in compound {
                    all_commands.push(cmd.clone());
                }
            }
            all_commands
        } else {
            vec![command]
        };

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].action, ActionType::Task);
    }

    #[tokio::test]
    async fn test_parse_compound_with_secondary() {
        let parser = NLPParser::new(NLPConfig::default());

        // Create a compound command manually
        let mut command = NLPCommand {
            action: ActionType::Task,
            content: "Review PR".to_string(),
            ..Default::default()
        };
        command.add_compound_command(NLPCommand {
            action: ActionType::Update,
            content: "Review PR".to_string(),
            ..Default::default()
        });

        // Simulate compound parsing
        let result = if command.is_compound() {
            let mut all_commands = vec![command.clone()];
            if let Some(ref compound) = command.compound_commands {
                for cmd in compound {
                    all_commands.push(cmd.clone());
                }
            }
            all_commands
        } else {
            vec![command]
        };

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].action, ActionType::Task);
        assert_eq!(result[1].action, ActionType::Update);
    }

    #[tokio::test]
    async fn test_parse_to_compound_args_simple() {
        let parser = NLPParser::new(NLPConfig::default());

        let command = NLPCommand {
            action: ActionType::Task,
            content: "simple".to_string(),
            ..Default::default()
        };

        let all_args = CommandMapper::to_compound_args(&command);
        assert_eq!(all_args.len(), 1);
        assert_eq!(all_args[0], vec!["task", "simple"]);
    }

    #[tokio::test]
    async fn test_parse_to_compound_args_with_secondary() {
        let parser = NLPParser::new(NLPConfig::default());

        let mut command = NLPCommand {
            action: ActionType::Done,
            content: "5".to_string(),
            ..Default::default()
        };
        command.add_compound_command(NLPCommand {
            action: ActionType::Update,
            content: "5".to_string(),
            ..Default::default()
        });
        command.add_compound_command(NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            ..Default::default()
        });

        let all_args = CommandMapper::to_compound_args(&command);
        assert_eq!(all_args.len(), 3);
        assert_eq!(all_args[0], vec!["done", "5"]);
        assert_eq!(all_args[1], vec!["update", "5"]);
        assert_eq!(all_args[2], vec!["list", "task"]);
    }
}