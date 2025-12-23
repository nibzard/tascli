//! Conditional logic execution for NLP commands
//!
//! This module provides support for conditional command execution,
//! allowing commands to be executed based on runtime conditions.

use super::types::*;
use rusqlite::Connection;
use std::collections::HashMap;
use chrono::Timelike;

/// Evaluator for conditional expressions
pub struct ConditionEvaluator;

impl ConditionEvaluator {
    /// Create a new condition evaluator
    pub fn new() -> Self {
        Self
    }

    /// Evaluate a condition against the current state
    pub fn evaluate(
        &self,
        condition: &Condition,
        conn: &Connection,
        context: &SequentialContext,
    ) -> Result<bool, String> {
        match condition {
            Condition::Single(expr) => self.evaluate_expression(expr, conn, context),
            Condition::And(conditions) => {
                for cond in conditions {
                    if !self.evaluate(cond, conn, context)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            },
            Condition::Or(conditions) => {
                for cond in conditions {
                    if self.evaluate(cond, conn, context)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            },
            Condition::Not(cond) => {
                Ok(!self.evaluate(cond, conn, context)?)
            },
        }
    }

    /// Evaluate a single condition expression
    fn evaluate_expression(
        &self,
        expr: &ConditionExpression,
        conn: &Connection,
        context: &SequentialContext,
    ) -> Result<bool, String> {
        match expr {
            ConditionExpression::TaskExists { content } => {
                self.task_exists(conn, content)
            },

            ConditionExpression::TaskCount { operator, value } => {
                let count = self.get_task_count(conn)?;
                self.compare_numbers(count, *operator, *value)
            },

            ConditionExpression::CategoryHasTasks { category } => {
                self.category_has_tasks(conn, category)
            },

            ConditionExpression::CategoryEmpty { category } => {
                Ok(!self.category_has_tasks(conn, category)?)
            },

            ConditionExpression::PreviousSuccess => {
                Ok(context.previous_results
                    .last()
                    .map_or(false, |r| r.success))
            },

            ConditionExpression::PreviousFailed => {
                Ok(context.previous_results
                    .last()
                    .map_or(false, |r| !r.success))
            },

            ConditionExpression::TimeCondition { operator, hour, minute } => {
                let now = chrono::Local::now();
                let current_hour = now.hour() as i32;
                let current_min = now.minute() as i32;

                let current_value = current_hour * 60 + current_min;
                let target_value = hour.unwrap_or(0) * 60 + minute.unwrap_or(0);

                self.compare_numbers(current_value, *operator, target_value)
            },

            ConditionExpression::DayOfWeek { days } => {
                let now = chrono::Local::now();
                let current_day = now.format("%A").to_string().to_lowercase();
                Ok(days.iter()
                    .any(|d| d.to_lowercase() == current_day))
            },

            ConditionExpression::VariableEquals { name, value } => {
                Ok(context.get_var(name)
                    .map_or(false, |v| v == value))
            },

            ConditionExpression::VariableExists { name } => {
                Ok(context.get_var(name).is_some())
            },
        }
    }

    /// Check if a task exists matching the content
    fn task_exists(&self, conn: &Connection, content: &str) -> Result<bool, String> {
        let pattern = format!("%{}%", content);

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM items WHERE content LIKE ? AND status != 'cancelled'",
            [&pattern],
            |row| row.get(0),
        ).map_err(|e| format!("Database error: {}", e))?;

        Ok(count > 0)
    }

    /// Get the total count of non-cancelled tasks
    fn get_task_count(&self, conn: &Connection) -> Result<i32, String> {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM items WHERE status != 'cancelled'",
            [],
            |row| row.get(0),
        ).map_err(|e| format!("Database error: {}", e))?;

        Ok(count as i32)
    }

    /// Check if a category has any tasks
    fn category_has_tasks(&self, conn: &Connection, category: &str) -> Result<bool, String> {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM items WHERE category = ? AND status != 'cancelled'",
            [category],
            |row| row.get(0),
        ).map_err(|e| format!("Database error: {}", e))?;

