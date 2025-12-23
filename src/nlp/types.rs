//! Types and structures for natural language processing

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

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

impl fmt::Display for ActionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActionType::Task => write!(f, "task"),
            ActionType::Record => write!(f, "record"),
            ActionType::Done => write!(f, "done"),
            ActionType::Update => write!(f, "update"),
            ActionType::Delete => write!(f, "delete"),
            ActionType::List => write!(f, "list"),
        }
    }
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

/// Conditional expressions for conditional command execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Condition {
    /// Single condition
    Single(Box<ConditionExpression>),
    /// Logical AND of conditions
    And(Vec<Condition>),
    /// Logical OR of conditions
    Or(Vec<Condition>),
    /// Logical NOT of condition
    Not(Box<Condition>),
}

/// Individual condition expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionExpression {
    /// Task exists with given content/pattern
    TaskExists { content: String },
    /// Task count matches criteria
    TaskCount { operator: ComparisonOperator, value: i32 },
    /// Category has tasks
    CategoryHasTasks { category: String },
    /// Category is empty
    CategoryEmpty { category: String },
    /// Previous command succeeded
    PreviousSuccess,
    /// Previous command failed
    PreviousFailed,
    /// Time-based condition
    TimeCondition { operator: ComparisonOperator, hour: Option<i32>, minute: Option<i32> },
    /// Day of week condition
    DayOfWeek { days: Vec<String> },
    /// Variable exists and equals value
    VariableEquals { name: String, value: String },
    /// Variable exists
    VariableExists { name: String },
}

/// Comparison operators for conditions
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterOrEqual,
    LessOrEqual,
}

/// Conditional branch for if-then-else execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalBranch {
    /// The condition to evaluate
    pub condition: Condition,
    /// Commands to execute if condition is true
    pub then_commands: Vec<NLPCommand>,
    /// Commands to execute if condition is false (optional)
    pub else_commands: Option<Vec<NLPCommand>>,
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
    /// For compound commands: additional commands to execute
    pub compound_commands: Option<Vec<NLPCommand>>,
    /// Conditional execution for this command
    pub condition: Option<Condition>,
    /// Confidence score for NLP interpretation (0.0 to 1.0)
    pub confidence: Option<f64>,
    /// Source of the command interpretation (pattern, ai, learning, personalization)
    pub interpretation_source: Option<String>,
}

/// Represents a compound command with multiple operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompoundCommand {
    /// The primary/first command
    pub primary: NLPCommand,
    /// Additional commands to execute
    pub secondary: Vec<NLPCommand>,
    /// How commands should be executed
    pub execution_mode: CompoundExecutionMode,
}

/// Execution mode for compound commands
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompoundExecutionMode {
    /// Execute all commands sequentially
    Sequential,
    /// Execute all commands in parallel (independent)
    Parallel,
    /// Execute with dependency resolution
    Dependent,
    /// Stop on first error
    StopOnError,
    /// Continue on error, collect all results
    ContinueOnError,
    /// Execute with conditional logic
    Conditional,
}

/// Execution result for a single command in a compound sequence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandExecutionResult {
    /// Index of the command in the sequence
    pub index: usize,
    /// Whether the command succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Output data from the command
    pub output: Option<CommandOutput>,
}

/// Output data from executed commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandOutput {
    /// ID of created/modified item
    pub item_id: Option<i64>,
    /// Content of the command
    pub content: String,
    /// Category if applicable
    pub category: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Execution context shared between sequential commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequentialContext {
    /// Results from previous commands
    pub previous_results: Vec<CommandExecutionResult>,
    /// Last created item ID
    pub last_item_id: Option<i64>,
    /// Last used category
    pub last_category: Option<String>,
    /// Last content
    pub last_content: Option<String>,
    /// Variables set during execution
    pub variables: HashMap<String, String>,
}

impl Default for SequentialContext {
    fn default() -> Self {
        Self {
            previous_results: Vec::new(),
            last_item_id: None,
            last_category: None,
            last_content: None,
            variables: HashMap::new(),
        }
    }
}

