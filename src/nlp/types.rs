//! Types and structures for natural language processing

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the different types of actions tascli can perform
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionType {
    Task,
    Record,
    Done,
    Update,
    Delete,
    List,
}

/// Represents the different status options for items
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StatusType {
    Ongoing,
    Done,
    Cancelled,
    Duplicate,
    Suspended,
    Pending,
    Open,
    Closed,
    All,
}

/// Complex query types for advanced filtering
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QueryType {
    /// Tasks with deadlines in the past that are not done
    Overdue,
    /// Tasks with deadlines coming up soon
    Upcoming,
    /// Tasks with no deadline
    Unscheduled,
    /// Tasks due within a specific timeframe
    DueToday,
    DueTomorrow,
    DueThisWeek,
    DueThisMonth,
    /// High priority or urgent tasks
    Urgent,
    /// Tasks matching all specified criteria
    All,
}

/// Main structure for parsed natural language commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NLPCommand {
    /// The primary action to perform
    pub action: ActionType,
    /// Main content or description of the task/record
    pub content: String,
    /// Optional category for the item
    pub category: Option<String>,
    /// Optional deadline for tasks
    pub deadline: Option<String>,
    /// Optional schedule for recurring tasks
    pub schedule: Option<String>,
    /// Optional status filter for listing
    pub status: Option<StatusType>,
    /// Optional complex query type for advanced filtering
    pub query_type: Option<QueryType>,
    /// Optional search terms
    pub search: Option<String>,
    /// Additional filters
    pub filters: HashMap<String, String>,
    /// Modifications for update commands
    pub modifications: HashMap<String, String>,
    /// Days filter for listing (e.g., "7" for last 7 days)
    pub days: Option<i32>,
    /// Limit for listing results
    pub limit: Option<i32>,
}

impl Default for NLPCommand {
    fn default() -> Self {
        Self {
            action: ActionType::Task,
            content: String::new(),
            category: None,
            deadline: None,
            schedule: None,
            status: None,
            query_type: None,
            search: None,
            filters: HashMap::new(),
            modifications: HashMap::new(),
            days: None,
            limit: None,
        }
    }
}

/// Configuration for the NLP system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NLPConfig {
    /// Whether NLP is enabled
    pub enabled: bool,
    /// OpenAI API key
    pub api_key: Option<String>,
    /// Model to use (default: gpt-5-nano)
    pub model: String,
    /// Whether to fallback to traditional commands on error
    pub fallback_to_traditional: bool,
    /// Whether to cache command parses
    pub cache_commands: bool,
    /// Context window size for conversation
    pub context_window: usize,
    /// Maximum API calls per minute
    pub max_api_calls_per_minute: u32,
    /// API base URL (can be overridden for testing)
    pub api_base_url: String,
}

impl Default for NLPConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_key: None,
            model: "gpt-5-nano".to_string(),
            fallback_to_traditional: true,
            cache_commands: true,
            context_window: 10,
            max_api_calls_per_minute: 20,
            api_base_url: "https://api.openai.com/v1".to_string(),
        }
    }
}

/// Errors that can occur during NLP processing
#[derive(Debug, thiserror::Error)]
pub enum NLPError {
    #[error("API error: {0}")]
    APIError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Invalid API key")]
    InvalidAPIKey,

    #[error("Rate limited")]
    RateLimited,

    #[error("JSON serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Command validation failed: {0}")]
    ValidationError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Result type for NLP operations
pub type NLPResult<T> = Result<T, NLPError>;

#[cfg(test)]
mod tests {
    use super::*;

    // === ActionType Tests ===

    #[test]
    fn test_action_type_equality() {
        assert_eq!(ActionType::Task, ActionType::Task);
        assert_ne!(ActionType::Task, ActionType::Record);
    }

    #[test]
    fn test_action_type_clone() {
        let action = ActionType::List;
        let cloned = action.clone();
        assert_eq!(action, cloned);
    }

