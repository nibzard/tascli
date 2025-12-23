//! Command batching for efficient compound command execution

use super::types::*;
use std::collections::{HashMap, HashSet};

/// Analyzer for determining command batchability
pub struct BatchAnalyzer;

impl BatchAnalyzer {
    /// Analyze commands and return batchable groups
    pub fn analyze_batches(commands: &[NLPCommand]) -> Vec<CommandBatch> {
        let mut batches: Vec<CommandBatch> = Vec::new();
        let mut processed = HashSet::new();

        for (i, cmd) in commands.iter().enumerate() {
            if processed.contains(&i) {
                continue;
            }

            // Try to find batchable partners
            let batch_group = Self::find_batch_group(i, commands, &processed);

            if batch_group.len() > 1 {
                // Create a batch
                let batch_commands: Vec<NLPCommand> = batch_group
                    .iter()
                    .map(|&idx| commands[idx].clone())
                    .collect();

                let batch_type = Self::determine_batch_type(&batch_commands);
                batches.push(CommandBatch::new(batch_type, batch_group.clone(), batch_commands));

                // Mark as processed
                for idx in batch_group {
                    processed.insert(idx);
                }
            } else {
                // Single command, not batchable
                batches.push(CommandBatch::single(i, cmd.clone()));
                processed.insert(i);
            }
        }

        batches
    }

    /// Find all commands that can be batched with the command at index
    fn find_batch_group(
        start_idx: usize,
        commands: &[NLPCommand],
        processed: &HashSet<usize>,
    ) -> Vec<usize> {
        let mut group = vec![start_idx];
        let start_cmd = &commands[start_idx];

        // Commands can only be batched if they:
        // 1. Are of the same action type
        // 2. Don't depend on each other's results
        // 3. Are read-only or independent writes

        for (i, cmd) in commands.iter().enumerate() {
            if i <= start_idx || processed.contains(&i) {
                continue;
            }

            if Self::can_batch(start_cmd, cmd) {
                group.push(i);
            }
        }

        group
    }

    /// Determine if two commands can be batched together
    fn can_batch(cmd1: &NLPCommand, cmd2: &NLPCommand) -> bool {
        // Same action type required for batching
        if cmd1.action != cmd2.action {
            return false;
        }

        match cmd1.action {
            // List commands are always batchable (read-only)
            ActionType::List => true,

            // Task/Record creation: batchable if independent (no dependencies)
            ActionType::Task | ActionType::Record => {
                // Can't batch if using context references like "it", "that"
                !Self::has_context_reference(cmd1) && !Self::has_context_reference(cmd2)
            },

            // Done/Delete/Update: generally not batchable due to potential dependencies
            ActionType::Done | ActionType::Delete | ActionType::Update => false,
        }
    }

    /// Check if command has context references that prevent batching
    fn has_context_reference(cmd: &NLPCommand) -> bool {
        let refs = ["it", "that", "this", "the task", "the previous"];
        let lower = cmd.content.to_lowercase();
        refs.iter().any(|&r| lower == r || lower.starts_with(&format!("{} ", r)))
    }

    /// Determine the batch type based on commands
    fn determine_batch_type(commands: &[NLPCommand]) -> BatchType {
        if commands.is_empty() {
            return BatchType::Mixed;
        }

        match commands[0].action {
            ActionType::Task => BatchType::TaskCreation,
            ActionType::Record => BatchType::RecordCreation,
            ActionType::List => BatchType::Query,
            _ => BatchType::Mixed,
        }
    }
}

/// A batch of commands that can be executed together
#[derive(Debug, Clone)]
pub struct CommandBatch {
    /// Type of batch
    pub batch_type: BatchType,
    /// Indices of original commands in this batch
    pub indices: Vec<usize>,
    /// Commands in this batch
    pub commands: Vec<NLPCommand>,
}

impl CommandBatch {
    /// Create a new command batch
    pub fn new(batch_type: BatchType, indices: Vec<usize>, commands: Vec<NLPCommand>) -> Self {
        Self {
            batch_type,
            indices,
            commands,
        }
    }

