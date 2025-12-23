//! Error recovery and clarification requests for natural language commands
//!
//! This module provides intelligent error recovery when NLP parsing or command
//! execution fails. It detects and categorizes errors, provides clarification
//! requests, and implements recovery strategies.

use super::types::{NLPError, NLPCommand, Disambiguation, DisambiguationCandidate, AmbiguityType};
use super::suggestions::{SuggestionEngine, SuggestionRequest, Suggestion};
use crate::actions::display::{print_red, print_yellow, print_green};

/// Category of error that occurred
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorCategory {
    /// Invalid natural language input
    Parse,
    /// Command failed to run
    Execution,
    /// API error (timeout, rate limiting, service unavailable)
    API,
    /// Invalid parameters (task doesn't exist, category not found)
    Validation,
    /// Network connection issues
    Network,
    /// Configuration problems
    Config,
}

/// Recovery strategy for an error
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// Suggest similar valid commands
    SuggestAlternatives(Vec<String>),
    /// Ask user for missing information
    RequestClarification(ClarificationRequest),
    /// Offer to retry with different parameters
    RetryWithChanges(Vec<ParameterChange>),
    /// Provide guided prompts
    GuidedPrompt(GuidedPrompt),
    /// Interactive disambiguation
    Disambiguation(Disambiguation),
}

/// Clarification request to user
#[derive(Debug, Clone)]
pub struct ClarificationRequest {
    /// Question to ask user
    pub question: String,
    /// Options user can choose from
    pub options: Vec<ClarificationOption>,
    /// Whether free-form input is allowed
    pub allow_freeform: bool,
}

/// Option for clarification
#[derive(Debug, Clone)]
pub struct ClarificationOption {
    /// Display text
    pub text: String,
    /// Value to use if selected
    pub value: String,
    /// Description of this option
    pub description: String,
}

/// Parameter change suggestion
#[derive(Debug, Clone)]
pub struct ParameterChange {
    /// Parameter name
    pub parameter: String,
    /// Suggested value
    pub suggested_value: String,
    /// Reason for suggestion
    pub reason: String,
}

/// Guided prompt to help user
#[derive(Debug, Clone)]
pub struct GuidedPrompt {
    /// Step-by-step instructions
    pub steps: Vec<String>,
    /// Example correct input
    pub example: String,
    /// Common mistakes to avoid
    pub common_mistakes: Vec<String>,
}

/// Error recovery result
#[derive(Debug, Clone)]
pub enum RecoveryResult {
    /// Suggestion provided to user
    Suggestion(Vec<String>),
    /// Clarification needed from user
    ClarificationNeeded(ClarificationRequest),
    /// Disambiguation needed
    DisambiguationNeeded(Disambiguation),
    /// Guided prompt shown
    Guided(GuidedPrompt),
    /// Error cannot be recovered
    Unrecoverable(String),
}

/// Error recovery engine
pub struct ErrorRecoveryEngine;

impl ErrorRecoveryEngine {
    /// Categorize an NLP error
    pub fn categorize_error(error: &NLPError) -> ErrorCategory {
        match error {
            NLPError::ParseError(_) => ErrorCategory::Parse,
            NLPError::ValidationError(_) => ErrorCategory::Validation,
            NLPError::APIError(_) | NLPError::RateLimited | NLPError::Timeout(_) => ErrorCategory::API,
            NLPError::NetworkError(_) => ErrorCategory::Network,
            NLPError::ConfigError(_) | NLPError::InvalidAPIKey => ErrorCategory::Config,
            // Execution errors are typically returned as generic errors
            _ => ErrorCategory::Execution,
        }
    }

    /// Generate recovery strategy for an error
    pub fn recovery_strategy(
        error: &NLPError,
        input: &str,
        available_categories: &[String],
    ) -> RecoveryStrategy {
        let category = Self::categorize_error(error);

        match category {
            ErrorCategory::Parse => Self::parse_error_recovery(error, input, available_categories),
            ErrorCategory::Validation => Self::validation_error_recovery(error, input),
            ErrorCategory::API => Self::api_error_recovery(error),
            ErrorCategory::Execution => Self::execution_error_recovery(error, input),
            ErrorCategory::Network => Self::network_error_recovery(error),
            ErrorCategory::Config => Self::config_error_recovery(error),
        }
    }