impl SequentialContext {
    /// Update context with execution result
    pub fn update_with_result(&mut self, result: &CommandExecutionResult) {
        self.previous_results.push(result.clone());
        if let Some(ref output) = result.output {
            if output.item_id.is_some() {
                self.last_item_id = output.item_id;
            }
            if output.category.is_some() {
                self.last_category = output.category.clone();
            }
            self.last_content = Some(output.content.clone());
        }
    }

    /// Get a variable value
    pub fn get_var(&self, key: &str) -> Option<&String> {
        self.variables.get(key)
    }

    /// Set a variable value
    pub fn set_var(&mut self, key: String, value: String) {
        self.variables.insert(key, value);
    }
}

/// Summary of compound command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSummary {
    /// Total commands executed
    pub total: usize,
    /// Successful commands
    pub successful: usize,
    /// Failed commands
    pub failed: usize,
    /// Individual results
    pub results: Vec<CommandExecutionResult>,
    /// Final context state
    pub final_context: SequentialContext,
}

impl ExecutionSummary {
    pub fn new(total: usize, results: Vec<CommandExecutionResult>, final_context: SequentialContext) -> Self {
        let successful = results.iter().filter(|r| r.success).count();
        let failed = results.iter().filter(|r| !r.success).count();

        Self {
            total,
            successful,
            failed,
            results,
            final_context,
        }
    }

    /// Whether all commands succeeded
    pub fn is_complete_success(&self) -> bool {
        self.failed == 0
    }

    /// Get human-readable summary
    pub fn to_summary_string(&self) -> String {
        if self.is_complete_success() {
            format!("All {} command(s) executed successfully", self.total)
        } else {
            format!("Executed {} command(s): {} succeeded, {} failed",
                self.total, self.successful, self.failed)
        }
    }
}

impl NLPCommand {
    /// Check if this command has compound commands
    pub fn is_compound(&self) -> bool {
        self.compound_commands.as_ref().map_or(false, |v| !v.is_empty())
    }

    /// Get all compound commands if present
    pub fn compound(&self) -> Option<&[NLPCommand]> {
        self.compound_commands.as_deref()
    }

    /// Convert to a CompoundCommand structure
    pub fn to_compound(self) -> Option<CompoundCommand> {
        if self.is_compound() {
            Some(CompoundCommand {
                primary: NLPCommand {
                    compound_commands: None,
                    ..self.clone()
                },
                secondary: self.compound_commands.unwrap_or_default(),
                execution_mode: CompoundExecutionMode::Sequential,
            })
        } else {
            None
        }
    }

    /// Add a compound command
    pub fn add_compound_command(&mut self, command: NLPCommand) {
        if self.compound_commands.is_none() {
            self.compound_commands = Some(Vec::new());
        }
        if let Some(ref mut commands) = self.compound_commands {
            commands.push(command);
        }
    }
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
            compound_commands: None,
            condition: None,
            confidence: None,
            interpretation_source: None,
        }
    }
}

impl Default for CompoundExecutionMode {
    fn default() -> Self {
        Self::Sequential
    }
}

impl CompoundCommand {
    /// Create a new compound command from a primary command
    pub fn new(primary: NLPCommand) -> Self {
        Self {
            primary,
            secondary: Vec::new(),
            execution_mode: CompoundExecutionMode::Sequential,
        }
    }

    /// Add a secondary command
    pub fn add_command(mut self, command: NLPCommand) -> Self {
        self.secondary.push(command);
        self
    }

    /// Set the execution mode
    pub fn with_execution_mode(mut self, mode: CompoundExecutionMode) -> Self {
        self.execution_mode = mode;
        self
    }

    /// Get all commands in execution order
    pub fn all_commands(&self) -> Vec<&NLPCommand> {
        let mut commands = vec![&self.primary];
        commands.extend(self.secondary.iter());
        commands
    }