    /// Create a single-command batch
    pub fn single(index: usize, command: NLPCommand) -> Self {
        let batch_type = match command.action {
            ActionType::Task => BatchType::TaskCreation,
            ActionType::Record => BatchType::RecordCreation,
            ActionType::List => BatchType::Query,
            _ => BatchType::Mixed,
        };

        Self {
            batch_type,
            indices: vec![index],
            commands: vec![command],
        }
    }

    /// Check if this is a multi-command batch
    pub fn is_batched(&self) -> bool {
        self.commands.len() > 1
    }

    /// Number of commands in batch
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Check if batch is empty
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

/// Type of command batch
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchType {
    /// Multiple task creations
    TaskCreation,
    /// Multiple record creations
    RecordCreation,
    /// Multiple queries (read-only)
    Query,
    /// Mixed operations
    Mixed,
}

/// Executor for batched commands
pub struct BatchExecutor {
    /// Whether to show progress
    verbose: bool,
}

impl BatchExecutor {
    /// Create a new batch executor
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    /// Execute commands with automatic batching optimization
    pub fn execute_batched(
        &self,
        conn: &rusqlite::Connection,
        commands: &[NLPCommand],
        show_preview: bool,
    ) -> Result<BatchExecutionSummary, String> {
        // Analyze and create batches
        let batches = BatchAnalyzer::analyze_batches(commands);

        if show_preview {
            self.show_batch_preview(&batches, commands.len())?;
        }

        let mut all_results = Vec::new();
        let mut batch_results: Vec<BatchResult> = Vec::new();

        for batch in &batches {
            if self.verbose {
                println!("Processing batch: {} command(s)", batch.len());
            }

            let result = self.execute_batch(conn, batch)?;
            batch_results.push(result.clone());
            all_results.extend(result.command_results);
        }

        Ok(BatchExecutionSummary {
            total_commands: commands.len(),
            batch_count: batches.len(),
            batch_results,
            all_commands_result: all_results,
        })
    }

    /// Execute a single batch
    fn execute_batch(
        &self,
        conn: &rusqlite::Connection,
        batch: &CommandBatch,
    ) -> Result<BatchResult, String> {
        let mut command_results = Vec::new();

        match batch.batch_type {
            BatchType::TaskCreation | BatchType::RecordCreation => {
                // For creations, we can optimize by tracking created IDs
                for (idx, cmd) in batch.commands.iter().enumerate() {
                    let result = self.execute_single_command(conn, cmd, batch.indices[idx])?;
                    command_results.push(result);
                }
            },
            BatchType::Query => {
                // Queries are independent and safe to batch
                for (idx, cmd) in batch.commands.iter().enumerate() {
                    let result = self.execute_single_command(conn, cmd, batch.indices[idx])?;
                    command_results.push(result);
                }
            },
            BatchType::Mixed => {
                // Execute sequentially for mixed batches
                for (idx, cmd) in batch.commands.iter().enumerate() {
                    let result = self.execute_single_command(conn, cmd, batch.indices[idx])?;
                    command_results.push(result);
                }
            },
        }

        let all_success = command_results.iter().all(|r| r.success);

        Ok(BatchResult {
            batch_type: batch.batch_type,
            indices: batch.indices.clone(),
            command_results,
            all_success,
        })
    }

    /// Execute a single command
    fn execute_single_command(
        &self,
        conn: &rusqlite::Connection,
        command: &NLPCommand,
        original_index: usize,
    ) -> Result<CommandExecutionResult, String> {
        use super::mapper::CommandMapper;
        let args = CommandMapper::to_tascli_args(command);

        match execute_parsed_command(conn, &args) {
            Ok(()) => Ok(CommandExecutionResult {
                index: original_index,
                success: true,
                error: None,
                output: Some(CommandOutput {
                    item_id: None,
                    content: command.content.clone(),
                    category: command.category.clone(),
                    metadata: HashMap::new(),
                }),
            }),
            Err(e) => Ok(CommandExecutionResult {
                index: original_index,
                success: false,
                error: Some(e),
                output: None,
            }),
        }
    }