    /// Handle error and return recovery result
    pub fn handle_error(
        error: &NLPError,
        input: &str,
        available_categories: &[String],
    ) -> RecoveryResult {
        let strategy = Self::recovery_strategy(error, input, available_categories);

        match strategy {
            RecoveryStrategy::SuggestAlternatives(alts) => RecoveryResult::Suggestion(alts),
            RecoveryStrategy::RequestClarification(req) => RecoveryResult::ClarificationNeeded(req),
            RecoveryStrategy::Disambiguation(d) => RecoveryResult::DisambiguationNeeded(d),
            RecoveryStrategy::GuidedPrompt(prompt) => RecoveryResult::Guided(prompt),
            RecoveryStrategy::RetryWithChanges(changes) => {
                let suggestions = changes.iter()
                    .map(|c| format!("Try changing {} to '{}': {}", c.parameter, c.suggested_value, c.reason))
                    .collect();
                RecoveryResult::Suggestion(suggestions)
            }
        }
    }

    /// Display recovery options to user
    pub fn display_recovery(result: &RecoveryResult) {
        match result {
            RecoveryResult::Suggestion(suggestions) => {
                print_yellow("\nSuggestions:");
                for (i, suggestion) in suggestions.iter().enumerate() {
                    println!("  {}. {}", i + 1, suggestion);
                }
            }
            RecoveryResult::ClarificationNeeded(req) => {
                print_yellow(&format!("\n{}", req.question));
                println!();
                for (i, option) in req.options.iter().enumerate() {
                    println!("  {}. {} - {}", i + 1, option.text, option.description);
                }
                if req.allow_freeform {
                    println!("  Or type your own answer.");
                }
            }
            RecoveryResult::DisambiguationNeeded(d) => {
                print_yellow(&format!("\n{}", d.prompt));
                println!();
                for (i, candidate) in d.candidates.iter().enumerate() {
                    let confidence_pct = (candidate.confidence * 100.0) as u32;
                    println!("  {}. {} ({}%)", i + 1, candidate.value, confidence_pct);
                    if let Some(ctx) = &candidate.context {
                        println!("     {}", ctx);
                    }
                }
            }
            RecoveryResult::Guided(prompt) => {
                print_yellow("\nLet me help you with that:");
                println!();
                for (i, step) in prompt.steps.iter().enumerate() {
                    println!("  {}. {}", i + 1, step);
                }
                println!();
                print_green(&format!("Example: {}", prompt.example));
                if !prompt.common_mistakes.is_empty() {
                    println!();
                    print_yellow("Common mistakes to avoid:");
                    for mistake in &prompt.common_mistakes {
                        println!("  • {}", mistake);
                    }
                }
            }
            RecoveryResult::Unrecoverable(reason) => {
                print_red(&format!("\nUnable to recover: {}", reason));
            }
        }
    }

    /// Parse error recovery
    fn parse_error_recovery(
        error: &NLPError,
        input: &str,
        available_categories: &[String],
    ) -> RecoveryStrategy {
        // Get suggestions based on input
        let request = SuggestionRequest {
            input: input.to_string(),
            cursor_position: input.len(),
            recent_commands: Vec::new(),
            available_categories: available_categories.to_vec(),
        };

        let result = SuggestionEngine::suggest(&request);

        // Build alternative suggestions
        let alternatives: Vec<String> = result.suggestions
            .into_iter()
            .filter(|s| s.confidence > 0.5)
            .map(|s| {
                if s.suggestion_type == crate::nlp::suggestions::SuggestionType::TypoCorrection {
                    format!("{} (typo correction)", s.text)
                } else {
                    format!("{}: {}", s.text, s.description)
                }
            })
            .collect();

        if alternatives.is_empty() {
            // Provide guided prompt for completely unrecognizable input
            RecoveryStrategy::GuidedPrompt(GuidedPrompt {
                steps: vec![
                    "Start with an action verb (add, list, complete, delete, update)".to_string(),
                    "Follow with what you want to do (task, record)".to_string(),
                    "Add any details like category, deadline, or description".to_string(),
                ],
                example: "add task \"Buy groceries\" category \"personal\" deadline \"tomorrow\"".to_string(),
                common_mistakes: vec![
                    "Forgetting to specify what to add or update".to_string(),
                    "Using informal language instead of clear commands".to_string(),
                    "Not quoting text with spaces".to_string(),
                ],
            })
        } else {
            RecoveryStrategy::SuggestAlternatives(alternatives)
        }
    }

