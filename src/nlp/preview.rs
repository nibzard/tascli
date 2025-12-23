//! Command preview and confirmation functionality
//!
//! This module provides preview capabilities for NLP commands before execution,
//! allowing users to review and confirm interpreted commands.

use super::types::*;
use super::mapper::CommandMapper;
use std::io::{self, Write};

/// Result of user confirmation prompt
#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmationResult {
    /// User confirmed execution
    Confirmed,
    /// User cancelled execution
    Cancelled,
    /// User wants to edit the command
    Edit,
}

/// Represents a command ready for preview
#[derive(Debug, Clone)]
pub struct PreviewCommand {
    /// Index in compound command sequence
    pub index: usize,
    /// Human-readable description
    pub description: String,
    /// CLI arguments that will be executed
    pub args: Vec<String>,
    /// Command type
    pub command_type: String,
}

impl PreviewCommand {
    /// Create a new preview command from an NLPCommand
    pub fn from_nlp_command(cmd: &NLPCommand, index: usize) -> Self {
        let description = CommandMapper::describe_command(cmd);
        let args = CommandMapper::to_tascli_args(cmd);
        let command_type = format!("{:?}", cmd.action);

        Self {
            index,
            description,
            args,
            command_type,
        }
    }

    /// Format for display
    pub fn format(&self) -> String {
        let mut output = format!("{}. {}\n", self.index + 1, self.description);
        output.push_str(&format!("   Type: {}\n", self.command_type));
        output.push_str(&format!("   Command: {}\n", self.args.join(" ")));
        output
    }
}

/// Preview formatter for displaying commands
pub struct PreviewFormatter {
    /// Show detailed information
    detailed: bool,
    /// Use colored output
    colored: bool,
}

impl PreviewFormatter {
    /// Create a new preview formatter
    pub fn new(detailed: bool, colored: bool) -> Self {
        Self {
            detailed,
            colored,
        }
    }

    /// Format a list of preview commands
    pub fn format_commands(&self, commands: &[PreviewCommand], mode: &CompoundExecutionMode) -> String {
        let mut output = String::new();

        // Header
        if self.colored {
            output.push_str("\x1b[1;36m"); // Cyan bold
        }
        output.push_str("\n=== Command Preview ===\n");
        if self.colored {
            output.push_str("\x1b[0m"); // Reset
        }

        // Execution mode
        output.push_str(&format!("Execution mode: {:?}\n", mode));
        output.push_str(&format!("Total commands: {}\n\n", commands.len()));

        // Commands
        for cmd in commands {
            output.push_str(&cmd.format());
        }

        output
    }

    /// Format execution summary
    pub fn format_summary(&self, total: usize, successful: usize, failed: usize) -> String {
        let mut output = String::new();

        if self.colored {
            output.push_str("\n\x1b[1;36m"); // Cyan bold
        }
        output.push_str("=== Execution Summary ===\n");
        if self.colored {
            output.push_str("\x1b[0m"); // Reset
        }

        if failed == 0 {
            if self.colored {
                output.push_str("\x1b[1;32m"); // Green bold
            }
            output.push_str(&format!("All {} command(s) executed successfully\n", total));
            if self.colored {
                output.push_str("\x1b[0m"); // Reset
            }
        } else {
            if self.colored {
                output.push_str("\x1b[1;33m"); // Yellow bold
            }
            output.push_str(&format!("Executed {} command(s): {} succeeded, {} failed\n",
                total, successful, failed));
            if self.colored {
                output.push_str("\x1b[0m"); // Reset
            }
        }

        output
    }
}

/// Manages command preview and user confirmation
pub struct PreviewManager {
    /// Whether preview is enabled
    enabled: bool,
    /// Whether to auto-confirm without asking
    auto_confirm: bool,
    /// Formatter for display
    formatter: PreviewFormatter,
}

impl PreviewManager {
    /// Create a new preview manager
    pub fn new(enabled: bool, auto_confirm: bool) -> Self {
        let formatter = PreviewFormatter::new(true, true);
        Self {
            enabled,
            auto_confirm,
            formatter,
        }
    }

    /// Check if preview is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Preview and confirm a single command
    pub fn preview_command(&self, cmd: &PreviewCommand) -> Result<ConfirmationResult, String> {
        if !self.enabled {
            return Ok(ConfirmationResult::Confirmed);
        }

        // Display the command
        print!("{}", self.formatter.format_commands(&[cmd.clone()], &CompoundExecutionMode::Sequential));

        // Auto-confirm if enabled
        if self.auto_confirm {
            println!("Auto-confirm enabled, proceeding with execution...\n");
            return Ok(ConfirmationResult::Confirmed);
        }

        // Prompt for confirmation
        self.prompt_confirmation()
    }

    /// Preview and confirm compound commands
    pub fn preview_compound(
        &self,
        commands: &[PreviewCommand],
        mode: &CompoundExecutionMode,
    ) -> Result<ConfirmationResult, String> {
        if !self.enabled {
            return Ok(ConfirmationResult::Confirmed);
        }

        // Display all commands
        print!("{}", self.formatter.format_commands(commands, mode));

        // Auto-confirm if enabled
        if self.auto_confirm {
            println!("Auto-confirm enabled, proceeding with execution...\n");
            return Ok(ConfirmationResult::Confirmed);
        }

        // Prompt for confirmation
        self.prompt_confirmation()
    }

    /// Show execution summary
    pub fn show_summary(&self, total: usize, successful: usize, failed: usize) {
        print!("{}", self.formatter.format_summary(total, successful, failed));
    }

