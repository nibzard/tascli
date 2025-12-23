//! Sequential command execution with state management and error handling

use super::types::*;
use rusqlite::Connection;

/// Executor for sequential compound commands
pub struct SequentialExecutor {
    /// Whether to stop on first error
    stop_on_error: bool,
    /// Whether to show progress
    verbose: bool,
}

impl SequentialExecutor {
    /// Create a new sequential executor
    pub fn new(stop_on_error: bool, verbose: bool) -> Self {
        Self {
            stop_on_error,
            verbose,
        }
    }

    /// Execute a compound command with state management
    pub fn execute_compound(
        &self,
        conn: &Connection,
        commands: &[NLPCommand],
        execution_mode: &CompoundExecutionMode,
        show_preview: bool,
    ) -> Result<ExecutionSummary, String> {
        // Show preview if requested
        if show_preview {
            self.show_preview(commands, execution_mode)?;
        }

        match execution_mode {
            CompoundExecutionMode::Sequential | CompoundExecutionMode::StopOnError => {
                self.execute_stop_on_error(conn, commands)
            },
            CompoundExecutionMode::Parallel => {
                self.execute_parallel(conn, commands)
            },
            CompoundExecutionMode::Dependent => {
                self.execute_dependent(conn, commands)
            },
            CompoundExecutionMode::ContinueOnError => {
                self.execute_continue_on_error(conn, commands)
            },
        }
    }

    /// Execute commands stopping on first error
    fn execute_stop_on_error(
        &self,
        conn: &Connection,
        commands: &[NLPCommand],
    ) -> Result<ExecutionSummary, String> {
        let mut context = SequentialContext::default();
        let mut results = Vec::new();

        for (index, command) in commands.iter().enumerate() {
            if self.verbose {
                println!("Executing command {}/{}...", index + 1, commands.len());
            }

            match self.execute_single(conn, command, &context) {
                Ok(result) => {
                    context.update_with_result(&result);
                    results.push(result);
                },
                Err(e) => {
                    let result = CommandExecutionResult {
                        index,
                        success: false,
                        error: Some(e.clone()),
                        output: None,
                    };
                    results.push(result);
                    return Ok(ExecutionSummary::new(commands.len(), results, context));
                },
            }
        }

        Ok(ExecutionSummary::new(commands.len(), results, context))
    }

    /// Execute commands continuing on error
    fn execute_continue_on_error(
        &self,
        conn: &Connection,
        commands: &[NLPCommand],
    ) -> Result<ExecutionSummary, String> {
        let mut context = SequentialContext::default();
        let mut results = Vec::new();

        for (index, command) in commands.iter().enumerate() {
            if self.verbose {
                println!("Executing command {}/{}...", index + 1, commands.len());
            }

            let result = match self.execute_single(conn, command, &context) {
                Ok(result) => {
                    context.update_with_result(&result);
                    result
                },
                Err(e) => {
                    CommandExecutionResult {
                        index,
                        success: false,
                        error: Some(e),
                        output: None,
                    }
                },
            };
            results.push(result);
        }

        Ok(ExecutionSummary::new(commands.len(), results, context))
    }

    /// Execute commands in parallel (simplified - executes sequentially but independently)
    fn execute_parallel(
        &self,
        conn: &Connection,
        commands: &[NLPCommand],
    ) -> Result<ExecutionSummary, String> {
        let context = SequentialContext::default();
        let mut results = Vec::new();

        for (index, command) in commands.iter().enumerate() {
            if self.verbose {
                println!("Executing command {}/{}...", index + 1, commands.len());
            }

            let result = match self.execute_single(conn, command, &context) {
                Ok(result) => result,
                Err(e) => CommandExecutionResult {
                    index,
                    success: false,
                    error: Some(e),
                    output: None,
                },
            };
            results.push(result);
        }

        Ok(ExecutionSummary::new(commands.len(), results, context))
    }

    /// Execute commands with dependency resolution
    fn execute_dependent(
        &self,
        conn: &Connection,
        commands: &[NLPCommand],
    ) -> Result<ExecutionSummary, String> {
        let mut context = SequentialContext::default();
        let mut results = Vec::new();

        for (index, command) in commands.iter().enumerate() {
            if self.verbose {
                println!("Executing command {}/{}...", index + 1, commands.len());
            }

            // Apply context substitutions to the command
            let resolved_command = self.resolve_context(command, &context);

            match self.execute_single(conn, &resolved_command, &context) {
                Ok(result) => {
                    context.update_with_result(&result);
                    results.push(result);
                },
                Err(e) => {
                    let result = CommandExecutionResult {
                        index,
                        success: false,
                        error: Some(e),
                        output: None,
                    };
                    results.push(result);
                    return Ok(ExecutionSummary::new(commands.len(), results, context));
                },
            }
        }

        Ok(ExecutionSummary::new(commands.len(), results, context))
    }

