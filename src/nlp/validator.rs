//! Command validation for NLP parsed commands

use super::types::*;

pub struct CommandValidator;

impl CommandValidator {
    /// Validate an NLP command and return an error if it's invalid
    pub fn validate(command: &NLPCommand) -> NLPResult<()> {
        // Basic validation
        // List and Delete commands can have empty content (Delete can use status filter)
        let can_have_empty_content = command.action == ActionType::List
            || (command.action == ActionType::Delete && command.status.is_some());

        if command.content.trim().is_empty() && !can_have_empty_content {
            return Err(NLPError::ValidationError(
                "Command content is required for this action".to_string()
            ));
        }

        match command.action {
            ActionType::Task => Self::validate_task(command),
            ActionType::Record => Self::validate_record(command),
            ActionType::Done => Self::validate_done(command),
            ActionType::Update => Self::validate_update(command),
            ActionType::Delete => Self::validate_delete(command),
            ActionType::List => Self::validate_list(command),
        }
    }

    fn validate_task(command: &NLPCommand) -> NLPResult<()> {
        // Validate that we don't have both deadline and schedule for simple tasks
        if command.deadline.is_some() && command.schedule.is_some() {
            return Err(NLPError::ValidationError(
                "Task cannot have both a deadline and a recurring schedule".to_string()
            ));
        }

        // Validate content length
        if command.content.len() > 200 {
            return Err(NLPError::ValidationError(
                "Task content is too long (max 200 characters)".to_string()
            ));
        }

        // Validate category if provided
        if let Some(category) = &command.category {
            Self::validate_category(category)?;
        }

        Ok(())
    }

    fn validate_record(command: &NLPCommand) -> NLPResult<()> {
        // Records shouldn't have deadlines or schedules
        if command.deadline.is_some() {
            return Err(NLPError::ValidationError(
                "Records cannot have deadlines".to_string()
            ));
        }

        if command.schedule.is_some() {
            return Err(NLPError::ValidationError(
                "Records cannot have recurring schedules".to_string()
            ));
        }

        // Validate content length
        if command.content.len() > 200 {
            return Err(NLPError::ValidationError(
                "Record content is too long (max 200 characters)".to_string()
            ));
        }

        // Validate category if provided
        if let Some(category) = &command.category {
            Self::validate_category(category)?;
        }

        Ok(())
    }

    fn validate_done(command: &NLPCommand) -> NLPResult<()> {
        // Validate that we have some way to identify the task
        if command.content.trim().is_empty() {
            return Err(NLPError::ValidationError(
                "Task identifier is required for marking as done".to_string()
            ));
        }

        Ok(())
    }

    fn validate_update(command: &NLPCommand) -> NLPResult<()> {
        // Validate that we have a target to update
        if command.content.trim().is_empty() {
            return Err(NLPError::ValidationError(
                "Task identifier is required for update".to_string()
            ));
        }

        // Validate that we have some modifications
        if command.modifications.is_empty() {
            return Err(NLPError::ValidationError(
                "At least one modification is required for update".to_string()
            ));
        }

        // Validate specific modifications
        for (key, value) in &command.modifications {
            match key.as_str() {
                "content" => {
                    if value.trim().is_empty() {
                        return Err(NLPError::ValidationError(
                            "Content cannot be empty".to_string()
                        ));
                    }
                    if value.len() > 200 {
                        return Err(NLPError::ValidationError(
                            "Content is too long (max 200 characters)".to_string()
                        ));
                    }
                },
                "category" => {
                    Self::validate_category(value)?;
                },
                "deadline" | "schedule" => {
                    // Basic validation - could be enhanced with time parsing
                    if value.trim().is_empty() {
                        return Err(NLPError::ValidationError(
                            format!("{} cannot be empty", key)
                        ));
                    }
                },
                _ => {
                    return Err(NLPError::ValidationError(
                        format!("Unknown modification type: {}", key)
                    ));
                }
            }
        }

        Ok(())
    }

