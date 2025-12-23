//! Personalized pattern recognition for natural language commands
//!
//! This module provides user-specific pattern recognition that adapts to individual
//! users' preferred terminology, phrasing, and command patterns over time.

use super::types::{NLPCommand, ActionType};
use rusqlite::params;
use std::path::Path;
use std::collections::HashMap;

/// User profile tracking individual patterns
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserProfile {
    /// Unique user identifier (based on system username)
    pub user_id: String,
    /// Timestamp when profile was created
    pub created_at: i64,
    /// Timestamp of last activity
    pub last_active: i64,
    /// Total commands processed
    pub total_commands: u32,
    /// Preferred action mappings
    pub preferred_actions: HashMap<String, ActionType>,
    /// Preferred category mappings
    pub preferred_categories: HashMap<String, String>,
    /// Common phrases and their interpretations
    pub common_phrases: HashMap<String, PhrasePattern>,
}

/// A phrase pattern used by the user
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PhrasePattern {
    /// The phrase pattern
    pub pattern: String,
    /// How often this pattern is used
    pub usage_count: u32,
    /// The intended action
    pub action: ActionType,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// First seen timestamp
    pub first_seen: i64,
    /// Last seen timestamp
    pub last_seen: i64,
}

/// Frequency tracking for command patterns
#[derive(Debug, Clone)]
pub struct PatternFrequency {
    /// The pattern being tracked
    pub pattern: String,
    /// Usage count
    pub count: u32,
    /// First used timestamp
    pub first_used: i64,
    /// Last used timestamp
    pub last_used: i64,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
}

/// Personalized shortcut for frequently used commands
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PersonalizedShortcut {
    /// Shortcut name/alias
    pub shortcut: String,
    /// The full command it expands to
    pub command: NLPCommand,
    /// How many times used
    pub usage_count: u32,
    /// Confidence score
    pub confidence: f64,
    /// Created timestamp
    pub created_at: i64,
    /// Last used timestamp
    pub last_used_at: i64,
}

/// Database for storing personalization data
pub struct PersonalizationDB {
    pub conn: rusqlite::Connection,
    pub user_id: String,
}

