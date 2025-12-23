//! Learning from user corrections for natural language commands
//!
//! This module provides adaptive learning that improves NLP command parsing
//! based on user corrections. When users correct misinterpreted commands,
//! the system learns from these corrections to provide better results in the future.

use super::types::{NLPCommand, ActionType};
use rusqlite::params;
use std::path::Path;

/// A correction learned from user input
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LearnedCorrection {
    /// The original (incorrect) input
    pub original_input: String,
    /// The corrected/actual command that was intended
    pub intended_command: NLPCommand,
    /// How many times this correction has been confirmed
    pub confirmation_count: u32,
    /// Timestamp when first learned (Unix seconds)
    pub learned_at: i64,
    /// Timestamp of last use (Unix seconds)
    pub last_used_at: i64,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
}

/// Pattern learned from corrections
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LearnedPattern {
    /// Pattern type
    pub pattern_type: PatternType,
    /// The pattern to match
    pub pattern: String,
    /// The correction to apply
    pub correction: PatternCorrection,
    /// How many times this pattern has been confirmed
    pub confirmation_count: u32,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Timestamp when first learned
    pub learned_at: i64,
}

/// Types of learned patterns
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PatternType {
    /// Word substitution (e.g., "finish" -> "done")
    WordSubstitution,
    /// Category mapping (e.g., "office" -> "work")
    CategoryMapping,
    /// Action mapping (e.g., "create" -> "add task")
    ActionMapping,
    /// Deadline interpretation (e.g., "by friday" -> "due this friday")
    DeadlineInterpretation,
    /// Phrase pattern (e.g., "I need to" -> "add task")
    PhrasePattern,
}

/// Correction to apply when pattern matches
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PatternCorrection {
    /// Replace with specific word
    Word(String),
    /// Replace with specific action
    Action(ActionType),
    /// Replace with specific category
    Category(String),
    /// Replace with deadline format
    Deadline(String),
    /// Custom transformation (original -> replacement)
    Transform { original: String, replacement: String },
}

/// Learning database for storing and retrieving corrections
pub struct LearningDB {
    conn: rusqlite::Connection,
}

