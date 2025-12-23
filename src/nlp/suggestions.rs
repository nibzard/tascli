//! Auto-completion and suggestions for natural language commands
//!
//! This module provides intelligent suggestions for partial natural language input,
//! helping users discover available commands and complete their input faster.

use super::types::{NLPCommand, ActionType, StatusType, QueryType};
use super::pattern_matcher::{PatternMatcher, PatternMatch};

/// A suggestion for completing or improving user input
#[derive(Debug, Clone)]
pub struct Suggestion {
    /// The suggested text
    pub text: String,
    /// Type of suggestion
    pub suggestion_type: SuggestionType,
    /// How confident we are this is correct (0.0 to 1.0)
    pub confidence: f64,
    /// Description of what this suggestion does
    pub description: String,
}

/// Types of suggestions we can provide
#[derive(Debug, Clone, PartialEq)]
pub enum SuggestionType {
    /// Complete a partial command
    CommandCompletion,
    /// Suggest a similar command
    SimilarCommand,
    /// Correct a potential typo
    TypoCorrection,
    /// Suggest available options
    AvailableOption,
    /// Context-aware suggestion based on previous commands
    Contextual,
}

/// Suggestion request with context
#[derive(Debug, Clone)]
pub struct SuggestionRequest {
    /// The current partial input
    pub input: String,
    /// Cursor position in input
    pub cursor_position: usize,
    /// Recent commands for context
    pub recent_commands: Vec<String>,
    /// Available categories in database
    pub available_categories: Vec<String>,
}

/// Result of a suggestion request
#[derive(Debug, Clone)]
pub struct SuggestionResult {
    /// Suggestions for completing input
    pub suggestions: Vec<Suggestion>,
    /// Whether input is recognized as valid
    pub is_valid: bool,
    /// Parsed command if input is complete
    pub parsed_command: Option<NLPCommand>,
}

/// Suggestion engine using pattern matching and context
pub struct SuggestionEngine;

impl SuggestionEngine {
    /// Get suggestions for partial input
    pub fn suggest(request: &SuggestionRequest) -> SuggestionResult {
        let input = request.input.trim();
        let mut suggestions = Vec::new();

        // Check if input is already a complete valid command
        let pattern_match = PatternMatcher::match_input(input);
        let is_valid = matches!(pattern_match, PatternMatch::Matched(_));

        let parsed_command = match pattern_match {
            PatternMatch::Matched(cmd) => Some(cmd),
            _ => None,
        };

        // Empty input - suggest common commands
        if input.is_empty() {
            suggestions = Self::common_commands_suggestions();
            // Add contextual suggestions for empty input too
            suggestions.extend(Self::contextual_suggestions(input, &request.recent_commands));
        } else {
            // Get suggestions based on input
            suggestions.extend(Self::completion_suggestions(input, &request.available_categories));
            suggestions.extend(Self::typo_correction_suggestions(input));
            suggestions.extend(Self::contextual_suggestions(input, &request.recent_commands));
        }

        // Sort by confidence and limit results
        suggestions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        suggestions.truncate(8);

        SuggestionResult {
            suggestions,
            is_valid,
            parsed_command,
        }
    }

    /// Common commands to suggest for empty input
    fn common_commands_suggestions() -> Vec<Suggestion> {
        vec![
            Suggestion {
                text: "add task ".to_string(),
                suggestion_type: SuggestionType::CommandCompletion,
                confidence: 1.0,
                description: "Add a new task".to_string(),
            },
            Suggestion {
                text: "list".to_string(),
                suggestion_type: SuggestionType::CommandCompletion,
                confidence: 0.95,
                description: "List all tasks".to_string(),
            },
            Suggestion {
                text: "done ".to_string(),
                suggestion_type: SuggestionType::CommandCompletion,
                confidence: 0.90,
                description: "Mark a task as complete".to_string(),
            },
            Suggestion {
                text: "delete ".to_string(),
                suggestion_type: SuggestionType::CommandCompletion,
                confidence: 0.85,
                description: "Delete a task".to_string(),
            },
            Suggestion {
                text: "overdue".to_string(),
                suggestion_type: SuggestionType::CommandCompletion,
                confidence: 0.80,
                description: "Show overdue tasks".to_string(),
            },
            Suggestion {
                text: "due today".to_string(),
                suggestion_type: SuggestionType::CommandCompletion,
                confidence: 0.75,
                description: "Show tasks due today".to_string(),
            },
        ]
    }

    /// Suggestions for completing partial input
    fn completion_suggestions(input: &str, categories: &[String]) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();
        let input_lower = input.to_lowercase();