    /// Check if this is a compound command (has secondary commands)
    pub fn is_compound(&self) -> bool {
        !self.secondary.is_empty()
    }

    /// Count total number of commands
    pub fn command_count(&self) -> usize {
        1 + self.secondary.len()
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
    /// API request timeout in seconds (default: 30)
    pub timeout_seconds: u64,
    /// Whether to show preview before executing commands
    pub preview_enabled: bool,
    /// Whether to auto-confirm preview without asking
    pub auto_confirm: bool,
    /// Whether to show NLP interpretation transparency
    pub show_transparency: bool,
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
            timeout_seconds: 30,
            preview_enabled: true,
            auto_confirm: false,
            show_transparency: true,
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

    #[error("Request timeout after {0} seconds")]
    Timeout(u64),
}

/// Disambiguation information for ambiguous inputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Disambiguation {
    /// The ambiguous input
    pub input: String,
    /// The type of ambiguity
    pub ambiguity_type: AmbiguityType,
    /// Possible matches with their scores
    pub candidates: Vec<DisambiguationCandidate>,
    /// A helpful message asking for clarification
    pub prompt: String,
}

/// Types of ambiguity that can occur
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AmbiguityType {
    /// Multiple categories match the input
    Category,
    /// Multiple tasks match the input
    Task,
    /// Time/deadline is ambiguous
    Deadline,
}

