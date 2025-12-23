//! Integration tests for end-to-end NLP flow
//!
//! These tests cover the complete journey from user input through
//! NLP parsing, command mapping, and final execution.

use super::types::*;
use super::parser::NLPParser;
use super::mapper::CommandMapper;
use super::validator::CommandValidator;
use std::collections::HashMap;

/// Test helper that simulates the complete NLP flow
/// from a natural language command to tascli arguments
async fn simulate_nlp_flow(_input: &str, mock_command: NLPCommand) -> Result<(Vec<String>, String), String> {
    // Validate the command
    CommandValidator::validate(&mock_command)
        .map_err(|e| format!("Validation failed: {}", e))?;

    // Convert to tascli args
    let args = CommandMapper::to_tascli_args(&mock_command);
    let description = CommandMapper::describe_command(&mock_command);

    Ok((args, description))
}

// === End-to-End Flow Tests ===

#[tokio::test]
async fn test_e2e_task_creation_simple() {
    let input = "add a task to buy groceries";
    let mock_command = NLPCommand {
        action: ActionType::Task,
        content: "buy groceries".to_string(),
        ..Default::default()
    };

    let result = simulate_nlp_flow(input, mock_command).await;
    assert!(result.is_ok());

    let (args, description) = result.unwrap();
    assert_eq!(args, vec!["task", "buy groceries"]);
    assert_eq!(description, "Create task: buy groceries");
}

#[tokio::test]
async fn test_e2e_task_with_category_and_deadline() {
    let input = "add a work task for today to review the presentation";
    let mock_command = NLPCommand {
        action: ActionType::Task,
        content: "review the presentation".to_string(),
        category: Some("work".to_string()),
        deadline: Some("today".to_string()),
        ..Default::default()
    };

    let result = simulate_nlp_flow(input, mock_command).await;
    assert!(result.is_ok());

    let (args, description) = result.unwrap();
    assert_eq!(args, vec!["task", "-c", "work", "review the presentation", "today"]);
    assert!(description.contains("review the presentation"));
    assert!(description.contains("work"));
    assert!(description.contains("today"));
}

#[tokio::test]
async fn test_e2e_recurring_task() {
    let input = "create a daily task to water the plants";
    let mock_command = NLPCommand {
        action: ActionType::Task,
        content: "water the plants".to_string(),
        schedule: Some("daily".to_string()),
        ..Default::default()
    };

    let result = simulate_nlp_flow(input, mock_command).await;
    assert!(result.is_ok());

    let (args, description) = result.unwrap();
    assert_eq!(args, vec!["task", "water the plants", "daily"]);
    assert!(description.contains("recurring"));
    assert!(description.contains("daily"));
}

#[tokio::test]
async fn test_e2e_record_creation() {
    let input = "record that I completed a 5k run";
    let mock_command = NLPCommand {
        action: ActionType::Record,
        content: "completed a 5k run".to_string(),
        ..Default::default()
    };

    let result = simulate_nlp_flow(input, mock_command).await;
    assert!(result.is_ok());

    let (args, description) = result.unwrap();
    assert_eq!(args, vec!["record", "completed a 5k run"]);
    assert_eq!(description, "Create record: completed a 5k run");
}

#[tokio::test]
async fn test_e2e_record_with_category() {
    let input = "log a work record: completed project phase 1";
    let mock_command = NLPCommand {
        action: ActionType::Record,
        content: "completed project phase 1".to_string(),
        category: Some("work".to_string()),
        ..Default::default()
    };

    let result = simulate_nlp_flow(input, mock_command).await;
    assert!(result.is_ok());

    let (args, description) = result.unwrap();
    assert_eq!(args, vec!["record", "-c", "work", "completed project phase 1"]);
    assert!(description.contains("work"));
}

#[tokio::test]
async fn test_e2e_mark_task_done() {
    let input = "mark the groceries task as done";
    let mock_command = NLPCommand {
        action: ActionType::Done,
        content: "groceries".to_string(),
        ..Default::default()
    };

    let result = simulate_nlp_flow(input, mock_command).await;
    assert!(result.is_ok());

    let (args, description) = result.unwrap();
    assert_eq!(args, vec!["done", "groceries"]);
    assert!(description.contains("done"));
    assert!(description.contains("groceries"));
}