        Ok(count > 0)
    }

    /// Compare two numbers using the given operator
    fn compare_numbers(&self, left: i32, operator: ComparisonOperator, right: i32) -> Result<bool, String> {
        Ok(match operator {
            ComparisonOperator::Equal => left == right,
            ComparisonOperator::NotEqual => left != right,
            ComparisonOperator::GreaterThan => left > right,
            ComparisonOperator::LessThan => left < right,
            ComparisonOperator::GreaterOrEqual => left >= right,
            ComparisonOperator::LessOrEqual => left <= right,
        })
    }
}

impl Default for ConditionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Executor for conditional command sequences
pub struct ConditionalExecutor {
    /// Whether to show progress
    verbose: bool,
    /// Condition evaluator
    evaluator: ConditionEvaluator,
}

impl ConditionalExecutor {
    /// Create a new conditional executor
    pub fn new(verbose: bool) -> Self {
        Self {
            verbose,
            evaluator: ConditionEvaluator::new(),
        }
    }

    /// Execute a command with conditional checking
    pub fn execute_conditional(
        &self,
        conn: &Connection,
        command: &NLPCommand,
        context: &SequentialContext,
    ) -> Result<ConditionalExecutionResult, String> {
        // Check if command has a condition
        if let Some(ref condition) = command.condition {
            let should_execute = self.evaluator.evaluate(condition, conn, context)?;

            if self.verbose {
                let status = if should_execute { "true" } else { "false" };
                println!("Condition evaluated: {}", status);
            }

            if should_execute {
                // Execute the command
                match self.execute_single(conn, command) {
                    Ok(output) => Ok(ConditionalExecutionResult {
                        executed: true,
                        skipped: false,
                        output: Some(output),
                    }),
                    Err(e) => Err(e),
                }
            } else {
                // Command was skipped due to condition
                Ok(ConditionalExecutionResult {
                    executed: false,
                    skipped: true,
                    output: None,
                })
            }
        } else {
            // No condition, execute normally
            match self.execute_single(conn, command) {
                Ok(output) => Ok(ConditionalExecutionResult {
                    executed: true,
                    skipped: false,
                    output: Some(output),
                }),
                Err(e) => Err(e),
            }
        }
    }

    /// Execute a conditional branch (if-then-else)
    pub fn execute_branch(
        &self,
        conn: &Connection,
        branch: &ConditionalBranch,
        context: &SequentialContext,
    ) -> Result<BranchExecutionResult, String> {
        let condition_result = self.evaluator.evaluate(&branch.condition, conn, context)?;

        let executed_commands = if condition_result {
            // Execute then_commands
            let mut results = Vec::new();
            for cmd in &branch.then_commands {
                match self.execute_conditional(conn, cmd, context) {
                    Ok(result) => {
                        if result.executed {
                            results.push(CommandExecutionResult {
                                index: results.len(),
                                success: true,
                                error: None,
                                output: result.output,
                            });
                        }
                    },
                    Err(e) => {
                        results.push(CommandExecutionResult {
                            index: results.len(),
                            success: false,
                            error: Some(e),
                            output: None,
                        });
                    },
                }
            }
            results
        } else {
            // Execute else_commands if present
            if let Some(ref else_commands) = branch.else_commands {
                let mut results = Vec::new();
                for cmd in else_commands {
                    match self.execute_conditional(conn, cmd, context) {
                        Ok(result) => {
                            if result.executed {
                                results.push(CommandExecutionResult {
                                    index: results.len(),
                                    success: true,
                                    error: None,
                                    output: result.output,
                                });
                            }
                        },
                        Err(e) => {
                            results.push(CommandExecutionResult {
                                index: results.len(),
                                success: false,
                                error: Some(e),
                                output: None,
                            });
                        },
                    }
                }
                results
            } else {
                Vec::new()
            }
        };

        Ok(BranchExecutionResult {
            condition_met: condition_result,
            executed_commands,
        })
    }

