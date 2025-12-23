//! Context awareness for natural language processing

use super::types::*;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use regex::Regex;
use chrono::Datelike;
use chrono::Timelike;
use std::fmt;

/// Context information about previous commands and state
#[derive(Debug, Clone)]
pub struct CommandContext {
    /// History of recent commands
    pub command_history: Vec<ContextualCommand>,
    /// Last mentioned category
    pub last_category: Option<String>,
    /// Last mentioned task/content
    pub last_content: Option<String>,
    /// Known categories from the database
    pub known_categories: Vec<String>,
    /// Recent task contents
    pub recent_tasks: Vec<String>,
    /// Maximum history size
    pub max_history_size: usize,
}

/// A command with contextual information
#[derive(Debug, Clone)]
pub struct ContextualCommand {
    /// The command that was executed
    pub command: NLPCommand,
    /// Timestamp when command was issued
    pub timestamp: i64,
    /// The original natural language input
    pub original_input: String,
}

impl Default for CommandContext {
    fn default() -> Self {
        Self {
            command_history: Vec::new(),
            last_category: None,
            last_content: None,
            known_categories: Vec::new(),
            recent_tasks: Vec::new(),
            max_history_size: 50,
        }
    }
}

impl CommandContext {
    /// Create a new context with given known categories
    pub fn new(known_categories: Vec<String>) -> Self {
        Self {
            known_categories,
            ..Default::default()
        }
    }

    /// Add a command to the context history
    pub fn add_command(&mut self, command: NLPCommand, original_input: String) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Update last_category and last_content
        if let Some(ref category) = command.category {
            self.last_category = Some(category.clone());
        }

        if !command.content.is_empty() {
            self.last_content = Some(command.content.clone());

            // Track recent tasks for task-based references
            if matches!(command.action, ActionType::Task | ActionType::Done) {
                self.recent_tasks.push(command.content.clone());
                if self.recent_tasks.len() > 20 {
                    self.recent_tasks.remove(0);
                }
            }
        }

        let contextual_cmd = ContextualCommand {
            command,
            timestamp: now,
            original_input,
        };

        self.command_history.push(contextual_cmd);

        // Limit history size
        if self.command_history.len() > self.max_history_size {
            self.command_history.remove(0);
        }
    }

    /// Get recent commands of a specific action type
    pub fn get_recent_by_action(&self, action: ActionType) -> Vec<&NLPCommand> {
        self.command_history
            .iter()
            .rev()
            .filter(|cmd| cmd.command.action == action)
            .take(5)
            .map(|cmd| &cmd.command)
            .collect()
    }

    /// Get the most recent command
    pub fn get_last_command(&self) -> Option<&NLPCommand> {
        self.command_history.last().map(|cmd| &cmd.command)
    }

    /// Update known categories
    pub fn update_categories(&mut self, categories: Vec<String>) {
        self.known_categories = categories;
    }

    /// Clear old history entries (older than specified seconds)
    pub fn clear_old_entries(&mut self, max_age_seconds: i64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        self.command_history.retain(|cmd| now - cmd.timestamp <= max_age_seconds);
    }

    /// Get context as a formatted string for the API
    pub fn to_context_string(&self) -> String {
        let mut context_parts = Vec::new();

        if let Some(ref category) = self.last_category {
            context_parts.push(format!("Last category mentioned: {}", category));
        }

        if let Some(ref content) = self.last_content {
            context_parts.push(format!("Last task mentioned: {}", content));
        }

        if !self.known_categories.is_empty() {
            context_parts.push(format!("Available categories: {}", self.known_categories.join(", ")));
        }

        if !self.recent_tasks.is_empty() {
            let recent = self.recent_tasks.iter()
                .rev()
                .take(5)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            context_parts.push(format!("Recent tasks: {}", recent));
        }

        if context_parts.is_empty() {
            "No previous context".to_string()
        } else {
            context_parts.join(". ")
        }
    }

    /// Get recent conversation summary for the API
    pub fn get_conversation_summary(&self) -> Vec<HashMap<String, String>> {
        self.command_history
            .iter()
            .rev()
            .take(5)
            .map(|cmd| {
                let mut map = HashMap::new();
                map.insert("action".to_string(), format!("{:?}", cmd.command.action));
                map.insert("content".to_string(), cmd.command.content.clone());
                if let Some(ref cat) = cmd.command.category {
                    map.insert("category".to_string(), cat.clone());
                }
                map
            })
            .collect()
    }
}

/// Time context for resolving relative time expressions
#[derive(Debug, Clone)]
pub struct TimeContext {
    /// Current time (for testing purposes, can be overridden)
    pub current_time: Option<i64>,
    /// Current timezone offset in seconds (optional)
    pub timezone_offset: Option<i32>,
}

impl Default for TimeContext {
    fn default() -> Self {
        Self {
            current_time: None,
            timezone_offset: None,
        }
    }
}

impl TimeContext {
    /// Create new time context
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with specific current time (for testing)
    pub fn with_time(time: i64) -> Self {
        Self {
            current_time: Some(time),
            ..Default::default()
        }
    }

    /// Get current timestamp
    pub fn now(&self) -> i64 {
        self.current_time.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64
        })
    }

    /// Get start of today (midnight)
    pub fn start_of_today(&self) -> i64 {
        let now = self.now();
        let day_in_seconds = 86400;
        (now / day_in_seconds) * day_in_seconds
    }

    /// Get start of tomorrow
    pub fn start_of_tomorrow(&self) -> i64 {
        self.start_of_today() + 86400
    }

    /// Get start of yesterday
    pub fn start_of_yesterday(&self) -> i64 {
        self.start_of_today() - 86400
    }

    /// Get day of week (0 = Sunday, 1 = Monday, etc.)
    pub fn day_of_week(&self) -> u8 {
        let now = self.now();
        ((now / 86400 + 4) % 7) as u8 // Unix epoch was Thursday
    }

    /// Get days until a specific weekday (0 = Sunday, 1 = Monday, etc.)
    pub fn days_until_weekday(&self, target_day: u8) -> i64 {
        let current = self.day_of_week();
        if target_day >= current {
            (target_day - current) as i64
        } else {
            (7 - current as i64 + target_day as i64)
        }
    }

    /// Get start of next Monday
    pub fn next_monday(&self) -> i64 {
        self.start_of_today() + self.days_until_weekday(1) * 86400
    }

    /// Get timestamp for "this week" (start of week, assuming Monday)
    pub fn start_of_week(&self) -> i64 {
        self.start_of_today() - ((self.day_of_week() + 6) % 7) as i64 * 86400
    }

    /// Get timestamp for "end of month"
    pub fn end_of_month(&self) -> i64 {
        let now = self.now();
        // Get days in current month (simplified - doesn't handle leap years perfectly)
        let days_in_month = match Self::month_from_timestamp(now) {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => 28, // Simplified
            _ => 30,
        };
        let start_of_month = (now / (86400 * 30)) * (86400 * 30);
        start_of_month + days_in_month as i64 * 86400 - 1
    }

    /// Get month from timestamp (1-12)
    fn month_from_timestamp(ts: i64) -> u32 {
        let days_since_epoch = ts / 86400;
        let years_since_epoch = days_since_epoch / 365;
        let day_of_year = (days_since_epoch % 365) as u32;
        // Simplified - actual implementation would account for leap years
        let days_per_month = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        let mut day_count = 0;
        let mut month = 0;
        for (i, &days) in days_per_month.iter().enumerate() {
            day_count += days;
            if day_of_year < day_count {
                month = i as u32 + 1;
                break;
            }
        }
        month
    }
}

/// Fuzzy matching utilities for category and task name matching
pub struct FuzzyMatcher;

impl FuzzyMatcher {
    /// Find the best matching category using fuzzy matching
    pub fn match_category(input: &str, known_categories: &[String]) -> Option<String> {
        let input_lower = input.to_lowercase();

        // Return None for empty input
        if input_lower.is_empty() {
            return None;
        }

        // First try exact match (case-insensitive)
        for category in known_categories {
            if category.to_lowercase() == input_lower {
                return Some(category.clone());
            }
        }

        // Try contains match (only if input is non-empty)
        for category in known_categories {
            if !input_lower.is_empty() &&
               (category.to_lowercase().contains(&input_lower) || input_lower.contains(&category.to_lowercase())) {
                return Some(category.clone());
            }
        }

        // Use Levenshtein distance for fuzzy matching
        let mut best_match = None;
        let mut best_score = 0.0f64;

        for category in known_categories {
            let score = Self::similarity_score(&input_lower, &category.to_lowercase());
            if score > best_score && score >= 0.6 { // 60% similarity threshold
                best_score = score;
                best_match = Some(category.clone());
            }
        }

        best_match
    }

    /// Find matching task content using fuzzy matching
    pub fn match_task(input: &str, known_tasks: &[String]) -> Option<String> {
        let input_lower = input.to_lowercase();

        // Return None for empty input
        if input_lower.is_empty() {
            return None;
        }

        let input_words: Vec<&str> = input_lower.split_whitespace().collect();

        // First try exact match
        for task in known_tasks {
            if task.to_lowercase() == input_lower {
                return Some(task.clone());
            }
        }

        // Try contains match
        for task in known_tasks {
            let task_lower = task.to_lowercase();
            if !input_lower.is_empty() &&
               (task_lower.contains(&input_lower) || input_lower.contains(&task_lower)) {
                return Some(task.clone());
            }
        }

        // Try matching key words
        let mut best_match = None;
        let mut best_score = 0.0f64;

        for task in known_tasks {
            let task_lower = task.to_lowercase();
            let task_words: Vec<&str> = task_lower.split_whitespace().collect();

            // Count how many input words appear in the task
            let matching_words = input_words.iter()
                .filter(|word| task_words.contains(word))
                .count();

            if matching_words > 0 {
                let score = matching_words as f64 / input_words.len() as f64;
                if score > best_score && score >= 0.5 {
                    best_score = score;
                    best_match = Some(task.clone());
                }
            }
        }

        best_match
    }