    /// Show preview of batched execution
    fn show_batch_preview(
        &self,
        batches: &[CommandBatch],
        total_commands: usize,
    ) -> Result<(), String> {
        use super::mapper::CommandMapper;

        println!("\n=== Batched Execution Preview ===");
        println!("Total commands: {}", total_commands);
        println!("Optimized into {} batch(es)", batches.len());
        println!();

        let batchable_count: usize = batches.iter().filter(|b| b.is_batched()).count();
        if batchable_count > 0 {
            println!("Batch optimization: {} command(s) can be executed efficiently",
                batches.iter().filter(|b| b.is_batched()).map(|b| b.len()).sum::<usize>());
            println!();
        }

        for (i, batch) in batches.iter().enumerate() {
            println!("Batch {} ({:?}, {} command(s)):", i + 1, batch.batch_type, batch.len());
            for cmd in &batch.commands {
                println!("  - {}", CommandMapper::describe_command(cmd));
            }
            println!();
        }

        print!("Execute with batch optimization? [Y/n] ");

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

impl Default for BatchExecutor {
    fn default() -> Self {
        Self { verbose: true }
    }
}

/// Result of executing a batch
#[derive(Debug, Clone)]
pub struct BatchResult {
    /// Type of batch
    pub batch_type: BatchType,
    /// Original indices of commands in this batch
    pub indices: Vec<usize>,
    /// Results of each command in the batch
    pub command_results: Vec<CommandExecutionResult>,
    /// Whether all commands in batch succeeded
    pub all_success: bool,
}

/// Summary of batched execution
#[derive(Debug, Clone)]
pub struct BatchExecutionSummary {
    /// Total number of commands
    pub total_commands: usize,
    /// Number of batches created
    pub batch_count: usize,
    /// Results for each batch
    pub batch_results: Vec<BatchResult>,
    /// All command results in order
    pub all_commands_result: Vec<CommandExecutionResult>,
}

impl BatchExecutionSummary {
    /// Check if all commands succeeded
    pub fn is_complete_success(&self) -> bool {
        self.all_commands_result.iter().all(|r| r.success)
    }

    /// Get number of successful commands
    pub fn successful_count(&self) -> usize {
        self.all_commands_result.iter().filter(|r| r.success).count()
    }

    /// Get number of failed commands
    pub fn failed_count(&self) -> usize {
        self.all_commands_result.iter().filter(|r| !r.success).count()
    }

    /// Get human-readable summary
    pub fn to_summary_string(&self) -> String {
        if self.is_complete_success() {
            format!("All {} command(s) executed successfully across {} batch(es)",
                self.total_commands, self.batch_count)
        } else {
            format!("Executed {} command(s) across {} batch(es): {} succeeded, {} failed",
                self.total_commands, self.batch_count,
                self.successful_count(), self.failed_count())
        }
    }

    /// Get batching efficiency
    pub fn efficiency_ratio(&self) -> f64 {
        if self.batch_count == 0 {
            return 1.0;
        }
        // Ratio of batches to original commands (lower is better)
        self.batch_count as f64 / self.total_commands as f64
    }
}

/// Execute a parsed command (re-exported from sequential.rs)
fn execute_parsed_command(
    conn: &rusqlite::Connection,
    args: &[String],
) -> Result<(), String> {
    if args.is_empty() {
        return Err("No command to execute".to_string());
    }

    use crate::args::parser::CliArgs;
    use clap::Parser;

    let cmd_args: Vec<&str> = std::iter::once("tascli")
        .chain(args.iter().map(|s| s.as_str()))
        .collect();

    let parsed_args = CliArgs::try_parse_from(cmd_args)
        .map_err(|e| format!("Failed to parse generated command: {}", e))?;

    crate::actions::handler::handle_commands(conn, parsed_args)
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Batch Type Tests ===

    #[test]
    fn test_batch_type_equality() {
        assert_eq!(BatchType::TaskCreation, BatchType::TaskCreation);
        assert_ne!(BatchType::TaskCreation, BatchType::RecordCreation);
    }

    #[test]
    fn test_batch_type_all_variants() {
        let types = vec![
            BatchType::TaskCreation,
            BatchType::RecordCreation,
            BatchType::Query,
            BatchType::Mixed,
        ];
        assert_eq!(types.len(), 4);
    }

    // === CommandBatch Tests ===

    #[test]
    fn test_command_batch_single() {
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "test task".to_string(),
            ..Default::default()
        };

        let batch = CommandBatch::single(0, cmd);

        assert_eq!(batch.batch_type, BatchType::TaskCreation);
        assert_eq!(batch.len(), 1);
        assert!(!batch.is_batched());
        assert!(!batch.is_empty());
        assert_eq!(batch.indices, vec![0]);
    }

    #[test]
    fn test_command_batch_multiple() {
        let commands = vec![
            NLPCommand {
                action: ActionType::Task,
                content: "task 1".to_string(),
                ..Default::default()
            },
            NLPCommand {
                action: ActionType::Task,
                content: "task 2".to_string(),
                ..Default::default()
            },
        ];

        let batch = CommandBatch::new(
            BatchType::TaskCreation,
            vec![0, 1],
            commands,
        );

        assert_eq!(batch.batch_type, BatchType::TaskCreation);
        assert_eq!(batch.len(), 2);
        assert!(batch.is_batched());
        assert!(!batch.is_empty());
    }

    #[test]
    fn test_command_batch_empty() {
        let batch = CommandBatch::new(
            BatchType::Mixed,
            vec![],
            vec![],
        );

        assert!(batch.is_empty());
        assert_eq!(batch.len(), 0);
        assert!(!batch.is_batched());
    }

    // === BatchAnalyzer Tests ===

    #[test]
    fn test_analyze_empty_commands() {
        let batches = BatchAnalyzer::analyze_batches(&[]);
        assert!(batches.is_empty());
    }

    #[test]
    fn test_analyze_single_command() {
        let commands = vec![NLPCommand {
            action: ActionType::Task,
            content: "single task".to_string(),
            ..Default::default()
        }];

        let batches = BatchAnalyzer::analyze_batches(&commands);

        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].len(), 1);
        assert!(!batches[0].is_batched());
    }