    /// Execute a single command
    fn execute_single(
        &self,
        conn: &Connection,
        command: &NLPCommand,
    ) -> Result<CommandOutput, String> {
        use super::mapper::CommandMapper;
        let args = CommandMapper::to_tascli_args(command);

        match execute_parsed_command(conn, &args) {
            Ok(()) => {
                let output = self.extract_output(command);
                Ok(output)
            },
            Err(e) => Err(e),
        }
    }

    /// Extract output information from a command
    fn extract_output(&self, command: &NLPCommand) -> CommandOutput {
        CommandOutput {
            item_id: None,
            content: command.content.clone(),
            category: command.category.clone(),
            metadata: HashMap::new(),
        }
    }
}

/// Result of conditional execution
#[derive(Debug, Clone)]
pub struct ConditionalExecutionResult {
    /// Whether the command was executed
    pub executed: bool,
    /// Whether the command was skipped due to condition
    pub skipped: bool,
    /// Output from the command
    pub output: Option<CommandOutput>,
}

/// Result of branch execution
#[derive(Debug, Clone)]
pub struct BranchExecutionResult {
    /// Whether the condition was met
    pub condition_met: bool,
    /// Results from executed commands
    pub executed_commands: Vec<CommandExecutionResult>,
}

impl Default for ConditionalExecutor {
    fn default() -> Self {
        Self {
            verbose: true,
            evaluator: ConditionEvaluator::new(),
        }
    }
}

/// Execute a parsed command (from sequential.rs - needed for reusability)
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

/// Helper functions for building conditions
pub struct ConditionBuilder;

impl ConditionBuilder {
    /// Build a task exists condition
    pub fn task_exists(content: &str) -> Condition {
        Condition::Single(Box::new(ConditionExpression::TaskExists {
            content: content.to_string(),
        }))
    }

    /// Build a task count condition
    pub fn task_count(operator: ComparisonOperator, value: i32) -> Condition {
        Condition::Single(Box::new(ConditionExpression::TaskCount {
            operator,
            value,
        }))
    }

    /// Build a category has tasks condition
    pub fn category_has_tasks(category: &str) -> Condition {
        Condition::Single(Box::new(ConditionExpression::CategoryHasTasks {
            category: category.to_string(),
        }))
    }

    /// Build a category empty condition
    pub fn category_empty(category: &str) -> Condition {
        Condition::Single(Box::new(ConditionExpression::CategoryEmpty {
            category: category.to_string(),
        }))
    }

    /// Build a previous success condition
    pub fn previous_success() -> Condition {
        Condition::Single(Box::new(ConditionExpression::PreviousSuccess))
    }

    /// Build a previous failed condition
    pub fn previous_failed() -> Condition {
        Condition::Single(Box::new(ConditionExpression::PreviousFailed))
    }

    /// Build a time condition
    pub fn time_condition(operator: ComparisonOperator, hour: Option<i32>, minute: Option<i32>) -> Condition {
        Condition::Single(Box::new(ConditionExpression::TimeCondition {
            operator,
            hour,
            minute,
        }))
    }

    /// Build a day of week condition
    pub fn day_of_week(days: Vec<&str>) -> Condition {
        Condition::Single(Box::new(ConditionExpression::DayOfWeek {
            days: days.into_iter().map(|s| s.to_string()).collect(),
        }))
    }

    /// Build an AND condition
    pub fn and(conditions: Vec<Condition>) -> Condition {
        Condition::And(conditions)
    }

    /// Build an OR condition
    pub fn or(conditions: Vec<Condition>) -> Condition {
        Condition::Or(conditions)
    }