impl PersonalizationDB {
    /// Create a new personalization database at the specified path
    pub fn new<P: AsRef<Path>>(path: P, user_id: String) -> Result<Self, crate::nlp::NLPError> {
        let conn = rusqlite::Connection::open(path)
            .map_err(|e| crate::nlp::NLPError::ConfigError(format!("Failed to open personalization database: {}", e)))?;

        // Create user_profiles table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS user_profiles (
                user_id TEXT PRIMARY KEY,
                created_at INTEGER NOT NULL,
                last_active INTEGER NOT NULL,
                total_commands INTEGER NOT NULL DEFAULT 0
            )",
            [],
        ).map_err(|e| crate::nlp::NLPError::ConfigError(format!("Failed to create user_profiles table: {}", e)))?;

        // Create command_patterns table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS command_patterns (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT NOT NULL,
                pattern TEXT NOT NULL,
                action TEXT NOT NULL,
                usage_count INTEGER NOT NULL DEFAULT 1,
                success_rate REAL NOT NULL DEFAULT 1.0,
                first_used INTEGER NOT NULL,
                last_used INTEGER NOT NULL,
                UNIQUE(user_id, pattern)
            )",
            [],
        ).map_err(|e| crate::nlp::NLPError::ConfigError(format!("Failed to create command_patterns table: {}", e)))?;

        // Create preferred_categories table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS preferred_categories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT NOT NULL,
                input_word TEXT NOT NULL,
                category TEXT NOT NULL,
                usage_count INTEGER NOT NULL DEFAULT 1,
                confidence REAL NOT NULL DEFAULT 0.5,
                UNIQUE(user_id, input_word)
            )",
            [],
        ).map_err(|e| crate::nlp::NLPError::ConfigError(format!("Failed to create preferred_categories table: {}", e)))?;

        // Create shortcuts table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS shortcuts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT NOT NULL,
                shortcut TEXT NOT NULL,
                command_data BLOB NOT NULL,
                usage_count INTEGER NOT NULL DEFAULT 1,
                confidence REAL NOT NULL DEFAULT 0.5,
                created_at INTEGER NOT NULL,
                last_used_at INTEGER NOT NULL,
                UNIQUE(user_id, shortcut)
            )",
            [],
        ).map_err(|e| crate::nlp::NLPError::ConfigError(format!("Failed to create shortcuts table: {}", e)))?;

        // Create indexes for faster lookups
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_cmd_patterns_user ON command_patterns(user_id)",
            [],
        ).map_err(|e| crate::nlp::NLPError::ConfigError(format!("Failed to create index: {}", e)))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pref_cats_user ON preferred_categories(user_id)",
            [],
        ).map_err(|e| crate::nlp::NLPError::ConfigError(format!("Failed to create index: {}", e)))?;

        Ok(Self { conn, user_id })
    }

    /// Initialize user profile if not exists
    pub fn ensure_profile(&self) -> Result<(), crate::nlp::NLPError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| crate::nlp::NLPError::ConfigError(format!("Time error: {}", e)))?
            .as_secs() as i64;

        self.conn.execute(
            "INSERT OR IGNORE INTO user_profiles (user_id, created_at, last_active, total_commands)
             VALUES (?1, ?2, ?3, 0)",
            params![&self.user_id, now, now],
        ).map_err(|e| crate::nlp::NLPError::ConfigError(format!("Failed to create user profile: {}", e)))?;

        Ok(())
    }

    /// Record a command pattern
    pub fn record_pattern(&self, pattern: &str, action: &ActionType, success: bool) -> Result<(), crate::nlp::NLPError> {
        self.ensure_profile()?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| crate::nlp::NLPError::ConfigError(format!("Time error: {}", e)))?
            .as_secs() as i64;

        let action_str = format!("{:?}", action);

        // Check if pattern exists
        let existing = self.conn.query_row(
            "SELECT id, usage_count, success_rate FROM command_patterns
             WHERE user_id = ?1 AND pattern = ?2",
            params![&self.user_id, pattern],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, u32>(1)?,
                    row.get::<_, f64>(2)?,
                ))
            },
        );

        match existing {
            Ok((id, count, rate)) => {
                // Update existing pattern
                let new_count = count + 1;
                // Update success rate using exponential moving average
                let new_rate = if success {
                    (rate * 0.9) + (1.0 * 0.1)
                } else {
                    (rate * 0.9) + (0.0 * 0.1)
                };

                self.conn.execute(
                    "UPDATE command_patterns
                     SET usage_count = ?1, success_rate = ?2, last_used = ?3
                     WHERE id = ?4",
                    params![new_count, new_rate, now, id],
                ).map_err(|e| crate::nlp::NLPError::ConfigError(format!("Failed to update pattern: {}", e)))?;
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                // Insert new pattern
                self.conn.execute(
                    "INSERT INTO command_patterns (user_id, pattern, action, usage_count, success_rate, first_used, last_used)
                     VALUES (?1, ?2, ?3, 1, ?4, ?5, ?5)",
                    params![&self.user_id, pattern, &action_str, if success { 1.0 } else { 0.0 }, now],
                ).map_err(|e| crate::nlp::NLPError::ConfigError(format!("Failed to insert pattern: {}", e)))?;
            }
            Err(e) => return Err(crate::nlp::NLPError::ConfigError(format!("Database error: {}", e))),
        }

        // Update total command count
        self.conn.execute(
            "UPDATE user_profiles SET total_commands = total_commands + 1, last_active = ?1 WHERE user_id = ?2",
            params![now, &self.user_id],
        ).map_err(|e| crate::nlp::NLPError::ConfigError(format!("Failed to update profile: {}", e)))?;

        Ok(())
    }

    /// Get frequent patterns for the user
    pub fn get_frequent_patterns(&self, min_count: u32) -> Result<Vec<PatternFrequency>, crate::nlp::NLPError> {
        let mut stmt = self.conn.prepare(
            "SELECT pattern, usage_count, first_used, last_used, success_rate
             FROM command_patterns
             WHERE user_id = ?1 AND usage_count >= ?2
             ORDER BY usage_count DESC"
        ).map_err(|e| crate::nlp::NLPError::ConfigError(format!("Failed to prepare query: {}", e)))?;

        let patterns = stmt.query_map(params![&self.user_id, min_count], |row| {
            Ok(PatternFrequency {
                pattern: row.get(0)?,
                count: row.get(1)?,
                first_used: row.get(2)?,
                last_used: row.get(3)?,
                success_rate: row.get(4)?,
            })
        }).map_err(|e| crate::nlp::NLPError::ConfigError(format!("Failed to query patterns: {}", e)))?
        .flatten()
        .collect();

        Ok(patterns)
    }

    /// Record a preferred category mapping
    pub fn record_category_preference(&self, input_word: &str, category: &str) -> Result<(), crate::nlp::NLPError> {
        self.ensure_profile()?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| crate::nlp::NLPError::ConfigError(format!("Time error: {}", e)))?
            .as_secs() as i64;

        // Check if mapping exists
        let existing = self.conn.query_row(
            "SELECT id, usage_count, confidence FROM preferred_categories
             WHERE user_id = ?1 AND input_word = ?2",
            params![&self.user_id, input_word],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, u32>(1)?,
                    row.get::<_, f64>(2)?,
                ))
            },
        );

        match existing {
            Ok((id, count, conf)) => {
                // Update with increased confidence
                let new_count = count + 1;
                let new_confidence = (conf + 0.1).min(0.95);

                self.conn.execute(
                    "UPDATE preferred_categories
                     SET usage_count = ?1, confidence = ?2
                     WHERE id = ?3",
                    params![new_count, new_confidence, id],
                ).map_err(|e| crate::nlp::NLPError::ConfigError(format!("Failed to update category pref: {}", e)))?;
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                // Insert new mapping
                self.conn.execute(
                    "INSERT INTO preferred_categories (user_id, input_word, category, usage_count, confidence)
                     VALUES (?1, ?2, ?3, 1, 0.5)",
                    params![&self.user_id, input_word, category],
                ).map_err(|e| crate::nlp::NLPError::ConfigError(format!("Failed to insert category pref: {}", e)))?;
            }
            Err(e) => return Err(crate::nlp::NLPError::ConfigError(format!("Database error: {}", e))),
        }

        Ok(())
    }

    /// Get preferred category for a word
    pub fn get_preferred_category(&self, input_word: &str) -> Option<String> {
        self.conn.query_row(
            "SELECT category FROM preferred_categories
             WHERE user_id = ?1 AND input_word = ?2 AND confidence >= 0.5",
            params![&self.user_id, input_word],
            |row| row.get(0)
        ).ok()
    }

    /// Create a personalized shortcut
    pub fn create_shortcut(&self, shortcut: &str, command: &NLPCommand) -> Result<(), crate::nlp::NLPError> {
        self.ensure_profile()?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| crate::nlp::NLPError::ConfigError(format!("Time error: {}", e)))?
            .as_secs() as i64;

        // Store shortcut in lowercase for case-insensitive matching
        let shortcut_lower = shortcut.to_lowercase();

        let command_data = serde_json::to_vec(command)
            .map_err(|e| crate::nlp::NLPError::SerializationError(serde_json::Error::from(e)))?;

        self.conn.execute(
            "INSERT OR REPLACE INTO shortcuts (user_id, shortcut, command_data, usage_count, confidence, created_at, last_used_at)
             VALUES (?1, ?2, ?3, 1, 0.5, ?4, ?4)",
            params![&self.user_id, shortcut_lower, command_data, now],
        ).map_err(|e| crate::nlp::NLPError::ConfigError(format!("Failed to create shortcut: {}", e)))?;

        Ok(())
    }

    /// Get a shortcut
    pub fn get_shortcut(&self, shortcut: &str) -> Option<PersonalizedShortcut> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_secs() as i64;

        // Lowercase the shortcut for case-insensitive matching
        let shortcut_lower = shortcut.to_lowercase();

        let result = self.conn.query_row(
            "SELECT id, shortcut, command_data, usage_count, confidence, created_at
             FROM shortcuts WHERE user_id = ?1 AND shortcut = ?2",
            params![&self.user_id, shortcut_lower],
            |row| {
                let data: Vec<u8> = row.get(2)?;
                let command: NLPCommand = serde_json::from_slice(&data)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    command,
                    row.get::<_, u32>(3)?,
                    row.get::<_, f64>(4)?,
                    row.get::<_, i64>(5)?,
                ))
            },
        ).ok()?;

        // Update last_used_at and usage_count
        let _ = self.conn.execute(
            "UPDATE shortcuts SET usage_count = usage_count + 1, last_used_at = ?1 WHERE id = ?2",
            params![now, result.0],
        );

        Some(PersonalizedShortcut {
            shortcut: result.1,
            command: result.2,
            usage_count: result.3 + 1,
            confidence: result.4,
            created_at: result.5,
            last_used_at: now,
        })
    }

    /// Get all shortcuts
    pub fn get_all_shortcuts(&self) -> Result<Vec<PersonalizedShortcut>, crate::nlp::NLPError> {
        let mut stmt = self.conn.prepare(
            "SELECT shortcut, command_data, usage_count, confidence, created_at, last_used_at
             FROM shortcuts WHERE user_id = ?1"
        ).map_err(|e| crate::nlp::NLPError::ConfigError(format!("Failed to prepare query: {}", e)))?;

        let shortcuts = stmt.query_map(params![&self.user_id], |row| {
            let data: Vec<u8> = row.get(1)?;
            let command: NLPCommand = serde_json::from_slice(&data)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
            Ok(PersonalizedShortcut {
                shortcut: row.get(0)?,
                command,
                usage_count: row.get(2)?,
                confidence: row.get(3)?,
                created_at: row.get(4)?,
                last_used_at: row.get(5)?,
            })
        }).map_err(|e| crate::nlp::NLPError::ConfigError(format!("Failed to query shortcuts: {}", e)))?
        .flatten()
        .collect();

        Ok(shortcuts)
    }

    /// Get personalization statistics
    pub fn get_stats(&self) -> Result<PersonalizationStats, crate::nlp::NLPError> {
        let total_patterns: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM command_patterns WHERE user_id = ?1",
            params![&self.user_id],
            |row| row.get(0),
        ).unwrap_or(0);

        let total_shortcuts: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM shortcuts WHERE user_id = ?1",
            params![&self.user_id],
            |row| row.get(0),
        ).unwrap_or(0);

        let total_categories: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM preferred_categories WHERE user_id = ?1",
            params![&self.user_id],
            |row| row.get(0),
        ).unwrap_or(0);

        let total_commands: i64 = self.conn.query_row(
            "SELECT total_commands FROM user_profiles WHERE user_id = ?1",
            params![&self.user_id],
            |row| row.get(0),
        ).unwrap_or(0);

        let created_at: i64 = self.conn.query_row(
            "SELECT created_at FROM user_profiles WHERE user_id = ?1",
            params![&self.user_id],
            |row| row.get(0),
        ).unwrap_or(0);

        let last_active: i64 = self.conn.query_row(
            "SELECT last_active FROM user_profiles WHERE user_id = ?1",
            params![&self.user_id],
            |row| row.get(0),
        ).unwrap_or(0);

        Ok(PersonalizationStats {
            user_id: self.user_id.clone(),
            total_patterns: total_patterns as usize,
            total_shortcuts: total_shortcuts as usize,
            total_categories: total_categories as usize,
            total_commands: total_commands as u32,
            profile_created_at: created_at,
            last_active,
        })
    }

    /// Clear all personalization data for the user
    pub fn clear(&self) -> Result<(), crate::nlp::NLPError> {
        self.conn.execute("DELETE FROM command_patterns WHERE user_id = ?1", params![&self.user_id])
            .map_err(|e| crate::nlp::NLPError::ConfigError(format!("Failed to clear patterns: {}", e)))?;

        self.conn.execute("DELETE FROM shortcuts WHERE user_id = ?1", params![&self.user_id])
            .map_err(|e| crate::nlp::NLPError::ConfigError(format!("Failed to clear shortcuts: {}", e)))?;

        self.conn.execute("DELETE FROM preferred_categories WHERE user_id = ?1", params![&self.user_id])
            .map_err(|e| crate::nlp::NLPError::ConfigError(format!("Failed to clear categories: {}", e)))?;

        Ok(())
    }

    /// Export personalization data as JSON
    pub fn export_data(&self) -> Result<String, crate::nlp::NLPError> {
        let profile = UserProfile {
            user_id: self.user_id.clone(),
            created_at: self.conn.query_row(
                "SELECT created_at FROM user_profiles WHERE user_id = ?1",
                params![&self.user_id],
                |row| row.get(0),
            ).unwrap_or(0),
            last_active: self.conn.query_row(
                "SELECT last_active FROM user_profiles WHERE user_id = ?1",
                params![&self.user_id],
                |row| row.get(0),
            ).unwrap_or(0),
            total_commands: self.conn.query_row(
                "SELECT total_commands FROM user_profiles WHERE user_id = ?1",
                params![&self.user_id],
                |row| row.get(0),
            ).unwrap_or(0),
            preferred_actions: HashMap::new(),
            preferred_categories: HashMap::new(),
            common_phrases: HashMap::new(),
        };

        serde_json::to_string_pretty(&profile)
            .map_err(|e| crate::nlp::NLPError::SerializationError(serde_json::Error::from(e)))
    }
}

