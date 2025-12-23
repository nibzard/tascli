//! Fast pattern matching for simple commands
//!
//! This module provides regex-based pattern matching for common simple commands
//! that don't require full AI processing. This significantly reduces latency and
//! API usage for frequently used commands.

use super::types::{NLPCommand, ActionType, StatusType, QueryType, Condition, ConditionExpression, ComparisonOperator};
use super::conditional::ConditionBuilder;
use regex::Regex;
use std::sync::LazyLock;

/// Result of pattern matching
#[derive(Debug, Clone)]
pub enum PatternMatch {
    /// A command was matched
    Matched(NLPCommand),
    /// No pattern matched - needs AI processing
    NeedsAI,
    /// Input is ambiguous
    Ambiguous(String),
}

/// Pattern matcher for simple commands
pub struct PatternMatcher;

// === Task Addition Patterns ===
// "add task ...", "create task ...", "new task ...", "task ..."
static ADD_TASK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^(?:(add|create|new)\s+)?task\s+(.+)$").unwrap()
});

// === Record Addition Patterns ===
// "add record ...", "log ...", "record ..."
static ADD_RECORD_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^(?:add\s+)?(?:record|log)\s+(.+)$").unwrap()
});

// === Completion Patterns ===
// "complete #", "done #", "finish #", "check #"
static COMPLETE_TASK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^(?:complete|done|finish|check|tick)\s+#?(\d+)$").unwrap()
});

// === Deletion Patterns ===
// "delete #", "remove #", "del #"
static DELETE_TASK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^(?:delete|remove|del)\s+#?(\d+)$").unwrap()
});

// === Simple List Patterns ===
// "list", "list tasks", "show tasks", "ls"
static LIST_ALL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^(?:list\s+tasks?|show\s+tasks?|ls|list|show)$").unwrap()
});

// === List Records Pattern ===
// "list records", "show records", "records"
static LIST_RECORDS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^(?:list\s+records|show\s+records|records)$").unwrap()
});

// === List by Category Patterns ===
// "list work tasks", "show personal", "work tasks"
static LIST_CATEGORY_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^(?:list|show)?\s*(\w+)\s+tasks?$").unwrap()
});

// === List by Status Patterns ===
// "list done", "show pending", "completed tasks"
static LIST_STATUS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^(?:list|show)?\s*(done|pending|ongoing|cancelled|all)\s+tasks?$").unwrap()
});

// === Query Type Patterns ===
// "overdue", "upcoming", "due today", "due tomorrow", "unscheduled"
static QUERY_TYPE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^(overdue|upcoming|due today|due tomorrow|unscheduled|urgent|due this week|due this month)(?:\s+tasks?)?$").unwrap()
});

// === Update Patterns ===
// "update #", "edit #", "modify #"
static UPDATE_TASK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^(?:update|edit|modify)\s+#?(\d+)(?:\s+(.+))?$").unwrap()
});

// === Help Patterns ===
// "help", "what can i do", "how to use"
static HELP_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^(?:help|what\s+can\s+i\s+do|how\s+to\s+use|\?)$").unwrap()
});

// === Clear/Reset Patterns ===
// "clear all", "reset"
static CLEAR_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^(?:clear\s+all|reset)(?:\s+tasks?)?$").unwrap()
});

// === Category Setting Patterns ===
// "set category to ...", "change category to ..."
static SET_CATEGORY_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^set\s+(\w+)\s+category\s+to\s+(\w+)$").unwrap()
});

// === Priority Patterns ===
// "high priority tasks", "urgent tasks"
static PRIORITY_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^(high|low|medium)\s+priority\s+tasks?$").unwrap()
});

// === Date-based Quick Patterns ===
// "today's tasks", "tomorrow's tasks"
static DATE_QUICK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^(today|tomorrow|yesterday)'?s?\s+tasks?$").unwrap()
});

// === Search Patterns ===
// "search for ...", "find ..."
static SEARCH_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^search\s+(.+)$").unwrap()
});

// === Conditional Patterns ===
// "if <category> has tasks then ...", "if category <category> is not empty then ..."
static IF_CATEGORY_HAS_TASKS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^if\s+(\w+)\s+category\s+(?:has\s+tasks|is\s+not\s+empty)\s+then\s+(.+)$").unwrap()
});