impl LearningDB {
    /// Create a new learning database at the specified path
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, crate::nlp::NLPError> {
        let conn = rusqlite::Connection::open(path)
            .map_err(|e| crate::nlp::NLPError::ConfigError(format!("Failed to open learning database: {}", e)))?;

        // Create corrections table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS corrections (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                original_input TEXT NOT NULL,
                intended_command BLOB NOT NULL,
                confirmation_count INTEGER NOT NULL DEFAULT 1,
                learned_at INTEGER NOT NULL,
                last_used_at INTEGER NOT NULL,
                confidence REAL NOT NULL DEFAULT 0.5
            )",
            [],
        ).map_err(|e| crate::nlp::NLPError::ConfigError(format!("Failed to create corrections table: {}", e)))?;

        // Create patterns table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS patterns (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                pattern_type TEXT NOT NULL,
                pattern TEXT NOT NULL,
                correction BLOB NOT NULL,
                confirmation_count INTEGER NOT NULL DEFAULT 1,
                confidence REAL NOT NULL DEFAULT 0.5,
                learned_at INTEGER NOT NULL,
                UNIQUE(pattern_type, pattern)
            )",
            [],
        ).map_err(|e| crate::nlp::NLPError::ConfigError(format!("Failed to create patterns table: {}", e)))?;

        // Create indexes for faster lookups
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_original_input ON corrections(original_input)",
            [],
        ).map_err(|e| crate::nlp::NLPError::ConfigError(format!("Failed to create index: {}", e)))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pattern ON patterns(pattern)",
            [],
        ).map_err(|e| crate::nlp::NLPError::ConfigError(format!("Failed to create index: {}", e)))?;

        Ok(Self { conn })
    }

    /// Store a learned correction
    pub fn store_correction(&self, original_input: &str, intended_command: &NLPCommand) -> Result<(), crate::nlp::NLPError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| crate::nlp::NLPError::ConfigError(format!("Time error: {}", e)))?
            .as_secs() as i64;

        // Check if this correction already exists
        let existing = self.conn.query_row(
            "SELECT id, confirmation_count, confidence FROM corrections WHERE original_input = ?1",
            params![original_input],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, u32>(1)?,
                    row.get::<_, f64>(2)?,
                ))
            },
        );

        match existing {
            Ok((id, count, old_confidence)) => {
                // Update existing correction with increased confidence
                let new_count = count + 1;
                let new_confidence = (old_confidence + 0.1).min(0.95); // Increase confidence

                self.conn.execute(
                    "UPDATE corrections SET confirmation_count = ?1, last_used_at = ?2, confidence = ?3 WHERE id = ?4",
                    params![new_count, now, new_confidence, id],
                ).map_err(|e| crate::nlp::NLPError::ConfigError(format!("Failed to update correction: {}", e)))?;
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                // Insert new correction
                let command_data = serde_json::to_vec(intended_command)
                    .map_err(|e| crate::nlp::NLPError::SerializationError(serde_json::Error::from(e)))?;

                self.conn.execute(
                    "INSERT INTO corrections (original_input, intended_command, confirmation_count, learned_at, last_used_at, confidence)
                     VALUES (?1, ?2, 1, ?3, ?3, 0.5)",
                    params![original_input, command_data, now],
                ).map_err(|e| crate::nlp::NLPError::ConfigError(format!("Failed to store correction: {}", e)))?;
            }
            Err(e) => return Err(crate::nlp::NLPError::ConfigError(format!("Database error: {}", e))),
        }

        Ok(())
    }

    /// Get a learned correction for input
    pub fn get_correction(&self, input: &str) -> Option<LearnedCorrection> {
        let normalized_input = input.trim().to_lowercase();

        // Try exact match first
        let result = self.conn.query_row(
            "SELECT intended_command, confirmation_count, learned_at, last_used_at, confidence
             FROM corrections WHERE original_input = ?1",
            params![normalized_input],
            |row| {
                let data: Vec<u8> = row.get(0)?;
                let command: NLPCommand = serde_json::from_slice(&data)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
                Ok((
                    command,
                    row.get::<_, u32>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, f64>(4)?,
                ))
            },
        );

        if let Ok((command, count, learned_at, last_used, confidence)) = result {
            return Some(LearnedCorrection {
                original_input: normalized_input,
                intended_command: command,
                confirmation_count: count,
                learned_at,
                last_used_at: last_used,
                confidence,
            });
        }

        // Try fuzzy match
        self.fuzzy_find_correction(&normalized_input)
    }

    /// Fuzzy search for corrections
    fn fuzzy_find_correction(&self, input: &str) -> Option<LearnedCorrection> {
        let mut stmt = self.conn.prepare(
            "SELECT original_input, intended_command, confirmation_count, learned_at, last_used_at, confidence
             FROM corrections"
        ).ok()?;

        let rows = stmt.query_map([], |row| {
            let data: Vec<u8> = row.get(1)?;
            let command: NLPCommand = serde_json::from_slice(&data)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
            Ok((
                row.get::<_, String>(0)?,
                command,
                row.get::<_, u32>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, f64>(5)?,
            ))
        });

        if rows.is_err() {
            return None;
        }

        let mut best_match: Option<LearnedCorrection> = None;
        let mut best_similarity = 0.5; // Minimum similarity threshold

        for row in rows.ok()?.flatten() {
            let (original, command, count, learned, last_used, conf) = row;
            let similarity = Self::string_similarity(input, &original);

            if similarity > best_similarity {
                best_similarity = similarity;
                best_match = Some(LearnedCorrection {
                    original_input: original.clone(),
                    intended_command: command,
                    confirmation_count: count,
                    learned_at: learned,
                    last_used_at: last_used,
                    confidence: conf * similarity, // Adjust confidence based on similarity
                });
            }
        }

        best_match
    }

    /// Calculate string similarity using Jaro-Winkler distance
    fn string_similarity(s1: &str, s2: &str) -> f64 {
        let len1 = s1.chars().count();
        let len2 = s2.chars().count();

        if len1 == 0 || len2 == 0 {
            return 0.0;
        }

        if s1 == s2 {
            return 1.0;
        }

        // Jaro similarity
        let match_distance = len1.max(len2) / 2 - 1;
        if match_distance < 0 {
            return 0.0;
        }

        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();

        let mut s1_matches = vec![false; len1];
        let mut s2_matches = vec![false; len2];

        let mut matches = 0;
        for i in 0..len1 {
            let start = i.saturating_sub(match_distance);
            let end = (i + match_distance + 1).min(len2);

            for j in start..end {
                if !s2_matches[j] && s1_chars[i] == s2_chars[j] {
                    s1_matches[i] = true;
                    s2_matches[j] = true;
                    matches += 1;
                    break;
                }
            }
        }

        if matches == 0 {
            return 0.0;
        }

        let mut transpositions = 0;
        let mut k = 0;
        for i in 0..len1 {
            if s1_matches[i] {
                while !s2_matches[k] {
                    k += 1;
                }
                if s1_chars[i] != s2_chars[k] {
                    transpositions += 1;
                }
                k += 1;
            }
        }

        let jaro = (
            matches as f64 / len1 as f64 +
            matches as f64 / len2 as f64 +
            (matches as f64 - transpositions as f64 / 2.0) / matches as f64
        ) / 3.0;

        // Jaro-Winkler similarity (prefix scaling)
        let prefix = s1_chars.iter()
            .zip(s2_chars.iter())
            .take_while(|(a, b)| a == b)
            .take(4)
            .count() as f64;

        jaro + (prefix * 0.1 * (1.0 - jaro))
    }

    /// Store a learned pattern
    pub fn store_pattern(&self, pattern: LearnedPattern) -> Result<(), crate::nlp::NLPError> {
        let correction_data = serde_json::to_vec(&pattern.correction)
            .map_err(|e| crate::nlp::NLPError::SerializationError(serde_json::Error::from(e)))?;

        let pattern_type_str = format!("{:?}", pattern.pattern_type);

        self.conn.execute(
            "INSERT OR REPLACE INTO patterns (pattern_type, pattern, correction, confirmation_count, confidence, learned_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                pattern_type_str,
                pattern.pattern,
                correction_data,
                pattern.confirmation_count,
                pattern.confidence,
                pattern.learned_at,
            ],
        ).map_err(|e| crate::nlp::NLPError::ConfigError(format!("Failed to store pattern: {}", e)))?;

        Ok(())
    }

    /// Get all matching patterns for input
    pub fn get_matching_patterns(&self, input: &str) -> Vec<LearnedPattern> {
        let mut patterns = Vec::new();
        let input_lower = input.to_lowercase();

        let mut stmt = self.conn.prepare(
            "SELECT pattern_type, pattern, correction, confirmation_count, confidence, learned_at
             FROM patterns"
        ).ok();

        if let Some(mut stmt) = stmt {
            let rows = stmt.query_map([], |row| {
                let pattern_type: String = row.get(0)?;
                let pattern: String = row.get(1)?;
                let correction_data: Vec<u8> = row.get(2)?;
                let correction: PatternCorrection = serde_json::from_slice(&correction_data)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
                Ok((
                    pattern_type,
                    pattern,
                    correction,
                    row.get::<_, u32>(3)?,
                    row.get::<_, f64>(4)?,
                    row.get::<_, i64>(5)?,
                ))
            });

            if let Ok(rows) = rows {
                for row in rows.flatten() {
                    let (ptype_str, pattern_str, correction, count, conf, learned) = row;

                    // Check if pattern matches input
                    if input_lower.contains(&pattern_str.to_lowercase()) {
                        let pattern_type = match ptype_str.as_str() {
                            "WordSubstitution" => PatternType::WordSubstitution,
                            "CategoryMapping" => PatternType::CategoryMapping,
                            "ActionMapping" => PatternType::ActionMapping,
                            "DeadlineInterpretation" => PatternType::DeadlineInterpretation,
                            "PhrasePattern" => PatternType::PhrasePattern,
                            _ => continue,
                        };

                        patterns.push(LearnedPattern {
                            pattern_type,
                            pattern: pattern_str,
                            correction,
                            confirmation_count: count,
                            confidence: conf,
                            learned_at: learned,
                        });
                    }
                }
            }
        }

        // Sort by confidence
        patterns.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        patterns
    }

    /// Get learning statistics
    pub fn stats(&self) -> LearningStats {
        let total_corrections = self.conn.query_row(
            "SELECT COUNT(*) FROM corrections",
            [],
            |row| row.get::<_, i64>(0)
        ).unwrap_or(0);

        let total_patterns = self.conn.query_row(
            "SELECT COUNT(*) FROM patterns",
            [],
            |row| row.get::<_, i64>(0)
        ).unwrap_or(0);

        let avg_confidence = self.conn.query_row(
            "SELECT AVG(confidence) FROM corrections",
            [],
            |row| row.get::<_, f64>(0)
        ).unwrap_or(0.0);

        let total_confirmations = self.conn.query_row(
            "SELECT SUM(confirmation_count) FROM corrections",
            [],
            |row| row.get::<_, i64>(0)
        ).unwrap_or(0);

        LearningStats {
            total_corrections: total_corrections as usize,
            total_patterns: total_patterns as usize,
            average_confidence: avg_confidence,
            total_confirmations: total_confirmations as u32,
        }
    }

    /// Clear all learned data
    pub fn clear(&self) -> Result<(), crate::nlp::NLPError> {
        self.conn.execute("DELETE FROM corrections", [])
            .map_err(|e| crate::nlp::NLPError::ConfigError(format!("Failed to clear corrections: {}", e)))?;
        self.conn.execute("DELETE FROM patterns", [])
            .map_err(|e| crate::nlp::NLPError::ConfigError(format!("Failed to clear patterns: {}", e)))?;
        Ok(())
    }
}