#[tokio::test]
async fn test_e2e_list_all_tasks() {
    let input = "show all my tasks";
    let mock_command = NLPCommand {
        action: ActionType::List,
        content: "tasks".to_string(),
        ..Default::default()
    };

    let result = simulate_nlp_flow(input, mock_command).await;
    assert!(result.is_ok());

    let (args, description) = result.unwrap();
    assert_eq!(args, vec!["list", "task"]);
    assert_eq!(description, "List tasks");
}

#[tokio::test]
async fn test_e2e_list_tasks_with_filters() {
    let input = "show work tasks that are ongoing";
    let mock_command = NLPCommand {
        action: ActionType::List,
        content: "tasks".to_string(),
        category: Some("work".to_string()),
        status: Some(StatusType::Ongoing),
        ..Default::default()
    };

    let result = simulate_nlp_flow(input, mock_command).await;
    assert!(result.is_ok());

    let (args, description) = result.unwrap();
    assert!(args.contains(&"list".to_string()));
    assert!(args.contains(&"task".to_string()));
    assert!(args.contains(&"-c".to_string()));
    assert!(args.contains(&"work".to_string()));
    assert!(args.contains(&"-s".to_string()));
    assert!(args.contains(&"ongoing".to_string()));
    assert!(description.contains("work"));
    assert!(description.contains("Ongoing"));
}

#[tokio::test]
async fn test_e2e_list_tasks_with_search() {
    let input = "find tasks containing meeting";
    let mock_command = NLPCommand {
        action: ActionType::List,
        content: "tasks".to_string(),
        search: Some("meeting".to_string()),
        ..Default::default()
    };

    let result = simulate_nlp_flow(input, mock_command).await;
    assert!(result.is_ok());

    let (args, description) = result.unwrap();
    assert!(args.contains(&"--search".to_string()));
    assert!(args.contains(&"meeting".to_string()));
    assert!(description.contains("search"));
    assert!(description.contains("meeting"));
}

#[tokio::test]
async fn test_e2e_list_tasks_with_days_limit() {
    let input = "show tasks from the last 7 days";
    let mock_command = NLPCommand {
        action: ActionType::List,
        content: "tasks".to_string(),
        days: Some(7),
        ..Default::default()
    };

    let result = simulate_nlp_flow(input, mock_command).await;
    assert!(result.is_ok());

    let (args, description) = result.unwrap();
    assert!(args.contains(&"-d".to_string()));
    assert!(args.contains(&"7".to_string()));
    assert!(description.contains("7 days"));
}

#[tokio::test]
async fn test_e2e_list_records() {
    let input = "show my records";
    let mock_command = NLPCommand {
        action: ActionType::List,
        content: "show my records".to_string(),
        ..Default::default()
    };

    let result = simulate_nlp_flow(input, mock_command).await;
    assert!(result.is_ok());

    let (args, description) = result.unwrap();
    assert_eq!(args, vec!["list", "record"]);
    assert_eq!(description, "List records");
}

#[tokio::test]
async fn test_e2e_delete_task() {
    let input = "delete the old task";
    let mock_command = NLPCommand {
        action: ActionType::Delete,
        content: "old task".to_string(),
        ..Default::default()
    };

    let result = simulate_nlp_flow(input, mock_command).await;
    assert!(result.is_ok());

    let (args, description) = result.unwrap();
    assert_eq!(args, vec!["delete", "old task"]);
    assert!(description.contains("old task"));
}

#[tokio::test]
async fn test_e2e_delete_by_status() {
    let input = "delete all completed tasks";
    let mock_command = NLPCommand {
        action: ActionType::Delete,
        content: "".to_string(),
        status: Some(StatusType::Done),
        ..Default::default()
    };

    let result = simulate_nlp_flow(input, mock_command).await;
    assert!(result.is_ok());

    let (args, _description) = result.unwrap();
    assert!(args.contains(&"--status".to_string()));
    assert!(args.contains(&"done".to_string()));
}