// "if <category> is empty then ...", "if category <category> has no tasks then ..."
static IF_CATEGORY_EMPTY_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^if\s+(\w+)\s+category\s+(?:is\s+empty|has\s+no\s+tasks)\s+then\s+(.+)$").unwrap()
});

// "if task count is <operator> <number> then ..."
static IF_TASK_COUNT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^if\s+task\s+count\s+is\s+(>=|<=|>|<|=|!=)\s+(\d+)\s+then\s+(.+)$").unwrap()
});

// "if today is ... then ..."
static IF_DAY_OF_WEEK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^if\s+today\s+is\s+(monday|tuesday|wednesday|thursday|friday|saturday|sunday|weekend|weekday)\s+then\s+(.+)$").unwrap()
});

// "if time is >= HH:MM then ..."
static IF_TIME_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^if\s+time\s+is\s+(>=|<=|>|<|=)\s+(\d{1,2}):(\d{2})\s+then\s+(.+)$").unwrap()
});

// "if previous succeeded then ..."
static IF_PREVIOUS_SUCCESS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^if\s+previous\s+(?:command\s+)?succeeded\s+then\s+(.+)$").unwrap()
});

// "if previous failed then ..."
static IF_PREVIOUS_FAILED_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^if\s+previous\s+(?:command\s+)?failed\s+then\s+(.+)$").unwrap()
});