    /// Prompt user for confirmation
    fn prompt_confirmation(&self) -> Result<ConfirmationResult, String> {
        print!("Execute these commands? [Y/n/e] ");

        io::stdout().flush()
            .map_err(|e| format!("Failed to flush stdout: {}", e))?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)
            .map_err(|e| format!("Failed to read input: {}", e))?;

        let input = input.trim().to_lowercase();
        println!();

        match input.as_str() {
            "" | "y" | "yes" => Ok(ConfirmationResult::Confirmed),
            "n" | "no" => Ok(ConfirmationResult::Cancelled),
            "e" | "edit" => Ok(ConfirmationResult::Edit),
            _ => {
                println!("Invalid input. Please enter Y (yes), N (no), or E (edit).");
                self.prompt_confirmation()
            }
        }
    }
}

impl Default for PreviewManager {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_confirm: false,
            formatter: PreviewFormatter::new(true, true),
        }
    }
}

/// Convert NLPCommands to PreviewCommands
pub fn commands_to_previews(commands: &[NLPCommand]) -> Vec<PreviewCommand> {
    commands.iter()
        .enumerate()
        .map(|(i, cmd)| PreviewCommand::from_nlp_command(cmd, i))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preview_command_from_nlp() {
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "test task".to_string(),
            category: Some("work".to_string()),
            ..Default::default()
        };

        let preview = PreviewCommand::from_nlp_command(&cmd, 0);

        assert_eq!(preview.index, 0);
        assert!(preview.description.contains("test task"));
        assert!(preview.args.contains(&"task".to_string()));
        assert_eq!(preview.command_type, "Task");
    }

    #[test]
    fn test_preview_command_format() {
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "test".to_string(),
            ..Default::default()
        };

        let preview = PreviewCommand::from_nlp_command(&cmd, 0);
        let formatted = preview.format();

        assert!(formatted.contains("1."));
        assert!(formatted.contains("test"));
        assert!(formatted.contains("Type:"));
        assert!(formatted.contains("Command:"));
    }

    #[test]
    fn test_commands_to_previews() {
        let commands = vec![
            NLPCommand {
                action: ActionType::Task,
                content: "task1".to_string(),
                ..Default::default()
            },
            NLPCommand {
                action: ActionType::Done,
                content: "task1".to_string(),
                ..Default::default()
            },
        ];

        let previews = commands_to_previews(&commands);

        assert_eq!(previews.len(), 2);
        assert_eq!(previews[0].index, 0);
        assert_eq!(previews[1].index, 1);
    }

    #[test]
    fn test_confirmation_result_variants() {
        assert_eq!(ConfirmationResult::Confirmed, ConfirmationResult::Confirmed);
        assert_eq!(ConfirmationResult::Cancelled, ConfirmationResult::Cancelled);
        assert_eq!(ConfirmationResult::Edit, ConfirmationResult::Edit);

        assert_ne!(ConfirmationResult::Confirmed, ConfirmationResult::Cancelled);
    }

    #[test]
    fn test_preview_manager_default() {
        let manager = PreviewManager::default();
        assert!(manager.is_enabled());
    }

    #[test]
    fn test_preview_manager_disabled() {
        let manager = PreviewManager::new(false, false);
        assert!(!manager.is_enabled());
    }

    #[test]
    fn test_preview_manager_auto_confirm() {
        let manager = PreviewManager::new(true, true);

        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "test".to_string(),
            ..Default::default()
        };

        let preview = PreviewCommand::from_nlp_command(&cmd, 0);

        // Auto-confirm should return Confirmed without prompting
        let result = manager.preview_command(&preview).unwrap();
        assert_eq!(result, ConfirmationResult::Confirmed);
    }

    #[test]
    fn test_formatter_format_commands() {
        let formatter = PreviewFormatter::new(true, false);
        let commands = vec![
            PreviewCommand {
                index: 0,
                description: "Test command".to_string(),
                args: vec!["task".to_string(), "test".to_string()],
                command_type: "Task".to_string(),
            },
        ];

        let output = formatter.format_commands(&commands, &CompoundExecutionMode::Sequential);

        assert!(output.contains("Command Preview"));
        assert!(output.contains("Execution mode"));
        assert!(output.contains("Total commands: 1"));
        assert!(output.contains("Test command"));
    }

    #[test]
    fn test_formatter_format_summary_success() {
        let formatter = PreviewFormatter::new(true, false);
        let output = formatter.format_summary(3, 3, 0);

        assert!(output.contains("Execution Summary"));
        assert!(output.contains("All 3 command(s) executed successfully"));
    }

    #[test]
    fn test_formatter_format_summary_partial_failure() {
        let formatter = PreviewFormatter::new(true, false);
        let output = formatter.format_summary(3, 2, 1);

        assert!(output.contains("Execution Summary"));
        assert!(output.contains("2 succeeded, 1 failed"));
    }

    #[test]
    fn test_preview_manager_disabled_auto_confirms() {
        let manager = PreviewManager::new(false, false);

        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "test".to_string(),
            ..Default::default()
        };

        let preview = PreviewCommand::from_nlp_command(&cmd, 0);

        // Disabled preview should auto-confirm
        let result = manager.preview_command(&preview).unwrap();
        assert_eq!(result, ConfirmationResult::Confirmed);
    }

    #[test]
    fn test_compound_command_preview() {
        let manager = PreviewManager::new(true, true);

        let commands = vec![
            NLPCommand {
                action: ActionType::Task,
                content: "task1".to_string(),
                ..Default::default()
            },
            NLPCommand {
                action: ActionType::Task,
                content: "task2".to_string(),
                ..Default::default()
            },
        ];

        let previews = commands_to_previews(&commands);
        let result = manager.preview_compound(&previews, &CompoundExecutionMode::Sequential).unwrap();

        assert_eq!(result, ConfirmationResult::Confirmed);
    }
}