/// Personalization statistics
#[derive(Debug, Clone)]
pub struct PersonalizationStats {
    pub user_id: String,
    pub total_patterns: usize,
    pub total_shortcuts: usize,
    pub total_categories: usize,
    pub total_commands: u32,
    pub profile_created_at: i64,
    pub last_active: i64,
}

/// Pattern matcher for personalized patterns
pub struct PersonalizedPatternMatcher {
    db: Option<PersonalizationDB>,
}

impl PersonalizedPatternMatcher {
    /// Create a new matcher without database
    pub fn new() -> Self {
        Self { db: None }
    }

    /// Create with database
    pub fn with_db(db: PersonalizationDB) -> Self {
        Self { db: Some(db) }
    }

    /// Match input against personalized patterns
    pub fn match_pattern(&self, input: &str) -> Option<NLPCommand> {
        let db = self.db.as_ref()?;

        // Check for shortcuts first
        let normalized = input.trim().to_lowercase();
        if let Some(shortcut) = db.get_shortcut(&normalized) {
            return Some(shortcut.command);
        }

        // Check for category preferences
        let words: Vec<&str> = normalized.split_whitespace().collect();
        for word in &words {
            if let Some(category) = db.get_preferred_category(word) {
                // Return a command with this category
                return Some(NLPCommand {
                    category: Some(category),
                    ..Default::default()
                });
            }
        }

        None
    }