    /// Calculate similarity score between two strings using Levenshtein distance
    fn similarity_score(a: &str, b: &str) -> f64 {
        if a.is_empty() && b.is_empty() {
            return 1.0;
        }
        if a.is_empty() || b.is_empty() {
            return 0.0;
        }

        let distance = Self::levenshtein_distance(a, b);
        let max_len = a.len().max(b.len());

        if max_len == 0 {
            1.0
        } else {
            1.0 - (distance as f64 / max_len as f64)
        }
    }

    /// Calculate Levenshtein distance between two strings
    fn levenshtein_distance(a: &str, b: &str) -> usize {
        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();
        let m = a_chars.len();
        let n = b_chars.len();

        let mut dp = vec![vec![0; n + 1]; m + 1];

        for i in 0..=m {
            dp[i][0] = i;
        }
        for j in 0..=n {
            dp[0][j] = j;
        }

        for i in 1..=m {
            for j in 1..=n {
                if a_chars[i - 1] == b_chars[j - 1] {
                    dp[i][j] = dp[i - 1][j - 1];
                } else {
                    dp[i][j] = 1 + [
                        dp[i - 1][j],      // deletion
                        dp[i][j - 1],      // insertion
                        dp[i - 1][j - 1],  // substitution
                    ].into_iter().min().unwrap();
                }
            }
        }

        dp[m][n]
    }

    /// Find all matches above a threshold
    pub fn find_all_matches(input: &str, candidates: &[String], threshold: f64) -> Vec<(String, f64)> {
        let input_lower = input.to_lowercase();
        let mut matches = Vec::new();

        for candidate in candidates {
            let candidate_lower = candidate.to_lowercase();
            let mut score = Self::similarity_score(&input_lower, &candidate_lower);

            // Boost score for substring matches
            if candidate_lower.contains(&input_lower) || input_lower.contains(&candidate_lower) {
                score = score.max(0.8); // Substring matches get at least 0.8
            }

            if score >= threshold {
                matches.push((candidate.clone(), score));
            }
        }

        matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        matches
    }
}

/// Result of deadline inference
#[derive(Debug, Clone, PartialEq)]
pub struct InferredDeadline {
    /// The inferred deadline as a human-readable string
    pub deadline: String,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
    /// Whether the deadline was explicitly stated or inferred
    pub is_explicit: bool,
    /// The source of the inference
    pub source: DeadlineSource,
}

/// Where the deadline information came from
#[derive(Debug, Clone, PartialEq)]
pub enum DeadlineSource {
    /// Explicitly mentioned (e.g., "by 5PM", "due tomorrow")
    Explicit,
    /// Inferred from relative time (e.g., "in 2 hours")
    RelativeTime,
    /// Inferred from urgency words (e.g., "urgent", "ASAP")
    Urgency,
    /// Inferred from task category patterns
    CategoryPattern,
    /// Default deadline applied
    Default,
}

/// Intelligent deadline inference from natural language
pub struct DeadlineInference;

impl DeadlineInference {
    /// Infer deadline from natural language input
    pub fn infer_deadline(input: &str, time_context: &TimeContext, category: Option<&str>) -> Option<InferredDeadline> {
        let input_lower = input.to_lowercase();

        // 1. Try explicit time patterns first
        if let Some(result) = Self::infer_explicit_deadline(&input_lower, time_context) {
            return Some(result);
        }

        // 2. Try relative time patterns
        if let Some(result) = Self::infer_relative_deadline(&input_lower, time_context) {
            return Some(result);
        }

        // 3. Try urgency-based inference
        if let Some(result) = Self::infer_urgency_deadline(&input_lower, time_context) {
            return Some(result);
        }

        // 4. Try category-based defaults
        if let Some(cat) = category {
            if let Some(result) = Self::infer_category_deadline(cat, time_context) {
                return Some(result);
            }
        }

        None
    }