    #[test]
    fn test_action_type_all_variants() {
        let actions = vec![
            ActionType::Task,
            ActionType::Record,
            ActionType::Done,
            ActionType::Update,
            ActionType::Delete,
            ActionType::List,
        ];
        assert_eq!(actions.len(), 6);
    }

    // === QueryType Tests ===

    #[test]
    fn test_query_type_overdue() {
        let qt = QueryType::Overdue;
        assert_eq!(qt, QueryType::Overdue);
    }

    #[test]
    fn test_query_type_all_variants() {
        let types = vec![
            QueryType::Overdue,
            QueryType::Upcoming,
            QueryType::Unscheduled,
            QueryType::DueToday,
            QueryType::DueTomorrow,
            QueryType::DueThisWeek,
            QueryType::DueThisMonth,
            QueryType::Urgent,
            QueryType::All,
        ];
        assert_eq!(types.len(), 9);
    }

    // === StatusType Tests ===

    #[test]
    fn test_status_type_equality() {
        assert_eq!(StatusType::Ongoing, StatusType::Ongoing);
        assert_ne!(StatusType::Done, StatusType::Cancelled);
    }

    #[test]
    fn test_status_type_clone() {
        let status = StatusType::Pending;
        let cloned = status.clone();
        assert_eq!(status, cloned);
    }

    #[test]
    fn test_status_type_all_variants() {
        let statuses = vec![
            StatusType::Ongoing,
            StatusType::Done,
            StatusType::Cancelled,
            StatusType::Duplicate,
            StatusType::Suspended,
            StatusType::Pending,
            StatusType::Open,
            StatusType::Closed,
            StatusType::All,
        ];
        assert_eq!(statuses.len(), 9);
    }

    // === NLPCommand Tests ===

    #[test]
    fn test_nlp_command_default() {
        let cmd = NLPCommand::default();
        assert_eq!(cmd.action, ActionType::Task);
        assert_eq!(cmd.content, "");
        assert!(cmd.category.is_none());
        assert!(cmd.deadline.is_none());
        assert!(cmd.schedule.is_none());
        assert!(cmd.status.is_none());
        assert!(cmd.query_type.is_none());
        assert!(cmd.search.is_none());
        assert!(cmd.filters.is_empty());
        assert!(cmd.modifications.is_empty());
        assert!(cmd.days.is_none());
        assert!(cmd.limit.is_none());
    }

    #[test]
    fn test_nlp_command_clone() {
        let mut cmd = NLPCommand::default();
        cmd.content = "test task".to_string();
        cmd.category = Some("work".to_string());

        let cloned = cmd.clone();
        assert_eq!(cmd.content, cloned.content);
        assert_eq!(cmd.category, cloned.category);
    }

    #[test]
    fn test_nlp_command_with_all_fields() {
        let mut filters = HashMap::new();
        filters.insert("key1".to_string(), "value1".to_string());

        let mut modifications = HashMap::new();
        modifications.insert("content".to_string(), "new content".to_string());

        let cmd = NLPCommand {
            action: ActionType::Update,
            content: "original task".to_string(),
            category: Some("urgent".to_string()),
            deadline: Some("tomorrow".to_string()),
            schedule: None,
            status: Some(StatusType::Ongoing),
            query_type: Some(QueryType::Overdue),
            search: Some("keyword".to_string()),
            filters,
            modifications,
            days: Some(7),
            limit: Some(10),
        };

        assert_eq!(cmd.action, ActionType::Update);
        assert_eq!(cmd.content, "original task");
        assert_eq!(cmd.category, Some("urgent".to_string()));
        assert_eq!(cmd.deadline, Some("tomorrow".to_string()));
        assert_eq!(cmd.status, Some(StatusType::Ongoing));
        assert_eq!(cmd.query_type, Some(QueryType::Overdue));
        assert_eq!(cmd.search, Some("keyword".to_string()));
        assert_eq!(cmd.filters.len(), 1);
        assert_eq!(cmd.modifications.len(), 1);
        assert_eq!(cmd.days, Some(7));
        assert_eq!(cmd.limit, Some(10));
    }

