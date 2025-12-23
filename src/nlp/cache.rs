//! Persistent caching for OpenAI API responses
//!
//! This module provides disk-based caching for NLP responses to reduce
//! redundant API calls and improve performance. Cached responses are stored
//! in SQLite with SHA256 hashes as keys.

use super::types::*;
use sha2::{Sha256, Digest};
use std::path::Path;
use rusqlite::params;

/// Cache entry for storing NLP responses
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct CacheEntry {
    /// The parsed command
    command: NLPCommand,
    /// Timestamp when cached (Unix seconds)
    cached_at: i64,
    /// Number of times this entry was accessed
    access_count: u32,
}

/// Persistent cache for NLP responses
pub struct ResponseCache {
    conn: rusqlite::Connection,
    ttl_seconds: i64,
}

impl ResponseCache {
    /// Create a new cache at the specified path
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, NLPError> {
        let conn = rusqlite::Connection::open(path)
            .map_err(|e| NLPError::ConfigError(format!("Failed to open cache database: {}", e)))?;

        // Create table if not exists
        conn.execute(
            "CREATE TABLE IF NOT EXISTS nlp_responses (
                hash TEXT PRIMARY KEY,
                input TEXT NOT NULL,
                response_data BLOB NOT NULL,
                cached_at INTEGER NOT NULL,
                last_accessed INTEGER NOT NULL,
                access_count INTEGER NOT NULL DEFAULT 1
            )",
            [],
        ).map_err(|e| NLPError::ConfigError(format!("Failed to create cache table: {}", e)))?;