    /// Suggest personalized completions for input
    pub fn suggest_completions(&self, input: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        let db = match &self.db {
            Some(d) => d,
            None => return suggestions,
        };

        let normalized = input.trim().to_lowercase();

        // Get matching shortcuts
        if let Ok(shortcuts) = db.get_all_shortcuts() {
            for shortcut in shortcuts {
                if shortcut.shortcut.starts_with(&normalized) {
                    suggestions.push(shortcut.shortcut);
                }
            }
        }

        suggestions
    }
}

impl Default for PersonalizedPatternMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Personalization engine for coordinating all personalization features
pub struct PersonalizationEngine {
    db: Option<PersonalizationDB>,
    matcher: PersonalizedPatternMatcher,
}

impl PersonalizationEngine {
    /// Create a new personalization engine
    pub fn new() -> Self {
        Self {
            db: None,
            matcher: PersonalizedPatternMatcher::new(),
        }
    }

    /// Initialize with database path
    pub fn with_db<P: AsRef<Path>>(path: P, user_id: String) -> Result<Self, crate::nlp::NLPError> {
        let path_ref = path.as_ref();
        let db = PersonalizationDB::new(path_ref, user_id.clone())?;
        let matcher = PersonalizedPatternMatcher::with_db(
            PersonalizationDB::new(path_ref, user_id)?
        );

        // Ensure profile exists
        db.ensure_profile()?;

        Ok(Self {
            db: Some(db),
            matcher,
        })
    }