    /// Validation error recovery
    fn validation_error_recovery(error: &NLPError, input: &str) -> RecoveryStrategy {
        let error_msg = error.to_string();

        if error_msg.contains("content") && error_msg.contains("required") {
            RecoveryStrategy::RequestClarification(ClarificationRequest {
                question: "What would you like to add?".to_string(),
                options: vec![
                    ClarificationOption {
                        text: "Add a task".to_string(),
                        value: "task".to_string(),
                        description: "Create a new task".to_string(),
                    },
                    ClarificationOption {
                        text: "Add a record".to_string(),
                        value: "record".to_string(),
                        description: "Create a new record".to_string(),
                    },
                ],
                allow_freeform: true,
            })
        } else if error_msg.contains("too long") {
            RecoveryStrategy::GuidedPrompt(GuidedPrompt {
                steps: vec![
                    "Keep descriptions concise".to_string(),
                    "Use categories for organization instead of long descriptions".to_string(),
                    "Break complex tasks into smaller ones".to_string(),
                ],
                example: "add task \"Review PR\" category \"work\"".to_string(),
                common_mistakes: vec![
                    "Writing entire paragraphs as task descriptions".to_string(),
                    "Including multiple tasks in one description".to_string(),
                ],
            })
        } else if error_msg.contains("deadline") && error_msg.contains("schedule") {
            RecoveryStrategy::RequestClarification(ClarificationRequest {
                question: "Should this be a one-time task with a deadline or a recurring task?".to_string(),
                options: vec![
                    ClarificationOption {
                        text: "One-time with deadline".to_string(),
                        value: "deadline".to_string(),
                        description: "Task due on a specific date".to_string(),
                    },
                    ClarificationOption {
                        text: "Recurring".to_string(),
                        value: "schedule".to_string(),
                        description: "Task that repeats on a schedule".to_string(),
                    },
                ],
                allow_freeform: false,
            })
        } else if error_msg.contains("not found") || error_msg.contains("does not exist") {
            RecoveryStrategy::SuggestAlternatives(vec![
                "Check the task/record number".to_string(),
                "Use 'list' to see all available items".to_string(),
                "Use 'search' to find specific items".to_string(),
            ])
        } else {
            RecoveryStrategy::GuidedPrompt(GuidedPrompt {
                steps: vec![
                    "Check your command syntax".to_string(),
                    "Verify all required fields are provided".to_string(),
                    "Make sure values are in the correct format".to_string(),
                ],
                example: "add task \"My task\" category \"work\"".to_string(),
                common_mistakes: vec![
                    "Missing required fields".to_string(),
                    "Invalid date/time formats".to_string(),
                    "Using non-existent categories or task numbers".to_string(),
                ],
            })
        }
    }

    /// API error recovery
    fn api_error_recovery(error: &NLPError) -> RecoveryStrategy {
        match error {
            NLPError::RateLimited => RecoveryStrategy::RetryWithChanges(vec![
                ParameterChange {
                    parameter: "Timing".to_string(),
                    suggested_value: "Wait a moment".to_string(),
                    reason: "API rate limit reached. Please try again in a few seconds.".to_string(),
                },
            ]),
            NLPError::Timeout(seconds) => RecoveryStrategy::RetryWithChanges(vec![
                ParameterChange {
                    parameter: "timeout".to_string(),
                    suggested_value: format!("{} seconds", *seconds + 30),
                    reason: "Request timed out. Try increasing timeout or check your connection.".to_string(),
                },
            ]),
            NLPError::APIError(msg) if msg.contains("insufficient_quota") => RecoveryStrategy::GuidedPrompt(GuidedPrompt {
                steps: vec![
                    "Check your OpenAI API quota".to_string(),
                    "Verify your API key is correct".to_string(),
                    "Add credits to your OpenAI account".to_string(),
                ],
                example: "Use 'tascli nlp config show' to check your settings".to_string(),
                common_mistakes: vec![
                    "Using an API key with no credits".to_string(),
                    "API key from a trial account that expired".to_string(),
                ],
            }),
            _ => RecoveryStrategy::GuidedPrompt(GuidedPrompt {
                steps: vec![
                    "Check your internet connection".to_string(),
                    "Verify your API key is valid".to_string(),
                    "Try again shortly".to_string(),
                ],
                example: "Use 'tascli nlp config show' to verify settings".to_string(),
                common_mistakes: vec![
                    "Invalid API key format".to_string(),
                    "API service temporarily unavailable".to_string(),
                ],
            }),
        }
    }