/// Learning engine for applying learned corrections
pub struct LearningEngine {
    db: Option<LearningDB>,
}

impl LearningEngine {
    /// Create a new learning engine
    pub fn new() -> Self {
        Self { db: None }
    }

    /// Initialize with database path
    pub fn with_db<P: AsRef<Path>>(path: P) -> Result<Self, crate::nlp::NLPError> {
        Ok(Self {
            db: Some(LearningDB::new(path)?),
        })
    }

    /// Learn from a user correction
    pub fn learn_from_correction(&self, original_input: &str, intended_command: &NLPCommand) -> Result<(), crate::nlp::NLPError> {
        if let Some(ref db) = self.db {
            db.store_correction(original_input, intended_command)?;

            // Also extract and learn patterns from the correction
            let patterns = Self::extract_patterns(original_input, intended_command);
            for pattern in patterns {
                let _ = db.store_pattern(pattern);
            }
        }
        Ok(())
    }

    /// Apply learned corrections to input
    pub fn apply_learning(&self, input: &str) -> Option<NLPCommand> {
        if let Some(ref db) = self.db {
            if let Some(correction) = db.get_correction(input) {
                // Only return if confidence is high enough
                if correction.confidence > 0.6 {
                    return Some(correction.intended_command);
                }
            }
        }
        None
    }