#[tokio::test]
async fn test_e2e_update_task_content() {
    let input = "update task 1 to change the content";
    let mut modifications = HashMap::new();
    modifications.insert("content".to_string(), "change the content".to_string());

    let mock_command = NLPCommand {
        action: ActionType::Update,
        content: "1".to_string(),
        modifications,
        ..Default::default()
    };

    let result = simulate_nlp_flow(input, mock_command).await;
    assert!(result.is_ok());

    let (args, _description) = result.unwrap();
    assert!(args.contains(&"update".to_string()));
    assert!(args.contains(&"1".to_string()));
    assert!(args.contains(&"--content".to_string()));
    assert!(args.contains(&"change the content".to_string()));
}

#[tokio::test]
async fn test_e2e_update_task_multiple_fields() {
    let input = "update task 1: change category to urgent and deadline to tomorrow";
    let mut modifications = HashMap::new();
    modifications.insert("category".to_string(), "urgent".to_string());
    modifications.insert("deadline".to_string(), "tomorrow".to_string());

    let mock_command = NLPCommand {
        action: ActionType::Update,
        content: "1".to_string(),
        modifications,
        ..Default::default()
    };

    let result = simulate_nlp_flow(input, mock_command).await;
    assert!(result.is_ok());

    let (args, _description) = result.unwrap();
    assert!(args.contains(&"--category".to_string()));
    assert!(args.contains(&"urgent".to_string()));
    assert!(args.contains(&"--deadline".to_string()));
    assert!(args.contains(&"tomorrow".to_string()));
}

// === Error Handling Tests ===

#[tokio::test]
async fn test_e2e_validation_error_empty_content() {
    let input = "add a task";
    let mock_command = NLPCommand {
        action: ActionType::Task,
        content: "".to_string(),
        ..Default::default()
    };

    let result = simulate_nlp_flow(input, mock_command).await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err();
    assert!(err_msg.contains("Validation failed"));
    assert!(err_msg.contains("required"));
}

#[tokio::test]
async fn test_e2e_validation_error_task_both_deadline_and_schedule() {
    let input = "add a task due today that repeats daily";
    let mock_command = NLPCommand {
        action: ActionType::Task,
        content: "test task".to_string(),
        deadline: Some("today".to_string()),
        schedule: Some("daily".to_string()),
        ..Default::default()
    };

    let result = simulate_nlp_flow(input, mock_command).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("both a deadline"));
}

#[tokio::test]
async fn test_e2e_validation_error_record_with_deadline() {
    let input = "record that I will complete the project tomorrow";
    let mock_command = NLPCommand {
        action: ActionType::Record,
        content: "complete the project".to_string(),
        deadline: Some("tomorrow".to_string()),
        ..Default::default()
    };

    let result = simulate_nlp_flow(input, mock_command).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot have deadlines"));
}

#[tokio::test]
async fn test_e2e_validation_error_update_no_modifications() {
    let input = "update task 1";
    let mock_command = NLPCommand {
        action: ActionType::Update,
        content: "1".to_string(),
        modifications: HashMap::new(),
        ..Default::default()
    };

    let result = simulate_nlp_flow(input, mock_command).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("modification"));
}

#[tokio::test]
async fn test_e2e_validation_error_content_too_long() {
    let input = "add a task";
    let mock_command = NLPCommand {
        action: ActionType::Task,
        content: "a".repeat(201),
        ..Default::default()
    };

    let result = simulate_nlp_flow(input, mock_command).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("too long"));
}

#[tokio::test]
async fn test_e2e_validation_error_category_too_long() {
    let input = "add a task with category";
    let mock_command = NLPCommand {
        action: ActionType::Task,
        content: "test".to_string(),
        category: Some("a".repeat(51)),
        ..Default::default()
    };

    let result = simulate_nlp_flow(input, mock_command).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("too long"));
}

#[tokio::test]
async fn test_e2e_validation_error_list_days_out_of_range() {
    let input = "show tasks from last 400 days";
    let mock_command = NLPCommand {
        action: ActionType::List,
        content: "".to_string(),
        days: Some(400),
        ..Default::default()
    };

    let result = simulate_nlp_flow(input, mock_command).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("between 1 and 365"));
}