    /// Build a NOT condition
    pub fn not(condition: Condition) -> Condition {
        Condition::Not(Box::new(condition))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_condition_builder_task_exists() {
        let cond = ConditionBuilder::task_exists("test");
        assert!(matches!(cond, Condition::Single(_)));
    }

    #[test]
    fn test_condition_builder_task_count() {
        let cond = ConditionBuilder::task_count(ComparisonOperator::GreaterThan, 5);
        assert!(matches!(cond, Condition::Single(_)));
    }

    #[test]
    fn test_condition_builder_category_has_tasks() {
        let cond = ConditionBuilder::category_has_tasks("work");
        assert!(matches!(cond, Condition::Single(_)));
    }

    #[test]
    fn test_condition_builder_category_empty() {
        let cond = ConditionBuilder::category_empty("personal");
        assert!(matches!(cond, Condition::Single(_)));
    }

    #[test]
    fn test_condition_builder_previous_success() {
        let cond = ConditionBuilder::previous_success();
        assert!(matches!(cond, Condition::Single(_)));
    }

    #[test]
    fn test_condition_builder_previous_failed() {
        let cond = ConditionBuilder::previous_failed();
        assert!(matches!(cond, Condition::Single(_)));
    }

    #[test]
    fn test_condition_builder_time_condition() {
        let cond = ConditionBuilder::time_condition(ComparisonOperator::GreaterOrEqual, Some(9), Some(0));
        assert!(matches!(cond, Condition::Single(_)));
    }

    #[test]
    fn test_condition_builder_day_of_week() {
        let cond = ConditionBuilder::day_of_week(vec!["Monday", "Tuesday"]);
        assert!(matches!(cond, Condition::Single(_)));
    }

    #[test]
    fn test_condition_builder_and() {
        let cond1 = ConditionBuilder::task_exists("test");
        let cond2 = ConditionBuilder::previous_success();
        let and_cond = ConditionBuilder::and(vec![cond1, cond2]);
        assert!(matches!(and_cond, Condition::And(_)));
    }

    #[test]
    fn test_condition_builder_or() {
        let cond1 = ConditionBuilder::category_has_tasks("work");
        let cond2 = ConditionBuilder::category_has_tasks("personal");
        let or_cond = ConditionBuilder::or(vec![cond1, cond2]);
        assert!(matches!(or_cond, Condition::Or(_)));
    }

    #[test]
    fn test_condition_builder_not() {
        let cond = ConditionBuilder::task_exists("test");
        let not_cond = ConditionBuilder::not(cond);
        assert!(matches!(not_cond, Condition::Not(_)));
    }

    #[test]
    fn test_comparison_operator_equality() {
        assert_eq!(ComparisonOperator::Equal, ComparisonOperator::Equal);
        assert_ne!(ComparisonOperator::Equal, ComparisonOperator::NotEqual);
    }

    #[test]
    fn test_comparison_operator_clone() {
        let op = ComparisonOperator::GreaterThan;
        let cloned = op.clone();
        assert_eq!(op, cloned);
    }

    #[test]
    fn test_condition_expression_task_exists() {
        let expr = ConditionExpression::TaskExists {
            content: "test".to_string(),
        };
        assert!(matches!(expr, ConditionExpression::TaskExists { .. }));
    }

    #[test]
    fn test_condition_expression_task_count() {
        let expr = ConditionExpression::TaskCount {
            operator: ComparisonOperator::Equal,
            value: 5,
        };
        assert!(matches!(expr, ConditionExpression::TaskCount { .. }));
    }

    #[test]
    fn test_condition_expression_category_has_tasks() {
        let expr = ConditionExpression::CategoryHasTasks {
            category: "work".to_string(),
        };
        assert!(matches!(expr, ConditionExpression::CategoryHasTasks { .. }));
    }

    #[test]
    fn test_condition_expression_category_empty() {
        let expr = ConditionExpression::CategoryEmpty {
            category: "personal".to_string(),
        };
        assert!(matches!(expr, ConditionExpression::CategoryEmpty { .. }));
    }

    #[test]
    fn test_condition_expression_previous_success() {
        let expr = ConditionExpression::PreviousSuccess;
        assert!(matches!(expr, ConditionExpression::PreviousSuccess));
    }

    #[test]
    fn test_condition_expression_previous_failed() {
        let expr = ConditionExpression::PreviousFailed;
        assert!(matches!(expr, ConditionExpression::PreviousFailed));
    }

    #[test]
    fn test_condition_expression_time_condition() {
        let expr = ConditionExpression::TimeCondition {
            operator: ComparisonOperator::GreaterOrEqual,
            hour: Some(9),
            minute: Some(0),
        };
        assert!(matches!(expr, ConditionExpression::TimeCondition { .. }));
    }

    #[test]
    fn test_condition_expression_day_of_week() {
        let expr = ConditionExpression::DayOfWeek {
            days: vec!["Monday".to_string(), "Friday".to_string()],
        };
        assert!(matches!(expr, ConditionExpression::DayOfWeek { .. }));
    }

    #[test]
    fn test_condition_expression_variable_equals() {
        let expr = ConditionExpression::VariableEquals {
            name: "key".to_string(),
            value: "value".to_string(),
        };
        assert!(matches!(expr, ConditionExpression::VariableEquals { .. }));
    }

    #[test]
    fn test_condition_expression_variable_exists() {
        let expr = ConditionExpression::VariableExists {
            name: "key".to_string(),
        };
        assert!(matches!(expr, ConditionExpression::VariableExists { .. }));
    }

    #[test]
    fn test_condition_clone() {
        let cond = Condition::Single(Box::new(ConditionExpression::PreviousSuccess));
        let cloned = cond.clone();
        assert_eq!(cond, cloned);
    }

    #[test]
    fn test_condition_and_clone() {
        let cond = Condition::And(vec![
            Condition::Single(Box::new(ConditionExpression::PreviousSuccess)),
            Condition::Single(Box::new(ConditionExpression::PreviousFailed)),
        ]);
        let cloned = cond.clone();
        assert_eq!(cond, cloned);
    }

    #[test]
    fn test_condition_or_clone() {
        let cond = Condition::Or(vec![
            Condition::Single(Box::new(ConditionExpression::PreviousSuccess)),
            Condition::Single(Box::new(ConditionExpression::PreviousFailed)),
        ]);
        let cloned = cond.clone();
        assert_eq!(cond, cloned);
    }

    #[test]
    fn test_condition_not_clone() {
        let cond = Condition::Not(Box::new(Condition::Single(Box::new(
            ConditionExpression::PreviousSuccess
        ))));
        let cloned = cond.clone();
        assert_eq!(cond, cloned);
    }

    #[test]
    fn test_conditional_branch_default_fields() {
        let branch = ConditionalBranch {
            condition: Condition::Single(Box::new(ConditionExpression::PreviousSuccess)),
            then_commands: vec![],
            else_commands: None,
        };

        assert!(branch.then_commands.is_empty());
        assert!(branch.else_commands.is_none());
    }

    #[test]
    fn test_conditional_branch_with_else() {
        let branch = ConditionalBranch {
            condition: Condition::Single(Box::new(ConditionExpression::PreviousSuccess)),
            then_commands: vec![],
            else_commands: Some(vec![]),
        };

        assert!(branch.else_commands.is_some());
        assert!(branch.else_commands.unwrap().is_empty());
    }

    #[test]
    fn test_conditional_execution_result_default() {
        let result = ConditionalExecutionResult {
            executed: false,
            skipped: false,
            output: None,
        };

        assert!(!result.executed);
        assert!(!result.skipped);
        assert!(result.output.is_none());
    }

    #[test]
    fn test_branch_execution_result_default() {
        let result = BranchExecutionResult {
            condition_met: false,
            executed_commands: vec![],
        };

        assert!(!result.condition_met);
        assert!(result.executed_commands.is_empty());
    }
}