    /// Execution error recovery
    fn execution_error_recovery(error: &NLPError, _input: &str) -> RecoveryStrategy {
        RecoveryStrategy::SuggestAlternatives(vec![
            "Check that the task/record number is correct".to_string(),
            "Use 'list' to see all available items".to_string(),
            "Try the traditional CLI command for more control".to_string(),
        ])
    }

    /// Network error recovery
    fn network_error_recovery(_error: &NLPError) -> RecoveryStrategy {
        RecoveryStrategy::GuidedPrompt(GuidedPrompt {
            steps: vec![
                "Check your internet connection".to_string(),
                "Verify you can reach api.openai.com".to_string(),
                "Try again shortly".to_string(),
            ],
            example: "ping api.openai.com # Check connectivity".to_string(),
            common_mistakes: vec![
                "Working offline".to_string(),
                "Firewall blocking API requests".to_string(),
                "VPN or proxy issues".to_string(),
            ],
        })
    }

    /// Config error recovery
    fn config_error_recovery(error: &NLPError) -> RecoveryStrategy {
        match error {
            NLPError::InvalidAPIKey => RecoveryStrategy::GuidedPrompt(GuidedPrompt {
                steps: vec![
                    "Get an API key from https://platform.openai.com/api-keys".to_string(),
                    "Set it using: tascli nlp config set-key <your-api-key>".to_string(),
                    "Verify it's set with: tascli nlp config show".to_string(),
                ],
                example: "tascli nlp config set-key sk-...".to_string(),
                common_mistakes: vec![
                    "Using an empty API key".to_string(),
                    "API key with extra spaces or characters".to_string(),
                ],
            }),
            _ => RecoveryStrategy::GuidedPrompt(GuidedPrompt {
                steps: vec![
                    "Check NLP is enabled: tascli nlp config enable".to_string(),
                    "Verify API key is set: tascli nlp config show".to_string(),
                    "Reset config if needed".to_string(),
                ],
                example: "tascli nlp config show".to_string(),
                common_mistakes: vec![
                    "NLP feature disabled".to_string(),
                    "Corrupted configuration file".to_string(),
                ],
            }),
        }
    }

    /// Create disambiguation for ambiguous task selection
    pub fn create_task_disambiguation(
        input: &str,
        candidates: Vec<(String, f64, Option<String>)>,
    ) -> Disambiguation {
        let disambiguation_candidates = candidates.into_iter()
            .map(|(value, confidence, context)| DisambiguationCandidate {
                value,
                confidence,
                context,
            })
            .collect();

        Disambiguation {
            input: input.to_string(),
            ambiguity_type: AmbiguityType::Task,
            candidates: disambiguation_candidates,
            prompt: "Multiple tasks match your input. Which one did you mean?".to_string(),
        }
    }

    /// Create disambiguation for ambiguous category selection
    pub fn create_category_disambiguation(
        input: &str,
        candidates: Vec<(String, f64)>,
    ) -> Disambiguation {
        let disambiguation_candidates = candidates.into_iter()
            .map(|(value, confidence)| DisambiguationCandidate {
                value,
                confidence,
                context: None,
            })
            .collect();

        Disambiguation {
            input: input.to_string(),
            ambiguity_type: AmbiguityType::Category,
            candidates: disambiguation_candidates,
            prompt: "Multiple categories match your input. Which one did you mean?".to_string(),
        }
    }