    #[test]
    fn test_nlp_command_minimal() {
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "buy groceries".to_string(),
            ..Default::default()
        };

        assert_eq!(cmd.action, ActionType::Task);
        assert_eq!(cmd.content, "buy groceries");
    }

    #[test]
    fn test_nlp_command_with_filters() {
        let mut cmd = NLPCommand::default();
        cmd.filters.insert("priority".to_string(), "high".to_string());
        cmd.filters.insert("assignee".to_string(), "john".to_string());

        assert_eq!(cmd.filters.len(), 2);
        assert_eq!(cmd.filters.get("priority"), Some(&"high".to_string()));
    }

    #[test]
    fn test_nlp_command_with_modifications() {
        let mut cmd = NLPCommand::default();
        cmd.modifications.insert("content".to_string(), "updated".to_string());
        cmd.modifications.insert("category".to_string(), "work".to_string());

        assert_eq!(cmd.modifications.len(), 2);
        assert_eq!(cmd.modifications.get("content"), Some(&"updated".to_string()));
    }

    // === NLPConfig Tests ===

    #[test]
    fn test_nlp_config_default() {
        let config = NLPConfig::default();
        assert!(!config.enabled);
        assert!(config.api_key.is_none());
        assert_eq!(config.model, "gpt-5-nano");
        assert!(config.fallback_to_traditional);
        assert!(config.cache_commands);
        assert_eq!(config.context_window, 10);
        assert_eq!(config.max_api_calls_per_minute, 20);
        assert_eq!(config.api_base_url, "https://api.openai.com/v1");
    }

    #[test]
    fn test_nlp_config_clone() {
        let config = NLPConfig {
            enabled: true,
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };

        let cloned = config.clone();
        assert_eq!(config.enabled, cloned.enabled);
        assert_eq!(config.api_key, cloned.api_key);
    }

    #[test]
    fn test_nlp_config_custom() {
        let config = NLPConfig {
            enabled: true,
            api_key: Some("sk-test-123".to_string()),
            model: "gpt-4".to_string(),
            fallback_to_traditional: false,
            cache_commands: false,
            context_window: 20,
            max_api_calls_per_minute: 100,
            api_base_url: "https://custom.api.com/v1".to_string(),
        };

        assert!(config.enabled);
        assert_eq!(config.api_key, Some("sk-test-123".to_string()));
        assert_eq!(config.model, "gpt-4");
        assert!(!config.fallback_to_traditional);
        assert!(!config.cache_commands);
        assert_eq!(config.context_window, 20);
        assert_eq!(config.max_api_calls_per_minute, 100);
        assert_eq!(config.api_base_url, "https://custom.api.com/v1");
    }

    // === NLPError Tests ===

    #[test]
    fn test_nlp_error_api_error() {
        let err = NLPError::APIError("Something went wrong".to_string());
        assert!(err.to_string().contains("API error"));
        assert!(err.to_string().contains("Something went wrong"));
    }

    #[test]
    fn test_nlp_error_parse_error() {
        let err = NLPError::ParseError("Invalid JSON".to_string());
        assert!(err.to_string().contains("Parse error"));
        assert!(err.to_string().contains("Invalid JSON"));
    }

    #[test]
    fn test_nlp_error_invalid_api_key() {
        let err = NLPError::InvalidAPIKey;
        assert_eq!(err.to_string(), "Invalid API key");
    }

    #[test]
    fn test_nlp_error_rate_limited() {
        let err = NLPError::RateLimited;
        assert_eq!(err.to_string(), "Rate limited");
    }