    /// Record a command for pattern learning
    pub fn record_command(&self, input: &str, command: &NLPCommand, success: bool) -> Result<(), crate::nlp::NLPError> {
        if let Some(ref db) = self.db {
            // Extract key phrases from input
            let normalized = input.trim().to_lowercase();
            let words: Vec<&str> = normalized.split_whitespace().collect();

            // Record the main action pattern
            let pattern = if words.is_empty() {
                normalized.clone()
            } else {
                // Use first two words as pattern
                words.iter().take(2).cloned().collect::<Vec<_>>().join(" ")
            };
            db.record_pattern(&pattern, &command.action, success)?;

            // Record category preferences
            if let Some(ref category) = command.category {
                for word in &words {
                    if word.len() > 3 && *word != category.as_str() {
                        let _ = db.record_category_preference(word, category);
                    }
                }
            }
        }
        Ok(())
    }

    /// Get personalized command for input
    pub fn get_personalized_command(&self, input: &str) -> Option<NLPCommand> {
        self.matcher.match_pattern(input)
    }

    /// Get suggestions for input
    pub fn get_suggestions(&self, input: &str) -> Vec<String> {
        self.matcher.suggest_completions(input)
    }

    /// Create a shortcut
    pub fn create_shortcut(&self, shortcut: &str, command: &NLPCommand) -> Result<(), crate::nlp::NLPError> {
        if let Some(ref db) = self.db {
            db.create_shortcut(shortcut, command)?;
        }
        Ok(())
    }