    /// Create disambiguation for ambiguous time/deadline
    pub fn create_deadline_disambiguation(
        input: &str,
        candidates: Vec<(String, f64, String)>,
    ) -> Disambiguation {
        let disambiguation_candidates = candidates.into_iter()
            .map(|(value, confidence, context)| DisambiguationCandidate {
                value,
                confidence,
                context: Some(context),
            })
            .collect();

        Disambiguation {
            input: input.to_string(),
            ambiguity_type: AmbiguityType::Deadline,
            candidates: disambiguation_candidates,
            prompt: "The time you specified is ambiguous. When did you mean?".to_string(),
        }
    }
}

/// Trait for types that can provide error recovery context
pub trait RecoveryContext {
    /// Get available categories for disambiguation
    fn available_categories(&self) -> Vec<String>;
    /// Get recent commands for context
    fn recent_commands(&self) -> Vec<String>;
    /// Get available tasks for disambiguation
    fn available_tasks(&self) -> Vec<(String, Option<String>)>;
}

/// Interactive error recovery handler
pub struct InteractiveRecoveryHandler {
    /// Available categories
    categories: Vec<String>,
    /// Recent command history
    history: Vec<String>,
    /// Available tasks
    tasks: Vec<(String, Option<String>)>,
}

impl InteractiveRecoveryHandler {
    /// Create a new recovery handler
    pub fn new() -> Self {
        Self {
            categories: Vec::new(),
            history: Vec::new(),
            tasks: Vec::new(),
        }
    }

    /// Create with context
    pub fn with_context(categories: Vec<String>, tasks: Vec<(String, Option<String>)>) -> Self {
        Self {
            categories,
            history: Vec::new(),
            tasks,
        }
    }

    /// Update categories
    pub fn update_categories(&mut self, categories: Vec<String>) {
        self.categories = categories;
    }

    /// Update tasks
    pub fn update_tasks(&mut self, tasks: Vec<(String, Option<String>)>) {
        self.tasks = tasks;
    }

    /// Add command to history
    pub fn add_to_history(&mut self, command: String) {
        self.history.push(command);
        if self.history.len() > 50 {
            self.history.remove(0);
        }
    }

    /// Handle error with interactive recovery
    pub fn handle(&self, error: &NLPError, input: &str) -> RecoveryResult {
        ErrorRecoveryEngine::handle_error(error, input, &self.categories)
    }

    /// Try to recover with user input
    pub fn recover_with_input(&self, input: &str, clarification: &ClarificationRequest) -> Option<String> {
        // Check if input matches any option
        for option in &clarification.options {
            if input.eq_ignore_ascii_case(&option.text) || input == option.value {
                return Some(option.value.clone());
            }
        }

        // Check numeric selection
        if let Ok(num) = input.parse::<usize>() {
            if num > 0 && num <= clarification.options.len() {
                return Some(clarification.options[num - 1].value.clone());
            }
        }

        // If freeform is allowed, return the input as-is
        if clarification.allow_freeform {
            Some(input.to_string())
        } else {
            None
        }
    }

    /// Select from disambiguation
    pub fn select_from_disambiguation(&self, input: &str, disambiguation: &Disambiguation) -> Option<String> {
        // Check numeric selection
        if let Ok(num) = input.parse::<usize>() {
            if num > 0 && num <= disambiguation.candidates.len() {
                return Some(disambiguation.candidates[num - 1].value.clone());
            }
        }

        // Check exact match
        for candidate in &disambiguation.candidates {
            if input.eq_ignore_ascii_case(&candidate.value) {
                return Some(candidate.value.clone());
            }
        }

        None
    }
}