    /// Suggest corrections based on learning
    pub fn suggest_corrections(&self, input: &str) -> Vec<String> {
        let mut suggestions = Vec::new();

        if let Some(ref db) = self.db {
            if let Some(correction) = db.get_correction(input) {
                suggestions.push(format!("Did you mean: {} {}?",
                    format_action(&correction.intended_command.action),
                    correction.intended_command.content
                ));
            }

            // Get pattern-based suggestions
            for pattern in db.get_matching_patterns(input) {
                if pattern.confidence > 0.5 {
                    match pattern.correction {
                        PatternCorrection::Word(ref word) => {
                            suggestions.push(format!("Consider using '{}' instead of '{}'", word, pattern.pattern));
                        }
                        PatternCorrection::Category(ref cat) => {
                            suggestions.push(format!("Did you mean category '{}'?", cat));
                        }
                        PatternCorrection::Action(ref action) => {
                            suggestions.push(format!("Did you mean to {} something?", format_action(action)));
                        }
                        _ => {}
                    }
                }
            }
        }

        suggestions
    }

    /// Extract patterns from a correction
    fn extract_patterns(original: &str, command: &NLPCommand) -> Vec<LearnedPattern> {
        let mut patterns = Vec::new();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let original_lower = original.to_lowercase();
        let words: Vec<&str> = original_lower.split_whitespace().collect();

        // Extract action mappings
        for word in words.iter() {
            match *word {
                "make" | "create" | "start" | "begin" => {
                    patterns.push(LearnedPattern {
                        pattern_type: PatternType::ActionMapping,
                        pattern: word.to_string(),
                        correction: PatternCorrection::Action(command.action.clone()),
                        confirmation_count: 1,
                        confidence: 0.6,
                        learned_at: now,
                    });
                }
                "finish" | "complete" | "done" if command.action == ActionType::Done => {
                    patterns.push(LearnedPattern {
                        pattern_type: PatternType::WordSubstitution,
                        pattern: word.to_string(),
                        correction: PatternCorrection::Word("done".to_string()),
                        confirmation_count: 1,
                        confidence: 0.7,
                        learned_at: now,
                    });
                }
                _ => {}
            }
        }

        // Extract category mappings
        if let Some(ref category) = command.category {
            for word in &words {
                if word.len() > 3 && *word != category.as_str() {
                    // Potential category mapping
                    patterns.push(LearnedPattern {
                        pattern_type: PatternType::CategoryMapping,
                        pattern: word.to_string(),
                        correction: PatternCorrection::Category(category.clone()),
                        confirmation_count: 1,
                        confidence: 0.5,
                        learned_at: now,
                    });
                }
            }
        }

        patterns
    }