impl PatternMatcher {
    /// Try to match input against known patterns
    /// Returns PatternMatch::Matched if a simple pattern is found
    /// Returns PatternMatch::NeedsAI if input requires AI processing
    pub fn match_input(input: &str) -> PatternMatch {
        let input = input.trim();
        let input_lower = input.to_lowercase();

        // Handle empty input
        if input.is_empty() {
            return PatternMatch::NeedsAI;
        }

        // Keywords that indicate complex input requiring AI
        // We check these with word boundaries to avoid false positives
        // E.g., "unscheduled" contains "schedule" but is a valid query type

        // Check for specific complex patterns that require AI
        // These patterns indicate the user wants to do something complex that AI should handle
        if input_lower.contains("deadline") ||
           input_lower.contains(" to work category") ||
           input_lower.contains(" to personal category") ||
           input_lower.ends_with(" category") && input_lower.len() > "category".len() + 5 ||
           input_lower.contains("every day") ||
           input_lower.contains("every week") ||
           input_lower.contains("recurring") ||
           input_lower.contains("repeat") {
            return PatternMatch::NeedsAI;
        }

        // Handle single word list requests
        if input == "list" || input == "ls" || input == "show" {
            return PatternMatch::Matched(NLPCommand {
                action: ActionType::List,
                content: String::new(),
                ..Default::default()
            });
        }

        // === Task Addition ===
        if let Some(caps) = ADD_TASK_RE.captures(input) {
            let content = caps.get(2).map(|m| m.as_str().to_string()).unwrap_or_default();
            return PatternMatch::Matched(NLPCommand {
                action: ActionType::Task,
                content,
                ..Default::default()
            });
        }

        // === Record Addition ===
        if let Some(caps) = ADD_RECORD_RE.captures(input) {
            let content = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
            return PatternMatch::Matched(NLPCommand {
                action: ActionType::Record,
                content,
                ..Default::default()
            });
        }

        // === Completion ===
        if let Some(caps) = COMPLETE_TASK_RE.captures(input) {
            if let Some(id) = caps.get(1) {
                return PatternMatch::Matched(NLPCommand {
                    action: ActionType::Done,
                    content: id.as_str().to_string(),
                    ..Default::default()
                });
            }
        }

        // === Deletion ===
        if let Some(caps) = DELETE_TASK_RE.captures(input) {
            if let Some(id) = caps.get(1) {
                return PatternMatch::Matched(NLPCommand {
                    action: ActionType::Delete,
                    content: id.as_str().to_string(),
                    ..Default::default()
                });
            }
        }

        // === Simple List ===
        if LIST_ALL_RE.is_match(input) {
            return PatternMatch::Matched(NLPCommand {
                action: ActionType::List,
                content: String::new(),
                ..Default::default()
            });
        }

        // === List Records ===
        if LIST_RECORDS_RE.is_match(input) {
            return PatternMatch::Matched(NLPCommand {
                action: ActionType::List,
                content: String::new(),
                filters: {
                    let mut f = std::collections::HashMap::new();
                    f.insert("type".to_string(), "record".to_string());
                    f
                },
                ..Default::default()
            });
        }

        // === List by Category ===
        if let Some(caps) = LIST_CATEGORY_RE.captures(input) {
            if let Some(category) = caps.get(1) {
                let cat = category.as_str().to_string();
                // Skip if this is actually a status word
                if !matches!(cat.to_lowercase().as_str(), "done" | "pending" | "ongoing" | "cancelled" | "all" | "overdue" | "upcoming" | "urgent") {
                    return PatternMatch::Matched(NLPCommand {
                        action: ActionType::List,
                        content: String::new(),
                        category: Some(cat),
                        ..Default::default()
                    });
                }
            }
        }

        // === List by Status ===
        if let Some(caps) = LIST_STATUS_RE.captures(input) {
            if let Some(status) = caps.get(1) {
                let status_type = match status.as_str().to_lowercase().as_str() {
                    "done" | "completed" => StatusType::Done,
                    "pending" => StatusType::Pending,
                    "ongoing" | "open" => StatusType::Ongoing,
                    "cancelled" | "canceled" => StatusType::Cancelled,
                    "all" => StatusType::All,
                    _ => return PatternMatch::NeedsAI,
                };
                return PatternMatch::Matched(NLPCommand {
                    action: ActionType::List,
                    content: String::new(),
                    status: Some(status_type),
                    ..Default::default()
                });
            }
        }

        // === Query Types ===
        if let Some(caps) = QUERY_TYPE_RE.captures(input) {
            if let Some(query) = caps.get(1) {
                let query_type = match query.as_str().to_lowercase().as_str() {
                    "overdue" => QueryType::Overdue,
                    "upcoming" => QueryType::Upcoming,
                    "unscheduled" => QueryType::Unscheduled,
                    "due today" => QueryType::DueToday,
                    "due tomorrow" => QueryType::DueTomorrow,
                    "due this week" => QueryType::DueThisWeek,
                    "due this month" => QueryType::DueThisMonth,
                    "urgent" => QueryType::Urgent,
                    _ => return PatternMatch::NeedsAI,
                };
                return PatternMatch::Matched(NLPCommand {
                    action: ActionType::List,
                    content: String::new(),
                    query_type: Some(query_type),
                    ..Default::default()
                });
            }
        }

        // === Update ===
        if let Some(caps) = UPDATE_TASK_RE.captures(input) {
            if let Some(id) = caps.get(1) {
                let content = caps.get(2).map(|m| m.as_str().to_string()).unwrap_or_default();
                let mut modifications = std::collections::HashMap::new();
                if !content.is_empty() {
                    modifications.insert("content".to_string(), content);
                }
                return PatternMatch::Matched(NLPCommand {
                    action: ActionType::Update,
                    content: id.as_str().to_string(),
                    modifications,
                    ..Default::default()
                });
            }
        }

        // === Help ===
        if HELP_RE.is_match(input) {
            return PatternMatch::Ambiguous("Help requested - showing available commands".to_string());
        }

        // === Clear/Reset ===
        if CLEAR_RE.is_match(input) {
            return PatternMatch::Ambiguous("Clear all tasks? Confirm with 'yes'".to_string());
        }

        // === Search Pattern ===
        if let Some(caps) = SEARCH_RE.captures(input) {
            if let Some(search_term) = caps.get(1) {
                return PatternMatch::Matched(NLPCommand {
                    action: ActionType::List,
                    content: String::new(),
                    search: Some(search_term.as_str().to_string()),
                    ..Default::default()
                });
            }
        }

        // === Priority Pattern ===
        if let Some(caps) = PRIORITY_RE.captures(input) {
            if let Some(priority) = caps.get(1) {
                let mut filters = std::collections::HashMap::new();
                filters.insert("priority".to_string(), priority.as_str().to_string());
                return PatternMatch::Matched(NLPCommand {
                    action: ActionType::List,
                    content: String::new(),
                    filters,
                    ..Default::default()
                });
            }
        }

        // === Date Quick Pattern ===
        if let Some(caps) = DATE_QUICK_RE.captures(input) {
            if let Some(day) = caps.get(1) {
                let query_type = match day.as_str().to_lowercase().as_str() {
                    "today" => QueryType::DueToday,
                    "tomorrow" => QueryType::DueTomorrow,
                    "yesterday" => QueryType::Overdue,
                    _ => return PatternMatch::NeedsAI,
                };
                return PatternMatch::Matched(NLPCommand {
                    action: ActionType::List,
                    content: String::new(),
                    query_type: Some(query_type),
                    ..Default::default()
                });
            }
        }

        // === Category Setting Pattern ===
        if let Some(caps) = SET_CATEGORY_RE.captures(input) {
            if let (Some(item_name), Some(category)) = (caps.get(1), caps.get(2)) {
                let mut modifications = std::collections::HashMap::new();
                modifications.insert("category".to_string(), category.as_str().to_string());
                return PatternMatch::Matched(NLPCommand {
                    action: ActionType::Update,
                    content: item_name.as_str().to_string(),
                    modifications,
                    ..Default::default()
                });
            }
        }

        // === Single number (treat as show task details) ===
        if let Some(caps) = Regex::new(r"^#?(\d+)$").unwrap().captures(input) {
            return PatternMatch::Matched(NLPCommand {
                action: ActionType::List,
                content: caps.get(1).unwrap().as_str().to_string(),
                filters: {
                    let mut f = std::collections::HashMap::new();
                    f.insert("id".to_string(), caps.get(1).unwrap().as_str().to_string());
                    f
                },
                ..Default::default()
            });
        }

        // === Very simple "add <content>" pattern ===
        if let Some(caps) = Regex::new(r"^add\s+(.+)$").unwrap().captures(input) {
            let content = caps.get(1).unwrap().as_str().to_string();
            return PatternMatch::Matched(NLPCommand {
                action: ActionType::Task,
                content,
                ..Default::default()
            });
        }

        // === Conditional Patterns ===
        // Note: Complex conditionals may require AI processing for full accuracy

        // "if <category> has tasks then ..." or "if <category> category is not empty then ..."
        if let Some(caps) = IF_CATEGORY_HAS_TASKS_RE.captures(input) {
            if let (Some(category), Some(then_command)) = (caps.get(1), caps.get(2)) {
                let condition = ConditionBuilder::category_has_tasks(category.as_str());
                let then_content = then_command.as_str().to_string();
                return PatternMatch::Matched(NLPCommand {
                    action: ActionType::Task,
                    content: then_content,
                    condition: Some(condition),
                    ..Default::default()
                });
            }
        }

        // "if <category> is empty then ..." or "if <category> category has no tasks then ..."
        if let Some(caps) = IF_CATEGORY_EMPTY_RE.captures(input) {
            if let (Some(category), Some(then_command)) = (caps.get(1), caps.get(2)) {
                let condition = ConditionBuilder::category_empty(category.as_str());
                let then_content = then_command.as_str().to_string();
                return PatternMatch::Matched(NLPCommand {
                    action: ActionType::Task,
                    content: then_content,
                    condition: Some(condition),
                    ..Default::default()
                });
            }
        }

        // "if task count is <operator> <number> then ..."
        if let Some(caps) = IF_TASK_COUNT_RE.captures(input) {
            if let (Some(op_str), Some(value_str), Some(then_command)) = (caps.get(1), caps.get(2), caps.get(3)) {
                let operator = match op_str.as_str() {
                    ">=" => ComparisonOperator::GreaterOrEqual,
                    "<=" => ComparisonOperator::LessOrEqual,
                    ">" => ComparisonOperator::GreaterThan,
                    "<" => ComparisonOperator::LessThan,
                    "=" | "==" => ComparisonOperator::Equal,
                    "!=" => ComparisonOperator::NotEqual,
                    _ => return PatternMatch::NeedsAI,
                };
                let value: i32 = value_str.as_str().parse().unwrap_or(0);
                let condition = ConditionBuilder::task_count(operator, value);
                let then_content = then_command.as_str().to_string();
                return PatternMatch::Matched(NLPCommand {
                    action: ActionType::Task,
                    content: then_content,
                    condition: Some(condition),
                    ..Default::default()
                });
            }
        }

        // "if today is ... then ..."
        if let Some(caps) = IF_DAY_OF_WEEK_RE.captures(input) {
            if let (Some(day), Some(then_command)) = (caps.get(1), caps.get(2)) {
                let days = match day.as_str().to_lowercase().as_str() {
                    "monday" => vec!["Monday"],
                    "tuesday" => vec!["Tuesday"],
                    "wednesday" => vec!["Wednesday"],
                    "thursday" => vec!["Thursday"],
                    "friday" => vec!["Friday"],
                    "saturday" => vec!["Saturday"],
                    "sunday" => vec!["Sunday"],
                    "weekend" => vec!["Saturday", "Sunday"],
                    "weekday" => vec!["Monday", "Tuesday", "Wednesday", "Thursday", "Friday"],
                    _ => return PatternMatch::NeedsAI,
                };
                let condition = ConditionBuilder::day_of_week(days);
                let then_content = then_command.as_str().to_string();
                return PatternMatch::Matched(NLPCommand {
                    action: ActionType::Task,
                    content: then_content,
                    condition: Some(condition),
                    ..Default::default()
                });
            }
        }

        // "if time is >= HH:MM then ..."
        if let Some(caps) = IF_TIME_RE.captures(input) {
            if let (Some(op_str), Some(hour_str), Some(minute_str), Some(then_command)) =
                (caps.get(1), caps.get(2), caps.get(3), caps.get(4)) {
                let operator = match op_str.as_str() {
                    ">=" => ComparisonOperator::GreaterOrEqual,
                    "<=" => ComparisonOperator::LessOrEqual,
                    ">" => ComparisonOperator::GreaterThan,
                    "<" => ComparisonOperator::LessThan,
                    "=" | "==" => ComparisonOperator::Equal,
                    "!=" => ComparisonOperator::NotEqual,
                    _ => return PatternMatch::NeedsAI,
                };
                let hour: i32 = hour_str.as_str().parse().unwrap_or(0);
                let minute: i32 = minute_str.as_str().parse().unwrap_or(0);
                let condition = ConditionBuilder::time_condition(operator, Some(hour), Some(minute));
                let then_content = then_command.as_str().to_string();
                return PatternMatch::Matched(NLPCommand {
                    action: ActionType::Task,
                    content: then_content,
                    condition: Some(condition),
                    ..Default::default()
                });
            }
        }

        // "if previous succeeded then ..."
        if let Some(caps) = IF_PREVIOUS_SUCCESS_RE.captures(input) {
            if let Some(then_command) = caps.get(1) {
                let condition = ConditionBuilder::previous_success();
                let then_content = then_command.as_str().to_string();
                return PatternMatch::Matched(NLPCommand {
                    action: ActionType::Task,
                    content: then_content,
                    condition: Some(condition),
                    ..Default::default()
                });
            }
        }

        // "if previous failed then ..."
        if let Some(caps) = IF_PREVIOUS_FAILED_RE.captures(input) {
            if let Some(then_command) = caps.get(1) {
                let condition = ConditionBuilder::previous_failed();
                let then_content = then_command.as_str().to_string();
                return PatternMatch::Matched(NLPCommand {
                    action: ActionType::Task,
                    content: then_content,
                    condition: Some(condition),
                    ..Default::default()
                });
            }
        }

        // No pattern matched
        PatternMatch::NeedsAI
    }