    #[test]
    fn test_analyze_batchable_tasks() {
        let commands = vec![
            NLPCommand {
                action: ActionType::Task,
                content: "task 1".to_string(),
                ..Default::default()
            },
            NLPCommand {
                action: ActionType::Task,
                content: "task 2".to_string(),
                ..Default::default()
            },
            NLPCommand {
                action: ActionType::Task,
                content: "task 3".to_string(),
                ..Default::default()
            },
        ];

        let batches = BatchAnalyzer::analyze_batches(&commands);

        // All tasks should be in one batch
        assert_eq!(batches.len(), 1);
        assert!(batches[0].is_batched());
        assert_eq!(batches[0].len(), 3);
        assert_eq!(batches[0].batch_type, BatchType::TaskCreation);
    }

    #[test]
    fn test_analyze_batchable_records() {
        let commands = vec![
            NLPCommand {
                action: ActionType::Record,
                content: "record 1".to_string(),
                ..Default::default()
            },
            NLPCommand {
                action: ActionType::Record,
                content: "record 2".to_string(),
                ..Default::default()
            },
        ];

        let batches = BatchAnalyzer::analyze_batches(&commands);

        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].batch_type, BatchType::RecordCreation);
        assert_eq!(batches[0].len(), 2);
    }

    #[test]
    fn test_analyze_batchable_queries() {
        let commands = vec![
            NLPCommand {
                action: ActionType::List,
                content: "".to_string(),
                query_type: Some(QueryType::Overdue),
                ..Default::default()
            },
            NLPCommand {
                action: ActionType::List,
                content: "".to_string(),
                query_type: Some(QueryType::Upcoming),
                ..Default::default()
            },
        ];

        let batches = BatchAnalyzer::analyze_batches(&commands);

        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].batch_type, BatchType::Query);
        assert_eq!(batches[0].len(), 2);
    }

    #[test]
    fn test_analyze_non_batchable_mixed() {
        let commands = vec![
            NLPCommand {
                action: ActionType::Task,
                content: "new task".to_string(),
                ..Default::default()
            },
            NLPCommand {
                action: ActionType::Done,
                content: "task 1".to_string(),
                ..Default::default()
            },
        ];

        let batches = BatchAnalyzer::analyze_batches(&commands);

        // Different action types cannot be batched
        assert_eq!(batches.len(), 2);
        assert!(!batches[0].is_batched());
        assert!(!batches[1].is_batched());
    }

    #[test]
    fn test_analyze_non_batchable_update() {
        let commands = vec![
            NLPCommand {
                action: ActionType::Update,
                content: "task 1".to_string(),
                ..Default::default()
            },
            NLPCommand {
                action: ActionType::Update,
                content: "task 2".to_string(),
                ..Default::default()
            },
        ];

        let batches = BatchAnalyzer::analyze_batches(&commands);

        // Update commands are not batchable (potential dependencies)
        assert_eq!(batches.len(), 2);
        assert!(!batches[0].is_batched());
        assert!(!batches[1].is_batched());
    }

    #[test]
    fn test_analyze_non_batchable_with_context_refs() {
        let commands = vec![
            NLPCommand {
                action: ActionType::Task,
                content: "first task".to_string(),
                ..Default::default()
            },
            NLPCommand {
                action: ActionType::Task,
                content: "it".to_string(),  // Context reference
                ..Default::default()
            },
        ];

        let batches = BatchAnalyzer::analyze_batches(&commands);

        // "it" is a context reference, prevents batching
        assert_eq!(batches.len(), 2);
    }

    #[test]
    fn test_analyze_mixed_batchable_and_non() {
        let commands = vec![
            NLPCommand {
                action: ActionType::Task,
                content: "task 1".to_string(),
                ..Default::default()
            },
            NLPCommand {
                action: ActionType::Task,
                content: "task 2".to_string(),
                ..Default::default()
            },
            NLPCommand {
                action: ActionType::Done,
                content: "task 1".to_string(),
                ..Default::default()
            },
        ];

        let batches = BatchAnalyzer::analyze_batches(&commands);

        assert_eq!(batches.len(), 2);
        // First batch should contain the two tasks
        assert_eq!(batches[0].len(), 2);
        assert_eq!(batches[0].batch_type, BatchType::TaskCreation);
        // Second batch is the done command
        assert_eq!(batches[1].len(), 1);
    }

    // === BatchAnalyzer Context Reference Tests ===

    #[test]
    fn test_has_context_reference_it() {
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "it".to_string(),
            ..Default::default()
        };

        assert!(BatchAnalyzer::has_context_reference(&cmd));
    }

    #[test]
    fn test_has_context_reference_that() {
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "that".to_string(),
            ..Default::default()
        };

        assert!(BatchAnalyzer::has_context_reference(&cmd));
    }

    #[test]
    fn test_has_context_reference_this() {
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "this".to_string(),
            ..Default::default()
        };

        assert!(BatchAnalyzer::has_context_reference(&cmd));
    }

    #[test]
    fn test_has_context_reference_the_task() {
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "the task".to_string(),
            ..Default::default()
        };

        assert!(BatchAnalyzer::has_context_reference(&cmd));
    }

    #[test]
    fn test_has_context_reference_false() {
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "buy groceries".to_string(),
            ..Default::default()
        };

        assert!(!BatchAnalyzer::has_context_reference(&cmd));
    }

    #[test]
    fn test_has_context_reference_false_with_it_in_word() {
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "edit the document".to_string(),
            ..Default::default()
        };

        assert!(!BatchAnalyzer::has_context_reference(&cmd));
    }

    // === BatchAnalyzer Can Batch Tests ===

    #[test]
    fn test_can_batch_same_task() {
        let cmd1 = NLPCommand {
            action: ActionType::Task,
            content: "task 1".to_string(),
            ..Default::default()
        };
        let cmd2 = NLPCommand {
            action: ActionType::Task,
            content: "task 2".to_string(),
            ..Default::default()
        };

        assert!(BatchAnalyzer::can_batch(&cmd1, &cmd2));
    }

    #[test]
    fn test_can_batch_same_record() {
        let cmd1 = NLPCommand {
            action: ActionType::Record,
            content: "record 1".to_string(),
            ..Default::default()
        };
        let cmd2 = NLPCommand {
            action: ActionType::Record,
            content: "record 2".to_string(),
            ..Default::default()
        };

        assert!(BatchAnalyzer::can_batch(&cmd1, &cmd2));
    }

    #[test]
    fn test_can_batch_same_list() {
        let cmd1 = NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            ..Default::default()
        };
        let cmd2 = NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            ..Default::default()
        };

        assert!(BatchAnalyzer::can_batch(&cmd1, &cmd2));
    }

    #[test]
    fn test_cannot_batch_different_actions() {
        let cmd1 = NLPCommand {
            action: ActionType::Task,
            content: "task".to_string(),
            ..Default::default()
        };
        let cmd2 = NLPCommand {
            action: ActionType::Record,
            content: "record".to_string(),
            ..Default::default()
        };

        assert!(!BatchAnalyzer::can_batch(&cmd1, &cmd2));
    }

    #[test]
    fn test_cannot_batch_with_context_ref() {
        let cmd1 = NLPCommand {
            action: ActionType::Task,
            content: "task 1".to_string(),
            ..Default::default()
        };
        let cmd2 = NLPCommand {
            action: ActionType::Task,
            content: "it".to_string(),
            ..Default::default()
        };

        assert!(!BatchAnalyzer::can_batch(&cmd1, &cmd2));
    }

    #[test]
    fn test_cannot_batch_done_commands() {
        let cmd1 = NLPCommand {
            action: ActionType::Done,
            content: "task 1".to_string(),
            ..Default::default()
        };
        let cmd2 = NLPCommand {
            action: ActionType::Done,
            content: "task 2".to_string(),
            ..Default::default()
        };

        assert!(!BatchAnalyzer::can_batch(&cmd1, &cmd2));
    }

    #[test]
    fn test_cannot_batch_delete_commands() {
        let cmd1 = NLPCommand {
            action: ActionType::Delete,
            content: "task 1".to_string(),
            ..Default::default()
        };
        let cmd2 = NLPCommand {
            action: ActionType::Delete,
            content: "task 2".to_string(),
            ..Default::default()
        };

        assert!(!BatchAnalyzer::can_batch(&cmd1, &cmd2));
    }

    #[test]
    fn test_cannot_batch_update_commands() {
        let cmd1 = NLPCommand {
            action: ActionType::Update,
            content: "task 1".to_string(),
            ..Default::default()
        };
        let cmd2 = NLPCommand {
            action: ActionType::Update,
            content: "task 2".to_string(),
            ..Default::default()
        };

        assert!(!BatchAnalyzer::can_batch(&cmd1, &cmd2));
    }

    // === BatchExecutor Tests ===

    #[test]
    fn test_batch_executor_default() {
        let executor = BatchExecutor::default();
        assert!(executor.verbose);
    }

    #[test]
    fn test_batch_executor_new() {
        let executor = BatchExecutor::new(false);
        assert!(!executor.verbose);
    }

    // === BatchExecutionSummary Tests ===

    #[test]
    fn test_batch_execution_summary_all_success() {
        let results = vec![
            BatchResult {
                batch_type: BatchType::TaskCreation,
                indices: vec![0, 1],
                command_results: vec![
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
                ],
                all_success: true,
            },
        ];

        let summary = BatchExecutionSummary {
            total_commands: 2,
            batch_count: 1,
            batch_results: results,
            all_commands_result: vec![
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
            ],
        };

        assert!(summary.is_complete_success());
        assert_eq!(summary.successful_count(), 2);
        assert_eq!(summary.failed_count(), 0);
    }

    #[test]
    fn test_batch_execution_summary_partial_failure() {
        let summary = BatchExecutionSummary {
            total_commands: 3,
            batch_count: 2,
            batch_results: vec![],
            all_commands_result: vec![
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
                CommandExecutionResult {
                    index: 2,
                    success: true,
                    error: None,
                    output: None,
                },
            ],
        };

        assert!(!summary.is_complete_success());
        assert_eq!(summary.successful_count(), 2);
        assert_eq!(summary.failed_count(), 1);
    }

    #[test]
    fn test_batch_execution_summary_efficiency() {
        let summary = BatchExecutionSummary {
            total_commands: 10,
            batch_count: 3,
            batch_results: vec![],
            all_commands_result: vec![],
        };

        assert_eq!(summary.efficiency_ratio(), 0.3);
    }

    #[test]
    fn test_batch_execution_summary_efficiency_empty() {
        let summary = BatchExecutionSummary {
            total_commands: 0,
            batch_count: 0,
            batch_results: vec![],
            all_commands_result: vec![],
        };

        assert_eq!(summary.efficiency_ratio(), 1.0);
    }

    #[test]
    fn test_batch_execution_summary_to_string_all_success() {
        let summary = BatchExecutionSummary {
            total_commands: 5,
            batch_count: 2,
            batch_results: vec![],
            all_commands_result: vec![
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
                CommandExecutionResult {
                    index: 2,
                    success: true,
                    error: None,
                    output: None,
                },
                CommandExecutionResult {
                    index: 3,
                    success: true,
                    error: None,
                    output: None,
                },
                CommandExecutionResult {
                    index: 4,
                    success: true,
                    error: None,
                    output: None,
                },
            ],
        };

        let s = summary.to_summary_string();
        assert!(s.contains("5 command(s)"));
        assert!(s.contains("2 batch(es)"));
        assert!(s.contains("successfully"));
    }

    #[test]
    fn test_batch_execution_summary_to_string_with_failure() {
        let summary = BatchExecutionSummary {
            total_commands: 4,
            batch_count: 2,
            batch_results: vec![],
            all_commands_result: vec![
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
                CommandExecutionResult {
                    index: 2,
                    success: true,
                    error: None,
                    output: None,
                },
                CommandExecutionResult {
                    index: 3,
                    success: false,
                    error: Some("error".to_string()),
                    output: None,
                },
            ],
        };

        let s = summary.to_summary_string();
        assert!(s.contains("4 command(s)"));
        assert!(s.contains("2 batch(es)"));
        assert!(s.contains("3 succeeded"));
        assert!(s.contains("1 failed"));
    }

    // === BatchResult Tests ===

    #[test]
    fn test_batch_result_all_success() {
        let result = BatchResult {
            batch_type: BatchType::TaskCreation,
            indices: vec![0, 1],
            command_results: vec![
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
            ],
            all_success: true,
        };

        assert!(result.all_success);
    }

    #[test]
    fn test_batch_result_partial_failure() {
        let result = BatchResult {
            batch_type: BatchType::TaskCreation,
            indices: vec![0, 1],
            command_results: vec![
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
            ],
            all_success: false,
        };

        assert!(!result.all_success);
    }

    // === Edge Cases ===

    #[test]
    fn test_analyze_large_batch() {
        let commands: Vec<NLPCommand> = (0..100)
            .map(|i| NLPCommand {
                action: ActionType::Task,
                content: format!("task {}", i),
                ..Default::default()
            })
            .collect();

        let batches = BatchAnalyzer::analyze_batches(&commands);

        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].len(), 100);
    }

    #[test]
    fn test_analyze_alternating_types() {
        let commands = vec![
            NLPCommand {
                action: ActionType::Task,
                content: "task 1".to_string(),
                ..Default::default()
            },
            NLPCommand {
                action: ActionType::List,
                content: "".to_string(),
                ..Default::default()
            },
            NLPCommand {
                action: ActionType::Task,
                content: "task 2".to_string(),
                ..Default::default()
            },
            NLPCommand {
                action: ActionType::List,
                content: "".to_string(),
                ..Default::default()
            },
        ];

        let batches = BatchAnalyzer::analyze_batches(&commands);

        // The analyzer processes commands sequentially and groups ALL
        // compatible commands, not just consecutive ones:
        // - task1 (index 0): finds task2 (index 2) compatible, forms batch
        // - list1 (index 1): finds list2 (index 3) compatible, forms batch
        // Result: 2 batches
        assert_eq!(batches.len(), 2);

        // First batch: both tasks
        assert_eq!(batches[0].len(), 2);
        assert_eq!(batches[0].batch_type, BatchType::TaskCreation);

        // Second batch: both list commands
        assert_eq!(batches[1].len(), 2);
        assert_eq!(batches[1].batch_type, BatchType::Query);
    }

    #[test]
    fn test_batch_clone() {
        let batch = CommandBatch::single(
            0,
            NLPCommand {
                action: ActionType::Task,
                content: "test".to_string(),
                ..Default::default()
            },
        );

        let cloned = batch.clone();
        assert_eq!(batch.batch_type, cloned.batch_type);
        assert_eq!(batch.len(), cloned.len());
    }
}