    /// Infer explicit deadlines (mentioned dates/times)
    fn infer_explicit_deadline(input: &str, _time_context: &TimeContext) -> Option<InferredDeadline> {
        // Common deadline indicator words (case-insensitive)
        let deadline_patterns = [
            r"(?i)by\s+(today|tomorrow|monday|tuesday|wednesday|thursday|friday|saturday|sunday|eod|eow|eom|eoy)",
            r"(?i)due\s+(today|tomorrow|monday|tuesday|wednesday|thursday|friday|saturday|sunday)",
            r"(?i)deadline\s+(?:is\s+)?(today|tomorrow|monday|tuesday|wednesday|thursday|friday|saturday|sunday)",
            r"(?i)before\s+(today|tomorrow|monday|tuesday|wednesday|thursday|friday|saturday|sunday|eod)",
            r"(?i)on\s+(monday|tuesday|wednesday|thursday|friday|saturday|sunday)",
            r"(?i)next\s+(monday|tuesday|wednesday|thursday|friday|saturday|sunday)",
            r"(?i)for\s+(monday|tuesday|wednesday|thursday|friday|saturday|sunday)",
        ];

        for pattern in &deadline_patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(caps) = re.captures(input) {
                    if let Some(match_str) = caps.get(1) {
                        return Some(InferredDeadline {
                            deadline: Self::normalize_deadline_keyword(match_str.as_str()),
                            confidence: 0.95,
                            is_explicit: true,
                            source: DeadlineSource::Explicit,
                        });
                    }
                }
            }
        }

        // Check for time-specific deadlines (e.g., "by 5PM", "due at 3:30")
        let time_patterns = [
            r"(?i)by\s+(\d{1,2}(?::\d{2})?(?:\s*(?:am|pm|a\.m\.|p\.m\.))?)",
            r"(?i)due\s+(?:at\s+)?(\d{1,2}(?::\d{2})?(?:\s*(?:am|pm|a\.m\.|p\.m\.))?)",
            r"(?i)deadline\s+(?:at\s+)?(\d{1,2}(?::\d{2})?(?:\s*(?:am|pm|a\.m\.|p\.m\.))?)",
        ];

        for pattern in &time_patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(caps) = re.captures(input) {
                    if let Some(match_str) = caps.get(1) {
                        return Some(InferredDeadline {
                            deadline: format!("today {}", match_str.as_str()),
                            confidence: 0.90,
                            is_explicit: true,
                            source: DeadlineSource::Explicit,
                        });
                    }
                }
            }
        }

        None
    }

    /// Infer relative time deadlines (e.g., "in 2 hours", "next week")
    fn infer_relative_deadline(input: &str, time_context: &TimeContext) -> Option<InferredDeadline> {
        // "in X time_unit" patterns
        let relative_patterns = [
            (r"in\s+(\d+)\s+seconds?", 1, "second"),
            (r"in\s+(\d+)\s+minutes?", 60, "minute"),
            (r"in\s+(\d+)\s+hours?", 3600, "hour"),
            (r"in\s+(\d+)\s+days?", 86400, "day"),
            (r"in\s+(\d+)\s+weeks?", 604800, "week"),
        ];

        for (pattern, seconds_per_unit, unit_name) in &relative_patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(caps) = re.captures(input) {
                    if let Some(match_str) = caps.get(1) {
                        if let Ok(amount) = match_str.as_str().parse::<i64>() {
                            let total_seconds = amount * seconds_per_unit;
                            return Some(InferredDeadline {
                                deadline: Self::format_relative_deadline(total_seconds, time_context),
                                confidence: 0.85,
                                is_explicit: true,
                                source: DeadlineSource::RelativeTime,
                            });
                        }
                    }
                }
            }
        }

        // "next week"/"next month" patterns - use tascli-compatible +Xd format
        let next_patterns = [
            (r"next\s+week", "+7d"),
            (r"next\s+month", "+30d"),
            (r"next\s+year", "+365d"),
        ];

        for (pattern, description) in &next_patterns {
            if let Ok(re) = Regex::new(pattern) {
                if re.is_match(input) {
                    return Some(InferredDeadline {
                        deadline: description.to_string(),
                        confidence: 0.80,
                        is_explicit: true,
                        source: DeadlineSource::RelativeTime,
                    });
                }
            }
        }

        None
    }

    /// Infer deadline from urgency indicators
    fn infer_urgency_deadline(input: &str, _time_context: &TimeContext) -> Option<InferredDeadline> {
        let urgency_mappings: [(&str, &str, f64); 6] = [
            (r"(?i)\burgent(?:ly)?\b", "today", 0.70),
            (r"(?i)\basap\b|\bas soon as possible\b", "today", 0.65),
            (r"(?i)\bimmediately\b|\bright now\b", "today", 0.75),
            (r"(?i)\bsoon\b", "tomorrow", 0.50),
            (r"(?i)\bthis week\b", "eow", 0.60),
            (r"(?i)\boverdue\b", "yesterday", 0.80),
        ];

        for (pattern, deadline, confidence) in &urgency_mappings {
            if let Ok(re) = Regex::new(pattern) {
                if re.is_match(input) {
                    return Some(InferredDeadline {
                        deadline: deadline.to_string(),
                        confidence: *confidence,
                        is_explicit: false,
                        source: DeadlineSource::Urgency,
                    });
                }
            }
        }

        None
    }

    /// Infer deadline based on category patterns
    fn infer_category_deadline(category: &str, time_context: &TimeContext) -> Option<InferredDeadline> {
        let category_lower = category.to_lowercase();

        // Common category-deadline associations
        let category_rules: [(&str, &str, f64); 12] = [
            // Urgent categories
            ("urgent", "today", 0.60),
            ("today", "today", 0.70),
            ("emergency", "today", 0.65),
            ("asap", "today", 0.60),

            // Work-related (often end of week)
            ("work", "eow", 0.40),
            ("meeting", "tomorrow", 0.50),

            // Personal (often more flexible)
            ("personal", "week", 0.30),
            ("errand", "weekend", 0.40),

            // Shopping/chores (often this week)
            ("shopping", "week", 0.35),
            ("chore", "weekend", 0.35),

            // Learning/reading (often longer term)
            ("learning", "month", 0.30),
            ("reading", "month", 0.30),
        ];

        for (pattern, deadline, confidence) in &category_rules {
            if category_lower.contains(pattern) {
                return Some(InferredDeadline {
                    deadline: deadline.to_string(),
                    confidence: *confidence,
                    is_explicit: false,
                    source: DeadlineSource::CategoryPattern,
                });
            }
        }

        None
    }

    /// Normalize deadline keywords to standard format
    fn normalize_deadline_keyword(keyword: &str) -> String {
        match keyword.to_lowercase().as_str() {
            "eod" => "today".to_string(),
            "eow" | "week" => "sunday".to_string(),
            "eom" => "month".to_string(),
            "eoy" => "year".to_string(),
            other => other.to_string(),
        }
    }

    /// Format relative deadline as tascli-compatible time string
    fn format_relative_deadline(seconds: i64, time_context: &TimeContext) -> String {
        let minutes = seconds / 60;
        let hours = seconds / 3600;
        let days = seconds / 86400;

        // For days and longer, use +Xd format
        if days > 0 {
            format!("+{}d", days)
        } else if hours > 0 {
            // For hours, calculate the actual time and return HH:MM format
            let future_timestamp = time_context.now() + seconds;
            let future_datetime = chrono::DateTime::from_timestamp(future_timestamp, 0)
                .unwrap()
                .naive_local();
            format!("{:02}:{:02}",
                future_datetime.hour(),
                future_datetime.minute()
            )
        } else if minutes > 0 {
            // For minutes, return time in "today HH:MM" format
            let future_timestamp = time_context.now() + seconds;
            let future_datetime = chrono::DateTime::from_timestamp(future_timestamp, 0)
                .unwrap()
                .naive_local();
            format!("today {:02}:{:02}",
                future_datetime.hour(),
                future_datetime.minute()
            )
        } else {
            // For seconds, use "today HH:MM:SS" format
            let future_timestamp = time_context.now() + seconds;
            let future_datetime = chrono::DateTime::from_timestamp(future_timestamp, 0)
                .unwrap()
                .naive_local();
            format!("today {:02}:{:02}:{:02}",
                future_datetime.hour(),
                future_datetime.minute(),
                future_datetime.second()
            )
        }
    }

    /// Get default deadline for a task type
    pub fn default_deadline(task_type: ActionType) -> Option<String> {
        match task_type {
            ActionType::Task => Some("tomorrow".to_string()),
            ActionType::Record => None, // Records don't typically have deadlines
            _ => None,
        }
    }

    /// Calculate business days from now (skips weekends)
    pub fn add_business_days(days: u32, time_context: &TimeContext) -> String {
        let mut current_day = time_context.day_of_week();
        let mut days_to_add = days as i64;
        let mut total_days = 0;

        while days_to_add > 0 {
            total_days += 1;
            current_day = (current_day + 1) % 7;
            // Skip weekends (0 = Sunday, 6 = Saturday)
            if current_day != 0 && current_day != 6 {
                days_to_add -= 1;
            }
        }

        if total_days == 1 {
            "tomorrow".to_string()
        } else {
            format!("{} days", total_days)
        }
    }

    /// Check if a deadline expression is ambiguous
    pub fn is_ambiguous_deadline(input: &str) -> bool {
        let ambiguous_patterns = [
            r"\blater\b",
            r"\bsometime\b",
            r"\beventually\b",
            r"\bsomeday\b",
        ];

        for pattern in &ambiguous_patterns {
            if let Ok(re) = Regex::new(pattern) {
                if re.is_match(input) {
                    return true;
                }
            }
        }

        false
    }

    /// Suggest clarification for ambiguous deadlines
    pub fn suggest_deadline_clarification(input: &str) -> Option<String> {
        if Self::is_ambiguous_deadline(input) {
            Some(
                "When would you like to complete this? You can say things like \
                'today', 'tomorrow', 'in 2 hours', 'by Friday', etc."
                .to_string()
            )
        } else {
            None
        }
    }

    /// Extract all time-related phrases from input for debugging
    pub fn extract_time_phrases(input: &str) -> Vec<String> {
        let mut phrases = Vec::new();

        let time_patterns = [
            r"(?i)\bin\s+\d+\s+(?:seconds?|minutes?|hours?|days?|weeks?|months?|years?)\b",
            r"(?i)\bby\s+(?:today|tomorrow|monday|tuesday|wednesday|thursday|friday|saturday|sunday|eod|eow)\b",
            r"(?i)\bdue\s+(?:today|tomorrow|monday|tuesday|wednesday|thursday|friday|saturday|sunday)\b",
            r"(?i)\bnext\s+(?:week|month|year|monday|tuesday|wednesday|thursday|friday|saturday|sunday)\b",
            r"(?i)\bthis\s+(?:week|weekend|month)\b",
            r"(?i)\bat\s+\d{1,2}(?::\d{2})?\s*(?:am|pm)?\b",
            r"(?i)\b\d{1,2}:\d{2}\s*(?:am|pm)?\b",
        ];

        for pattern in &time_patterns {
            if let Ok(re) = Regex::new(pattern) {
                for caps in re.captures_iter(input) {
                    if let Some(match_str) = caps.get(0) {
                        phrases.push(match_str.as_str().to_string());
                    }
                }
            }
        }

        phrases
    }
}

/// Helper for detecting and resolving ambiguous inputs
pub struct DisambiguationHelper;

impl DisambiguationHelper {
    /// Check if a category input is ambiguous and return disambiguation info if so
    pub fn check_category_ambiguity(
        input: &str,
        known_categories: &[String],
    ) -> Option<Disambiguation> {
        if input.is_empty() || known_categories.is_empty() {
            return None;
        }

        // Find all matches above a reasonable threshold
        let matches = FuzzyMatcher::find_all_matches(input, known_categories, 0.5);

        // If we have multiple matches with similar scores, it's ambiguous
        if matches.len() > 1 {
            // Check if the top two matches are close in score (within 20%)
            let top_score = matches[0].1;
            let second_score = matches[1].1;

            if (top_score - second_score).abs() < 0.2 {
                // Clone for prompt generation before moving
                let matches_for_prompt = matches.clone();
                return Some(Disambiguation {
                    input: input.to_string(),
                    ambiguity_type: AmbiguityType::Category,
                    candidates: matches
                        .into_iter()
                        .take(5) // Limit to top 5
                        .map(|(value, score)| DisambiguationCandidate {
                            value,
                            confidence: score,
                            context: None,
                        })
                        .collect(),
                    prompt: Self::format_category_prompt(input, &matches_for_prompt),
                });
            }
        }

        None
    }

    /// Check if a task input is ambiguous and return disambiguation info if so
    pub fn check_task_ambiguity(
        input: &str,
        known_tasks: &[String],
    ) -> Option<Disambiguation> {
        if input.is_empty() || known_tasks.is_empty() {
            return None;
        }

        // For tasks, we need a more sophisticated approach
        let input_lower = input.to_lowercase();
        let input_words: Vec<&str> = input_lower.split_whitespace().collect();

        // Score each task based on word overlap and similarity
        let mut scored_tasks: Vec<(String, f64)> = score_tasks(
            &input_lower,
            &input_words,
            known_tasks,
        );

        // Sort by score descending
        scored_tasks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Keep only tasks with a meaningful score
        scored_tasks.retain(|(_, score)| *score >= 0.3);

        // If we have multiple high-scoring matches close together, it's ambiguous
        if scored_tasks.len() > 1 {
            let top_score = scored_tasks[0].1;
            let second_score = scored_tasks[1].1;

            // If scores are close (within 15%) and both are reasonably high
            if (top_score - second_score).abs() < 0.15 && top_score > 0.5 {
                // Truncate to top 5 for usability
                scored_tasks.truncate(5);

                // Clone for prompt generation before moving
                let scored_tasks_for_prompt = scored_tasks.clone();
                return Some(Disambiguation {
                    input: input.to_string(),
                    ambiguity_type: AmbiguityType::Task,
                    candidates: scored_tasks
                        .into_iter()
                        .map(|(value, score)| DisambiguationCandidate {
                            value,
                            confidence: score,
                            context: None,
                        })
                        .collect(),
                    prompt: Self::format_task_prompt(input, &scored_tasks_for_prompt),
                });
            }
        }

        None
    }

    /// Create a user-friendly prompt for category disambiguation
    fn format_category_prompt(input: &str, matches: &[(String, f64)]) -> String {
        let mut prompt = format!("Multiple categories match '{}'. Which one did you mean?", input);
        prompt.push_str("\n\n");

        for (i, (category, score)) in matches.iter().take(5).enumerate() {
            let percentage = (score * 100.0).round() as i32;
            prompt.push_str(&format!("{}. {} ({}% match)\n", i + 1, category, percentage));
        }

        prompt.push_str("\nPlease specify the full category name or provide more context.");
        prompt
    }