        // Task addition patterns
        if input_lower.starts_with("add") || input_lower.starts_with("task") {
            suggestions.push(Suggestion {
                text: format!("{} ", input.trim()),
                suggestion_type: SuggestionType::CommandCompletion,
                confidence: 0.9,
                description: "Adding a task/record".to_string(),
            });
        }

        // Complete/done patterns
        if input_lower.starts_with("com") {
            suggestions.push(Suggestion {
                text: "complete ".to_string(),
                suggestion_type: SuggestionType::CommandCompletion,
                confidence: 0.95,
                description: "Complete a task by number".to_string(),
            });
        }

        // List patterns
        if input_lower.starts_with("li") || input_lower.starts_with("sh") {
            suggestions.push(Suggestion {
                text: "list".to_string(),
                suggestion_type: SuggestionType::CommandCompletion,
                confidence: 0.95,
                description: "List all tasks".to_string(),
            });
            suggestions.push(Suggestion {
                text: "list work tasks".to_string(),
                suggestion_type: SuggestionType::CommandCompletion,
                confidence: 0.80,
                description: "List tasks by category".to_string(),
            });
            suggestions.push(Suggestion {
                text: "list done tasks".to_string(),
                suggestion_type: SuggestionType::CommandCompletion,
                confidence: 0.80,
                description: "List completed tasks".to_string(),
            });
        }

        // Update patterns
        if input_lower.starts_with("up") || input_lower.starts_with("ed") {
            suggestions.push(Suggestion {
                text: "update ".to_string(),
                suggestion_type: SuggestionType::CommandCompletion,
                confidence: 0.95,
                description: "Update a task by number".to_string(),
            });
        }

        // Query type suggestions
        if input_lower.starts_with("over") {
            suggestions.push(Suggestion {
                text: "overdue".to_string(),
                suggestion_type: SuggestionType::CommandCompletion,
                confidence: 0.95,
                description: "Show overdue tasks".to_string(),
            });
        }

        if input_lower.starts_with("upc") {
            suggestions.push(Suggestion {
                text: "upcoming".to_string(),
                suggestion_type: SuggestionType::CommandCompletion,
                confidence: 0.95,
                description: "Show upcoming tasks".to_string(),
            });
        }

        // Category-based suggestions
        if !categories.is_empty() {
            for category in categories.iter().take(3) {
                if input_lower.starts_with("list") || input_lower.starts_with("show") {
                    suggestions.push(Suggestion {
                        text: format!("list {} tasks", category),
                        suggestion_type: SuggestionType::CommandCompletion,
                        confidence: 0.70,
                        description: format!("List {} tasks", category),
                    });
                }
            }
        }

        suggestions
    }

    /// Correct common typos
    fn typo_correction_suggestions(input: &str) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();
        let input_lower = input.to_lowercase();

        // Common typo corrections
        let corrections: &[(&str, &str, f64)] = &[
            ("ad", "add ", 0.9),
            ("complet", "complete ", 0.9),
            ("delet", "delete ", 0.9),
            ("updte", "update ", 0.85),
            ("lis", "list", 0.9),
            ("shwo", "show", 0.85),
            ("don", "done ", 0.85),
            ("ta sk", "task ", 0.8),
            ("recrd", "record ", 0.8),
        ];

        for (typo, correction, confidence) in corrections {
            if input_lower.starts_with(typo) {
                suggestions.push(Suggestion {
                    text: format!("{}{}", correction, &input[typo.len()..]),
                    suggestion_type: SuggestionType::TypoCorrection,
                    confidence: *confidence,
                    description: format!("Did you mean '{}'?", correction.trim()),
                });
            }
        }

        suggestions
    }

    /// Context-aware suggestions based on recent commands
    fn contextual_suggestions(input: &str, recent_commands: &[String]) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();
        let input_lower = input.to_lowercase();

        // If user just added a task, suggest completing it
        for recent in recent_commands.iter().take(5) {
            let recent_lower = recent.to_lowercase();
            if recent_lower.starts_with("add task") || recent_lower.starts_with("task ") {
                // Extract task number if available
                if input_lower.starts_with("done") || input_lower.starts_with("complete") {
                    suggestions.push(Suggestion {
                        text: format!("complete 1"),
                        suggestion_type: SuggestionType::Contextual,
                        confidence: 0.75,
                        description: "Complete the most recent task".to_string(),
                    });
                }
            }
        }

        // If user just listed tasks, suggest operations on them
        if recent_commands.iter().any(|c| c.starts_with("list")) {
            if input.is_empty() || input_lower.starts_with("d") {
                suggestions.push(Suggestion {
                    text: "done ".to_string(),
                    suggestion_type: SuggestionType::Contextual,
                    confidence: 0.70,
                    description: "Mark a task as complete".to_string(),
                });
            }
        }

        suggestions
    }

    /// Get available command patterns for help
    pub fn command_patterns() -> Vec<(&'static str, &'static str)> {
        vec![
            ("add task <description>", "Add a new task"),
            ("add record <description>", "Add a new record"),
            ("complete <number>", "Mark task as complete"),
            ("done <number>", "Mark task as complete"),
            ("delete <number>", "Delete a task"),
            ("list", "List all tasks"),
            ("list <category> tasks", "List tasks by category"),
            ("list done tasks", "List completed tasks"),
            ("list pending tasks", "List pending tasks"),
            ("overdue", "Show overdue tasks"),
            ("upcoming", "Show upcoming tasks"),
            ("due today", "Show tasks due today"),
            ("due tomorrow", "Show tasks due tomorrow"),
            ("update <number>", "Update a task"),
            ("search <term>", "Search for tasks"),
            ("high priority tasks", "Show high priority tasks"),
            ("if <category> has tasks then ...", "Conditional execution"),
        ]
    }

    /// Format suggestions for display
    pub fn format_suggestions(suggestions: &[Suggestion]) -> String {
        if suggestions.is_empty() {
            return "No suggestions available".to_string();
        }

        let mut output = String::from("Suggestions:\n");

        for (i, suggestion) in suggestions.iter().enumerate() {
            let icon = match suggestion.suggestion_type {
                SuggestionType::CommandCompletion => "▶",
                SuggestionType::TypoCorrection => "✓",
                SuggestionType::SimilarCommand => "≈",
                SuggestionType::AvailableOption => "•",
                SuggestionType::Contextual => "◇",
            };

            output.push_str(&format!("  {} {}\n", i + 1, suggestion.text));
            if !suggestion.description.is_empty() {
                output.push_str(&format!("     {} {}\n", icon, suggestion.description));
            }
        }

        output
    }
}