    /// Check if input might be matchable by patterns
    /// (useful for caching decisions)
    pub fn is_simple_input(input: &str) -> bool {
        let input = input.trim().to_lowercase();

        // Exclude complex phrases
        if input.starts_with("show me") ||
           input.starts_with("list all") ||
           input.contains("deadline") ||
           (input.contains("category") && !input.ends_with("category") && !input.ends_with("category tasks")) ||
           input.contains("every day") ||
           input.contains("every week") ||
           input.contains("recurring") {
            return false;
        }

        // Quick heuristics for simple inputs (expanded with new patterns)
        input.starts_with("add ")
            || input.starts_with("task ")
            || input.starts_with("complete ")
            || input.starts_with("done ")
            || input.starts_with("delete ")
            || input.starts_with("list ")
            || input == "list"
            || input == "ls"
            || (input.starts_with("show ") && !input.starts_with("show me"))
            || input.starts_with("update ")
            || input.starts_with("edit ")
            || input.starts_with("search ")
            || input.starts_with("set ")
            || Regex::new(r"^(overdue|upcoming|urgent|help|\?|clear|today|tomorrow|yesterday)").unwrap().is_match(&input)
    }

    /// Get statistics about pattern matching
    pub fn stats() -> PatternMatcherStats {
        PatternMatcherStats {
            total_patterns: 26,
            patterns_checked: vec![
                "add_task", "add_record", "complete", "delete", "list_all",
                "list_records", "list_category", "list_status", "query_type",
                "update", "help", "clear", "single_number", "simple_add",
                "search", "priority", "date_quick", "set_category",
                "if_category_has_tasks", "if_category_empty", "if_task_count",
                "if_day_of_week", "if_time", "if_previous_success", "if_previous_failed",
            ],
        }
    }
}