    /// Get personalization statistics
    pub fn get_stats(&self) -> Option<PersonalizationStats> {
        self.db.as_ref().and_then(|db| db.get_stats().ok())
    }

    /// Clear all personalization data
    pub fn clear(&self) -> Result<(), crate::nlp::NLPError> {
        if let Some(ref db) = self.db {
            db.clear()?;
        }
        Ok(())
    }

    /// Export personalization data
    pub fn export(&self) -> Result<String, crate::nlp::NLPError> {
        if let Some(ref db) = self.db {
            db.export_data()
        } else {
            Err(crate::nlp::NLPError::ConfigError("No database available".to_string()))
        }
    }

    /// Get frequent patterns
    pub fn get_frequent_patterns(&self, min_count: u32) -> Result<Vec<PatternFrequency>, crate::nlp::NLPError> {
        if let Some(ref db) = self.db {
            db.get_frequent_patterns(min_count)
        } else {
            Ok(Vec::new())
        }
    }

    /// Get all shortcuts
    pub fn get_shortcuts(&self) -> Result<Vec<PersonalizedShortcut>, crate::nlp::NLPError> {
        if let Some(ref db) = self.db {
            db.get_all_shortcuts()
        } else {
            Ok(Vec::new())
        }
    }
}

impl Default for PersonalizationEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Get current user ID for personalization
pub fn get_user_id() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "default".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_personalization_db() -> (PersonalizationDB, tempfile::NamedTempFile) {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let user_id = "test_user".to_string();
        let db = PersonalizationDB::new(temp_file.path(), user_id).unwrap();
        (db, temp_file)
    }

    #[test]
    fn test_personalization_db_new() {
        let (db, _temp) = create_test_personalization_db();
        assert_eq!(db.user_id, "test_user");
    }

    #[test]
    fn test_ensure_profile() {
        let (db, _temp) = create_test_personalization_db();
        db.ensure_profile().unwrap();
        let stats = db.get_stats().unwrap();
        assert_eq!(stats.user_id, "test_user");
        assert_eq!(stats.total_commands, 0);
    }

    #[test]
    fn test_record_pattern() {
        let (db, _temp) = create_test_personalization_db();
        db.record_pattern("add task", &ActionType::Task, true).unwrap();

        let patterns = db.get_frequent_patterns(1).unwrap();
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].pattern, "add task");
        assert_eq!(patterns[0].count, 1);
    }

    #[test]
    fn test_record_pattern_increments_count() {
        let (db, _temp) = create_test_personalization_db();
        db.record_pattern("list", &ActionType::List, true).unwrap();
        db.record_pattern("list", &ActionType::List, true).unwrap();

        let patterns = db.get_frequent_patterns(1).unwrap();
        assert_eq!(patterns[0].count, 2);
    }

    #[test]
    fn test_record_pattern_updates_success_rate() {
        let (db, _temp) = create_test_personalization_db();
        db.record_pattern("delete", &ActionType::Delete, true).unwrap();
        db.record_pattern("delete", &ActionType::Delete, false).unwrap();

        let patterns = db.get_frequent_patterns(1).unwrap();
        // After success (1.0) and failure (0.0), rate should be ~0.9
        assert!(patterns[0].success_rate > 0.8);
        assert!(patterns[0].success_rate < 1.0);
    }

    #[test]
    fn test_record_category_preference() {
        let (db, _temp) = create_test_personalization_db();
        db.record_category_preference("office", "work").unwrap();

        let category = db.get_preferred_category("office");
        assert_eq!(category, Some("work".to_string()));
    }

    #[test]
    fn test_get_preferred_category_none() {
        let (db, _temp) = create_test_personalization_db();
        let category = db.get_preferred_category("nonexistent");
        assert!(category.is_none());
    }

    #[test]
    fn test_create_shortcut() {
        let (db, _temp) = create_test_personalization_db();

        let command = NLPCommand {
            action: ActionType::List,
            content: "all work tasks".to_string(),
            category: Some("work".to_string()),
            ..Default::default()
        };

        // First verify that creation succeeds
        let result = db.create_shortcut("lw", &command);
        assert!(result.is_ok(), "create_shortcut failed: {:?}", result.err());

        // Check if we can get the shortcut
        let retrieved = db.get_shortcut("lw");
        assert!(retrieved.is_some(), "get_shortcut returned None");
        assert_eq!(retrieved.unwrap().shortcut, "lw");
    }

    #[test]
    fn test_get_shortcut_none() {
        let (db, _temp) = create_test_personalization_db();
        db.ensure_profile().unwrap();
        let shortcut = db.get_shortcut("nonexistent");
        assert!(shortcut.is_none());
    }

    #[test]
    fn test_get_all_shortcuts() {
        let (db, _temp) = create_test_personalization_db();

        let cmd1 = NLPCommand {
            action: ActionType::List,
            ..Default::default()
        };

        let cmd2 = NLPCommand {
            action: ActionType::Task,
            content: "test".to_string(),
            ..Default::default()
        };

        db.create_shortcut("list", &cmd1).unwrap();
        db.create_shortcut("task", &cmd2).unwrap();

        let shortcuts = db.get_all_shortcuts().unwrap();
        assert_eq!(shortcuts.len(), 2);
    }

    #[test]
    fn test_personalization_stats() {
        let (db, _temp) = create_test_personalization_db();
        db.record_pattern("test", &ActionType::Task, true).unwrap();
        db.record_category_preference("office", "work").unwrap();

        let stats = db.get_stats().unwrap();
        assert_eq!(stats.total_patterns, 1);
        assert_eq!(stats.total_categories, 1);
        assert_eq!(stats.total_commands, 1);
    }

    #[test]
    fn test_clear_personalization_data() {
        let (db, _temp) = create_test_personalization_db();
        db.record_pattern("test", &ActionType::Task, true).unwrap();
        db.create_shortcut("s", &NLPCommand::default()).unwrap();

        db.clear().unwrap();

        let stats = db.get_stats().unwrap();
        assert_eq!(stats.total_patterns, 0);
        assert_eq!(stats.total_shortcuts, 0);
    }

    #[test]
    fn test_export_data() {
        let (db, _temp) = create_test_personalization_db();
        db.ensure_profile().unwrap();

        let exported = db.export_data().unwrap();
        assert!(exported.contains("test_user"));
        assert!(exported.contains("created_at"));
    }

    #[test]
    fn test_pattern_frequency_clone() {
        let freq = PatternFrequency {
            pattern: "test".to_string(),
            count: 5,
            first_used: 100,
            last_used: 200,
            success_rate: 0.9,
        };

        let cloned = freq.clone();
        assert_eq!(freq.pattern, cloned.pattern);
        assert_eq!(freq.count, cloned.count);
    }

    #[test]
    fn test_phrase_pattern_clone() {
        let pattern = PhrasePattern {
            pattern: "test pattern".to_string(),
            usage_count: 10,
            action: ActionType::Task,
            confidence: 0.85,
            first_seen: 100,
            last_seen: 200,
        };

        let cloned = pattern.clone();
        assert_eq!(pattern.pattern, cloned.pattern);
        assert_eq!(pattern.usage_count, cloned.usage_count);
    }

    #[test]
    fn test_personalized_shortcut_clone() {
        let shortcut = PersonalizedShortcut {
            shortcut: "test".to_string(),
            command: NLPCommand::default(),
            usage_count: 5,
            confidence: 0.8,
            created_at: 100,
            last_used_at: 200,
        };

        let cloned = shortcut.clone();
        assert_eq!(shortcut.shortcut, cloned.shortcut);
        assert_eq!(shortcut.usage_count, cloned.usage_count);
    }

    #[test]
    fn test_user_profile_clone() {
        let profile = UserProfile {
            user_id: "test_user".to_string(),
            created_at: 100,
            last_active: 200,
            total_commands: 50,
            preferred_actions: HashMap::new(),
            preferred_categories: HashMap::new(),
            common_phrases: HashMap::new(),
        };

        let cloned = profile.clone();
        assert_eq!(profile.user_id, cloned.user_id);
        assert_eq!(profile.total_commands, cloned.total_commands);
    }

    #[test]
    fn test_personalized_pattern_matcher_new() {
        let matcher = PersonalizedPatternMatcher::new();
        assert!(matcher.match_pattern("test").is_none());
        assert!(matcher.suggest_completions("test").is_empty());
    }

    #[test]
    fn test_personalization_engine_new() {
        let engine = PersonalizationEngine::new();
        assert!(engine.get_stats().is_none());
        assert!(engine.get_shortcuts().unwrap().is_empty());
    }

    #[test]
    fn test_get_user_id() {
        let user_id = get_user_id();
        assert!(!user_id.is_empty());
    }

    #[test]
    fn test_personalization_stats_default() {
        let stats = PersonalizationStats {
            user_id: "test".to_string(),
            total_patterns: 0,
            total_shortcuts: 0,
            total_categories: 0,
            total_commands: 0,
            profile_created_at: 100,
            last_active: 200,
        };

        assert_eq!(stats.total_patterns, 0);
        assert_eq!(stats.user_id, "test");
    }
}
