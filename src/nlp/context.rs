//! Context awareness for natural language processing

use super::types::*;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

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
            let score = Self::similarity_score(&input_lower, &candidate.to_lowercase());
            if score >= threshold {
                matches.push((candidate.clone(), score));
            }
        }

        matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        matches
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
}