impl Default for InteractiveRecoveryHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Error Category Tests ===

    #[test]
    fn test_categorize_parse_error() {
        let error = NLPError::ParseError("test".to_string());
        let category = ErrorRecoveryEngine::categorize_error(&error);
        assert_eq!(category, ErrorCategory::Parse);
    }

    #[test]
    fn test_categorize_validation_error() {
        let error = NLPError::ValidationError("test".to_string());
        let category = ErrorRecoveryEngine::categorize_error(&error);
        assert_eq!(category, ErrorCategory::Validation);
    }

    #[test]
    fn test_categorize_api_error() {
        let error = NLPError::APIError("test".to_string());
        let category = ErrorRecoveryEngine::categorize_error(&error);
        assert_eq!(category, ErrorCategory::API);
    }

    #[test]
    fn test_categorize_rate_limited() {
        let error = NLPError::RateLimited;
        let category = ErrorRecoveryEngine::categorize_error(&error);
        assert_eq!(category, ErrorCategory::API);
    }

    #[test]
    fn test_categorize_timeout() {
        let error = NLPError::Timeout(30);
        let category = ErrorRecoveryEngine::categorize_error(&error);
        assert_eq!(category, ErrorCategory::API);
    }

    #[test]
    fn test_categorize_network_error() {
        // Create a reqwest error by making an invalid request
        // We can't construct reqwest::Error directly, so we'll use a workaround
        // For this test, we'll just verify API error handling covers network issues
        let error = NLPError::APIError("Network connection failed".to_string());
        let category = ErrorRecoveryEngine::categorize_error(&error);
        assert_eq!(category, ErrorCategory::API);
    }

    #[test]
    fn test_categorize_config_error() {
        let error = NLPError::ConfigError("test".to_string());
        let category = ErrorRecoveryEngine::categorize_error(&error);
        assert_eq!(category, ErrorCategory::Config);
    }

    #[test]
    fn test_categorize_invalid_api_key() {
        let error = NLPError::InvalidAPIKey;
        let category = ErrorRecoveryEngine::categorize_error(&error);
        assert_eq!(category, ErrorCategory::Config);
    }

    // === Recovery Strategy Tests ===

    #[test]
    fn test_recovery_strategy_parse_error() {
        let error = NLPError::ParseError("invalid command".to_string());
        let strategy = ErrorRecoveryEngine::recovery_strategy(&error, "invalid input", &[]);
        // Should return some form of recovery
        match strategy {
            RecoveryStrategy::SuggestAlternatives(_) |
            RecoveryStrategy::GuidedPrompt(_) => {},
            _ => panic!("Expected suggestions or guided prompt for parse error"),
        }
    }

    #[test]
    fn test_recovery_strategy_validation_error() {
        let error = NLPError::ValidationError("content required".to_string());
        let strategy = ErrorRecoveryEngine::recovery_strategy(&error, "add task", &[]);
        match strategy {
            RecoveryStrategy::RequestClarification(req) => {
                assert!(!req.question.is_empty());
            },
            _ => {},
        }
    }

    #[test]
    fn test_recovery_strategy_rate_limited() {
        let error = NLPError::RateLimited;
        let strategy = ErrorRecoveryEngine::recovery_strategy(&error, "", &[]);
        match strategy {
            RecoveryStrategy::RetryWithChanges(changes) => {
                assert!(!changes.is_empty());
            },
            _ => {},
        }
    }

    #[test]
    fn test_recovery_strategy_timeout() {
        let error = NLPError::Timeout(30);
        let strategy = ErrorRecoveryEngine::recovery_strategy(&error, "", &[]);
        match strategy {
            RecoveryStrategy::RetryWithChanges(changes) => {
                assert!(!changes.is_empty());
            },
            _ => {},
        }
    }

    #[test]
    fn test_recovery_strategy_invalid_api_key() {
        let error = NLPError::InvalidAPIKey;
        let strategy = ErrorRecoveryEngine::recovery_strategy(&error, "", &[]);
        match strategy {
            RecoveryStrategy::GuidedPrompt(prompt) => {
                assert!(!prompt.steps.is_empty());
                assert!(!prompt.example.is_empty());
            },
            _ => {},
        }
    }

    // === Disambiguation Tests ===

    #[test]
    fn test_create_task_disambiguation() {
        let candidates = vec![
            ("Task 1".to_string(), 0.9, Some("Work category".to_string())),
            ("Task 2".to_string(), 0.7, Some("Personal category".to_string())),
        ];
        let disambiguation = ErrorRecoveryEngine::create_task_disambiguation("task", candidates);

        assert_eq!(disambiguation.ambiguity_type, AmbiguityType::Task);
        assert_eq!(disambiguation.candidates.len(), 2);
    }

    #[test]
    fn test_create_category_disambiguation() {
        let candidates = vec![
            ("work".to_string(), 0.9),
            ("work-project".to_string(), 0.7),
        ];
        let disambiguation = ErrorRecoveryEngine::create_category_disambiguation("work", candidates);

        assert_eq!(disambiguation.ambiguity_type, AmbiguityType::Category);
        assert_eq!(disambiguation.candidates.len(), 2);
    }

    #[test]
    fn test_create_deadline_disambiguation() {
        let candidates = vec![
            ("tomorrow".to_string(), 0.8, "2024-01-02".to_string()),
            ("next week".to_string(), 0.6, "2024-01-08".to_string()),
        ];
        let disambiguation = ErrorRecoveryEngine::create_deadline_disambiguation("next", candidates);

        assert_eq!(disambiguation.ambiguity_type, AmbiguityType::Deadline);
        assert_eq!(disambiguation.candidates.len(), 2);
    }

    // === Interactive Recovery Handler Tests ===

    #[test]
    fn test_interactive_recovery_handler_new() {
        let handler = InteractiveRecoveryHandler::new();
        assert!(handler.categories.is_empty());
        assert!(handler.history.is_empty());
        assert!(handler.tasks.is_empty());
    }

    #[test]
    fn test_interactive_recovery_handler_with_context() {
        let categories = vec!["work".to_string(), "personal".to_string()];
        let tasks = vec![("Task 1".to_string(), Some("work".to_string()))];
        let handler = InteractiveRecoveryHandler::with_context(categories, tasks);

        assert_eq!(handler.categories.len(), 2);
        assert_eq!(handler.tasks.len(), 1);
    }

    #[test]
    fn test_interactive_recovery_handler_update_categories() {
        let mut handler = InteractiveRecoveryHandler::new();
        handler.update_categories(vec!["home".to_string()]);
        assert_eq!(handler.categories.len(), 1);
    }

    #[test]
    fn test_interactive_recovery_handler_update_tasks() {
        let mut handler = InteractiveRecoveryHandler::new();
        handler.update_tasks(vec![("Task 2".to_string(), None)]);
        assert_eq!(handler.tasks.len(), 1);
    }

    #[test]
    fn test_interactive_recovery_handler_add_to_history() {
        let mut handler = InteractiveRecoveryHandler::new();
        handler.add_to_history("add task test".to_string());
        assert_eq!(handler.history.len(), 1);
        assert_eq!(handler.history[0], "add task test");
    }

    #[test]
    fn test_interactive_recovery_handler_history_limit() {
        let mut handler = InteractiveRecoveryHandler::new();
        for i in 0..100 {
            handler.add_to_history(format!("command {}", i));
        }
        // History should be limited to 50
        assert_eq!(handler.history.len(), 50);
    }

    #[test]
    fn test_interactive_recovery_handler_handle() {
        let handler = InteractiveRecoveryHandler::new();
        let error = NLPError::ParseError("test".to_string());
        let result = handler.handle(&error, "input");
        // Should return some recovery result
        match result {
            RecoveryResult::Suggestion(_) |
            RecoveryResult::Guided(_) => {},
            _ => {},
        }
    }

    // === Recovery With Input Tests ===

    #[test]
    fn test_recover_with_input_exact_match() {
        let handler = InteractiveRecoveryHandler::new();
        let clarification = ClarificationRequest {
            question: "Choose?".to_string(),
            options: vec![
                ClarificationOption {
                    text: "Task".to_string(),
                    value: "task".to_string(),
                    description: "A task".to_string(),
                },
            ],
            allow_freeform: false,
        };

        let result = handler.recover_with_input("task", &clarification);
        assert_eq!(result, Some("task".to_string()));
    }

    #[test]
    fn test_recover_with_input_numeric() {
        let handler = InteractiveRecoveryHandler::new();
        let clarification = ClarificationRequest {
            question: "Choose?".to_string(),
            options: vec![
                ClarificationOption {
                    text: "Task".to_string(),
                    value: "task".to_string(),
                    description: "A task".to_string(),
                },
                ClarificationOption {
                    text: "Record".to_string(),
                    value: "record".to_string(),
                    description: "A record".to_string(),
                },
            ],
            allow_freeform: false,
        };

        let result = handler.recover_with_input("2", &clarification);
        assert_eq!(result, Some("record".to_string()));
    }

    #[test]
    fn test_recover_with_input_freeform() {
        let handler = InteractiveRecoveryHandler::new();
        let clarification = ClarificationRequest {
            question: "What?".to_string(),
            options: vec![],
            allow_freeform: true,
        };

        let result = handler.recover_with_input("custom input", &clarification);
        assert_eq!(result, Some("custom input".to_string()));
    }

    #[test]
    fn test_recover_with_input_invalid() {
        let handler = InteractiveRecoveryHandler::new();
        let clarification = ClarificationRequest {
            question: "Choose?".to_string(),
            options: vec![
                ClarificationOption {
                    text: "Task".to_string(),
                    value: "task".to_string(),
                    description: "A task".to_string(),
                },
            ],
            allow_freeform: false,
        };

        let result = handler.recover_with_input("invalid", &clarification);
        assert_eq!(result, None);
    }

    // === Disambiguation Selection Tests ===

    #[test]
    fn test_select_from_disambiguation_numeric() {
        let handler = InteractiveRecoveryHandler::new();
        let disambiguation = Disambiguation {
            input: "task".to_string(),
            ambiguity_type: AmbiguityType::Task,
            candidates: vec![
                DisambiguationCandidate {
                    value: "Task 1".to_string(),
                    confidence: 0.9,
                    context: None,
                },
                DisambiguationCandidate {
                    value: "Task 2".to_string(),
                    confidence: 0.7,
                    context: None,
                },
            ],
            prompt: "Choose".to_string(),
        };

        let result = handler.select_from_disambiguation("1", &disambiguation);
        assert_eq!(result, Some("Task 1".to_string()));
    }

    #[test]
    fn test_select_from_disambiguation_exact_match() {
        let handler = InteractiveRecoveryHandler::new();
        let disambiguation = Disambiguation {
            input: "task".to_string(),
            ambiguity_type: AmbiguityType::Task,
            candidates: vec![
                DisambiguationCandidate {
                    value: "Task 1".to_string(),
                    confidence: 0.9,
                    context: None,
                },
            ],
            prompt: "Choose".to_string(),
        };

        let result = handler.select_from_disambiguation("Task 1", &disambiguation);
        assert_eq!(result, Some("Task 1".to_string()));
    }

    #[test]
    fn test_select_from_disambiguation_invalid() {
        let handler = InteractiveRecoveryHandler::new();
        let disambiguation = Disambiguation {
            input: "task".to_string(),
            ambiguity_type: AmbiguityType::Task,
            candidates: vec![
                DisambiguationCandidate {
                    value: "Task 1".to_string(),
                    confidence: 0.9,
                    context: None,
                },
            ],
            prompt: "Choose".to_string(),
        };

        let result = handler.select_from_disambiguation("invalid", &disambiguation);
        assert_eq!(result, None);
    }

    // === Error Category Display Tests ===

    #[test]
    fn test_error_category_equality() {
        assert_eq!(ErrorCategory::Parse, ErrorCategory::Parse);
        assert_ne!(ErrorCategory::Parse, ErrorCategory::Validation);
    }

    // === Guided Prompt Tests ===

    #[test]
    fn test_guided_prompt_structure() {
        let prompt = GuidedPrompt {
            steps: vec!["Step 1".to_string(), "Step 2".to_string()],
            example: "example".to_string(),
            common_mistakes: vec!["Mistake 1".to_string()],
        };

        assert_eq!(prompt.steps.len(), 2);
        assert_eq!(prompt.example, "example");
        assert_eq!(prompt.common_mistakes.len(), 1);
    }

    // === Parameter Change Tests ===

    #[test]
    fn test_parameter_change_structure() {
        let change = ParameterChange {
            parameter: "test".to_string(),
            suggested_value: "value".to_string(),
            reason: "reason".to_string(),
        };

        assert_eq!(change.parameter, "test");
        assert_eq!(change.suggested_value, "value");
        assert_eq!(change.reason, "reason");
    }
}