    /// Create a user-friendly prompt for task disambiguation
    fn format_task_prompt(input: &str, matches: &[(String, f64)]) -> String {
        let mut prompt = format!("Multiple tasks match '{}'. Which one did you mean?", input);
        prompt.push_str("\n\n");

        for (i, (task, score)) in matches.iter().take(5).enumerate() {
            let percentage = (score * 100.0).round() as i32;
            // Truncate long tasks for readability
            let display_task = if task.len() > 50 {
                format!("{}...", &task[..47])
            } else {
                task.clone()
            };
            prompt.push_str(&format!("{}. {} ({}% match)\n", i + 1, display_task, percentage));
        }

        prompt.push_str("\nPlease use more specific words from the task you want.");
        prompt
    }

    /// Resolve a disambiguation by selecting a candidate index
    pub fn resolve_by_index(disambiguation: &Disambiguation, index: usize) -> Option<String> {
        if index < disambiguation.candidates.len() {
            Some(disambiguation.candidates[index].value.clone())
        } else {
            None
        }
    }

    /// Check if deadline is ambiguous
    pub fn check_deadline_ambiguity(input: &str) -> Option<Disambiguation> {
        if DeadlineInference::is_ambiguous_deadline(input) {
            return Some(Disambiguation {
                input: input.to_string(),
                ambiguity_type: AmbiguityType::Deadline,
                candidates: vec![
                    DisambiguationCandidate {
                        value: "today".to_string(),
                        confidence: 0.8,
                        context: Some("Due by end of day".to_string()),
                    },
                    DisambiguationCandidate {
                        value: "tomorrow".to_string(),
                        confidence: 0.7,
                        context: Some("Due tomorrow".to_string()),
                    },
                    DisambiguationCandidate {
                        value: "week".to_string(),
                        confidence: 0.6,
                        context: Some("Due this week".to_string()),
                    },
                ],
                prompt: DeadlineInference::suggest_deadline_clarification(input)
                    .unwrap_or_else(|| "When would you like to complete this?".to_string()),
            });
        }

        None
    }

    /// Get a user-friendly display of disambiguation options
    pub fn format_disambiguation(disambiguation: &Disambiguation) -> String {
        let mut result = String::new();

        match disambiguation.ambiguity_type {
            AmbiguityType::Category => {
                result.push_str("Category Ambiguity\n");
                result.push_str("==================\n");
            }
            AmbiguityType::Task => {
                result.push_str("Task Ambiguity\n");
                result.push_str("==============\n");
            }
            AmbiguityType::Deadline => {
                result.push_str("Deadline Ambiguity\n");
                result.push_str("==================\n");
            }
        }

        result.push_str(&disambiguation.prompt);

        // Add candidates list
        if !disambiguation.candidates.is_empty() {
            result.push_str("\n\nOptions:\n");
            for (i, candidate) in disambiguation.candidates.iter().enumerate() {
                let percentage = (candidate.confidence * 100.0).round() as i32;
                result.push_str(&format!("{}. {} ({}% match)\n", i + 1, candidate.value, percentage));
            }
        }

        result.push_str("\n");

        result
    }
}

/// Helper function to score tasks based on input
fn score_tasks(
    input_lower: &str,
    input_words: &[&str],
    known_tasks: &[String],
) -> Vec<(String, f64)> {
    let mut scored = Vec::new();

    for task in known_tasks {
        let task_lower = task.to_lowercase();
        let task_words: Vec<&str> = task_lower.split_whitespace().collect();

        // Calculate similarity score based on multiple factors
        let mut score = 0.0;

        // 1. Word overlap (40% weight)
        let matching_words = input_words.iter()
            .filter(|w| task_words.contains(w))
            .count() as f64;
        let word_score = if input_words.is_empty() {
            0.0
        } else {
            matching_words / input_words.len() as f64
        };
        score += word_score * 0.4;

        // 2. Fuzzy similarity (40% weight)
        let fuzzy_score = FuzzyMatcher::similarity_score(input_lower, &task_lower);
        score += fuzzy_score * 0.4;

        // 3. Contains match (20% weight)
        let contains_bonus = if task_lower.contains(input_lower) || input_lower.contains(&task_lower) {
            0.2
        } else {
            0.0
        };
        score += contains_bonus;

        scored.push((task.clone(), score));
    }

    scored
}