/// Auto-completer for shell integration
pub struct AutoCompleter {
    /// Available categories
    categories: Vec<String>,
    /// Recent command history
    history: Vec<String>,
    /// Max history size
    max_history: usize,
}

impl AutoCompleter {
    /// Create a new auto-completer
    pub fn new() -> Self {
        Self {
            categories: Vec::new(),
            history: Vec::new(),
            max_history: 50,
        }
    }

    /// Create with categories
    pub fn with_categories(categories: Vec<String>) -> Self {
        Self {
            categories,
            history: Vec::new(),
            max_history: 50,
        }
    }

    /// Update available categories
    pub fn update_categories(&mut self, categories: Vec<String>) {
        self.categories = categories;
    }

    /// Add a command to history
    pub fn add_to_history(&mut self, command: String) {
        self.history.push(command);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }

    /// Get completions for input
    pub fn complete(&self, input: &str) -> Vec<String> {
        let request = SuggestionRequest {
            input: input.to_string(),
            cursor_position: input.len(),
            recent_commands: self.history.clone(),
            available_categories: self.categories.clone(),
        };

        let result = SuggestionEngine::suggest(&request);
        result.suggestions.into_iter().map(|s| s.text).collect()
    }

    /// Get detailed suggestions
    pub fn suggest(&self, input: &str) -> SuggestionResult {
        let request = SuggestionRequest {
            input: input.to_string(),
            cursor_position: input.len(),
            recent_commands: self.history.clone(),
            available_categories: self.categories.clone(),
        };

        SuggestionEngine::suggest(&request)
    }
}