/// Statistics about the pattern matcher
#[derive(Debug, Clone)]
pub struct PatternMatcherStats {
    pub total_patterns: usize,
    pub patterns_checked: Vec<&'static str>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Task Addition Tests ===

    #[test]
    fn test_match_add_task() {
        let result = PatternMatcher::match_input("add task buy groceries");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::Task);
            assert_eq!(cmd.content, "buy groceries");
        }
    }

    #[test]
    fn test_match_create_task() {
        let result = PatternMatcher::match_input("create task review code");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::Task);
            assert_eq!(cmd.content, "review code");
        }
    }

    #[test]
    fn test_match_new_task() {
        let result = PatternMatcher::match_input("new task call mom");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::Task);
            assert_eq!(cmd.content, "call mom");
        }
    }

    #[test]
    fn test_match_task_only() {
        let result = PatternMatcher::match_input("task write report");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::Task);
            assert_eq!(cmd.content, "write report");
        }
    }

    // === Record Addition Tests ===

    #[test]
    fn test_match_add_record() {
        let result = PatternMatcher::match_input("add record meeting notes");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::Record);
            assert_eq!(cmd.content, "meeting notes");
        }
    }

    #[test]
    fn test_match_log() {
        let result = PatternMatcher::match_input("log completed project");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::Record);
            assert_eq!(cmd.content, "completed project");
        }
    }

    #[test]
    fn test_match_record_only() {
        let result = PatternMatcher::match_input("record daily standup");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::Record);
            assert_eq!(cmd.content, "daily standup");
        }
    }

    // === Completion Tests ===

    #[test]
    fn test_match_complete_number() {
        let result = PatternMatcher::match_input("complete 5");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::Done);
            assert_eq!(cmd.content, "5");
        }
    }

    #[test]
    fn test_match_done_number() {
        let result = PatternMatcher::match_input("done 3");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::Done);
            assert_eq!(cmd.content, "3");
        }
    }

    #[test]
    fn test_match_finish_number() {
        let result = PatternMatcher::match_input("finish 10");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::Done);
            assert_eq!(cmd.content, "10");
        }
    }

    #[test]
    fn test_match_complete_with_hash() {
        let result = PatternMatcher::match_input("complete #7");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::Done);
            assert_eq!(cmd.content, "7");
        }
    }

    // === Deletion Tests ===

    #[test]
    fn test_match_delete_number() {
        let result = PatternMatcher::match_input("delete 2");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::Delete);
            assert_eq!(cmd.content, "2");
        }
    }

    #[test]
    fn test_match_remove_number() {
        let result = PatternMatcher::match_input("remove 4");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::Delete);
            assert_eq!(cmd.content, "4");
        }
    }

    #[test]
    fn test_match_del_number() {
        let result = PatternMatcher::match_input("del 1");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::Delete);
            assert_eq!(cmd.content, "1");
        }
    }

    // === List Tests ===

    #[test]
    fn test_match_list() {
        let result = PatternMatcher::match_input("list");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::List);
        }
    }

    #[test]
    fn test_match_ls() {
        let result = PatternMatcher::match_input("ls");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::List);
        }
    }

    #[test]
    fn test_match_show() {
        let result = PatternMatcher::match_input("show");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::List);
        }
    }

    #[test]
    fn test_match_list_tasks() {
        let result = PatternMatcher::match_input("list tasks");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::List);
        }
    }

    #[test]
    fn test_match_show_tasks() {
        let result = PatternMatcher::match_input("show tasks");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::List);
        }
    }

    // === List Records Tests ===

    #[test]
    fn test_match_list_records() {
        let result = PatternMatcher::match_input("list records");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::List);
            assert_eq!(cmd.filters.get("type"), Some(&"record".to_string()));
        }
    }

    #[test]
    fn test_match_records_only() {
        let result = PatternMatcher::match_input("records");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::List);
        }
    }

    // === List by Category Tests ===

    #[test]
    fn test_match_list_category() {
        let result = PatternMatcher::match_input("list work tasks");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::List);
            assert_eq!(cmd.category, Some("work".to_string()));
        }
    }

    #[test]
    fn test_match_show_category() {
        let result = PatternMatcher::match_input("show personal tasks");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::List);
            assert_eq!(cmd.category, Some("personal".to_string()));
        }
    }

    #[test]
    fn test_match_category_only() {
        let result = PatternMatcher::match_input("work tasks");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::List);
            assert_eq!(cmd.category, Some("work".to_string()));
        }
    }

    // === List by Status Tests ===

    #[test]
    fn test_match_list_done() {
        let result = PatternMatcher::match_input("list done tasks");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::List);
            assert_eq!(cmd.status, Some(StatusType::Done));
        }
    }

    #[test]
    fn test_match_list_pending() {
        let result = PatternMatcher::match_input("list pending tasks");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::List);
            assert_eq!(cmd.status, Some(StatusType::Pending));
        }
    }

    #[test]
    fn test_match_list_ongoing() {
        let result = PatternMatcher::match_input("list ongoing tasks");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::List);
            assert_eq!(cmd.status, Some(StatusType::Ongoing));
        }
    }

    // === Query Type Tests ===

    #[test]
    fn test_match_overdue() {
        let result = PatternMatcher::match_input("overdue");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::List);
            assert_eq!(cmd.query_type, Some(QueryType::Overdue));
        }
    }

    #[test]
    fn test_match_overdue_tasks() {
        let result = PatternMatcher::match_input("overdue tasks");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::List);
            assert_eq!(cmd.query_type, Some(QueryType::Overdue));
        }
    }

    #[test]
    fn test_match_upcoming() {
        let result = PatternMatcher::match_input("upcoming");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::List);
            assert_eq!(cmd.query_type, Some(QueryType::Upcoming));
        }
    }

    #[test]
    fn test_match_due_today() {
        let result = PatternMatcher::match_input("due today");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::List);
            assert_eq!(cmd.query_type, Some(QueryType::DueToday));
        }
    }

    #[test]
    fn test_match_unscheduled() {
        let result = PatternMatcher::match_input("unscheduled");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::List);
            assert_eq!(cmd.query_type, Some(QueryType::Unscheduled));
        }
    }

    #[test]
    fn test_match_urgent() {
        let result = PatternMatcher::match_input("urgent");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::List);
            assert_eq!(cmd.query_type, Some(QueryType::Urgent));
        }
    }

    // === Update Tests ===

    #[test]
    fn test_match_update_number() {
        let result = PatternMatcher::match_input("update 5");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::Update);
            assert_eq!(cmd.content, "5");
        }
    }

    #[test]
    fn test_match_edit_number() {
        let result = PatternMatcher::match_input("edit 3");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::Update);
            assert_eq!(cmd.content, "3");
        }
    }

    #[test]
    fn test_match_update_with_content() {
        let result = PatternMatcher::match_input("update 7 new description");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::Update);
            assert_eq!(cmd.content, "7");
            assert_eq!(cmd.modifications.get("content"), Some(&"new description".to_string()));
        }
    }

    // === Help Tests ===

    #[test]
    fn test_match_help() {
        let result = PatternMatcher::match_input("help");
        assert!(matches!(result, PatternMatch::Ambiguous(_)));
    }

    #[test]
    fn test_match_question_mark() {
        let result = PatternMatcher::match_input("?");
        assert!(matches!(result, PatternMatch::Ambiguous(_)));
    }

    // === Clear Tests ===

    #[test]
    fn test_match_clear_all() {
        let result = PatternMatcher::match_input("clear all");
        assert!(matches!(result, PatternMatch::Ambiguous(_)));
    }

    #[test]
    fn test_match_reset() {
        let result = PatternMatcher::match_input("reset");
        assert!(matches!(result, PatternMatch::Ambiguous(_)));
    }

    // === Single Number Tests ===

    #[test]
    fn test_match_single_number() {
        let result = PatternMatcher::match_input("5");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::List);
            assert_eq!(cmd.filters.get("id"), Some(&"5".to_string()));
        }
    }

    #[test]
    fn test_match_single_number_with_hash() {
        let result = PatternMatcher::match_input("#12");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::List);
            assert_eq!(cmd.filters.get("id"), Some(&"12".to_string()));
        }
    }

    // === Simple Add Tests ===

    #[test]
    fn test_match_simple_add() {
        let result = PatternMatcher::match_input("add call john");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::Task);
            assert_eq!(cmd.content, "call john");
        }
    }

    // === Needs AI Tests ===

    #[test]
    fn test_needs_ai_complex_query() {
        let result = PatternMatcher::match_input("show me all the work tasks that are due tomorrow");
        assert!(matches!(result, PatternMatch::NeedsAI));
    }

    #[test]
    fn test_needs_ai_vague_input() {
        let result = PatternMatcher::match_input("I need to do something important");
        assert!(matches!(result, PatternMatch::NeedsAI));
    }

    #[test]
    fn test_needs_ai_with_deadline() {
        let result = PatternMatcher::match_input("add task with deadline next friday");
        assert!(matches!(result, PatternMatch::NeedsAI));
    }

    #[test]
    fn test_needs_ai_with_category() {
        let result = PatternMatcher::match_input("add review documentation to work category");
        assert!(matches!(result, PatternMatch::NeedsAI));
    }

    // === Edge Cases ===

    #[test]
    fn test_empty_input() {
        let result = PatternMatcher::match_input("");
        assert!(matches!(result, PatternMatch::NeedsAI));
    }

    #[test]
    fn test_whitespace_only() {
        let result = PatternMatcher::match_input("   ");
        assert!(matches!(result, PatternMatch::NeedsAI));
    }

    #[test]
    fn test_case_insensitive_list() {
        let result = PatternMatcher::match_input("LIST");
        assert!(matches!(result, PatternMatch::Matched(_)));
    }

    #[test]
    fn test_case_insensitive_done() {
        let result = PatternMatcher::match_input("DONE 5");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::Done);
        }
    }

    #[test]
    fn test_extra_whitespace() {
        let result = PatternMatcher::match_input("  add   task   buy milk  ");
        assert!(matches!(result, PatternMatch::Matched(_)));
        if let PatternMatch::Matched(cmd) = result {
            assert_eq!(cmd.action, ActionType::Task);
            assert_eq!(cmd.content, "buy milk");
        }
    }

    // === is_simple_input Tests ===

    #[test]
    fn test_is_simple_input_true() {
        assert!(PatternMatcher::is_simple_input("add task"));
        assert!(PatternMatcher::is_simple_input("complete 5"));
        assert!(PatternMatcher::is_simple_input("list work"));
        assert!(PatternMatcher::is_simple_input("overdue"));
    }

    #[test]
    fn test_is_simple_input_false() {
        assert!(!PatternMatcher::is_simple_input("show me all tasks"));
        assert!(!PatternMatcher::is_simple_input("what should I do"));
        assert!(!PatternMatcher::is_simple_input("add something with deadline tomorrow"));
    }

    // === Stats Tests ===

    #[test]
    fn test_pattern_stats() {
        let stats = PatternMatcher::stats();
        assert!(stats.total_patterns > 0);
        assert!(!stats.patterns_checked.is_empty());
    }

    // === Complex Queries Need AI ===

    #[test]
    fn test_compound_query_needs_ai() {
        let result = PatternMatcher::match_input("complete 5 and update it");
        assert!(matches!(result, PatternMatch::NeedsAI));
    }

    #[test]
    fn test_recurring_task_needs_ai() {
        let result = PatternMatcher::match_input("add daily standup every day at 9am");
        assert!(matches!(result, PatternMatch::NeedsAI));
    }

    #[test]
    fn test_modification_needs_ai() {
        let result = PatternMatcher::match_input("change task 5 category to work");
        assert!(matches!(result, PatternMatch::NeedsAI));
    }
}