    #[test]
    fn test_nlp_error_validation_error() {
        let err = NLPError::ValidationError("Invalid input".to_string());
        assert!(err.to_string().contains("Command validation failed"));
        assert!(err.to_string().contains("Invalid input"));
    }

    #[test]
    fn test_nlp_error_config_error() {
        let err = NLPError::ConfigError("Missing API key".to_string());
        assert!(err.to_string().contains("Configuration error"));
        assert!(err.to_string().contains("Missing API key"));
    }

    // === NLPResult Tests ===

    #[test]
    fn test_nlp_result_ok() {
        let result: NLPResult<String> = Ok("success".to_string());
        assert!(result.is_ok());
        assert!(!result.is_err());
        assert_eq!(result.unwrap(), "success");
    }

    #[test]
    fn test_nlp_result_err() {
        let result: NLPResult<String> = Err(NLPError::InvalidAPIKey);
        assert!(result.is_err());
        assert!(!result.is_ok());
        assert!(matches!(result.unwrap_err(), NLPError::InvalidAPIKey));
    }

    #[test]
    fn test_nlp_result_with_command() {
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "test".to_string(),
            ..Default::default()
        };
        let result: NLPResult<NLPCommand> = Ok(cmd.clone());
        assert!(result.is_ok());
        let unwrapped = result.unwrap();
        assert_eq!(unwrapped.action, ActionType::Task);
        assert_eq!(unwrapped.content, "test");
    }

    // === Serialization Tests ===

    #[test]
    fn test_action_type_serialize() {
        let action = ActionType::Task;
        let json = serde_json::to_string(&action).unwrap();
        assert_eq!(json, "\"task\"");
    }

    #[test]
    fn test_action_type_deserialize() {
        let json = "\"task\"";
        let action: ActionType = serde_json::from_str(json).unwrap();
        assert_eq!(action, ActionType::Task);
    }

    #[test]
    fn test_action_type_deserialize_all() {
        let task: ActionType = serde_json::from_str("\"task\"").unwrap();
        let record: ActionType = serde_json::from_str("\"record\"").unwrap();
        let done: ActionType = serde_json::from_str("\"done\"").unwrap();
        let update: ActionType = serde_json::from_str("\"update\"").unwrap();
        let delete: ActionType = serde_json::from_str("\"delete\"").unwrap();
        let list: ActionType = serde_json::from_str("\"list\"").unwrap();

        assert_eq!(task, ActionType::Task);
        assert_eq!(record, ActionType::Record);
        assert_eq!(done, ActionType::Done);
        assert_eq!(update, ActionType::Update);
        assert_eq!(delete, ActionType::Delete);
        assert_eq!(list, ActionType::List);
    }

    #[test]
    fn test_status_type_serialize() {
        let status = StatusType::Ongoing;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"ongoing\"");
    }

    #[test]
    fn test_status_type_deserialize() {
        let json = "\"done\"";
        let status: StatusType = serde_json::from_str(json).unwrap();
        assert_eq!(status, StatusType::Done);
    }

    #[test]
    fn test_nlp_command_serialize() {
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "buy groceries".to_string(),
            ..Default::default()
        };

        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("\"task\""));
        assert!(json.contains("buy groceries"));
    }

    #[test]
    fn test_nlp_command_deserialize() {
        let json = r#"{"action":"task","content":"test","category":"work","deadline":"today","filters":{},"modifications":{},"search":null,"status":null,"schedule":null,"days":null,"limit":null}"#;
        let cmd: NLPCommand = serde_json::from_str(json).unwrap();

        assert_eq!(cmd.action, ActionType::Task);
        assert_eq!(cmd.content, "test");
        assert_eq!(cmd.category, Some("work".to_string()));
        assert_eq!(cmd.deadline, Some("today".to_string()));
    }

    #[test]
    fn test_nlp_config_serialize() {
        let config = NLPConfig {
            enabled: true,
            model: "gpt-4".to_string(),
            ..Default::default()
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"gpt-4\""));
        assert!(json.contains("true"));
    }

    #[test]
    fn test_nlp_config_deserialize() {
        let json = r#"{"enabled":true,"api_key":"sk-test","model":"gpt-4","fallback_to_traditional":false,"cache_commands":false,"context_window":5,"max_api_calls_per_minute":10,"api_base_url":"https://api.test.com"}"#;
        let config: NLPConfig = serde_json::from_str(json).unwrap();

        assert!(config.enabled);
        assert_eq!(config.api_key, Some("sk-test".to_string()));
        assert_eq!(config.model, "gpt-4");
        assert!(!config.fallback_to_traditional);
        assert!(!config.cache_commands);
        assert_eq!(config.context_window, 5);
        assert_eq!(config.max_api_calls_per_minute, 10);
        assert_eq!(config.api_base_url, "https://api.test.com");
    }

    // === Edge Cases ===

    #[test]
    fn test_nlp_command_empty_content() {
        let cmd = NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            ..Default::default()
        };
        assert_eq!(cmd.content, "");
    }

    #[test]
    fn test_nlp_command_unicode_content() {
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "Task with emoji 🎉 and unicode 日本語".to_string(),
            ..Default::default()
        };
        assert!(cmd.content.contains("🎉"));
        assert!(cmd.content.contains("日本語"));
    }

    #[test]
    fn test_nlp_command_very_long_content() {
        let long_content = "a".repeat(1000);
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: long_content.clone(),
            ..Default::default()
        };
        assert_eq!(cmd.content.len(), 1000);
    }

    #[test]
    fn test_nlp_config_empty_api_key() {
        let config = NLPConfig {
            api_key: Some("".to_string()),
            ..Default::default()
        };
        assert_eq!(config.api_key, Some("".to_string()));
    }

    #[test]
    fn test_nlp_config_zero_max_api_calls() {
        let config = NLPConfig {
            max_api_calls_per_minute: 0,
            ..Default::default()
        };
        assert_eq!(config.max_api_calls_per_minute, 0);
    }

    #[test]
    fn test_nlp_config_large_context_window() {
        let config = NLPConfig {
            context_window: 100000,
            ..Default::default()
        };
        assert_eq!(config.context_window, 100000);
    }

    #[test]
    fn test_nlp_result_error_display() {
        let err = NLPError::APIError("Test error".to_string());
        assert_eq!(format!("{}", err), "API error: Test error");
    }

    #[test]
    fn test_nlp_command_debug_format() {
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "test".to_string(),
            ..Default::default()
        };
        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("Task"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_nlp_config_debug_format() {
        let config = NLPConfig {
            enabled: true,
            ..Default::default()
        };
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("enabled"));
        assert!(debug_str.contains("true"));
    }

    // === HashMap field tests ===

    #[test]
    fn test_nlp_command_filters_multiple_entries() {
        let mut cmd = NLPCommand::default();
        cmd.filters.insert("a".to_string(), "1".to_string());
        cmd.filters.insert("b".to_string(), "2".to_string());
        cmd.filters.insert("c".to_string(), "3".to_string());

        assert_eq!(cmd.filters.len(), 3);
    }

    #[test]
    fn test_nlp_command_modifications_multiple_entries() {
        let mut cmd = NLPCommand::default();
        cmd.modifications.insert("content".to_string(), "new".to_string());
        cmd.modifications.insert("deadline".to_string(), "today".to_string());

        assert_eq!(cmd.modifications.len(), 2);
    }

    #[test]
    fn test_nlp_command_with_negative_days() {
        let cmd = NLPCommand {
            days: Some(-7),
            ..Default::default()
        };
        assert_eq!(cmd.days, Some(-7));
    }

    #[test]
    fn test_nlp_command_with_zero_limit() {
        let cmd = NLPCommand {
            limit: Some(0),
            ..Default::default()
        };
        assert_eq!(cmd.limit, Some(0));
    }
}