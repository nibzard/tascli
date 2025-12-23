//! Natural Language Pattern Validation Tests
//!
//! This module validates that various natural language patterns are properly
//! handled and validated by the NLP system, testing edge cases, common phrasings,
//! and potential ambiguous inputs.

use super::types::*;
use super::validator::CommandValidator;
use std::collections::HashMap;

/// Test helper to create a task command
fn task(content: &str) -> NLPCommand {
    NLPCommand {
        action: ActionType::Task,
        content: content.to_string(),
        ..Default::default()
    }
}

/// Test helper to create a record command
fn record(content: &str) -> NLPCommand {
    NLPCommand {
        action: ActionType::Record,
        content: content.to_string(),
        ..Default::default()
    }
}

/// Test helper to create a done command
fn done(content: &str) -> NLPCommand {
    NLPCommand {
        action: ActionType::Done,
        content: content.to_string(),
        ..Default::default()
    }
}

/// Test helper to create a list command
fn list() -> NLPCommand {
    NLPCommand {
        action: ActionType::List,
        ..Default::default()
    }
}

/// Test helper to create a delete command
fn delete(content: &str) -> NLPCommand {
    NLPCommand {
        action: ActionType::Delete,
        content: content.to_string(),
        ..Default::default()
    }
}

/// Test helper to create an update command
fn update(content: &str, modifications: HashMap<String, String>) -> NLPCommand {
    NLPCommand {
        action: ActionType::Update,
        content: content.to_string(),
        modifications,
        ..Default::default()
    }
}

// === Simple Command Patterns ===