    /// Get learning statistics
    pub fn stats(&self) -> Option<LearningStats> {
        self.db.as_ref().map(|db| db.stats())
    }

    /// Clear all learned data
    pub fn clear(&self) -> Result<(), crate::nlp::NLPError> {
        if let Some(ref db) = self.db {
            db.clear()?;
        }
        Ok(())
    }
}

impl Default for LearningEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Format action type for display
fn format_action(action: &ActionType) -> &'static str {
    match action {
        ActionType::Task => "add",
        ActionType::Record => "record",
        ActionType::Done => "complete",
        ActionType::Update => "update",
        ActionType::Delete => "delete",
        ActionType::List => "list",
    }
}

/// Learning statistics
#[derive(Debug, Clone)]
pub struct LearningStats {
    /// Total number of learned corrections
    pub total_corrections: usize,
    /// Total number of learned patterns
    pub total_patterns: usize,
    /// Average confidence of learned corrections
    pub average_confidence: f64,
    /// Total number of confirmations
    pub total_confirmations: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_learning_db() -> (LearningDB, tempfile::NamedTempFile) {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let db = LearningDB::new(temp_file.path()).unwrap();
        (db, temp_file)
    }

    #[test]
    fn test_learning_db_new() {
        let (db, _temp) = create_test_learning_db();
        let stats = db.stats();
        assert_eq!(stats.total_corrections, 0);
        assert_eq!(stats.total_patterns, 0);
    }

    #[test]
    fn test_store_and_get_correction() {
        let (db, _temp) = create_test_learning_db();

        let original = "make task buy groceries";
        let intended = NLPCommand {
            action: ActionType::Task,
            content: "buy groceries".to_string(),
            ..Default::default()
        };

        db.store_correction(original, &intended).unwrap();

        let retrieved = db.get_correction(original);
        assert!(retrieved.is_some());
        let correction = retrieved.unwrap();
        assert_eq!(correction.intended_command.action, ActionType::Task);
        assert_eq!(correction.intended_command.content, "buy groceries");
    }

    #[test]
    fn test_correction_confidence_increases() {
        let (db, _temp) = create_test_learning_db();

        let original = "finish task 1";
        let intended = NLPCommand {
            action: ActionType::Done,
            content: "1".to_string(),
            ..Default::default()
        };

        db.store_correction(original, &intended).unwrap();

        let first = db.get_correction(original).unwrap();
        assert_eq!(first.confirmation_count, 1);

        // Store again to simulate confirmation
        db.store_correction(original, &intended).unwrap();

        let second = db.get_correction(original).unwrap();
        assert_eq!(second.confirmation_count, 2);
        assert!(second.confidence > first.confidence);
    }