impl Default for AutoCompleter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Suggestion Engine Tests ===

    #[test]
    fn test_empty_input_suggestions() {
        let request = SuggestionRequest {
            input: String::new(),
            cursor_position: 0,
            recent_commands: Vec::new(),
            available_categories: Vec::new(),
        };

        let result = SuggestionEngine::suggest(&request);
        assert!(!result.suggestions.is_empty());
        assert!(!result.is_valid);
    }

    #[test]
    fn test_add_task_completion() {
        let request = SuggestionRequest {
            input: "add".to_string(),
            cursor_position: 3,
            recent_commands: Vec::new(),
            available_categories: vec!["work".to_string(), "personal".to_string()],
        };

        let result = SuggestionEngine::suggest(&request);
        assert!(!result.suggestions.is_empty());
        // Should have completion suggestions
        assert!(result.suggestions.iter().any(|s| s.suggestion_type == SuggestionType::CommandCompletion));
    }

    #[test]
    fn test_typo_correction() {
        let request = SuggestionRequest {
            input: "complet".to_string(),
            cursor_position: 7,
            recent_commands: Vec::new(),
            available_categories: Vec::new(),
        };

        let result = SuggestionEngine::suggest(&request);
        // Should have typo correction
        assert!(result.suggestions.iter().any(|s| s.suggestion_type == SuggestionType::TypoCorrection));
    }

    #[test]
    fn test_valid_command_detection() {
        let request = SuggestionRequest {
            input: "list".to_string(),
            cursor_position: 4,
            recent_commands: Vec::new(),
            available_categories: Vec::new(),
        };

        let result = SuggestionEngine::suggest(&request);
        assert!(result.is_valid);
        assert!(result.parsed_command.is_some());
    }

    #[test]
    fn test_contextual_suggestions() {
        let request = SuggestionRequest {
            input: "".to_string(),
            cursor_position: 0,
            recent_commands: vec!["add task buy groceries".to_string(), "list".to_string()],
            available_categories: Vec::new(),
        };

        let result = SuggestionEngine::suggest(&request);
        // The "list" command in recent_commands should trigger contextual "done " suggestion
        assert!(result.suggestions.iter().any(|s| s.suggestion_type == SuggestionType::Contextual
            && s.text == "done "));
    }

    #[test]
    fn test_command_patterns() {
        let patterns = SuggestionEngine::command_patterns();
        assert!(!patterns.is_empty());
        assert!(patterns.iter().any(|(p, _)| p.contains("add task")));
    }

    // === Auto Completer Tests ===

    #[test]
    fn test_auto_completer_new() {
        let completer = AutoCompleter::new();
        assert!(completer.categories.is_empty());
        assert!(completer.history.is_empty());
    }

    #[test]
    fn test_auto_completer_with_categories() {
        let completer = AutoCompleter::with_categories(vec!["work".to_string()]);
        assert_eq!(completer.categories.len(), 1);
    }

    #[test]
    fn test_auto_completer_update_categories() {
        let mut completer = AutoCompleter::new();
        completer.update_categories(vec!["personal".to_string(), "home".to_string()]);
        assert_eq!(completer.categories.len(), 2);
    }

    #[test]
    fn test_auto_completer_add_to_history() {
        let mut completer = AutoCompleter::new();
        completer.add_to_history("add task test".to_string());
        assert_eq!(completer.history.len(), 1);
        assert_eq!(completer.history[0], "add task test");
    }

    #[test]
    fn test_auto_completer_history_limit() {
        let mut completer = AutoCompleter::with_max_history(3);
        for i in 0..5 {
            completer.add_to_history(format!("command {}", i));
        }
        assert_eq!(completer.history.len(), 3);
    }

    #[test]
    fn test_auto_completer_complete() {
        let completer = AutoCompleter::new();
        let completions = completer.complete("add");
        assert!(!completions.is_empty());
    }

    #[test]
    fn test_auto_completer_suggest() {
        let completer = AutoCompleter::new();
        let result = completer.suggest("list");
        assert!(result.is_valid);
    }

    // === Suggestion Type Tests ===

    #[test]
    fn test_suggestion_types() {
        assert_eq!(SuggestionType::CommandCompletion, SuggestionType::CommandCompletion);
        assert_ne!(SuggestionType::CommandCompletion, SuggestionType::TypoCorrection);
    }

    #[test]
    fn test_suggestion_clone() {
        let suggestion = Suggestion {
            text: "test".to_string(),
            suggestion_type: SuggestionType::CommandCompletion,
            confidence: 0.9,
            description: "Test suggestion".to_string(),
        };
        let cloned = suggestion.clone();
        assert_eq!(suggestion.text, cloned.text);
    }

    // === Suggestion Request Tests ===

    #[test]
    fn test_suggestion_request() {
        let request = SuggestionRequest {
            input: "test".to_string(),
            cursor_position: 4,
            recent_commands: vec!["previous".to_string()],
            available_categories: vec!["work".to_string()],
        };
        assert_eq!(request.input, "test");
        assert_eq!(request.cursor_position, 4);
        assert_eq!(request.recent_commands.len(), 1);
        assert_eq!(request.available_categories.len(), 1);
    }

    // === Format Suggestions Tests ===

    #[test]
    fn test_format_empty_suggestions() {
        let formatted = SuggestionEngine::format_suggestions(&[]);
        assert!(formatted.contains("No suggestions"));
    }

    #[test]
    fn test_format_suggestions_content() {
        let suggestions = vec![
            Suggestion {
                text: "add task".to_string(),
                suggestion_type: SuggestionType::CommandCompletion,
                confidence: 0.9,
                description: "Add a task".to_string(),
            },
        ];
        let formatted = SuggestionEngine::format_suggestions(&suggestions);
        assert!(formatted.contains("add task"));
        assert!(formatted.contains("Add a task"));
    }
}

/// Helper for creating AutoCompleter with max history
impl AutoCompleter {
    pub fn with_max_history(max_history: usize) -> Self {
        Self {
            categories: Vec::new(),
            history: Vec::new(),
            max_history,
        }
    }
}