    /// Execute a single command
    fn execute_single(
        &self,
        conn: &Connection,
        command: &NLPCommand,
        _context: &SequentialContext,
    ) -> Result<CommandExecutionResult, String> {
        // Convert to CLI args
        use super::mapper::CommandMapper;
        let args = CommandMapper::to_tascli_args(command);

        // Execute the command
        match execute_parsed_command(conn, &args) {
            Ok(()) => {
                // Try to extract item info for context
                let output = self.extract_output(command);
                Ok(CommandExecutionResult {
                    index: 0,
                    success: true,
                    error: None,
                    output: Some(output),
                })
            },
            Err(e) => Err(e),
        }
    }

    /// Extract output information from a command
    fn extract_output(&self, command: &NLPCommand) -> CommandOutput {
        CommandOutput {
            item_id: None, // Would need to query DB to get actual ID
            content: command.content.clone(),
            category: command.category.clone(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Resolve context references in a command
    fn resolve_context(&self, command: &NLPCommand, context: &SequentialContext) -> NLPCommand {
        let mut resolved = command.clone();

        // Resolve content references (e.g., "it", "that", "the task")
        if resolved.content == "it" || resolved.content == "that" {
            if let Some(ref last_content) = context.last_content {
                resolved.content = last_content.clone();
            }
        }

        // Apply category from context if not specified
        if resolved.category.is_none() {
            if let Some(ref last_category) = context.last_category {
                resolved.category = Some(last_category.clone());
            }
        }

        // Apply variable substitutions in modifications
        for (_, value) in resolved.modifications.iter_mut() {
            if value.starts_with("$") {
                let var_name = &value[1..];
                if let Some(var_value) = context.get_var(var_name) {
                    *value = var_value.clone();
                }
            }
        }

        resolved
    }

    /// Show preview of commands to be executed
    fn show_preview(&self, commands: &[NLPCommand], execution_mode: &CompoundExecutionMode) -> Result<(), String> {
        use super::mapper::CommandMapper;

        println!("\n=== Compound Command Preview ===");
        println!("Execution mode: {:?}", execution_mode);
        println!("Total commands: {}", commands.len());
        println!();

        for (i, cmd) in commands.iter().enumerate() {
            let description = CommandMapper::describe_command(cmd);
            let args = CommandMapper::to_tascli_args(cmd);
            println!("{}. {}", i + 1, description);
            println!("   Command: {}", args.join(" "));
            println!();
        }

        print!("Execute these commands? [Y/n] ");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)
            .map_err(|e| format!("Failed to read input: {}", e))?;

        let input = input.trim().to_lowercase();
        if !input.is_empty() && input != "y" && input != "yes" {
            return Err("Commands cancelled by user.".to_string());
        }

        println!();
        Ok(())
    }
}

/// Execute a parsed command (from actions/nlp.rs - moved here for reusability)
fn execute_parsed_command(conn: &Connection, args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("No command to execute".to_string());
    }

    use crate::args::parser::{CliArgs};
    use clap::Parser;

    let cmd_args: Vec<&str> = std::iter::once("tascli")
        .chain(args.iter().map(|s| s.as_str()))
        .collect();

    let parsed_args = CliArgs::try_parse_from(cmd_args)
        .map_err(|e| format!("Failed to parse generated command: {}", e))?;

    crate::actions::handler::handle_commands(conn, parsed_args)
}

impl Default for SequentialExecutor {
    fn default() -> Self {
        Self {
            stop_on_error: true,
            verbose: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequential_context_default() {
        let ctx = SequentialContext::default();
        assert!(ctx.previous_results.is_empty());
        assert!(ctx.last_item_id.is_none());
        assert!(ctx.last_category.is_none());
        assert!(ctx.last_content.is_none());
        assert!(ctx.variables.is_empty());
    }

    #[test]
    fn test_sequential_context_update() {
        let mut ctx = SequentialContext::default();

        let result = CommandExecutionResult {
            index: 0,
            success: true,
            error: None,
            output: Some(CommandOutput {
                item_id: Some(42),
                content: "test task".to_string(),
                category: Some("work".to_string()),
                metadata: std::collections::HashMap::new(),
            }),
        };

        ctx.update_with_result(&result);

        assert_eq!(ctx.previous_results.len(), 1);
        assert_eq!(ctx.last_item_id, Some(42));
        assert_eq!(ctx.last_category, Some("work".to_string()));
        assert_eq!(ctx.last_content, Some("test task".to_string()));
    }

    #[test]
    fn test_sequential_context_variables() {
        let mut ctx = SequentialContext::default();

        ctx.set_var("key1".to_string(), "value1".to_string());
        ctx.set_var("key2".to_string(), "value2".to_string());

        assert_eq!(ctx.get_var("key1"), Some(&"value1".to_string()));
        assert_eq!(ctx.get_var("key2"), Some(&"value2".to_string()));
        assert_eq!(ctx.get_var("nonexistent"), None);
    }

    #[test]
    fn test_execution_summary_complete_success() {
        let results = vec![
            CommandExecutionResult {
                index: 0,
                success: true,
                error: None,
                output: None,
            },
            CommandExecutionResult {
                index: 1,
                success: true,
                error: None,
                output: None,
            },
        ];

        let summary = ExecutionSummary::new(2, results, SequentialContext::default());

        assert!(summary.is_complete_success());
        assert_eq!(summary.total, 2);
        assert_eq!(summary.successful, 2);
        assert_eq!(summary.failed, 0);
    }

    #[test]
    fn test_execution_summary_partial_failure() {
        let results = vec![
            CommandExecutionResult {
                index: 0,
                success: true,
                error: None,
                output: None,
            },
            CommandExecutionResult {
                index: 1,
                success: false,
                error: Some("error".to_string()),
                output: None,
            },
        ];

        let summary = ExecutionSummary::new(2, results, SequentialContext::default());

        assert!(!summary.is_complete_success());
        assert_eq!(summary.total, 2);
        assert_eq!(summary.successful, 1);
        assert_eq!(summary.failed, 1);
    }

    #[test]
    fn test_execution_summary_string() {
        let results = vec![
            CommandExecutionResult {
                index: 0,
                success: true,
                error: None,
                output: None,
            },
        ];

        let summary = ExecutionSummary::new(1, results, SequentialContext::default());

        assert_eq!(summary.to_summary_string(), "All 1 command(s) executed successfully");

        let results = vec![
            CommandExecutionResult {
                index: 0,
                success: true,
                error: None,
                output: None,
            },
            CommandExecutionResult {
                index: 1,
                success: false,
                error: Some("error".to_string()),
                output: None,
            },
        ];

        let summary = ExecutionSummary::new(2, results, SequentialContext::default());

        assert_eq!(summary.to_summary_string(), "Executed 2 command(s): 1 succeeded, 1 failed");
    }

    #[test]
    fn test_sequential_executor_default() {
        let executor = SequentialExecutor::default();
        assert!(executor.stop_on_error);
        assert!(executor.verbose);
    }

    #[test]
    fn test_sequential_executor_new() {
        let executor = SequentialExecutor::new(false, false);
        assert!(!executor.stop_on_error);
        assert!(!executor.verbose);
    }

    #[test]
    fn test_resolve_context_content() {
        let executor = SequentialExecutor::default();

        let mut context = SequentialContext::default();
        context.last_content = Some("original task".to_string());

        let command = NLPCommand {
            action: ActionType::Done,
            content: "it".to_string(),
            ..Default::default()
        };

        let resolved = executor.resolve_context(&command, &context);

        assert_eq!(resolved.content, "original task");
    }

    #[test]
    fn test_resolve_context_category() {
        let executor = SequentialExecutor::default();

        let mut context = SequentialContext::default();
        context.last_category = Some("work".to_string());

        let command = NLPCommand {
            action: ActionType::Task,
            content: "new task".to_string(),
            category: None,
            ..Default::default()
        };

        let resolved = executor.resolve_context(&command, &context);

        assert_eq!(resolved.category, Some("work".to_string()));
    }

    #[test]
    fn test_resolve_context_variables() {
        let executor = SequentialExecutor::default();

        let mut context = SequentialContext::default();
        context.set_var("deadline".to_string(), "tomorrow".to_string());

        let mut command = NLPCommand {
            action: ActionType::Task,
            content: "new task".to_string(),
            ..Default::default()
        };
        command.modifications.insert("deadline".to_string(), "$deadline".to_string());

        let resolved = executor.resolve_context(&command, &context);

        assert_eq!(resolved.modifications.get("deadline"), Some(&"tomorrow".to_string()));
    }
}