    fn validate_delete(command: &NLPCommand) -> NLPResult<()> {
        // Either have specific content or a status filter for bulk deletion
        if command.content.trim().is_empty() && command.status.is_none() {
            return Err(NLPError::ValidationError(
                "Either specific item identifier or status filter is required for deletion".to_string()
            ));
        }

        Ok(())
    }

    fn validate_list(command: &NLPCommand) -> NLPResult<()> {
        // Validate days filter
        if let Some(days) = command.days {
            if days < 1 || days > 365 {
                return Err(NLPError::ValidationError(
                    "Days filter must be between 1 and 365".to_string()
                ));
            }
        }

        // Validate limit
        if let Some(limit) = command.limit {
            if limit < 1 || limit > 100 {
                return Err(NLPError::ValidationError(
                    "Limit must be between 1 and 100".to_string()
                ));
            }
        }

        // Validate category if provided
        if let Some(category) = &command.category {
            Self::validate_category(category)?;
        }

        Ok(())
    }

    fn validate_category(category: &str) -> NLPResult<()> {
        if category.trim().is_empty() {
            return Err(NLPError::ValidationError(
                "Category cannot be empty".to_string()
            ));
        }

        if category.len() > 50 {
            return Err(NLPError::ValidationError(
                "Category is too long (max 50 characters)".to_string()
            ));
        }

        // Check for control characters or invalid whitespace (tabs, newlines)
        for c in category.chars() {
            if c.is_control() || c == '\t' || c == '\n' || c == '\r' {
                return Err(NLPError::ValidationError(
                    "Category contains invalid whitespace characters".to_string()
                ));
            }
        }

        Ok(())
    }

    /// Suggest corrections for common validation errors
    pub fn suggest_corrections(command: &NLPCommand, error: &NLPError) -> Vec<String> {
        let mut suggestions = Vec::new();

        match error {
            NLPError::ValidationError(msg) => {
                if msg.contains("content is required") {
                    suggestions.push("Please specify what task or record you want to create/manage".to_string());
                }

                if msg.contains("too long") {
                    suggestions.push("Consider using shorter, more concise descriptions".to_string());
                }

                if msg.contains("both a deadline and a recurring schedule") {
                    suggestions.push("Choose either a one-time deadline or a recurring schedule".to_string());
                }

                if msg.contains("cannot have deadlines") {
                    suggestions.push("Records are for logging past events, not future tasks".to_string());
                }

                if msg.contains("identifier is required") {
                    suggestions.push("Please specify which task you want to modify or refer to".to_string());
                }

                if msg.contains("At least one modification is required") {
                    suggestions.push("Please specify what you want to change about the task".to_string());
                }
            },
            _ => {
                suggestions.push("Try rephrasing your command more clearly".to_string());
                suggestions.push("Use specific terms like 'task', 'record', 'list', 'done', 'update', or 'delete'".to_string());
            }
        }

        suggestions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_task_command(content: &str) -> NLPCommand {
        NLPCommand {
            action: ActionType::Task,
            content: content.to_string(),
            ..Default::default()
        }
    }

    fn make_record_command(content: &str) -> NLPCommand {
        NLPCommand {
            action: ActionType::Record,
            content: content.to_string(),
            ..Default::default()
        }
    }

    fn make_done_command(content: &str) -> NLPCommand {
        NLPCommand {
            action: ActionType::Done,
            content: content.to_string(),
            ..Default::default()
        }
    }

    fn make_list_command() -> NLPCommand {
        NLPCommand {
            action: ActionType::List,
            ..Default::default()
        }
    }

    fn make_update_command(content: &str, modifications: HashMap<String, String>) -> NLPCommand {
        NLPCommand {
            action: ActionType::Update,
            content: content.to_string(),
            modifications,
            ..Default::default()
        }
    }

    fn make_delete_command(content: &str) -> NLPCommand {
        NLPCommand {
            action: ActionType::Delete,
            content: content.to_string(),
            ..Default::default()
        }
    }

    // === Basic Validation Tests ===

    #[test]
    fn test_validate_valid_task() {
        let cmd = make_task_command("Buy groceries");
        assert!(CommandValidator::validate(&cmd).is_ok());
    }

    #[test]
    fn test_validate_empty_content_for_task() {
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "".to_string(),
            ..Default::default()
        };
        assert!(CommandValidator::validate(&cmd).is_err());
    }