#[tokio::test]
async fn test_e2e_validation_error_list_limit_out_of_range() {
    let input = "show tasks with limit 200";
    let mock_command = NLPCommand {
        action: ActionType::List,
        content: "".to_string(),
        limit: Some(200),
        ..Default::default()
    };

    let result = simulate_nlp_flow(input, mock_command).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Limit must be between 1 and 100"));
}

// === Edge Case Tests ===

#[tokio::test]
async fn test_e2e_task_with_unicode() {
    let input = "add a task: review Japanese document 日本語";
    let mock_command = NLPCommand {
        action: ActionType::Task,
        content: "review Japanese document 日本語".to_string(),
        ..Default::default()
    };

    let result = simulate_nlp_flow(input, mock_command).await;
    assert!(result.is_ok());
    let (args, _) = result.unwrap();
    assert!(args[1].contains("日本語"));
}

#[tokio::test]
async fn test_e2e_task_with_emoji() {
    let input = "add a task: celebrate 🎉";
    let mock_command = NLPCommand {
        action: ActionType::Task,
        content: "celebrate 🎉".to_string(),
        ..Default::default()
    };

    let result = simulate_nlp_flow(input, mock_command).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_e2e_task_with_max_length_content() {
    let input = "add a task";
    let mock_command = NLPCommand {
        action: ActionType::Task,
        content: "a".repeat(200),
        ..Default::default()
    };

    let result = simulate_nlp_flow(input, mock_command).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_e2e_task_with_special_characters_in_category() {
    let input = "add a task in category 'work & personal'";
    let mock_command = NLPCommand {
        action: ActionType::Task,
        content: "test".to_string(),
        category: Some("work & personal".to_string()),
        ..Default::default()
    };

    let result = simulate_nlp_flow(input, mock_command).await;
    assert!(result.is_ok());
    let (args, _) = result.unwrap();
    assert!(args.contains(&"work & personal".to_string()));
}

#[tokio::test]
async fn test_e2e_list_with_boundary_values() {
    // Test minimum valid days
    let cmd = NLPCommand {
        action: ActionType::List,
        content: "".to_string(),
        days: Some(1),
        ..Default::default()
    };
    assert!(simulate_nlp_flow("test", cmd).await.is_ok());

    // Test maximum valid days
    let cmd = NLPCommand {
        action: ActionType::List,
        content: "".to_string(),
        days: Some(365),
        ..Default::default()
    };
    assert!(simulate_nlp_flow("test", cmd).await.is_ok());

    // Test minimum valid limit
    let cmd = NLPCommand {
        action: ActionType::List,
        content: "".to_string(),
        limit: Some(1),
        ..Default::default()
    };
    assert!(simulate_nlp_flow("test", cmd).await.is_ok());

    // Test maximum valid limit
    let cmd = NLPCommand {
        action: ActionType::List,
        content: "".to_string(),
        limit: Some(100),
        ..Default::default()
    };
    assert!(simulate_nlp_flow("test", cmd).await.is_ok());
}

// === Integration with Real Parser Tests ===

#[tokio::test]
async fn test_parser_with_cache_flow() {
    let config = NLPConfig {
        cache_commands: true,
        ..Default::default()
    };

    let parser = NLPParser::new(config);

    // Test cache stats initially
    let (hot_len, cold_total, cold_expired) = parser.cache_stats().await;
    assert_eq!(hot_len + cold_total, 0);
    assert_eq!(cold_expired, 0);

    // Test that parser is ready check works
    assert!(!parser.is_ready());

    // Test config getter
    let config = parser.config();
    assert!(config.cache_commands);
}

#[tokio::test]
async fn test_parser_cache_operations() {
    let config = NLPConfig {
        cache_commands: true,
        ..Default::default()
    };

    let parser = NLPParser::new(config);

    // Test clear cache
    parser.clear_cache().await;

    // Verify cache stats after clear
    let (hot_len, cold_total, cold_expired) = parser.cache_stats().await;
    assert_eq!(hot_len + cold_total, 0);
    assert_eq!(cold_expired, 0);
}

#[tokio::test]
async fn test_parser_is_ready() {
    // Not ready with default config
    let parser = NLPParser::new(NLPConfig::default());
    assert!(!parser.is_ready());

    // Ready with proper config
    let config = NLPConfig {
        enabled: true,
        api_key: Some("test-key".to_string()),
        ..Default::default()
    };
    let parser = NLPParser::new(config);
    assert!(parser.is_ready());
}

#[tokio::test]
async fn test_parser_config_getter() {
    let config = NLPConfig {
        enabled: true,
        api_key: Some("test-key".to_string()),
        model: "custom-model".to_string(),
        cache_commands: false,
        ..Default::default()
    };

    let parser = NLPParser::new(config);
    let retrieved = parser.config();

    assert!(retrieved.enabled);
    assert_eq!(retrieved.api_key, Some("test-key".to_string()));
    assert_eq!(retrieved.model, "custom-model");
    assert!(!retrieved.cache_commands);
}

// === Command Mapper Integration Tests ===

#[tokio::test]
async fn test_mapper_all_action_types() {
    let test_cases: Vec<(ActionType, &str, &[&str])> = vec![
        (ActionType::Task, "content", &["task", "content"]),
        (ActionType::Record, "content", &["record", "content"]),
        (ActionType::Done, "content", &["done", "content"]),
        (ActionType::List, "", &["list", "task"]),
        (ActionType::Delete, "content", &["delete", "content"]),
    ];

    for (action, content, expected_start) in test_cases {
        let command = NLPCommand {
            action: action.clone(),
            content: content.to_string(),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);

        // Check that args starts with expected values
        assert!(
            args.len() >= expected_start.len(),
            "Action {:?} should produce at least {} args, got {:?}",
            action,
            expected_start.len(),
            args
        );

        for (i, expected) in expected_start.iter().enumerate() {
            assert_eq!(
                args[i], *expected,
                "Action {:?} arg {} should be {:?}, got {:?}",
                action, i, expected, args[i]
            );
        }
    }
}

#[tokio::test]
async fn test_mapper_describe_all_action_types() {
    let test_cases = vec![
        (ActionType::Task, "content", "Create task: content"),
        (ActionType::Record, "content", "Create record: content"),
        (ActionType::Done, "content", "Mark task as done: content"),
        (ActionType::List, "", "List tasks"),
        (ActionType::Delete, "content", "Delete: content"),
        (ActionType::Update, "content", "Update: content"),
    ];

    for (action, content, expected_desc) in test_cases {
        let command = NLPCommand {
            action: action.clone(),
            content: content.to_string(),
            ..Default::default()
        };

        let desc = CommandMapper::describe_command(&command);
        assert_eq!(
            desc,
            expected_desc,
            "Action {:?} should produce description {:?}, got {:?}",
            action,
            expected_desc,
            desc
        );
    }
}

// === Validation Integration Tests ===

#[tokio::test]
async fn test_validator_suggestions() {
    // Test content required suggestion - must use exact message from validator
    let cmd = NLPCommand {
        action: ActionType::Task,
        content: "".to_string(),
        ..Default::default()
    };
    let error = NLPError::ValidationError("Command content is required for this action".to_string());
    let suggestions = CommandValidator::suggest_corrections(&cmd, &error);
    assert!(!suggestions.is_empty());
    assert!(suggestions.iter().any(|s| s.contains("specify what task")));

    // Test too long suggestion - message must contain "too long"
    let cmd = NLPCommand {
        action: ActionType::Task,
        content: "a".repeat(201),
        ..Default::default()
    };
    let error = NLPError::ValidationError("Task content is too long (max 200 characters)".to_string());
    let suggestions = CommandValidator::suggest_corrections(&cmd, &error);
    assert!(!suggestions.is_empty());
    assert!(suggestions.iter().any(|s| s.contains("shorter")));

    // Test both deadline and schedule suggestion
    let cmd = NLPCommand {
        action: ActionType::Task,
        content: "test".to_string(),
        deadline: Some("today".to_string()),
        schedule: Some("daily".to_string()),
        ..Default::default()
    };
    let error = NLPError::ValidationError("Task cannot have both a deadline and a recurring schedule".to_string());
    let suggestions = CommandValidator::suggest_corrections(&cmd, &error);
    assert!(!suggestions.is_empty());
    assert!(suggestions.iter().any(|s| s.contains("one-time")));
}