/// A candidate for disambiguation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisambiguationCandidate {
    /// The candidate value
    pub value: String,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Additional context about this candidate
    pub context: Option<String>,
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
        assert!(cmd.compound_commands.is_none());
        assert!(cmd.condition.is_none());
        assert!(cmd.confidence.is_none());
        assert!(cmd.interpretation_source.is_none());
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
            compound_commands: None,
            condition: None,
            confidence: None,
            interpretation_source: None,
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
        assert!(cmd.condition.is_none());
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
        assert_eq!(config.timeout_seconds, 30);
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
            timeout_seconds: 45,
            preview_enabled: false,
            auto_confirm: true,
            show_transparency: false,
        };

        assert!(config.enabled);
        assert_eq!(config.api_key, Some("sk-test-123".to_string()));
        assert_eq!(config.model, "gpt-4");
        assert!(!config.fallback_to_traditional);
        assert!(!config.cache_commands);
        assert_eq!(config.context_window, 20);
        assert_eq!(config.max_api_calls_per_minute, 100);
        assert_eq!(config.api_base_url, "https://custom.api.com/v1");
        assert_eq!(config.timeout_seconds, 45);
        assert!(!config.preview_enabled);
        assert!(config.auto_confirm);
        assert!(!config.show_transparency);
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
            show_transparency: true,
            ..Default::default()
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"gpt-4\""));
        assert!(json.contains("true"));
    }

    #[test]
    fn test_nlp_config_deserialize() {
        let json = r#"{"enabled":true,"api_key":"sk-test","model":"gpt-4","fallback_to_traditional":false,"cache_commands":false,"context_window":5,"max_api_calls_per_minute":10,"api_base_url":"https://api.test.com","timeout_seconds":60,"preview_enabled":true,"auto_confirm":false,"show_transparency":true}"#;
        let config: NLPConfig = serde_json::from_str(json).unwrap();

        assert!(config.enabled);
        assert_eq!(config.api_key, Some("sk-test".to_string()));
        assert_eq!(config.model, "gpt-4");
        assert!(!config.fallback_to_traditional);
        assert!(!config.cache_commands);
        assert_eq!(config.context_window, 5);
        assert_eq!(config.max_api_calls_per_minute, 10);
        assert_eq!(config.api_base_url, "https://api.test.com");
        assert_eq!(config.timeout_seconds, 60);
        assert!(config.preview_enabled);
        assert!(!config.auto_confirm);
        assert!(config.show_transparency);
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
    fn test_nlp_error_timeout() {
        let err = NLPError::Timeout(30);
        assert!(err.to_string().contains("timeout"));
        assert!(err.to_string().contains("30"));
        assert!(err.to_string().contains("seconds"));
    }

    #[test]
    fn test_nlp_error_timeout_zero() {
        let err = NLPError::Timeout(0);
        assert_eq!(err.to_string(), "Request timeout after 0 seconds");
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

    // === Compound Command Tests ===

    #[test]
    fn test_nlp_command_not_compound_by_default() {
        let cmd = NLPCommand::default();
        assert!(!cmd.is_compound());
        assert!(cmd.compound().is_none());
        assert!(cmd.to_compound().is_none());
    }

    #[test]
    fn test_nlp_command_is_compound() {
        let mut cmd = NLPCommand::default();
        cmd.add_compound_command(NLPCommand {
            action: ActionType::Done,
            content: "secondary task".to_string(),
            ..Default::default()
        });
        assert!(cmd.is_compound());
        assert!(cmd.compound().is_some());
        assert_eq!(cmd.compound().unwrap().len(), 1);
    }

    #[test]
    fn test_nlp_command_add_multiple_compound() {
        let mut cmd = NLPCommand {
            action: ActionType::Task,
            content: "primary task".to_string(),
            ..Default::default()
        };
        cmd.add_compound_command(NLPCommand {
            action: ActionType::Done,
            content: "task 2".to_string(),
            ..Default::default()
        });
        cmd.add_compound_command(NLPCommand {
            action: ActionType::Delete,
            content: "task 3".to_string(),
            ..Default::default()
        });
        assert!(cmd.is_compound());
        assert_eq!(cmd.compound().unwrap().len(), 2);
    }

    #[test]
    fn test_compound_execution_mode_default() {
        let mode = CompoundExecutionMode::default();
        assert_eq!(mode, CompoundExecutionMode::Sequential);
    }

    #[test]
    fn test_compound_execution_mode_equality() {
        assert_eq!(CompoundExecutionMode::Sequential, CompoundExecutionMode::Sequential);
        assert_ne!(CompoundExecutionMode::Sequential, CompoundExecutionMode::Parallel);
        assert_ne!(CompoundExecutionMode::Parallel, CompoundExecutionMode::Dependent);
    }

    #[test]
    fn test_compound_command_new() {
        let primary = NLPCommand {
            action: ActionType::Task,
            content: "main task".to_string(),
            ..Default::default()
        };
        let compound = CompoundCommand::new(primary.clone());
        assert!(!compound.is_compound());
        assert_eq!(compound.command_count(), 1);
        assert_eq!(compound.primary.action, ActionType::Task);
    }

    #[test]
    fn test_compound_command_add_secondary() {
        let primary = NLPCommand {
            action: ActionType::Task,
            content: "main task".to_string(),
            ..Default::default()
        };
        let compound = CompoundCommand::new(primary)
            .add_command(NLPCommand {
                action: ActionType::Done,
                content: "secondary".to_string(),
                ..Default::default()
            });
        assert!(compound.is_compound());
        assert_eq!(compound.command_count(), 2);
        assert_eq!(compound.secondary.len(), 1);
    }

    #[test]
    fn test_compound_command_all_commands() {
        let primary = NLPCommand {
            action: ActionType::Task,
            content: "main".to_string(),
            ..Default::default()
        };
        let compound = CompoundCommand::new(primary)
            .add_command(NLPCommand {
                action: ActionType::Done,
                content: "done cmd".to_string(),
                ..Default::default()
            })
            .add_command(NLPCommand {
                action: ActionType::List,
                content: "list cmd".to_string(),
                ..Default::default()
            });
        let all = compound.all_commands();
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].action, ActionType::Task);
        assert_eq!(all[1].action, ActionType::Done);
        assert_eq!(all[2].action, ActionType::List);
    }

    #[test]
    fn test_compound_command_with_execution_mode() {
        let primary = NLPCommand::default();
        let compound = CompoundCommand::new(primary)
            .with_execution_mode(CompoundExecutionMode::Parallel);
        assert_eq!(compound.execution_mode, CompoundExecutionMode::Parallel);
    }

    #[test]
    fn test_nlp_command_to_compound() {
        let mut cmd = NLPCommand {
            action: ActionType::Task,
            content: "primary".to_string(),
            ..Default::default()
        };
        cmd.add_compound_command(NLPCommand {
            action: ActionType::Done,
            content: "secondary".to_string(),
            ..Default::default()
        });

        let compound = cmd.to_compound();
        assert!(compound.is_some());
        let c = compound.unwrap();
        assert_eq!(c.primary.content, "primary");
        assert_eq!(c.secondary.len(), 1);
        assert_eq!(c.secondary[0].content, "secondary");
    }

    #[test]
    fn test_nlp_command_to_compound_when_not_compound() {
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "single".to_string(),
            ..Default::default()
        };
        assert!(cmd.to_compound().is_none());
    }

    // === Serialization for Compound Commands ===

    #[test]
    fn test_compound_execution_mode_serialize() {
        let mode = CompoundExecutionMode::Sequential;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"sequential\"");
    }

    #[test]
    fn test_compound_execution_mode_deserialize() {
        let json = "\"parallel\"";
        let mode: CompoundExecutionMode = serde_json::from_str(json).unwrap();
        assert_eq!(mode, CompoundExecutionMode::Parallel);
    }

    #[test]
    fn test_nlp_command_with_compound_serialize() {
        let mut cmd = NLPCommand {
            action: ActionType::Task,
            content: "main".to_string(),
            ..Default::default()
        };
        cmd.add_compound_command(NLPCommand {
            action: ActionType::Done,
            content: "secondary".to_string(),
            ..Default::default()
        });

        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("compound_commands"));
        assert!(json.contains("secondary"));
    }

    #[test]
    fn test_nlp_command_with_compound_deserialize() {
        let json = r#"{"action":"task","content":"main","category":null,"deadline":null,"schedule":null,"status":null,"query_type":null,"search":null,"filters":{},"modifications":{},"days":null,"limit":null,"compound_commands":[{"action":"done","content":"secondary","category":null,"deadline":null,"schedule":null,"status":null,"query_type":null,"search":null,"filters":{},"modifications":{},"days":null,"limit":null,"compound_commands":null}]}"#;
        let cmd: NLPCommand = serde_json::from_str(json).unwrap();
        assert!(cmd.is_compound());
        assert_eq!(cmd.compound().unwrap().len(), 1);
        assert_eq!(cmd.compound().unwrap()[0].action, ActionType::Done);
    }

    #[test]
    fn test_compound_command_serialize() {
        let primary = NLPCommand {
            action: ActionType::Task,
            content: "main".to_string(),
            ..Default::default()
        };
        let compound = CompoundCommand::new(primary)
            .add_command(NLPCommand {
                action: ActionType::Done,
                content: "done".to_string(),
                ..Default::default()
            });

        let json = serde_json::to_string(&compound).unwrap();
        assert!(json.contains("primary"));
        assert!(json.contains("secondary"));
        assert!(json.contains("execution_mode"));
    }

    #[test]
    fn test_compound_command_deserialize() {
        let json = r#"{"primary":{"action":"task","content":"main","category":null,"deadline":null,"schedule":null,"status":null,"query_type":null,"search":null,"filters":{},"modifications":{},"days":null,"limit":null,"compound_commands":null},"secondary":[{"action":"done","content":"done","category":null,"deadline":null,"schedule":null,"status":null,"query_type":null,"search":null,"filters":{},"modifications":{},"days":null,"limit":null,"compound_commands":null}],"execution_mode":"sequential"}"#;
        let compound: CompoundCommand = serde_json::from_str(json).unwrap();
        assert_eq!(compound.primary.content, "main");
        assert_eq!(compound.secondary.len(), 1);
        assert_eq!(compound.secondary[0].action, ActionType::Done);
        assert_eq!(compound.execution_mode, CompoundExecutionMode::Sequential);
    }
}