        // Create index for faster cleanup
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_cached_at ON nlp_responses(cached_at)",
            [],
        ).map_err(|e| NLPError::ConfigError(format!("Failed to create index: {}", e)))?;

        Ok(Self {
            conn,
            ttl_seconds: 7 * 24 * 3600, // 7 days default TTL
        })
    }

    /// Create cache with custom TTL in seconds
    pub fn with_ttl<P: AsRef<Path>>(path: P, ttl_seconds: i64) -> Result<Self, NLPError> {
        let mut cache = Self::new(path)?;
        cache.ttl_seconds = ttl_seconds;
        Ok(cache)
    }

    /// Generate cache key hash from input
    fn hash_input(&self, input: &str) -> String {
        // Normalize: trim, lowercase, and collapse multiple spaces
        let normalized = input.split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();
        let mut hasher = Sha256::new();
        hasher.update(normalized.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Get cached response if available and not expired
    pub fn get(&self, input: &str) -> Option<NLPCommand> {
        let hash = self.hash_input(input);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_secs() as i64;

        let result = self.conn.query_row(
            "SELECT response_data, cached_at, access_count FROM nlp_responses WHERE hash = ?1",
            params![hash.as_str()],
            |row| {
                let data: Vec<u8> = row.get(0)?;
                let cached_at: i64 = row.get(1)?;
                let access_count: u32 = row.get(2)?;
                Ok((data, cached_at, access_count))
            }
        );

        match result {
            Ok((data, cached_at, access_count)) => {
                // Check if expired
                if now - cached_at > self.ttl_seconds {
                    // Remove expired entry
                    let _ = self.conn.execute(
                        "DELETE FROM nlp_responses WHERE hash = ?1",
                        params![hash.as_str()],
                    );
                    return None;
                }

                // Update access stats
                let _ = self.conn.execute(
                    "UPDATE nlp_responses SET last_accessed = ?1, access_count = ?2 WHERE hash = ?3",
                    params![now, i64::from(access_count + 1), hash.as_str()],
                );

                // Deserialize command
                match serde_json::from_slice::<CacheEntry>(&data) {
                    Ok(entry) => Some(entry.command),
                    Err(_) => None,
                }
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => None,
            Err(_) => None,
        }
    }

    /// Store a response in the cache
    pub fn put(&self, input: &str, command: &NLPCommand) -> Result<(), NLPError> {
        let hash = self.hash_input(input);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| NLPError::ConfigError(format!("Time error: {}", e)))?
            .as_secs() as i64;

        let entry = CacheEntry {
            command: command.clone(),
            cached_at: now,
            access_count: 1,
        };

        let data = serde_json::to_vec(&entry)
            .map_err(|e| NLPError::SerializationError(serde_json::Error::from(e)))?;

        self.conn.execute(
            "INSERT OR REPLACE INTO nlp_responses (hash, input, response_data, cached_at, last_accessed, access_count)
             VALUES (?1, ?2, ?3, ?4, ?5, 1)",
            params![hash.as_str(), input, data, now, now],
        ).map_err(|e| NLPError::ConfigError(format!("Failed to store cache entry: {}", e)))?;

        Ok(())
    }

    /// Clear all entries from the cache
    pub fn clear(&self) -> Result<(), NLPError> {
        self.conn.execute("DELETE FROM nlp_responses", [])
            .map_err(|e| NLPError::ConfigError(format!("Failed to clear cache: {}", e)))?;
        Ok(())
    }

    /// Remove expired entries
    pub fn cleanup(&self) -> Result<usize, NLPError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| NLPError::ConfigError(format!("Time error: {}", e)))?
            .as_secs() as i64;

        let cutoff = now - self.ttl_seconds;

        let deleted = self.conn.execute(
            "DELETE FROM nlp_responses WHERE cached_at < ?1",
            [cutoff],
        ).map_err(|e| NLPError::ConfigError(format!("Failed to cleanup cache: {}", e)))?;

        Ok(deleted)
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let total = self.conn.query_row(
            "SELECT COUNT(*) FROM nlp_responses",
            [],
            |row| row.get::<_, i64>(0)
        ).unwrap_or(0);

        let total_size = self.conn.query_row(
            "SELECT SUM(LENGTH(response_data)) FROM nlp_responses",
            [],
            |row| row.get::<_, i64>(0)
        ).unwrap_or(0);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let cutoff = now - self.ttl_seconds;

        let expired = self.conn.query_row(
            "SELECT COUNT(*) FROM nlp_responses WHERE cached_at < ?1",
            [cutoff],
            |row| row.get::<_, i64>(0)
        ).unwrap_or(0);

        let total_accesses = self.conn.query_row(
            "SELECT SUM(access_count) FROM nlp_responses",
            [],
            |row| row.get::<_, i64>(0)
        ).unwrap_or(0);

        CacheStats {
            total_entries: total as usize,
            total_bytes: total_size as usize,
            expired_entries: expired as usize,
            total_accesses: total_accesses as u32,
            ttl_seconds: self.ttl_seconds,
        }
    }

    /// Change the TTL for cache entries
    pub fn set_ttl(&mut self, ttl_seconds: i64) {
        self.ttl_seconds = ttl_seconds;
    }

    /// Get current TTL
    pub fn ttl(&self) -> i64 {
        self.ttl_seconds
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Total number of cached entries
    pub total_entries: usize,
    /// Total size in bytes
    pub total_bytes: usize,
    /// Number of expired entries (not yet cleaned up)
    pub expired_entries: usize,
    /// Total number of accesses across all entries
    pub total_accesses: u32,
    /// Time-to-live in seconds
    pub ttl_seconds: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_cache() -> (ResponseCache, tempfile::NamedTempFile) {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let cache = ResponseCache::new(temp_file.path()).unwrap();
        (cache, temp_file)
    }

    #[test]
    fn test_cache_miss() {
        let (cache, _temp) = create_test_cache();
        assert!(cache.get("nonexistent input").is_none());
    }

    #[test]
    fn test_cache_put_and_get() {
        let (cache, _temp) = create_test_cache();

        let command = NLPCommand {
            action: ActionType::Task,
            content: "test task".to_string(),
            category: Some("work".to_string()),
            ..Default::default()
        };

        cache.put("add test task", &command).unwrap();

        let retrieved = cache.get("add test task");
        assert!(retrieved.is_some());

        let cmd = retrieved.unwrap();
        assert_eq!(cmd.action, ActionType::Task);
        assert_eq!(cmd.content, "test task");
        assert_eq!(cmd.category, Some("work".to_string()));
    }

    #[test]
    fn test_cache_normalization() {
        let (cache, _temp) = create_test_cache();

        let command = NLPCommand {
            action: ActionType::Task,
            content: "test".to_string(),
            ..Default::default()
        };

        cache.put("  ADD Test Task  ", &command).unwrap();

        // Should find with different casing/spacing
        assert!(cache.get("add test task").is_some());
        assert!(cache.get("ADD TEST TASK").is_some());
        assert!(cache.get("  add  test  task  ").is_some());
    }

    #[test]
    fn test_cache_expiration() {
        let (mut cache, _temp) = create_test_cache();
        cache.set_ttl(1); // 1 second TTL

        let command = NLPCommand {
            action: ActionType::Task,
            content: "test".to_string(),
            ..Default::default()
        };

        cache.put("test", &command).unwrap();

        // Should be available immediately
        assert!(cache.get("test").is_some());

        // Wait for expiration
        std::thread::sleep(std::time::Duration::from_secs(2));

        // Should be expired
        assert!(cache.get("test").is_none());
    }

    #[test]
    fn test_cache_clear() {
        let (cache, _temp) = create_test_cache();

        let command = NLPCommand {
            action: ActionType::Task,
            content: "test".to_string(),
            ..Default::default()
        };

        cache.put("test1", &command).unwrap();
        cache.put("test2", &command).unwrap();

        assert!(cache.get("test1").is_some());
        assert!(cache.get("test2").is_some());

        cache.clear().unwrap();

        assert!(cache.get("test1").is_none());
        assert!(cache.get("test2").is_none());
    }

    #[test]
    fn test_cache_cleanup() {
        let (mut cache, _temp) = create_test_cache();
        cache.set_ttl(1);

        let command = NLPCommand {
            action: ActionType::Task,
            content: "test".to_string(),
            ..Default::default()
        };

        cache.put("old_entry", &command).unwrap();

        // Wait for expiration
        std::thread::sleep(std::time::Duration::from_secs(2));

        // After 2 seconds, with TTL of 1, the old entry is expired
        // Add new entry which is fresh
        cache.put("new_entry", &command).unwrap();

        let stats = cache.stats();
        assert_eq!(stats.total_entries, 2);
        // Only old_entry is expired (cached >1 sec ago), new_entry is fresh
        assert_eq!(stats.expired_entries, 1);

        // Cleanup should remove expired entries
        let removed = cache.cleanup().unwrap();
        assert_eq!(removed, 1);

        let stats = cache.stats();
        // Only new_entry remains
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.expired_entries, 0);
    }

    #[test]
    fn test_cache_stats() {
        let (cache, _temp) = create_test_cache();

        let stats = cache.stats();
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.total_bytes, 0);
        assert_eq!(stats.total_accesses, 0);

        let command = NLPCommand {
            action: ActionType::Task,
            content: "test task".to_string(),
            ..Default::default()
        };

        cache.put("test", &command).unwrap();

        let stats = cache.stats();
        assert_eq!(stats.total_entries, 1);
        assert!(stats.total_bytes > 0);

        // Access the entry to increase access count
        cache.get("test");
        cache.get("test");

        let stats = cache.stats();
        // Each get increments access_count: initial put(1) + get1(2) + get2(3) = 3
        assert_eq!(stats.total_accesses, 3);
    }

    #[test]
    fn test_cache_different_inputs() {
        let (cache, _temp) = create_test_cache();

        let cmd1 = NLPCommand {
            action: ActionType::Task,
            content: "first".to_string(),
            ..Default::default()
        };

        let cmd2 = NLPCommand {
            action: ActionType::Record,
            content: "second".to_string(),
            ..Default::default()
        };

        cache.put("input 1", &cmd1).unwrap();
        cache.put("input 2", &cmd2).unwrap();

        assert_eq!(cache.get("input 1").unwrap().action, ActionType::Task);
        assert_eq!(cache.get("input 2").unwrap().action, ActionType::Record);
    }

    #[test]
    fn test_cache_update_existing() {
        let (cache, _temp) = create_test_cache();

        let cmd1 = NLPCommand {
            action: ActionType::Task,
            content: "original".to_string(),
            ..Default::default()
        };

        let cmd2 = NLPCommand {
            action: ActionType::Task,
            content: "updated".to_string(),
            ..Default::default()
        };

        cache.put("same input", &cmd1).unwrap();
        cache.put("same input", &cmd2).unwrap();

        // Should get the updated command
        assert_eq!(cache.get("same input").unwrap().content, "updated");
    }

    #[test]
    fn test_cache_unicode() {
        let (cache, _temp) = create_test_cache();

        let command = NLPCommand {
            action: ActionType::Task,
            content: "Task with emoji 🎉".to_string(),
            ..Default::default()
        };

        cache.put("add task with emoji 🎉", &command).unwrap();

        let retrieved = cache.get("add task with emoji 🎉").unwrap();
        assert!(retrieved.content.contains("🎉"));
    }

    #[test]
    fn test_cache_with_compound_command() {
        let (cache, _temp) = create_test_cache();

        let mut command = NLPCommand {
            action: ActionType::Task,
            content: "primary".to_string(),
            ..Default::default()
        };

        command.add_compound_command(NLPCommand {
            action: ActionType::Done,
            content: "secondary".to_string(),
            ..Default::default()
        });

        cache.put("compound input", &command).unwrap();

        let retrieved = cache.get("compound input").unwrap();
        assert!(retrieved.is_compound());
        assert_eq!(retrieved.compound().unwrap().len(), 1);
    }

    #[test]
    fn test_cache_hash_consistency() {
        let (cache, _temp) = create_test_cache();

        // Same input should produce same hash
        let hash1 = cache.hash_input("Test Input");
        let hash2 = cache.hash_input("test input");
        let hash3 = cache.hash_input("  TEST  INPUT  ");

        assert_eq!(hash1, hash2);
        assert_eq!(hash2, hash3);

        // Different input should produce different hash
        let hash4 = cache.hash_input("Different Input");
        assert_ne!(hash1, hash4);
    }

    #[test]
    fn test_cache_set_ttl() {
        let (mut cache, _temp) = create_test_cache();
        assert_eq!(cache.ttl(), 7 * 24 * 3600); // Default 7 days

        cache.set_ttl(3600);
        assert_eq!(cache.ttl(), 3600);
    }

    #[test]
    fn test_cache_with_all_command_fields() {
        let (cache, _temp) = create_test_cache();

        let mut filters = std::collections::HashMap::new();
        filters.insert("key".to_string(), "value".to_string());

        let mut modifications = std::collections::HashMap::new();
        modifications.insert("content".to_string(), "new".to_string());

        let command = NLPCommand {
            action: ActionType::Update,
            content: "test content".to_string(),
            category: Some("work".to_string()),
            deadline: Some("tomorrow".to_string()),
            schedule: Some("daily".to_string()),
            status: Some(StatusType::Ongoing),
            query_type: Some(QueryType::Overdue),
            search: Some("keyword".to_string()),
            filters,
            modifications,
            days: Some(7),
            limit: Some(10),
            compound_commands: None,
            condition: None,
        };

        cache.put("complex input", &command).unwrap();

        let retrieved = cache.get("complex input").unwrap();
        assert_eq!(retrieved.action, ActionType::Update);
        assert_eq!(retrieved.content, "test content");
        assert_eq!(retrieved.category, Some("work".to_string()));
        assert_eq!(retrieved.deadline, Some("tomorrow".to_string()));
        assert_eq!(retrieved.schedule, Some("daily".to_string()));
        assert_eq!(retrieved.status, Some(StatusType::Ongoing));
        assert_eq!(retrieved.query_type, Some(QueryType::Overdue));
        assert_eq!(retrieved.search, Some("keyword".to_string()));
        assert_eq!(retrieved.filters.len(), 1);
        assert_eq!(retrieved.modifications.len(), 1);
        assert_eq!(retrieved.days, Some(7));
        assert_eq!(retrieved.limit, Some(10));
    }

    #[test]
    fn test_cache_persistence_across_instances() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path();

        let command = NLPCommand {
            action: ActionType::Task,
            content: "persistent task".to_string(),
            ..Default::default()
        };

        // Create first cache instance and store data
        {
            let cache1 = ResponseCache::new(path).unwrap();
            cache1.put("test", &command).unwrap();
        } // cache1 is dropped here

        // Create new cache instance and verify data persists
        let cache2 = ResponseCache::new(path).unwrap();
        let retrieved = cache2.get("test").unwrap();
        assert_eq!(retrieved.content, "persistent task");
    }

    #[test]
    fn test_cache_empty_input() {
        let (cache, _temp) = create_test_cache();

        let command = NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            ..Default::default()
        };

        cache.put("", &command).unwrap();
        assert!(cache.get("").is_some());
        assert!(cache.get("   ").is_some()); // Whitespace normalizes to empty
    }
}