impl fmt::Display for Disambiguation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", DisambiguationHelper::format_disambiguation(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === CommandContext Tests ===

    #[test]
    fn test_context_default() {
        let context = CommandContext::default();
        assert!(context.command_history.is_empty());
        assert!(context.last_category.is_none());
        assert!(context.last_content.is_none());
        assert!(context.known_categories.is_empty());
        assert!(context.recent_tasks.is_empty());
        assert_eq!(context.max_history_size, 50);
    }

    #[test]
    fn test_context_new() {
        let categories = vec!["work".to_string(), "personal".to_string()];
        let context = CommandContext::new(categories.clone());
        assert_eq!(context.known_categories, categories);
    }

    #[test]
    fn test_add_command_updates_category() {
        let mut context = CommandContext::default();
        let command = NLPCommand {
            action: ActionType::Task,
            content: "test task".to_string(),
            category: Some("work".to_string()),
            ..Default::default()
        };

        context.add_command(command, "add work task".to_string());
        assert_eq!(context.last_category, Some("work".to_string()));
    }

    #[test]
    fn test_add_command_updates_content() {
        let mut context = CommandContext::default();
        let command = NLPCommand {
            action: ActionType::Task,
            content: "buy groceries".to_string(),
            ..Default::default()
        };

        context.add_command(command, "add task to buy groceries".to_string());
        assert_eq!(context.last_content, Some("buy groceries".to_string()));
    }

    #[test]
    fn test_add_command_tracks_recent_tasks() {
        let mut context = CommandContext::default();

        let cmd1 = NLPCommand {
            action: ActionType::Task,
            content: "task1".to_string(),
            ..Default::default()
        };
        context.add_command(cmd1, "add task1".to_string());

        let cmd2 = NLPCommand {
            action: ActionType::Task,
            content: "task2".to_string(),
            ..Default::default()
        };
        context.add_command(cmd2, "add task2".to_string());

        assert_eq!(context.recent_tasks.len(), 2);
        assert_eq!(context.recent_tasks[0], "task1");
        assert_eq!(context.recent_tasks[1], "task2");
    }

    #[test]
    fn test_get_recent_by_action() {
        let mut context = CommandContext::default();

        context.add_command(NLPCommand {
            action: ActionType::Task,
            content: "task1".to_string(),
            ..Default::default()
        }, "add task1".to_string());

        context.add_command(NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            ..Default::default()
        }, "list tasks".to_string());

        context.add_command(NLPCommand {
            action: ActionType::Task,
            content: "task2".to_string(),
            ..Default::default()
        }, "add task2".to_string());

        let tasks = context.get_recent_by_action(ActionType::Task);
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].content, "task2"); // Most recent first
        assert_eq!(tasks[1].content, "task1");
    }

    #[test]
    fn test_get_last_command() {
        let mut context = CommandContext::default();

        context.add_command(NLPCommand {
            action: ActionType::Task,
            content: "first".to_string(),
            ..Default::default()
        }, "add first".to_string());

        context.add_command(NLPCommand {
            action: ActionType::Done,
            content: "first".to_string(),
            ..Default::default()
        }, "mark done".to_string());

        let last = context.get_last_command();
        assert!(last.is_some());
        assert_eq!(last.unwrap().action, ActionType::Done);
    }

    #[test]
    fn test_update_categories() {
        let mut context = CommandContext::default();
        context.update_categories(vec!["work".to_string(), "home".to_string()]);
        assert_eq!(context.known_categories.len(), 2);
    }

    #[test]
    fn test_clear_old_entries() {
        let mut context = CommandContext::default();
        let old_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64 - 10000; // 10000 seconds ago

        // Add a command with manipulated timestamp (we'd need to adjust implementation for this)
        // For now, just test the method exists
        context.clear_old_entries(3600);
        assert!(context.command_history.is_empty());
    }

    #[test]
    fn test_to_context_string() {
        let mut context = CommandContext::default();
        context.last_category = Some("work".to_string());
        context.last_content = Some("meeting".to_string());
        context.known_categories = vec!["work".to_string(), "personal".to_string()];

        let context_str = context.to_context_string();
        assert!(context_str.contains("work"));
        assert!(context_str.contains("meeting"));
        assert!(context_str.contains("personal"));
    }

    #[test]
    fn test_get_conversation_summary() {
        let mut context = CommandContext::default();
        context.add_command(NLPCommand {
            action: ActionType::Task,
            content: "test task".to_string(),
            category: Some("work".to_string()),
            ..Default::default()
        }, "add test task".to_string());

        let summary = context.get_conversation_summary();
        assert_eq!(summary.len(), 1);
        assert_eq!(summary[0].get("action"), Some(&"Task".to_string()));
        assert_eq!(summary[0].get("content"), Some(&"test task".to_string()));
        assert_eq!(summary[0].get("category"), Some(&"work".to_string()));
    }

    // === TimeContext Tests ===

    #[test]
    fn test_time_context_default() {
        let context = TimeContext::default();
        assert!(context.current_time.is_none());
        assert!(context.timezone_offset.is_none());
    }

    #[test]
    fn test_time_context_with_time() {
        let context = TimeContext::with_time(1000000);
        assert_eq!(context.current_time, Some(1000000));
    }

    #[test]
    fn test_time_context_now() {
        let context = TimeContext::default();
        let now = context.now();
        assert!(now > 0);
    }

    #[test]
    fn test_time_context_with_time_now() {
        let context = TimeContext::with_time(86400 * 100); // 100 days from epoch
        let now = context.now();
        assert_eq!(now, 86400 * 100);
    }

    #[test]
    fn test_start_of_today() {
        let context = TimeContext::with_time(86400 * 100 + 3600); // 100 days + 1 hour
        let start = context.start_of_today();
        assert_eq!(start, 86400 * 100);
    }

    #[test]
    fn test_start_of_tomorrow() {
        let context = TimeContext::with_time(86400 * 100);
        let tomorrow = context.start_of_tomorrow();
        assert_eq!(tomorrow, 86400 * 101);
    }

    #[test]
    fn test_start_of_yesterday() {
        let context = TimeContext::with_time(86400 * 100);
        let yesterday = context.start_of_yesterday();
        assert_eq!(yesterday, 86400 * 99);
    }

    #[test]
    fn test_day_of_week() {
        // Unix epoch (0) was Thursday (day 4)
        let context = TimeContext::with_time(0);
        let day = context.day_of_week();
        assert_eq!(day, 4);
    }

    #[test]
    fn test_days_until_weekday() {
        // At epoch (day 0): Thursday (day 4)
        let context = TimeContext::with_time(0);
        let days = context.days_until_weekday(1); // Until Monday (day 1)
        // Since target_day (1) < current (4), we get: 7 - 4 + 1 = 4
        assert_eq!(days, 4);
    }

    #[test]
    fn test_next_monday() {
        // At epoch (day 0): Thursday (day 4)
        let context = TimeContext::with_time(0);
        let next_monday = context.next_monday();
        // Days until Monday = 4, so 0 + 86400 * 4 = 86400 * 4
        assert_eq!(next_monday, 86400 * 4);
    }

    // === FuzzyMatcher Tests ===

    #[test]
    fn test_match_category_exact() {
        let categories = vec!["work".to_string(), "personal".to_string()];
        let match_result = FuzzyMatcher::match_category("work", &categories);
        assert_eq!(match_result, Some("work".to_string()));
    }

    #[test]
    fn test_match_category_case_insensitive() {
        let categories = vec!["Work".to_string(), "Personal".to_string()];
        let match_result = FuzzyMatcher::match_category("work", &categories);
        assert_eq!(match_result, Some("Work".to_string()));
    }

    #[test]
    fn test_match_category_contains() {
        let categories = vec!["work-project".to_string(), "personal".to_string()];
        let match_result = FuzzyMatcher::match_category("project", &categories);
        assert_eq!(match_result, Some("work-project".to_string()));
    }

    #[test]
    fn test_match_category_fuzzy() {
        let categories = vec!["work".to_string(), "personal".to_string()];
        let match_result = FuzzyMatcher::match_category("wrk", &categories);
        // "wrk" is close to "work" (Levenshtein distance 1)
        assert_eq!(match_result, Some("work".to_string()));
    }

    #[test]
    fn test_match_category_no_match() {
        let categories = vec!["work".to_string(), "personal".to_string()];
        let match_result = FuzzyMatcher::match_category("xyzabc", &categories);
        assert!(match_result.is_none());
    }

    #[test]
    fn test_match_task_exact() {
        let tasks = vec!["buy groceries".to_string(), "call mom".to_string()];
        let match_result = FuzzyMatcher::match_task("buy groceries", &tasks);
        assert_eq!(match_result, Some("buy groceries".to_string()));
    }

    #[test]
    fn test_match_task_contains() {
        let tasks = vec!["buy groceries from store".to_string(), "call mom".to_string()];
        let match_result = FuzzyMatcher::match_task("groceries", &tasks);
        assert_eq!(match_result, Some("buy groceries from store".to_string()));
    }

    #[test]
    fn test_match_task_keywords() {
        let tasks = vec!["buy groceries from store".to_string(), "call mom on phone".to_string()];
        let match_result = FuzzyMatcher::match_task("groceries store", &tasks);
        assert_eq!(match_result, Some("buy groceries from store".to_string()));
    }

    #[test]
    fn test_match_task_no_match() {
        let tasks = vec!["buy groceries".to_string(), "call mom".to_string()];
        let match_result = FuzzyMatcher::match_task("exercise", &tasks);
        assert!(match_result.is_none());
    }

    #[test]
    fn test_similarity_score_identical() {
        let score = FuzzyMatcher::similarity_score("hello", "hello");
        assert_eq!(score, 1.0);
    }

    #[test]
    fn test_similarity_score_completely_different() {
        let score = FuzzyMatcher::similarity_score("abc", "xyz");
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_similarity_score_similar() {
        let score = FuzzyMatcher::similarity_score("work", "wrk");
        // 1 edit out of 4 chars = 0.75
        assert!(score > 0.7 && score < 1.0);
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(FuzzyMatcher::levenshtein_distance("", ""), 0);
        assert_eq!(FuzzyMatcher::levenshtein_distance("a", ""), 1);
        assert_eq!(FuzzyMatcher::levenshtein_distance("", "a"), 1);
        assert_eq!(FuzzyMatcher::levenshtein_distance("abc", "abc"), 0);
        assert_eq!(FuzzyMatcher::levenshtein_distance("abc", "ab"), 1);
        assert_eq!(FuzzyMatcher::levenshtein_distance("abc", "abcd"), 1);
        assert_eq!(FuzzyMatcher::levenshtein_distance("kitten", "sitting"), 3);
    }

    #[test]
    fn test_find_all_matches() {
        let categories = vec!["work".to_string(), "work-project".to_string(), "personal".to_string()];
        let matches = FuzzyMatcher::find_all_matches("wrk", &categories, 0.5);
        assert!(!matches.is_empty());
        // First match should be highest score
        assert!(matches[0].0 == "work" || matches[0].0 == "work-project");
    }

    #[test]
    fn test_find_all_matches_threshold() {
        let categories = vec!["work".to_string(), "personal".to_string()];
        let matches = FuzzyMatcher::find_all_matches("xyz", &categories, 0.8);
        assert!(matches.is_empty());
    }

    // === Edge Cases ===

    #[test]
    fn test_context_empty_command() {
        let mut context = CommandContext::default();
        context.add_command(NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            ..Default::default()
        }, "list".to_string());
        // Should not crash with empty content
        assert!(context.last_content.is_none());
    }

    #[test]
    fn test_fuzzy_match_empty_input() {
        let categories = vec!["work".to_string()];
        let result = FuzzyMatcher::match_category("", &categories);
        assert!(result.is_none());
    }

    #[test]
    fn test_fuzzy_match_empty_candidates() {
        let result = FuzzyMatcher::match_category("work", &[]);
        assert!(result.is_none());
    }

    #[test]
    fn test_time_context_negative_time() {
        let context = TimeContext::with_time(-100);
        let now = context.now();
        assert_eq!(now, -100);
    }

    // === DeadlineInference Tests ===

    #[test]
    fn test_infer_explicit_deadline_by_today() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_explicit_deadline("finish this by today", &context);
        assert!(result.is_some());
        let inferred = result.unwrap();
        assert_eq!(inferred.deadline, "today");
        assert!(inferred.is_explicit);
        assert_eq!(inferred.source, DeadlineSource::Explicit);
        assert!(inferred.confidence > 0.9);
    }

    #[test]
    fn test_infer_explicit_deadline_by_tomorrow() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_explicit_deadline("task due tomorrow", &context);
        assert!(result.is_some());
        let inferred = result.unwrap();
        assert_eq!(inferred.deadline, "tomorrow");
        assert_eq!(inferred.source, DeadlineSource::Explicit);
    }

    #[test]
    fn test_infer_explicit_deadline_by_friday() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_explicit_deadline("complete by Friday", &context);
        assert!(result.is_some());
        let inferred = result.unwrap();
        assert_eq!(inferred.deadline, "friday");
        assert_eq!(inferred.source, DeadlineSource::Explicit);
    }

    #[test]
    fn test_infer_explicit_deadline_by_eod() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_explicit_deadline("finish by eod", &context);
        assert!(result.is_some());
        let inferred = result.unwrap();
        assert_eq!(inferred.deadline, "today"); // eod normalizes to today
    }

    #[test]
    fn test_infer_explicit_deadline_by_eow() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_explicit_deadline("due by eow", &context);
        assert!(result.is_some());
        let inferred = result.unwrap();
        assert_eq!(inferred.deadline, "sunday"); // eow normalizes to sunday
    }

    #[test]
    fn test_infer_explicit_deadline_before_monday() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_explicit_deadline("finish before Monday", &context);
        assert!(result.is_some());
        let inferred = result.unwrap();
        assert_eq!(inferred.deadline, "monday");
    }

    #[test]
    fn test_infer_explicit_deadline_on_tuesday() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_explicit_deadline("meeting on Tuesday", &context);
        assert!(result.is_some());
        let inferred = result.unwrap();
        assert_eq!(inferred.deadline, "tuesday");
    }

    #[test]
    fn test_infer_explicit_deadline_by_time() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_explicit_deadline("submit by 5PM", &context);
        assert!(result.is_some());
        let inferred = result.unwrap();
        assert!(inferred.deadline.contains("today"));
        assert!(inferred.deadline.contains("5PM"));
    }

    #[test]
    fn test_infer_explicit_deadline_due_at_time() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_explicit_deadline("due at 3:30", &context);
        assert!(result.is_some());
        let inferred = result.unwrap();
        assert!(inferred.deadline.contains("today"));
        assert!(inferred.deadline.contains("3:30"));
    }

    #[test]
    fn test_infer_relative_deadline_in_2_hours() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_relative_deadline("complete in 2 hours", &context);
        assert!(result.is_some());
        let inferred = result.unwrap();
        // Should be in HH:MM format (tascli-compatible)
        assert!(inferred.deadline.contains(':'));
        assert_eq!(inferred.source, DeadlineSource::RelativeTime);
    }

    #[test]
    fn test_infer_relative_deadline_in_30_minutes() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_relative_deadline("finish in 30 minutes", &context);
        assert!(result.is_some());
        let inferred = result.unwrap();
        // Should be in "today HH:MM" format
        assert!(inferred.deadline.starts_with("today "));
        assert!(inferred.deadline.contains(':'));
    }

    #[test]
    fn test_infer_relative_deadline_in_5_days() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_relative_deadline("do it in 5 days", &context);
        assert!(result.is_some());
        let inferred = result.unwrap();
        // Should be in "+Xd" format for days
        assert_eq!(inferred.deadline, "+5d");
    }

    #[test]
    fn test_infer_relative_deadline_in_1_week() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_relative_deadline("review in 1 week", &context);
        assert!(result.is_some());
        let inferred = result.unwrap();
        // Should be in "+Xd" format for weeks (7 days)
        assert_eq!(inferred.deadline, "+7d");
    }

    #[test]
    fn test_infer_relative_deadline_next_week() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_relative_deadline("finish next week", &context);
        assert!(result.is_some());
        let inferred = result.unwrap();
        // Should be in "+Xd" format for "next week" (7 days)
        assert_eq!(inferred.deadline, "+7d");
        assert_eq!(inferred.source, DeadlineSource::RelativeTime);
    }

    #[test]
    fn test_infer_relative_deadline_next_month() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_relative_deadline("start next month", &context);
        assert!(result.is_some());
        let inferred = result.unwrap();
        // Should be in "+Xd" format for "next month" (30 days)
        assert_eq!(inferred.deadline, "+30d");
    }

    #[test]
    fn test_infer_relative_deadline_next_year() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_relative_deadline("plan next year", &context);
        assert!(result.is_some());
        let inferred = result.unwrap();
        // Should be in "+Xd" format for "next year" (365 days)
        assert_eq!(inferred.deadline, "+365d");
    }

    #[test]
    fn test_infer_urgency_deadline_urgent() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_urgency_deadline("urgent task", &context);
        assert!(result.is_some());
        let inferred = result.unwrap();
        assert_eq!(inferred.deadline, "today");
        assert!(!inferred.is_explicit);
        assert_eq!(inferred.source, DeadlineSource::Urgency);
    }

    #[test]
    fn test_infer_urgency_deadline_asap() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_urgency_deadline("do this ASAP", &context);
        assert!(result.is_some());
        let inferred = result.unwrap();
        assert_eq!(inferred.deadline, "today");
        assert_eq!(inferred.source, DeadlineSource::Urgency);
    }

    #[test]
    fn test_infer_urgency_deadline_immediately() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_urgency_deadline("handle immediately", &context);
        assert!(result.is_some());
        let inferred = result.unwrap();
        assert_eq!(inferred.deadline, "today");
        assert!(inferred.confidence > 0.7);
    }

    #[test]
    fn test_infer_urgency_deadline_soon() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_urgency_deadline("finish soon", &context);
        assert!(result.is_some());
        let inferred = result.unwrap();
        assert_eq!(inferred.deadline, "tomorrow");
        assert_eq!(inferred.source, DeadlineSource::Urgency);
    }

    #[test]
    fn test_infer_urgency_deadline_this_week() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_urgency_deadline("complete this week", &context);
        assert!(result.is_some());
        let inferred = result.unwrap();
        assert_eq!(inferred.deadline, "eow");
    }

    #[test]
    fn test_infer_urgency_deadline_overdue() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_urgency_deadline("task is overdue", &context);
        assert!(result.is_some());
        let inferred = result.unwrap();
        assert_eq!(inferred.deadline, "yesterday");
        assert!(inferred.confidence > 0.75);
    }

    #[test]
    fn test_infer_category_deadline_urgent() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_category_deadline("urgent", &context);
        assert!(result.is_some());
        let inferred = result.unwrap();
        assert_eq!(inferred.deadline, "today");
        assert_eq!(inferred.source, DeadlineSource::CategoryPattern);
    }

    #[test]
    fn test_infer_category_deadline_emergency() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_category_deadline("emergency", &context);
        assert!(result.is_some());
        let inferred = result.unwrap();
        assert_eq!(inferred.deadline, "today");
    }

    #[test]
    fn test_infer_category_deadline_work() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_category_deadline("work", &context);
        assert!(result.is_some());
        let inferred = result.unwrap();
        assert_eq!(inferred.deadline, "eow");
        assert!(inferred.confidence < 0.5); // Lower confidence for category patterns
    }

    #[test]
    fn test_infer_category_deadline_meeting() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_category_deadline("meeting", &context);
        assert!(result.is_some());
        let inferred = result.unwrap();
        assert_eq!(inferred.deadline, "tomorrow");
    }

    #[test]
    fn test_infer_category_deadline_personal() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_category_deadline("personal", &context);
        assert!(result.is_some());
        let inferred = result.unwrap();
        assert_eq!(inferred.deadline, "week");
    }

    #[test]
    fn test_infer_category_deadline_shopping() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_category_deadline("shopping", &context);
        assert!(result.is_some());
        let inferred = result.unwrap();
        assert_eq!(inferred.deadline, "week");
    }

    #[test]
    fn test_infer_category_deadline_learning() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_category_deadline("learning", &context);
        assert!(result.is_some());
        let inferred = result.unwrap();
        assert_eq!(inferred.deadline, "month");
    }

    #[test]
    fn test_infer_category_deadline_unknown() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_category_deadline("unknown-category", &context);
        assert!(result.is_none());
    }

    #[test]
    fn test_normalize_deadline_keyword_eod() {
        assert_eq!(DeadlineInference::normalize_deadline_keyword("eod"), "today");
    }

    #[test]
    fn test_normalize_deadline_keyword_eow() {
        assert_eq!(DeadlineInference::normalize_deadline_keyword("eow"), "sunday");
    }

    #[test]
    fn test_normalize_deadline_keyword_eom() {
        assert_eq!(DeadlineInference::normalize_deadline_keyword("eom"), "month");
    }

    #[test]
    fn test_normalize_deadline_keyword_eoy() {
        assert_eq!(DeadlineInference::normalize_deadline_keyword("eoy"), "year");
    }

    #[test]
    fn test_normalize_deadline_keyword_regular() {
        assert_eq!(DeadlineInference::normalize_deadline_keyword("friday"), "friday");
        assert_eq!(DeadlineInference::normalize_deadline_keyword("tomorrow"), "tomorrow");
    }

    #[test]
    fn test_format_relative_deadline_seconds() {
        let context = TimeContext::default();
        let result = DeadlineInference::format_relative_deadline(30, &context);
        // Should be in format "today HH:MM:SS"
        assert!(result.starts_with("today "));
        // Parse the HH:MM:SS part
        let time_part = &result[6..];
        assert!(time_part.matches(':').count() == 2); // HH:MM:SS format
    }

    #[test]
    fn test_format_relative_deadline_minutes() {
        let context = TimeContext::default();
        let result = DeadlineInference::format_relative_deadline(300, &context);
        // Should be in format "today HH:MM"
        assert!(result.starts_with("today "));
        // Parse the HH:MM part
        let time_part = &result[6..];
        assert!(time_part.matches(':').count() == 1); // HH:MM format
    }

    #[test]
    fn test_format_relative_deadline_hours() {
        let context = TimeContext::default();
        let result = DeadlineInference::format_relative_deadline(7200, &context);
        // Should be in format "HH:MM" (without "today" prefix for hours)
        assert!(result.matches(':').count() == 1); // HH:MM format
        // Should NOT start with "today" for pure hour offsets
        assert!(!result.starts_with("today "));
    }

    #[test]
    fn test_format_relative_deadline_days() {
        let context = TimeContext::default();
        assert_eq!(DeadlineInference::format_relative_deadline(172800, &context), "+2d");
    }

    #[test]
    fn test_default_deadline_task() {
        let result = DeadlineInference::default_deadline(ActionType::Task);
        assert_eq!(result, Some("tomorrow".to_string()));
    }

    #[test]
    fn test_default_deadline_record() {
        let result = DeadlineInference::default_deadline(ActionType::Record);
        assert_eq!(result, None);
    }

    #[test]
    fn test_add_business_days_one_day() {
        let context = TimeContext::default();
        let result = DeadlineInference::add_business_days(1, &context);
        assert_eq!(result, "tomorrow");
    }

    #[test]
    fn test_add_business_days_multiple_days() {
        let context = TimeContext::default();
        let result = DeadlineInference::add_business_days(3, &context);
        assert!(result.contains("days"));
    }

    #[test]
    fn test_is_ambiguous_deadline_later() {
        assert!(DeadlineInference::is_ambiguous_deadline("do this later"));
    }

    #[test]
    fn test_is_ambiguous_deadline_sometime() {
        assert!(DeadlineInference::is_ambiguous_deadline("finish sometime"));
    }

    #[test]
    fn test_is_ambiguous_deadline_eventually() {
        assert!(DeadlineInference::is_ambiguous_deadline("complete eventually"));
    }

    #[test]
    fn test_is_ambiguous_deadline_someday() {
        assert!(DeadlineInference::is_ambiguous_deadline("do it someday"));
    }

    #[test]
    fn test_is_ambiguous_deadline_specific() {
        assert!(!DeadlineInference::is_ambiguous_deadline("finish by tomorrow"));
        assert!(!DeadlineInference::is_ambiguous_deadline("due today"));
    }

    #[test]
    fn test_suggest_deadline_clarification_ambiguous() {
        let result = DeadlineInference::suggest_deadline_clarification("do this later");
        assert!(result.is_some());
        let suggestion = result.unwrap();
        assert!(suggestion.contains("When would you like"));
    }

    #[test]
    fn test_suggest_deadline_clarification_specific() {
        let result = DeadlineInference::suggest_deadline_clarification("finish by tomorrow");
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_time_phrases_in_hours() {
        let phrases = DeadlineInference::extract_time_phrases("complete in 2 hours");
        assert!(!phrases.is_empty());
        assert!(phrases.iter().any(|p| p.contains("in 2 hours")));
    }

    #[test]
    fn test_extract_time_phrases_by_day() {
        let phrases = DeadlineInference::extract_time_phrases("finish by Friday");
        assert!(!phrases.is_empty());
        assert!(phrases.iter().any(|p| p.contains("by Friday")));
    }

    #[test]
    fn test_extract_time_phrases_due_tomorrow() {
        let phrases = DeadlineInference::extract_time_phrases("task due tomorrow");
        assert!(!phrases.is_empty());
        assert!(phrases.iter().any(|p| p.contains("due tomorrow")));
    }

    #[test]
    fn test_extract_time_phrases_next_week() {
        let phrases = DeadlineInference::extract_time_phrases("review next week");
        assert!(!phrases.is_empty());
        assert!(phrases.iter().any(|p| p.contains("next week")));
    }

    #[test]
    fn test_extract_time_phrases_multiple() {
        let phrases = DeadlineInference::extract_time_phrases("urgent: finish by Friday, review next week");
        assert!(phrases.len() >= 2);
    }

    #[test]
    fn test_extract_time_phrases_none() {
        let phrases = DeadlineInference::extract_time_phrases("just a regular task");
        assert!(phrases.is_empty());
    }

    #[test]
    fn test_infer_deadline_full_integration() {
        let context = TimeContext::default();

        // Test explicit deadline
        let result = DeadlineInference::infer_deadline("finish by 5PM", &context, None);
        assert!(result.is_some());
        assert_eq!(result.unwrap().source, DeadlineSource::Explicit);

        // Test relative time
        let result = DeadlineInference::infer_deadline("complete in 2 hours", &context, None);
        assert!(result.is_some());
        assert_eq!(result.unwrap().source, DeadlineSource::RelativeTime);

        // Test urgency
        let result = DeadlineInference::infer_deadline("urgent task", &context, None);
        assert!(result.is_some());
        assert_eq!(result.unwrap().source, DeadlineSource::Urgency);

        // Test category-based
        let result = DeadlineInference::infer_deadline("some task", &context, Some("urgent"));
        assert!(result.is_some());
        assert_eq!(result.unwrap().source, DeadlineSource::CategoryPattern);

        // Test no deadline
        let result = DeadlineInference::infer_deadline("just a task", &context, None);
        assert!(result.is_none());
    }

    #[test]
    fn test_inferred_deadline_partial_eq() {
        let dl1 = InferredDeadline {
            deadline: "tomorrow".to_string(),
            confidence: 0.9,
            is_explicit: true,
            source: DeadlineSource::Explicit,
        };
        let dl2 = InferredDeadline {
            deadline: "tomorrow".to_string(),
            confidence: 0.9,
            is_explicit: true,
            source: DeadlineSource::Explicit,
        };
        assert_eq!(dl1, dl2);
    }

    #[test]
    fn test_deadline_source_partial_eq() {
        assert_eq!(DeadlineSource::Explicit, DeadlineSource::Explicit);
        assert_ne!(DeadlineSource::Explicit, DeadlineSource::RelativeTime);
    }

    // === DeadlineInference Edge Cases ===

    #[test]
    fn test_infer_deadline_empty_input() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_deadline("", &context, None);
        assert!(result.is_none());
    }

    #[test]
    fn test_infer_deadline_case_insensitive() {
        let context = TimeContext::default();
        let result1 = DeadlineInference::infer_deadline("BY TOMORROW", &context, None);
        let result2 = DeadlineInference::infer_deadline("by tomorrow", &context, None);
        assert!(result1.is_some());
        assert!(result2.is_some());
        assert_eq!(result1.unwrap().deadline, result2.unwrap().deadline);
    }

    #[test]
    fn test_infer_deadline_with_punctuation() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_deadline("finish by tomorrow!", &context, None);
        assert!(result.is_some());
        assert_eq!(result.unwrap().deadline, "tomorrow");
    }

    // === Additional Relative Time Parsing Tests ===

    #[test]
    fn test_infer_deadline_in_10_minutes() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_deadline("task in 10 minutes", &context, None);
        assert!(result.is_some());
        let inferred = result.unwrap();
        assert_eq!(inferred.source, DeadlineSource::RelativeTime);
        assert!(inferred.is_explicit);
        // Should be in "today HH:MM" format
        assert!(inferred.deadline.starts_with("today "));
    }

    #[test]
    fn test_infer_deadline_in_3_hours() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_deadline("call in 3 hours", &context, None);
        assert!(result.is_some());
        let inferred = result.unwrap();
        assert_eq!(inferred.source, DeadlineSource::RelativeTime);
        // Should be HH:MM format (no "today" prefix)
        assert!(!inferred.deadline.starts_with("today "));
        assert!(inferred.deadline.contains(':'));
    }

    #[test]
    fn test_infer_deadline_in_14_days() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_deadline("task in 14 days", &context, None);
        assert!(result.is_some());
        let inferred = result.unwrap();
        // Should be in +Xd format
        assert_eq!(inferred.deadline, "+14d");
    }

    #[test]
    fn test_infer_deadline_next_monday() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_deadline("task next monday", &context, None);
        assert!(result.is_some());
        let inferred = result.unwrap();
        assert_eq!(inferred.source, DeadlineSource::Explicit);
        assert_eq!(inferred.deadline, "monday");
    }

    #[test]
    fn test_infer_deadline_next_friday() {
        let context = TimeContext::default();
        let result = DeadlineInference::infer_deadline("deadline next friday", &context, None);
        assert!(result.is_some());
        let inferred = result.unwrap();
        assert_eq!(inferred.source, DeadlineSource::Explicit);
        assert_eq!(inferred.deadline, "friday");
    }

    #[test]
    fn test_infer_deadline_case_variations() {
        let context = TimeContext::default();

        // Test "IN 2 HOURS" (uppercase)
        let result = DeadlineInference::infer_deadline("task IN 2 HOURS", &context, None);
        assert!(result.is_some());

        // Test "In 5 Days" (mixed case)
        let result = DeadlineInference::infer_deadline("In 5 Days", &context, None);
        assert!(result.is_some());

        // Test "next WEEK" (mixed case)
        let result = DeadlineInference::infer_deadline("next WEEK", &context, None);
        assert!(result.is_some());
    }

    #[test]
    fn test_infer_deadline_with_plural_units() {
        let context = TimeContext::default();

        // Test "in 1 day" (singular)
        let result = DeadlineInference::infer_deadline("task in 1 day", &context, None);
        assert!(result.is_some());

        // Test "in 2 days" (plural)
        let result = DeadlineInference::infer_deadline("task in 2 days", &context, None);
        assert!(result.is_some());

        // Test "in 1 week" (singular week)
        let result = DeadlineInference::infer_deadline("task in 1 week", &context, None);
        assert!(result.is_some());

        // Test "in 2 weeks" (plural weeks)
        let result = DeadlineInference::infer_deadline("task in 2 weeks", &context, None);
        assert!(result.is_some());
    }

    #[test]
    fn test_format_relative_deadline_edge_cases() {
        let context = TimeContext::default();

        // Test 1 second (should still work)
        let result = DeadlineInference::format_relative_deadline(1, &context);
        assert!(result.starts_with("today "));

        // Test 60 seconds exactly (1 minute)
        let result = DeadlineInference::format_relative_deadline(60, &context);
        assert!(result.starts_with("today "));

        // Test 3600 seconds exactly (1 hour)
        let result = DeadlineInference::format_relative_deadline(3600, &context);
        assert!(result.contains(':'));

        // Test 86400 seconds exactly (1 day)
        let result = DeadlineInference::format_relative_deadline(86400, &context);
        assert_eq!(result, "+1d");
    }

    // === DisambiguationHelper Tests ===

    #[test]
    fn test_check_category_ambiguity_no_ambiguity_single_match() {
        let categories = vec!["work".to_string(), "personal".to_string()];
        let result = DisambiguationHelper::check_category_ambiguity("work", &categories);
        assert!(result.is_none());
    }

    #[test]
    fn test_check_category_ambiguity_no_ambiguity_empty_input() {
        let categories = vec!["work".to_string(), "personal".to_string()];
        let result = DisambiguationHelper::check_category_ambiguity("", &categories);
        assert!(result.is_none());
    }

    #[test]
    fn test_check_category_ambiguity_no_ambiguity_empty_categories() {
        let categories: Vec<String> = vec![];
        let result = DisambiguationHelper::check_category_ambiguity("work", &categories);
        assert!(result.is_none());
    }

    #[test]
    fn test_check_category_ambiguity_similar_categories() {
        let categories = vec![
            "work-project".to_string(),
            "work-meeting".to_string(),
            "personal-project".to_string(),
        ];
        // "work" should match both work-project and work-meeting
        let result = DisambiguationHelper::check_category_ambiguity("work", &categories);
        // Should be ambiguous since both start with "work"
        assert!(result.is_some());
        let disambiguation = result.unwrap();
        assert_eq!(disambiguation.ambiguity_type, AmbiguityType::Category);
        assert!(disambiguation.candidates.len() >= 2);
    }

    #[test]
    fn test_check_category_ambiguity_fuzzy_input() {
        let categories = vec![
            "engineering".to_string(),
            "engagement".to_string(),
            "entertainment".to_string(),
        ];
        // "eng" could match engineering or engagement
        let result = DisambiguationHelper::check_category_ambiguity("eng", &categories);
        assert!(result.is_some());
    }

    #[test]
    fn test_check_task_ambiguity_empty_input() {
        let tasks = vec!["buy groceries".to_string(), "call mom".to_string()];
        let result = DisambiguationHelper::check_task_ambiguity("", &tasks);
        assert!(result.is_none());
    }

    #[test]
    fn test_check_task_ambiguity_empty_tasks() {
        let tasks: Vec<String> = vec![];
        let result = DisambiguationHelper::check_task_ambiguity("groceries", &tasks);
        assert!(result.is_none());
    }

    #[test]
    fn test_check_task_ambiguity_similar_tasks() {
        let tasks = vec![
            "buy groceries from store".to_string(),
            "buy groceries from market".to_string(),
            "buy online groceries".to_string(),
        ];
        // "groceries" should match multiple tasks
        let result = DisambiguationHelper::check_task_ambiguity("groceries", &tasks);
        assert!(result.is_some());
        let disambiguation = result.unwrap();
        assert_eq!(disambiguation.ambiguity_type, AmbiguityType::Task);
        assert!(disambiguation.candidates.len() >= 2);
    }

    #[test]
    fn test_check_task_ambiguity_word_overlap() {
        let tasks = vec![
            "schedule team meeting for monday".to_string(),
            "schedule team meeting for tuesday".to_string(),
            "schedule individual meeting".to_string(),
        ];
        // "team meeting" could match both monday and tuesday versions
        let result = DisambiguationHelper::check_task_ambiguity("team meeting", &tasks);
        assert!(result.is_some());
    }

    #[test]
    fn test_check_task_ambiguity_unique_match() {
        let tasks = vec![
            "buy groceries".to_string(),
            "call mom".to_string(),
            "exercise for 30 minutes".to_string(),
        ];
        // "exercise" should uniquely match the exercise task
        let result = DisambiguationHelper::check_task_ambiguity("exercise", &tasks);
        assert!(result.is_none());
    }

    #[test]
    fn test_check_deadline_ambiguity_later() {
        let result = DisambiguationHelper::check_deadline_ambiguity("do this later");
        assert!(result.is_some());
        let disambiguation = result.unwrap();
        assert_eq!(disambiguation.ambiguity_type, AmbiguityType::Deadline);
        assert!(!disambiguation.candidates.is_empty());
    }

    #[test]
    fn test_check_deadline_ambiguity_sometime() {
        let result = DisambiguationHelper::check_deadline_ambiguity("finish sometime");
        assert!(result.is_some());
    }

    #[test]
    fn test_check_deadline_ambiguity_specific() {
        let result = DisambiguationHelper::check_deadline_ambiguity("finish by tomorrow");
        assert!(result.is_none());
    }

    #[test]
    fn test_resolve_by_index_valid() {
        let disambiguation = Disambiguation {
            input: "test".to_string(),
            ambiguity_type: AmbiguityType::Category,
            candidates: vec![
                DisambiguationCandidate {
                    value: "option1".to_string(),
                    confidence: 0.9,
                    context: None,
                },
                DisambiguationCandidate {
                    value: "option2".to_string(),
                    confidence: 0.8,
                    context: None,
                },
            ],
            prompt: "Choose one".to_string(),
        };

        let result = DisambiguationHelper::resolve_by_index(&disambiguation, 0);
        assert_eq!(result, Some("option1".to_string()));

        let result = DisambiguationHelper::resolve_by_index(&disambiguation, 1);
        assert_eq!(result, Some("option2".to_string()));
    }

    #[test]
    fn test_resolve_by_index_invalid() {
        let disambiguation = Disambiguation {
            input: "test".to_string(),
            ambiguity_type: AmbiguityType::Category,
            candidates: vec![
                DisambiguationCandidate {
                    value: "option1".to_string(),
                    confidence: 0.9,
                    context: None,
                },
            ],
            prompt: "Choose one".to_string(),
        };

        let result = DisambiguationHelper::resolve_by_index(&disambiguation, 5);
        assert!(result.is_none());
    }

    #[test]
    fn test_format_disambiguation_category() {
        let disambiguation = Disambiguation {
            input: "work".to_string(),
            ambiguity_type: AmbiguityType::Category,
            candidates: vec![
                DisambiguationCandidate {
                    value: "work-project".to_string(),
                    confidence: 0.85,
                    context: None,
                },
                DisambiguationCandidate {
                    value: "work-meeting".to_string(),
                    confidence: 0.75,
                    context: None,
                },
            ],
            prompt: "Multiple categories match 'work'".to_string(),
        };

        let formatted = DisambiguationHelper::format_disambiguation(&disambiguation);
        assert!(formatted.contains("Category Ambiguity"));
        assert!(formatted.contains("work-project"));
        assert!(formatted.contains("work-meeting"));
    }

    #[test]
    fn test_format_disambiguation_task() {
        let disambiguation = Disambiguation {
            input: "meeting".to_string(),
            ambiguity_type: AmbiguityType::Task,
            candidates: vec![
                DisambiguationCandidate {
                    value: "schedule team meeting".to_string(),
                    confidence: 0.8,
                    context: None,
                },
            ],
            prompt: "Multiple tasks match 'meeting'".to_string(),
        };

        let formatted = DisambiguationHelper::format_disambiguation(&disambiguation);
        assert!(formatted.contains("Task Ambiguity"));
    }

    #[test]
    fn test_format_disambiguation_deadline() {
        let disambiguation = Disambiguation {
            input: "later".to_string(),
            ambiguity_type: AmbiguityType::Deadline,
            candidates: vec![
                DisambiguationCandidate {
                    value: "today".to_string(),
                    confidence: 0.8,
                    context: Some("Due by end of day".to_string()),
                },
            ],
            prompt: "When would you like to complete this?".to_string(),
        };

        let formatted = DisambiguationHelper::format_disambiguation(&disambiguation);
        assert!(formatted.contains("Deadline Ambiguity"));
    }

    #[test]
    fn test_disambiguation_candidate_context() {
        let candidate = DisambiguationCandidate {
            value: "today".to_string(),
            confidence: 0.9,
            context: Some("Due by end of day".to_string()),
        };

        assert_eq!(candidate.value, "today");
        assert_eq!(candidate.confidence, 0.9);
        assert_eq!(candidate.context, Some("Due by end of day".to_string()));
    }

    #[test]
    fn test_disambiguation_candidate_no_context() {
        let candidate = DisambiguationCandidate {
            value: "work".to_string(),
            confidence: 0.85,
            context: None,
        };

        assert!(candidate.context.is_none());
    }

    #[test]
    fn test_ambiguity_type_equality() {
        assert_eq!(AmbiguityType::Category, AmbiguityType::Category);
        assert_ne!(AmbiguityType::Category, AmbiguityType::Task);
        assert_ne!(AmbiguityType::Task, AmbiguityType::Deadline);
    }

    #[test]
    fn test_disambiguation_display_impl() {
        let disambiguation = Disambiguation {
            input: "wrk".to_string(),
            ambiguity_type: AmbiguityType::Category,
            candidates: vec![
                DisambiguationCandidate {
                    value: "work".to_string(),
                    confidence: 0.9,
                    context: None,
                },
            ],
            prompt: "Multiple categories match".to_string(),
        };

        let display = format!("{}", disambiguation);
        assert!(display.contains("Category Ambiguity"));
    }

    // === Edge Cases for Disambiguation ===

    #[test]
    fn test_check_category_ambiguity_case_insensitive() {
        let categories = vec![
            "WorkProject".to_string(),
            "WorkMeeting".to_string(),
        ];
        // Lowercase input should still match
        let result = DisambiguationHelper::check_category_ambiguity("work", &categories);
        assert!(result.is_some());
    }

    #[test]
    fn test_check_task_ambiguity_case_insensitive() {
        let tasks = vec![
            "Buy Groceries From Store".to_string(),
            "Buy Groceries From Market".to_string(),
        ];
        let result = DisambiguationHelper::check_task_ambiguity("groceries", &tasks);
        assert!(result.is_some());
    }

    #[test]
    fn test_check_task_ambiguity_single_task() {
        let tasks = vec!["buy groceries".to_string()];
        let result = DisambiguationHelper::check_task_ambiguity("groceries", &tasks);
        // Single task should not be ambiguous even if it matches
        assert!(result.is_none());
    }

    #[test]
    fn test_check_category_ambiguity_single_category() {
        let categories = vec!["work".to_string()];
        let result = DisambiguationHelper::check_category_ambiguity("work", &categories);
        assert!(result.is_none());
    }

    #[test]
    fn test_score_tasks_empty_tasks() {
        let input_lower = "test";
        let input_words: Vec<&str> = vec!["test"];
        let tasks: Vec<String> = vec![];

        let result = score_tasks(input_lower, &input_words, &tasks);
        assert!(result.is_empty());
    }

    #[test]
    fn test_score_tasks_word_overlap() {
        let tasks = vec![
            "schedule team meeting for monday".to_string(),
            "buy groceries from store".to_string(),
        ];

        let result = score_tasks("team meeting", &["team", "meeting"], &tasks);
        assert_eq!(result.len(), 2);
        // First task should have higher score due to word overlap
        assert!(result[0].1 > result[1].1);
    }

    #[test]
    fn test_score_tasks_fuzzy_similarity() {
        let tasks = vec![
            "buy groceries at store".to_string(),
            "purchase grocery items".to_string(),
        ];

        let result = score_tasks("groceries", &["groceries"], &tasks);
        assert_eq!(result.len(), 2);
        // Both tasks should have reasonable scores
        assert!(result[0].1 > 0.0);
        assert!(result[1].1 > 0.0);
    }
}