    #[test]
    fn test_fuzzy_correction_matching() {
        let (db, _temp) = create_test_learning_db();

        let original = "add task buy milk";
        let intended = NLPCommand {
            action: ActionType::Task,
            content: "buy milk".to_string(),
            ..Default::default()
        };

        db.store_correction(original, &intended).unwrap();

        // Try similar input
        let similar = "Add task buy milk"; // Different case
        let retrieved = db.get_correction(similar);
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_string_similarity() {
        let sim1 = LearningDB::string_similarity("hello", "hello");
        assert_eq!(sim1, 1.0);

        let sim2 = LearningDB::string_similarity("hello", "hallo");
        assert!(sim2 > 0.8);

        let sim3 = LearningDB::string_similarity("hello", "world");
        assert!(sim3 < 0.5);

        let sim4 = LearningDB::string_similarity("", "");
        assert_eq!(sim4, 0.0);
    }

    #[test]
    fn test_store_and_get_patterns() {
        let (db, _temp) = create_test_learning_db();

        let pattern = LearnedPattern {
            pattern_type: PatternType::WordSubstitution,
            pattern: "finish".to_string(),
            correction: PatternCorrection::Word("done".to_string()),
            confirmation_count: 1,
            confidence: 0.7,
            learned_at: 1000,
        };

        db.store_pattern(pattern.clone()).unwrap();

        let patterns = db.get_matching_patterns("I want to finish this task");
        assert!(!patterns.is_empty());
        assert_eq!(patterns[0].pattern, "finish");
    }

    #[test]
    fn test_learning_engine_without_db() {
        let engine = LearningEngine::new();
        assert!(engine.apply_learning("test").is_none());
        assert!(engine.suggest_corrections("test").is_empty());
        assert!(engine.stats().is_none());
    }

    #[test]
    fn test_learning_stats() {
        let (db, _temp) = create_test_learning_db();

        let stats = db.stats();
        assert_eq!(stats.total_corrections, 0);
        assert_eq!(stats.total_patterns, 0);
        assert_eq!(stats.total_confirmations, 0);

        let correction = NLPCommand {
            action: ActionType::Task,
            content: "test".to_string(),
            ..Default::default()
        };

        db.store_correction("test input", &correction).unwrap();

        let stats = db.stats();
        assert_eq!(stats.total_corrections, 1);
    }

    #[test]
    fn test_clear_learning_data() {
        let (db, _temp) = create_test_learning_db();

        let correction = NLPCommand {
            action: ActionType::Task,
            content: "test".to_string(),
            ..Default::default()
        };

        db.store_correction("test", &correction).unwrap();
        assert_eq!(db.stats().total_corrections, 1);

        db.clear().unwrap();
        assert_eq!(db.stats().total_corrections, 0);
    }

    #[test]
    fn test_learned_correction_clone() {
        let correction = LearnedCorrection {
            original_input: "test".to_string(),
            intended_command: NLPCommand::default(),
            confirmation_count: 1,
            learned_at: 100,
            last_used_at: 100,
            confidence: 0.5,
        };

        let cloned = correction.clone();
        assert_eq!(correction.original_input, cloned.original_input);
    }

    #[test]
    fn test_learned_pattern_clone() {
        let pattern = LearnedPattern {
            pattern_type: PatternType::WordSubstitution,
            pattern: "test".to_string(),
            correction: PatternCorrection::Word("done".to_string()),
            confirmation_count: 1,
            confidence: 0.7,
            learned_at: 100,
        };

        let cloned = pattern.clone();
        assert_eq!(pattern.pattern, cloned.pattern);
    }

    #[test]
    fn test_pattern_correction_clone() {
        let correction = PatternCorrection::Word("test".to_string());
        let cloned = correction.clone();
        match cloned {
            PatternCorrection::Word(s) => assert_eq!(s, "test"),
            _ => panic!("Wrong type"),
        }
    }

    #[test]
    fn test_pattern_type_clone() {
        let pt = PatternType::WordSubstitution;
        let cloned = pt.clone();
        assert_eq!(pt, cloned);
    }

    #[test]
    fn test_pattern_correction_transform() {
        let correction = PatternCorrection::Transform {
            original: "old".to_string(),
            replacement: "new".to_string(),
        };

        match correction {
            PatternCorrection::Transform { original, replacement } => {
                assert_eq!(original, "old");
                assert_eq!(replacement, "new");
            }
            _ => panic!("Wrong type"),
        }
    }

    #[test]
    fn test_format_action() {
        assert_eq!(format_action(&ActionType::Task), "add");
        assert_eq!(format_action(&ActionType::Done), "complete");
        assert_eq!(format_action(&ActionType::Delete), "delete");
    }

    #[test]
    fn test_learning_stats_default() {
        let stats = LearningStats {
            total_corrections: 0,
            total_patterns: 0,
            average_confidence: 0.0,
            total_confirmations: 0,
        };

        assert_eq!(stats.total_corrections, 0);
        assert_eq!(stats.average_confidence, 0.0);
    }
}