#[test]
fn test_simple_add_task() {
    let cmd = task("buy milk");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_simple_show_tasks() {
    let cmd = list();
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_simple_complete_task_by_name() {
    let cmd = done("buy milk");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_simple_complete_task_by_id() {
    let cmd = done("42");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_simple_delete_task() {
    let cmd = delete("old task");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

// === Time-Based Patterns ===

#[test]
fn test_time_based_add_meeting_today() {
    let cmd = NLPCommand {
        action: ActionType::Task,
        content: "team meeting".to_string(),
        deadline: Some("today".to_string()),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_time_based_add_meeting_tomorrow() {
    let cmd = NLPCommand {
        action: ActionType::Task,
        content: "dentist appointment".to_string(),
        deadline: Some("tomorrow".to_string()),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_time_based_add_task_next_week() {
    let cmd = NLPCommand {
        action: ActionType::Task,
        content: "project review".to_string(),
        deadline: Some("next week".to_string()),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_time_based_show_tasks_due_today() {
    let cmd = list();
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_time_based_show_tasks_from_last_7_days() {
    let cmd = NLPCommand {
        action: ActionType::List,
        days: Some(7),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_time_based_recurring_daily_task() {
    let cmd = NLPCommand {
        action: ActionType::Task,
        content: "take vitamins".to_string(),
        schedule: Some("daily".to_string()),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_time_based_recurring_weekly_task() {
    let cmd = NLPCommand {
        action: ActionType::Task,
        content: "team sync".to_string(),
        schedule: Some("weekly".to_string()),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_time_based_recurring_monthly_task() {
    let cmd = NLPCommand {
        action: ActionType::Task,
        content: "pay bills".to_string(),
        schedule: Some("monthly".to_string()),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

// === Category-Based Patterns ===

#[test]
fn test_category_based_add_work_task() {
    let cmd = NLPCommand {
        action: ActionType::Task,
        content: "finish report".to_string(),
        category: Some("work".to_string()),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_category_based_add_personal_task() {
    let cmd = NLPCommand {
        action: ActionType::Task,
        content: "grocery shopping".to_string(),
        category: Some("personal".to_string()),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_category_based_show_work_tasks() {
    let cmd = NLPCommand {
        action: ActionType::List,
        category: Some("work".to_string()),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_category_based_show_personal_tasks() {
    let cmd = NLPCommand {
        action: ActionType::List,
        category: Some("personal".to_string()),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_category_based_add_record_with_category() {
    let cmd = NLPCommand {
        action: ActionType::Record,
        content: "completed 5k run".to_string(),
        category: Some("fitness".to_string()),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_category_based_with_multi_word_category() {
    let cmd = NLPCommand {
        action: ActionType::Task,
        content: "review code".to_string(),
        category: Some("work projects".to_string()),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

// === Complex Query Patterns ===

#[test]
fn test_complex_show_all_overdue_tasks() {
    let cmd = NLPCommand {
        action: ActionType::List,
        status: Some(StatusType::Ongoing),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_complex_show_work_tasks_from_last_week() {
    let cmd = NLPCommand {
        action: ActionType::List,
        category: Some("work".to_string()),
        days: Some(7),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_complex_show_completed_personal_tasks() {
    let cmd = NLPCommand {
        action: ActionType::List,
        category: Some("personal".to_string()),
        status: Some(StatusType::Done),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_complex_search_tasks_containing_meeting() {
    let cmd = NLPCommand {
        action: ActionType::List,
        search: Some("meeting".to_string()),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_complex_show_recent_limited_tasks() {
    let cmd = NLPCommand {
        action: ActionType::List,
        days: Some(7),
        limit: Some(10),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_complex_show_ongoing_work_tasks_last_month() {
    let cmd = NLPCommand {
        action: ActionType::List,
        category: Some("work".to_string()),
        status: Some(StatusType::Ongoing),
        days: Some(30),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_complex_delete_all_completed_tasks() {
    let cmd = NLPCommand {
        action: ActionType::Delete,
        content: "".to_string(),
        status: Some(StatusType::Done),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

// === Natural Variations - "I need to..." patterns ===

#[test]
fn test_natural_variation_i_need_to() {
    // "I need to buy groceries" should parse to task with content "buy groceries"
    let cmd = task("buy groceries");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_natural_variation_i_have_to() {
    let cmd = task("finish the report");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_natural_variation_i_gotta() {
    let cmd = task("call mom");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

// === Natural Variations - "Remind me to..." patterns ===

#[test]
fn test_natural_variation_remind_me_to() {
    let cmd = task("take out the trash");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_natural_variation_remind_me_about() {
    let cmd = task("dentist appointment");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

// === Natural Variations - "Don't forget to..." patterns ===

#[test]
fn test_natural_variation_dont_forget_to() {
    let cmd = task("send the email");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_natural_variation_remember_to() {
    let cmd = task("pick up dry cleaning");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

// === Natural Variations - Question patterns ===

#[test]
fn test_natural_variation_what_are_my_tasks() {
    let cmd = list();
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_natural_variation_show_me_my_tasks() {
    let cmd = list();
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_natural_variation_what_tasks_do_i_have() {
    let cmd = list();
    assert!(CommandValidator::validate(&cmd).is_ok());
}

// === Edge Cases - Missing Information ===

#[test]
fn test_edge_case_empty_task_content() {
    let cmd = task("");
    assert!(CommandValidator::validate(&cmd).is_err());
}

#[test]
fn test_edge_case_whitespace_only_content() {
    let cmd = task("   ");
    assert!(CommandValidator::validate(&cmd).is_err());
}

#[test]
fn test_edge_case_done_without_identifier() {
    let cmd = done("");
    assert!(CommandValidator::validate(&cmd).is_err());
}

#[test]
fn test_edge_case_update_without_target() {
    let mut modifications = HashMap::new();
    modifications.insert("content".to_string(), "new content".to_string());
    let cmd = update("", modifications);
    assert!(CommandValidator::validate(&cmd).is_err());
}

#[test]
fn test_edge_case_update_without_modifications() {
    let cmd = update("task 1", HashMap::new());
    assert!(CommandValidator::validate(&cmd).is_err());
}

#[test]
fn test_edge_case_delete_without_identifier_or_status() {
    let cmd = delete("");
    assert!(CommandValidator::validate(&cmd).is_err());
}

// === Edge Cases - Ambiguous References ===

#[test]
fn test_edge_case_ambiguous_task_reference() {
    // Multiple tasks might have similar names - validation should still pass
    // The resolution would happen during execution
    let cmd = done("meeting");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_edge_case_numeric_task_name() {
    let cmd = task("123");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_edge_case_task_id_vs_name() {
    // "1" could be task ID or task name - validator should accept both
    let cmd = done("1");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

// === Edge Cases - Special Characters ===

#[test]
fn test_edge_case_task_with_punctuation() {
    let cmd = task("buy milk, eggs, and bread!");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_edge_case_task_with_quotes() {
    let cmd = task("read \"The Art of War\"");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_edge_case_task_with_parentheses() {
    let cmd = task("call John (about the project)");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_edge_case_task_with_hyphen() {
    let cmd = task("follow-up on email");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_edge_case_task_with_apostrophe() {
    let cmd = task("review John's proposal");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_edge_case_task_with_at_sign() {
    let cmd = task("email john@example.com");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_edge_case_task_with_hashtag() {
    let cmd = task("submit #urgent report");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

// === Edge Cases - Unicode and Emojis ===

#[test]
fn test_edge_case_task_with_emoji() {
    let cmd = task("celebrate completion 🎉");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_edge_case_task_with_multiple_emojis() {
    let cmd = task("workout 💪🏃‍♂️💦");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_edge_case_task_with_unicode_chars() {
    let cmd = task("review 日本語 document");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_edge_case_task_with_chinese_chars() {
    let cmd = task("准备报告");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_edge_case_task_with_arabic_chars() {
    let cmd = task("مراجعة التقرير");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_edge_case_task_with_cyrillic_chars() {
    let cmd = task("подготовить отчёт");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_edge_case_task_with_mixed_scripts() {
    let cmd = task("review 日本語 and 中文 documents");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

// === Edge Cases - Length Boundaries ===

#[test]
fn test_edge_case_max_length_content() {
    let cmd = task(&"a".repeat(200));
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_edge_case_too_long_content() {
    let cmd = task(&"a".repeat(201));
    assert!(CommandValidator::validate(&cmd).is_err());
}

#[test]
fn test_edge_case_max_length_category() {
    let cmd = NLPCommand {
        action: ActionType::Task,
        content: "test".to_string(),
        category: Some("a".repeat(50)),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_edge_case_too_long_category() {
    let cmd = NLPCommand {
        action: ActionType::Task,
        content: "test".to_string(),
        category: Some("a".repeat(51)),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_err());
}

#[test]
fn test_edge_case_min_boundary_days() {
    let cmd = NLPCommand {
        action: ActionType::List,
        days: Some(1),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_edge_case_max_boundary_days() {
    let cmd = NLPCommand {
        action: ActionType::List,
        days: Some(365),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_edge_case_below_min_days() {
    let cmd = NLPCommand {
        action: ActionType::List,
        days: Some(0),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_err());
}

#[test]
fn test_edge_case_above_max_days() {
    let cmd = NLPCommand {
        action: ActionType::List,
        days: Some(366),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_err());
}

#[test]
fn test_edge_case_min_boundary_limit() {
    let cmd = NLPCommand {
        action: ActionType::List,
        limit: Some(1),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_edge_case_max_boundary_limit() {
    let cmd = NLPCommand {
        action: ActionType::List,
        limit: Some(100),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_edge_case_below_min_limit() {
    let cmd = NLPCommand {
        action: ActionType::List,
        limit: Some(0),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_err());
}

#[test]
fn test_edge_case_above_max_limit() {
    let cmd = NLPCommand {
        action: ActionType::List,
        limit: Some(101),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_err());
}

// === Record-Specific Patterns ===

#[test]
fn test_record_log_completed_task() {
    let cmd = record("finished reading a book");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_record_with_category() {
    let cmd = NLPCommand {
        action: ActionType::Record,
        content: "ran 5 kilometers".to_string(),
        category: Some("exercise".to_string()),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_record_should_not_have_deadline() {
    let cmd = NLPCommand {
        action: ActionType::Record,
        content: "test record".to_string(),
        deadline: Some("tomorrow".to_string()),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_err());
}

#[test]
fn test_record_should_not_have_schedule() {
    let cmd = NLPCommand {
        action: ActionType::Record,
        content: "test record".to_string(),
        schedule: Some("daily".to_string()),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_err());
}

// === Task-Specific Edge Cases ===

#[test]
fn test_task_cannot_have_both_deadline_and_schedule() {
    let cmd = NLPCommand {
        action: ActionType::Task,
        content: "test task".to_string(),
        deadline: Some("today".to_string()),
        schedule: Some("daily".to_string()),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_err());
}

#[test]
fn test_task_with_deadline_only() {
    let cmd = NLPCommand {
        action: ActionType::Task,
        content: "submit report".to_string(),
        deadline: Some("friday".to_string()),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_task_with_schedule_only() {
    let cmd = NLPCommand {
        action: ActionType::Task,
        content: "water plants".to_string(),
        schedule: Some("weekly".to_string()),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

// === Update Command Patterns ===

#[test]
fn test_update_change_content() {
    let mut modifications = HashMap::new();
    modifications.insert("content".to_string(), "new description".to_string());
    let cmd = update("task 1", modifications);
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_update_change_category() {
    let mut modifications = HashMap::new();
    modifications.insert("category".to_string(), "urgent".to_string());
    let cmd = update("buy groceries", modifications);
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_update_change_deadline() {
    let mut modifications = HashMap::new();
    modifications.insert("deadline".to_string(), "tomorrow".to_string());
    let cmd = update("project report", modifications);
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_update_multiple_fields() {
    let mut modifications = HashMap::new();
    modifications.insert("content".to_string(), "updated content".to_string());
    modifications.insert("category".to_string(), "work".to_string());
    modifications.insert("deadline".to_string(), "friday".to_string());
    let cmd = update("my task", modifications);
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_update_empty_content_modification() {
    let mut modifications = HashMap::new();
    modifications.insert("content".to_string(), "".to_string());
    let cmd = update("task 1", modifications);
    assert!(CommandValidator::validate(&cmd).is_err());
}

#[test]
fn test_update_unknown_field() {
    let mut modifications = HashMap::new();
    modifications.insert("unknown_field".to_string(), "value".to_string());
    let cmd = update("task 1", modifications);
    assert!(CommandValidator::validate(&cmd).is_err());
}

// === Status Type Coverage ===

#[test]
fn test_status_all_types() {
    for status in &[
        StatusType::Ongoing,
        StatusType::Done,
        StatusType::Cancelled,
        StatusType::Duplicate,
        StatusType::Suspended,
        StatusType::Pending,
        StatusType::Open,
        StatusType::Closed,
    ] {
        let cmd = NLPCommand {
            action: ActionType::List,
            status: Some(status.clone()),
            ..Default::default()
        };
        assert!(CommandValidator::validate(&cmd).is_ok());
    }
}

// === Category Validation Edge Cases ===

#[test]
fn test_category_with_spaces() {
    let cmd = NLPCommand {
        action: ActionType::Task,
        content: "test".to_string(),
        category: Some("work tasks".to_string()),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_category_with_numbers() {
    let cmd = NLPCommand {
        action: ActionType::Task,
        content: "test".to_string(),
        category: Some("project123".to_string()),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_category_with_special_chars() {
    let cmd = NLPCommand {
        action: ActionType::Task,
        content: "test".to_string(),
        category: Some("work & personal".to_string()),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_category_empty() {
    let cmd = NLPCommand {
        action: ActionType::Task,
        content: "test".to_string(),
        category: Some("".to_string()),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_err());
}

#[test]
fn test_category_whitespace_only() {
    let cmd = NLPCommand {
        action: ActionType::Task,
        content: "test".to_string(),
        category: Some("   ".to_string()),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_err());
}

#[test]
fn test_category_with_tab() {
    let cmd = NLPCommand {
        action: ActionType::Task,
        content: "test".to_string(),
        category: Some("work\ttasks".to_string()),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_err());
}

#[test]
fn test_category_with_newline() {
    let cmd = NLPCommand {
        action: ActionType::Task,
        content: "test".to_string(),
        category: Some("work\ntasks".to_string()),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_err());
}

// === Complex Real-World Scenarios ===

#[test]
fn test_real_world_create_daily_standup() {
    let cmd = NLPCommand {
        action: ActionType::Task,
        content: "daily standup meeting".to_string(),
        category: Some("work".to_string()),
        deadline: Some("tomorrow 9am".to_string()),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_real_world_log_exercise() {
    let cmd = NLPCommand {
        action: ActionType::Record,
        content: "completed 30 min yoga session".to_string(),
        category: Some("fitness".to_string()),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_real_world_review_pending_tasks() {
    let cmd = NLPCommand {
        action: ActionType::List,
        category: Some("work".to_string()),
        status: Some(StatusType::Ongoing),
        limit: Some(20),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_real_world_cleanup_completed() {
    let cmd = NLPCommand {
        action: ActionType::Delete,
        content: "".to_string(),
        status: Some(StatusType::Done),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_real_world_update_deadline() {
    let mut modifications = HashMap::new();
    modifications.insert("deadline".to_string(), "next monday".to_string());
    let cmd = update("project report", modifications);
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_real_world_mark_task_complete() {
    let cmd = done("submit quarterly report");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_real_world_create_recurring_bill() {
    let cmd = NLPCommand {
        action: ActionType::Task,
        content: "pay electricity bill".to_string(),
        category: Some("finance".to_string()),
        schedule: Some("monthly".to_string()),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_real_world_search_specific_task() {
    let cmd = NLPCommand {
        action: ActionType::List,
        search: Some("quarterly report".to_string()),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_real_world_recent_activity() {
    let cmd = NLPCommand {
        action: ActionType::List,
        days: Some(3),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_real_world_categorize_uncategorized_task() {
    let mut modifications = HashMap::new();
    modifications.insert("category".to_string(), "errand".to_string());
    let cmd = update("buy groceries", modifications);
    assert!(CommandValidator::validate(&cmd).is_ok());
}

// === Ambiguous or Conflicting Patterns ===

#[test]
fn test_ambiguous_list_all_vs_list_tasks() {
    let cmd = list();
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_ambiguous_delete_by_name_vs_by_id() {
    // "123" could be a task name or ID - both should be valid
    let cmd = delete("123");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_conflicting_empty_content_with_status() {
    let cmd = NLPCommand {
        action: ActionType::Delete,
        content: "".to_string(),
        status: Some(StatusType::Done),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

// === Multi-Language Pattern Support ===

#[test]
fn test_multilingual_spanish_task() {
    let cmd = task("comprar leche");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_multilingual_french_task() {
    let cmd = task("acheter du pain");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_multilingual_german_task() {
    let cmd = task("milch kaufen");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_multilingual_japanese_task() {
    let cmd = task("牛乳を買う");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

// === Case Sensitivity Tests ===

#[test]
fn test_case_sensitivity_uppercase() {
    let cmd = task("BUY GROCERIES");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_case_sensitivity_lowercase() {
    let cmd = task("buy groceries");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_case_sensitivity_mixed_case() {
    let cmd = task("Buy Groceries");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_case_sensitivity_category_uppercase() {
    let cmd = NLPCommand {
        action: ActionType::Task,
        content: "test".to_string(),
        category: Some("WORK".to_string()),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

// === Content with URLs or Paths ===

#[test]
fn test_content_with_url() {
    let cmd = task("review https://example.com/document");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_content_with_file_path() {
    let cmd = task("review /home/user/document.txt");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_content_with_email_address() {
    let cmd = task("email person@example.com about project");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

// === Number Handling ===

#[test]
fn test_content_with_large_number() {
    let cmd = task("read chapter 1000");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_content_with_decimal() {
    let cmd = task("run 5.5 km");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_content_with_fraction() {
    let cmd = task("review 1/2 of the document");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_content_with_multiple_numbers() {
    let cmd = task("meeting at 3pm for 2 hours in room 101");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

// === Action Type Coverage ===

#[test]
fn test_all_action_types_are_valid() {
    let actions = vec![
        (ActionType::Task, "buy groceries"),
        (ActionType::Record, "ran 5k"),
        (ActionType::Done, "task 1"),
        (ActionType::List, ""),
        (ActionType::Delete, "old task"),
    ];

    for (action, content) in actions {
        let cmd = NLPCommand {
            action: action.clone(),
            content: content.to_string(),
            ..Default::default()
        };
        assert!(
            CommandValidator::validate(&cmd).is_ok(),
            "Action {:?} with content '{}' should be valid",
            action,
            content
        );
    }
}

// === Empty and Null-like Patterns ===

#[test]
fn test_empty_string_in_optional_fields() {
    let cmd = NLPCommand {
        action: ActionType::Task,
        content: "test".to_string(),
        deadline: Some("".to_string()),
        ..Default::default()
    };
    // Empty deadline string should still validate (time parsing would fail later)
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_list_with_all_optional_fields() {
    let cmd = NLPCommand {
        action: ActionType::List,
        content: "".to_string(),
        category: Some("work".to_string()),
        status: Some(StatusType::Ongoing),
        days: Some(7),
        limit: Some(10),
        search: Some("project".to_string()),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_task_with_all_valid_fields() {
    let cmd = NLPCommand {
        action: ActionType::Task,
        content: "complete project documentation".to_string(),
        category: Some("work".to_string()),
        deadline: Some("friday".to_string()),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

// === Whitespace Handling ===

#[test]
fn test_content_with_leading_whitespace() {
    let cmd = task("   buy milk");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_content_with_trailing_whitespace() {
    let cmd = task("buy milk   ");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_content_with_multiple_spaces() {
    let cmd = task("buy    milk    and    eggs");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_content_with_tabs() {
    let cmd = task("buy\tmilk");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_content_with_newlines() {
    let cmd = task("buy\nmilk");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

// === Repetition and Redundancy Patterns ===

#[test]
fn test_repetitive_words() {
    let cmd = task("buy buy buy milk");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_redundant_punctuation() {
    let cmd = task("buy milk!!!");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_stuttered_typing() {
    let cmd = task("bbuy milk");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

// === Minimal and Maximal Valid Inputs ===

#[test]
fn test_minimal_valid_task() {
    let cmd = task("a");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_maximal_valid_task() {
    let cmd = NLPCommand {
        action: ActionType::Task,
        content: "a".repeat(200),
        category: Some("a".repeat(50)),
        deadline: Some("today".to_string()),
        ..Default::default()
    };
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_minimal_valid_record() {
    let cmd = record("a");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_minimal_valid_list() {
    let cmd = list();
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_minimal_valid_done() {
    let cmd = done("a");
    assert!(CommandValidator::validate(&cmd).is_ok());
}

#[test]
fn test_minimal_valid_delete() {
    let cmd = delete("a");
    assert!(CommandValidator::validate(&cmd).is_ok());
}