    #[test]
    fn test_validate_whitespace_only_content() {
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "   ".to_string(),
            ..Default::default()
        };
        assert!(CommandValidator::validate(&cmd).is_err());
    }

    #[test]
    fn test_validate_list_without_content() {
        let cmd = make_list_command();
        assert!(CommandValidator::validate(&cmd).is_ok());
    }

    // === Task-Specific Validation Tests ===

    #[test]
    fn test_validate_task_with_both_deadline_and_schedule() {
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "Test task".to_string(),
            deadline: Some("today".to_string()),
            schedule: Some("daily".to_string()),
            ..Default::default()
        };
        let result = CommandValidator::validate(&cmd);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("both a deadline"));
    }

    #[test]
    fn test_validate_task_content_too_long() {
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "a".repeat(201),
            ..Default::default()
        };
        let result = CommandValidator::validate(&cmd);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too long"));
    }

    #[test]
    fn test_validate_task_content_max_length() {
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "a".repeat(200),
            ..Default::default()
        };
        assert!(CommandValidator::validate(&cmd).is_ok());
    }

    #[test]
    fn test_validate_task_with_valid_category() {
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "Test task".to_string(),
            category: Some("work".to_string()),
            ..Default::default()
        };
        assert!(CommandValidator::validate(&cmd).is_ok());
    }

    #[test]
    fn test_validate_task_with_empty_category() {
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "Test task".to_string(),
            category: Some("".to_string()),
            ..Default::default()
        };
        let result = CommandValidator::validate(&cmd);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Category cannot be empty"));
    }

    #[test]
    fn test_validate_task_with_whitespace_category() {
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "Test task".to_string(),
            category: Some("   ".to_string()),
            ..Default::default()
        };
        let result = CommandValidator::validate(&cmd);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Category cannot be empty"));
    }

    #[test]
    fn test_validate_task_with_long_category() {
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "Test task".to_string(),
            category: Some("a".repeat(51)),
            ..Default::default()
        };
        let result = CommandValidator::validate(&cmd);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Category is too long"));
    }

    #[test]
    fn test_validate_task_with_category_max_length() {
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "Test task".to_string(),
            category: Some("a".repeat(50)),
            ..Default::default()
        };
        assert!(CommandValidator::validate(&cmd).is_ok());
    }

    #[test]
    fn test_validate_task_with_category_spaces() {
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "Test task".to_string(),
            category: Some("work tasks".to_string()),
            ..Default::default()
        };
        assert!(CommandValidator::validate(&cmd).is_ok());
    }

    #[test]
    fn test_validate_task_with_category_tab() {
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "Test task".to_string(),
            category: Some("work\ttasks".to_string()),
            ..Default::default()
        };
        let result = CommandValidator::validate(&cmd);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid whitespace"));
    }

    #[test]
    fn test_validate_task_with_category_newline() {
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "Test task".to_string(),
            category: Some("work\ntasks".to_string()),
            ..Default::default()
        };
        let result = CommandValidator::validate(&cmd);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid whitespace"));
    }

    // === Record-Specific Validation Tests ===

    #[test]
    fn test_validate_valid_record() {
        let cmd = make_record_command("Logged 8 hours of work");
        assert!(CommandValidator::validate(&cmd).is_ok());
    }

    #[test]
    fn test_validate_record_with_deadline() {
        let cmd = NLPCommand {
            action: ActionType::Record,
            content: "Test record".to_string(),
            deadline: Some("today".to_string()),
            ..Default::default()
        };
        let result = CommandValidator::validate(&cmd);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot have deadlines"));
    }

    #[test]
    fn test_validate_record_with_schedule() {
        let cmd = NLPCommand {
            action: ActionType::Record,
            content: "Test record".to_string(),
            schedule: Some("daily".to_string()),
            ..Default::default()
        };
        let result = CommandValidator::validate(&cmd);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot have recurring schedules"));
    }

    #[test]
    fn test_validate_record_content_too_long() {
        let cmd = NLPCommand {
            action: ActionType::Record,
            content: "a".repeat(201),
            ..Default::default()
        };
        let result = CommandValidator::validate(&cmd);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too long"));
    }

    // === Done-Specific Validation Tests ===

    #[test]
    fn test_validate_valid_done() {
        let cmd = make_done_command("Buy groceries");
        assert!(CommandValidator::validate(&cmd).is_ok());
    }

    #[test]
    fn test_validate_done_empty_content() {
        let cmd = NLPCommand {
            action: ActionType::Done,
            content: "".to_string(),
            ..Default::default()
        };
        let result = CommandValidator::validate(&cmd);
        assert!(result.is_err());
        // The basic validation catches this first
        assert!(result.unwrap_err().to_string().contains("Command content is required"));
    }

    #[test]
    fn test_validate_done_whitespace_content() {
        let cmd = NLPCommand {
            action: ActionType::Done,
            content: "   ".to_string(),
            ..Default::default()
        };
        let result = CommandValidator::validate(&cmd);
        assert!(result.is_err());
        // The basic validation catches this first
        assert!(result.unwrap_err().to_string().contains("Command content is required"));
    }

    // === Update-Specific Validation Tests ===

    #[test]
    fn test_validate_valid_update() {
        let mut modifications = HashMap::new();
        modifications.insert("content".to_string(), "Updated task".to_string());
        let cmd = make_update_command("Original task", modifications);
        assert!(CommandValidator::validate(&cmd).is_ok());
    }

    #[test]
    fn test_validate_update_empty_content() {
        let modifications = HashMap::new();
        let cmd = make_update_command("", modifications);
        let result = CommandValidator::validate(&cmd);
        assert!(result.is_err());
        // The basic validation catches this first
        assert!(result.unwrap_err().to_string().contains("Command content is required"));
    }

    #[test]
    fn test_validate_update_no_modifications() {
        let modifications = HashMap::new();
        let cmd = make_update_command("Some task", modifications);
        let result = CommandValidator::validate(&cmd);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("At least one modification"));
    }

    #[test]
    fn test_validate_update_empty_content_modification() {
        let mut modifications = HashMap::new();
        modifications.insert("content".to_string(), "".to_string());
        let cmd = make_update_command("Original task", modifications);
        let result = CommandValidator::validate(&cmd);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Content cannot be empty"));
    }

    #[test]
    fn test_validate_update_too_long_content_modification() {
        let mut modifications = HashMap::new();
        modifications.insert("content".to_string(), "a".repeat(201));
        let cmd = make_update_command("Original task", modifications);
        let result = CommandValidator::validate(&cmd);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too long"));
    }

    #[test]
    fn test_validate_update_empty_category_modification() {
        let mut modifications = HashMap::new();
        modifications.insert("category".to_string(), "".to_string());
        let cmd = make_update_command("Original task", modifications);
        let result = CommandValidator::validate(&cmd);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Category cannot be empty"));
    }

    #[test]
    fn test_validate_update_empty_deadline_modification() {
        let mut modifications = HashMap::new();
        modifications.insert("deadline".to_string(), "".to_string());
        let cmd = make_update_command("Original task", modifications);
        let result = CommandValidator::validate(&cmd);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_validate_update_empty_schedule_modification() {
        let mut modifications = HashMap::new();
        modifications.insert("schedule".to_string(), "".to_string());
        let cmd = make_update_command("Original task", modifications);
        let result = CommandValidator::validate(&cmd);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_validate_update_unknown_modification_type() {
        let mut modifications = HashMap::new();
        modifications.insert("unknown_field".to_string(), "value".to_string());
        let cmd = make_update_command("Original task", modifications);
        let result = CommandValidator::validate(&cmd);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown modification type"));
    }

    #[test]
    fn test_validate_update_multiple_valid_modifications() {
        let mut modifications = HashMap::new();
        modifications.insert("content".to_string(), "Updated content".to_string());
        modifications.insert("category".to_string(), "work".to_string());
        modifications.insert("deadline".to_string(), "tomorrow".to_string());
        let cmd = make_update_command("Original task", modifications);
        assert!(CommandValidator::validate(&cmd).is_ok());
    }

    // === Delete-Specific Validation Tests ===

    #[test]
    fn test_validate_valid_delete_with_content() {
        let cmd = make_delete_command("Some task");
        assert!(CommandValidator::validate(&cmd).is_ok());
    }

    #[test]
    fn test_validate_valid_delete_with_status() {
        let cmd = NLPCommand {
            action: ActionType::Delete,
            content: "".to_string(),
            status: Some(StatusType::Done),
            ..Default::default()
        };
        assert!(CommandValidator::validate(&cmd).is_ok());
    }

    #[test]
    fn test_validate_valid_delete_with_both_content_and_status() {
        let cmd = NLPCommand {
            action: ActionType::Delete,
            content: "Some task".to_string(),
            status: Some(StatusType::Cancelled),
            ..Default::default()
        };
        assert!(CommandValidator::validate(&cmd).is_ok());
    }

    #[test]
    fn test_validate_delete_no_content_or_status() {
        let cmd = NLPCommand {
            action: ActionType::Delete,
            content: "".to_string(),
            status: None,
            ..Default::default()
        };
        let result = CommandValidator::validate(&cmd);
        assert!(result.is_err());
        // The basic validation catches this first (delete without status can't have empty content)
        assert!(result.unwrap_err().to_string().contains("Command content is required"));
    }

    // === List-Specific Validation Tests ===

    #[test]
    fn test_validate_valid_list() {
        let cmd = make_list_command();
        assert!(CommandValidator::validate(&cmd).is_ok());
    }

    #[test]
    fn test_validate_list_days_too_low() {
        let cmd = NLPCommand {
            action: ActionType::List,
            days: Some(0),
            ..Default::default()
        };
        let result = CommandValidator::validate(&cmd);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("between 1 and 365"));
    }

    #[test]
    fn test_validate_list_days_negative() {
        let cmd = NLPCommand {
            action: ActionType::List,
            days: Some(-5),
            ..Default::default()
        };
        let result = CommandValidator::validate(&cmd);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("between 1 and 365"));
    }

    #[test]
    fn test_validate_list_days_too_high() {
        let cmd = NLPCommand {
            action: ActionType::List,
            days: Some(366),
            ..Default::default()
        };
        let result = CommandValidator::validate(&cmd);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("between 1 and 365"));
    }

    #[test]
    fn test_validate_list_days_boundary_low() {
        let cmd = NLPCommand {
            action: ActionType::List,
            days: Some(1),
            ..Default::default()
        };
        assert!(CommandValidator::validate(&cmd).is_ok());
    }

    #[test]
    fn test_validate_list_days_boundary_high() {
        let cmd = NLPCommand {
            action: ActionType::List,
            days: Some(365),
            ..Default::default()
        };
        assert!(CommandValidator::validate(&cmd).is_ok());
    }

    #[test]
    fn test_validate_list_limit_too_low() {
        let cmd = NLPCommand {
            action: ActionType::List,
            limit: Some(0),
            ..Default::default()
        };
        let result = CommandValidator::validate(&cmd);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Limit must be between 1 and 100"));
    }

    #[test]
    fn test_validate_list_limit_too_high() {
        let cmd = NLPCommand {
            action: ActionType::List,
            limit: Some(101),
            ..Default::default()
        };
        let result = CommandValidator::validate(&cmd);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Limit must be between 1 and 100"));
    }

    #[test]
    fn test_validate_list_limit_boundary_low() {
        let cmd = NLPCommand {
            action: ActionType::List,
            limit: Some(1),
            ..Default::default()
        };
        assert!(CommandValidator::validate(&cmd).is_ok());
    }

    #[test]
    fn test_validate_list_limit_boundary_high() {
        let cmd = NLPCommand {
            action: ActionType::List,
            limit: Some(100),
            ..Default::default()
        };
        assert!(CommandValidator::validate(&cmd).is_ok());
    }

    #[test]
    fn test_validate_list_with_valid_category() {
        let cmd = NLPCommand {
            action: ActionType::List,
            category: Some("work".to_string()),
            ..Default::default()
        };
        assert!(CommandValidator::validate(&cmd).is_ok());
    }

    #[test]
    fn test_validate_list_with_invalid_category() {
        let cmd = NLPCommand {
            action: ActionType::List,
            category: Some("".to_string()),
            ..Default::default()
        };
        let result = CommandValidator::validate(&cmd);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Category cannot be empty"));
    }

    // === Suggestion Tests ===

    #[test]
    fn test_suggest_corrections_content_required() {
        let cmd = make_task_command("");
        let error = NLPError::ValidationError("Command content is required for this action".to_string());
        let suggestions = CommandValidator::suggest_corrections(&cmd, &error);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("specify what task")));
    }

    #[test]
    fn test_suggest_corrections_too_long() {
        let cmd = make_task_command(&"a".repeat(201));
        let error = NLPError::ValidationError("Task content is too long".to_string());
        let suggestions = CommandValidator::suggest_corrections(&cmd, &error);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("shorter")));
    }

    #[test]
    fn test_suggest_corrections_deadline_and_schedule() {
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "Test".to_string(),
            deadline: Some("today".to_string()),
            schedule: Some("daily".to_string()),
            ..Default::default()
        };
        let error = NLPError::ValidationError("Task cannot have both a deadline and a recurring schedule".to_string());
        let suggestions = CommandValidator::suggest_corrections(&cmd, &error);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("one-time")));
    }

    #[test]
    fn test_suggest_corrections_record_deadline() {
        let cmd = NLPCommand {
            action: ActionType::Record,
            content: "Test".to_string(),
            deadline: Some("today".to_string()),
            ..Default::default()
        };
        let error = NLPError::ValidationError("Records cannot have deadlines".to_string());
        let suggestions = CommandValidator::suggest_corrections(&cmd, &error);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("logging past events")));
    }

    #[test]
    fn test_suggest_corrections_identifier_required() {
        let cmd = NLPCommand {
            action: ActionType::Done,
            content: "".to_string(),
            ..Default::default()
        };
        let error = NLPError::ValidationError("Task identifier is required".to_string());
        let suggestions = CommandValidator::suggest_corrections(&cmd, &error);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("which task")));
    }

    #[test]
    fn test_suggest_corrections_modifications_required() {
        let cmd = NLPCommand {
            action: ActionType::Update,
            content: "Test".to_string(),
            modifications: HashMap::new(),
            ..Default::default()
        };
        let error = NLPError::ValidationError("At least one modification is required for update".to_string());
        let suggestions = CommandValidator::suggest_corrections(&cmd, &error);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("change")));
    }

    #[test]
    fn test_suggest_corrections_generic_error() {
        let cmd = make_task_command("Test");
        let error = NLPError::APIError("Something went wrong".to_string());
        let suggestions = CommandValidator::suggest_corrections(&cmd, &error);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("rephrasing")));
        assert!(suggestions.iter().any(|s| s.contains("specific terms")));
    }

    // === Category Validation Edge Cases ===

    #[test]
    fn test_validate_category_with_control_char() {
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "Test".to_string(),
            category: Some("work\u{0001}tasks".to_string()),
            ..Default::default()
        };
        let result = CommandValidator::validate(&cmd);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid whitespace"));
    }

    #[test]
    fn test_validate_task_with_deadline_only() {
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "Test task".to_string(),
            deadline: Some("tomorrow".to_string()),
            ..Default::default()
        };
        assert!(CommandValidator::validate(&cmd).is_ok());
    }

    #[test]
    fn test_validate_task_with_schedule_only() {
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "Test task".to_string(),
            schedule: Some("weekly".to_string()),
            ..Default::default()
        };
        assert!(CommandValidator::validate(&cmd).is_ok());
    }
}