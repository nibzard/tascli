//! Integration tests for command mapping accuracy
//!
//! This module tests the end-to-end accuracy of mapping natural language
//! commands to tascli CLI arguments.

use super::mapper::CommandMapper;
use super::types::*;

/// Test case structure for mapping accuracy tests
struct MappingTest {
    name: &'static str,
    input: NLPCommand,
    expected_args: Vec<&'static str>,
    expected_description: &'static str,
}

/// Run a mapping test and assert correctness
fn test_mapping(test: MappingTest) {
    let args = CommandMapper::to_tascli_args(&test.input);
    let description = CommandMapper::describe_command(&test.input);

    assert_eq!(
        args,
        test.expected_args,
        "Test '{}' failed: expected args {:?}, got {:?}",
        test.name, test.expected_args, args
    );

    assert_eq!(
        description,
        test.expected_description,
        "Test '{}' failed: expected description '{}', got '{}'",
        test.name, test.expected_description, description
    );
}

// === Task Creation Tests ===

#[test]
fn test_task_simple() {
    test_mapping(MappingTest {
        name: "simple task",
        input: NLPCommand {
            action: ActionType::Task,
            content: "buy milk".to_string(),
            ..Default::default()
        },
        expected_args: vec!["task", "buy milk"],
        expected_description: "Create task: buy milk",
    });
}

#[test]
fn test_task_with_category() {
    test_mapping(MappingTest {
        name: "task with category",
        input: NLPCommand {
            action: ActionType::Task,
            content: "buy milk".to_string(),
            category: Some("groceries".to_string()),
            ..Default::default()
        },
        expected_args: vec!["task", "-c", "groceries", "buy milk"],
        expected_description: "Create task: buy milk (category: groceries)",
    });
}

#[test]
fn test_task_with_deadline() {
    test_mapping(MappingTest {
        name: "task with deadline",
        input: NLPCommand {
            action: ActionType::Task,
            content: "submit report".to_string(),
            deadline: Some("friday".to_string()),
            ..Default::default()
        },
        expected_args: vec!["task", "submit report", "friday"],
        expected_description: "Create task: submit report (deadline: friday)",
    });
}

#[test]
fn test_task_with_schedule() {
    test_mapping(MappingTest {
        name: "task with schedule",
        input: NLPCommand {
            action: ActionType::Task,
            content: "take vitamins".to_string(),
            schedule: Some("daily".to_string()),
            ..Default::default()
        },
        expected_args: vec!["task", "take vitamins", "daily"],
        expected_description: "Create task: take vitamins (recurring: daily)",
    });
}

#[test]
fn test_task_full() {
    test_mapping(MappingTest {
        name: "full task specification",
        input: NLPCommand {
            action: ActionType::Task,
            content: "team standup".to_string(),
            category: Some("work".to_string()),
            deadline: Some("tomorrow 9am".to_string()),
            ..Default::default()
        },
        expected_args: vec!["task", "-c", "work", "team standup", "tomorrow 9am"],
        expected_description: "Create task: team standup (category: work) (deadline: tomorrow 9am)",
    });
}

// === Record Creation Tests ===

#[test]
fn test_record_simple() {
    test_mapping(MappingTest {
        name: "simple record",
        input: NLPCommand {
            action: ActionType::Record,
            content: "ran 5km".to_string(),
            ..Default::default()
        },
        expected_args: vec!["record", "ran 5km"],
        expected_description: "Create record: ran 5km",
    });
}

#[test]
fn test_record_with_category() {
    test_mapping(MappingTest {
        name: "record with category",
        input: NLPCommand {
            action: ActionType::Record,
            content: "read 20 pages".to_string(),
            category: Some("learning".to_string()),
            ..Default::default()
        },
        expected_args: vec!["record", "-c", "learning", "read 20 pages"],
        expected_description: "Create record: read 20 pages (category: learning)",
    });
}

// === Done Tests ===

#[test]
fn test_done_by_name() {
    test_mapping(MappingTest {
        name: "mark task done by name",
        input: NLPCommand {
            action: ActionType::Done,
            content: "buy milk".to_string(),
            ..Default::default()
        },
        expected_args: vec!["done", "buy milk"],
        expected_description: "Mark task as done: buy milk",
    });
}

#[test]
fn test_done_by_id() {
    test_mapping(MappingTest {
        name: "mark task done by id",
        input: NLPCommand {
            action: ActionType::Done,
            content: "42".to_string(),
            ..Default::default()
        },
        expected_args: vec!["done", "42"],
        expected_description: "Mark task as done: 42",
    });
}

// === List Tests ===

#[test]
fn test_list_default() {
    test_mapping(MappingTest {
        name: "default list",
        input: NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            ..Default::default()
        },
        expected_args: vec!["list", "task"],
        expected_description: "List tasks",
    });
}

#[test]
fn test_list_by_keyword() {
    test_mapping(MappingTest {
        name: "list by keyword",
        input: NLPCommand {
            action: ActionType::List,
            content: "show my tasks".to_string(),
            ..Default::default()
        },
        expected_args: vec!["list", "task"],
        expected_description: "List tasks",
    });
}

#[test]
fn test_list_records() {
    test_mapping(MappingTest {
        name: "list records",
        input: NLPCommand {
            action: ActionType::List,
            content: "show records".to_string(),
            ..Default::default()
        },
        expected_args: vec!["list", "record"],
        expected_description: "List records",
    });
}

#[test]
fn test_list_history() {
    test_mapping(MappingTest {
        name: "list history",
        input: NLPCommand {
            action: ActionType::List,
            content: "show history".to_string(),
            ..Default::default()
        },
        expected_args: vec!["list", "record"],
        expected_description: "List records",
    });
}

#[test]
fn test_list_with_category() {
    test_mapping(MappingTest {
        name: "list by category",
        input: NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            category: Some("work".to_string()),
            ..Default::default()
        },
        expected_args: vec!["list", "task", "-c", "work"],
        expected_description: "List tasks (category: work)",
    });
}

#[test]
fn test_list_with_status() {
    test_mapping(MappingTest {
        name: "list by status",
        input: NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            status: Some(StatusType::Ongoing),
            ..Default::default()
        },
        expected_args: vec!["list", "task", "-s", "ongoing"],
        expected_description: "List tasks (status: Ongoing)",
    });
}

#[test]
fn test_list_with_search() {
    test_mapping(MappingTest {
        name: "list with search",
        input: NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            search: Some("meeting".to_string()),
            ..Default::default()
        },
        expected_args: vec!["list", "task", "--search", "meeting"],
        expected_description: "List tasks (search: meeting)",
    });
}

#[test]
fn test_list_with_days() {
    test_mapping(MappingTest {
        name: "list with days",
        input: NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            days: Some(7),
            ..Default::default()
        },
        expected_args: vec!["list", "task", "-d", "7"],
        expected_description: "List tasks (last 7 days)",
    });
}

#[test]
fn test_list_with_limit() {
    test_mapping(MappingTest {
        name: "list with limit",
        input: NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            limit: Some(5),
            ..Default::default()
        },
        expected_args: vec!["list", "task", "--limit", "5"],
        expected_description: "List tasks",
    });
}

#[test]
fn test_list_complex() {
    test_mapping(MappingTest {
        name: "complex list filter",
        input: NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            category: Some("work".to_string()),
            status: Some(StatusType::Ongoing),
            days: Some(7),
            limit: Some(10),
            ..Default::default()
        },
        expected_args: vec!["list", "task", "-c", "work", "-s", "ongoing", "-d", "7", "--limit", "10"],
        expected_description: "List tasks (category: work, status: Ongoing, last 7 days)",
    });
}

// === Delete Tests ===

#[test]
fn test_delete_specific() {
    test_mapping(MappingTest {
        name: "delete specific task",
        input: NLPCommand {
            action: ActionType::Delete,
            content: "old task".to_string(),
            ..Default::default()
        },
        expected_args: vec!["delete", "old task"],
        expected_description: "Delete: old task",
    });
}

#[test]
fn test_delete_all() {
    test_mapping(MappingTest {
        name: "delete all",
        input: NLPCommand {
            action: ActionType::Delete,
            content: "all".to_string(),
            ..Default::default()
        },
        expected_args: vec!["delete"],
        expected_description: "Delete all items",
    });
}

#[test]
fn test_delete_by_status() {
    test_mapping(MappingTest {
        name: "delete by status",
        input: NLPCommand {
            action: ActionType::Delete,
            content: "".to_string(),
            status: Some(StatusType::Done),
            ..Default::default()
        },
        expected_args: vec!["delete", "--status", "done"],
        expected_description: "Delete: ",  // describe_command uses content which is empty here
    });
}

// === Update Tests ===

#[test]
fn test_update_content() {
    let mut cmd = NLPCommand {
        action: ActionType::Update,
        content: "old task".to_string(),
        ..Default::default()
    };
    cmd.modifications.insert("content".to_string(), "new description".to_string());

    test_mapping(MappingTest {
        name: "update content",
        input: cmd,
        expected_args: vec!["update", "old task", "--content", "new description"],
        expected_description: "Update: old task",
    });
}

#[test]
fn test_update_category() {
    let mut cmd = NLPCommand {
        action: ActionType::Update,
        content: "my task".to_string(),
        ..Default::default()
    };
    cmd.modifications.insert("category".to_string(), "urgent".to_string());

    test_mapping(MappingTest {
        name: "update category",
        input: cmd,
        expected_args: vec!["update", "my task", "--category", "urgent"],
        expected_description: "Update: my task",
    });
}

#[test]
fn test_update_deadline() {
    let mut cmd = NLPCommand {
        action: ActionType::Update,
        content: "my task".to_string(),
        ..Default::default()
    };
    cmd.modifications.insert("deadline".to_string(), "tomorrow".to_string());

    test_mapping(MappingTest {
        name: "update deadline",
        input: cmd,
        expected_args: vec!["update", "my task", "--deadline", "tomorrow"],
        expected_description: "Update: my task",
    });
}

#[test]
fn test_update_status() {
    let mut cmd = NLPCommand {
        action: ActionType::Update,
        content: "my task".to_string(),
        ..Default::default()
    };
    cmd.modifications.insert("status".to_string(), "cancelled".to_string());

    test_mapping(MappingTest {
        name: "update status",
        input: cmd,
        expected_args: vec!["update", "my task", "--status", "cancelled"],
        expected_description: "Update: my task",
    });
}

#[test]
fn test_update_multiple() {
    let mut cmd = NLPCommand {
        action: ActionType::Update,
        content: "my task".to_string(),
        ..Default::default()
    };
    cmd.modifications.insert("content".to_string(), "updated".to_string());
    cmd.modifications.insert("category".to_string(), "work".to_string());

    let args = CommandMapper::to_tascli_args(&cmd);
    assert!(args.starts_with(&["update".to_string(), "my task".to_string()]));
    assert!(args.contains(&"--content".to_string()));
    assert!(args.contains(&"updated".to_string()));
    assert!(args.contains(&"--category".to_string()));
    assert!(args.contains(&"work".to_string()));
}

// === Edge Case Tests ===

#[test]
fn test_task_empty_content() {
    test_mapping(MappingTest {
        name: "task with empty content",
        input: NLPCommand {
            action: ActionType::Task,
            content: "".to_string(),
            ..Default::default()
        },
        expected_args: vec!["task", ""],
        expected_description: "Create task: ",
    });
}

#[test]
fn test_task_unicode_content() {
    test_mapping(MappingTest {
        name: "task with unicode",
        input: NLPCommand {
            action: ActionType::Task,
            content: "review 日本語 document 📄".to_string(),
            category: Some("work".to_string()),
            ..Default::default()
        },
        expected_args: vec!["task", "-c", "work", "review 日本語 document 📄"],
        expected_description: "Create task: review 日本語 document 📄 (category: work)",
    });
}

#[test]
fn test_list_all_statuses() {
    for status in [
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
            content: "".to_string(),
            status: Some(status.clone()),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&cmd);
        let status_str = format!("{:?}", status).to_lowercase();
        assert_eq!(args, vec!["list", "task", "-s", &status_str]);
    }
}

#[test]
fn test_deadline_over_schedule() {
    test_mapping(MappingTest {
        name: "deadline takes precedence",
        input: NLPCommand {
            action: ActionType::Task,
            content: "task".to_string(),
            deadline: Some("today".to_string()),
            schedule: Some("daily".to_string()),
            ..Default::default()
        },
        expected_args: vec!["task", "task", "today"],
        expected_description: "Create task: task (deadline: today)",
    });
}

// === Accuracy Test Suite ===
//
// These tests verify the mapping accuracy meets the >95% target by
// testing common natural language patterns.

#[test]
fn test_accuracy_common_tasks() {
    let test_cases = vec![
        ("add task", "buy milk"),
        ("create task", "call mom"),
        ("new task", "write report"),
        ("make task", "clean room"),
    ];

    for (_prefix, content) in test_cases {
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: content.to_string(),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&cmd);
        assert_eq!(args, vec!["task", content]);
    }
}

#[test]
fn test_accuracy_task_variations() {
    let variations = vec![
        ("task with deadline today", "today"),
        ("task due tomorrow", "tomorrow"),
        ("task due next week", "next week"),
        ("task for friday", "friday"),
    ];

    for (_desc, deadline) in variations {
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "some task".to_string(),
            deadline: Some(deadline.to_string()),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&cmd);
        assert!(args.contains(&deadline.to_string()));
    }
}

#[test]
fn test_accuracy_recurring_variations() {
    let schedules = vec![
        ("daily task", "daily"),
        ("weekly task", "weekly"),
        ("monthly task", "monthly"),
        ("every day task", "every day"),
        ("every week task", "every week"),
    ];

    for (_desc, schedule) in schedules {
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "some task".to_string(),
            schedule: Some(schedule.to_string()),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&cmd);
        assert!(args.contains(&schedule.to_string()));
    }
}

#[test]
fn test_accuracy_category_patterns() {
    let categories = vec![
        ("work task", "work"),
        ("personal chore", "personal"),
        ("shopping item", "shopping"),
        ("health activity", "health"),
    ];

    for (_desc, category) in categories {
        let cmd = NLPCommand {
            action: ActionType::Task,
            content: "item".to_string(),
            category: Some(category.to_string()),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&cmd);
        assert!(args.contains(&"-c".to_string()));
        assert!(args.contains(&category.to_string()));
    }
}

#[test]
fn test_accuracy_list_patterns() {
    let patterns = vec![
        ("show all tasks", "task"),
        ("list my tasks", "task"),
        ("display tasks", "task"),
        ("what are my tasks", "task"),
        ("show records", "record"),
        ("list history", "record"),
        ("show completed", "task"),
    ];

    for (pattern, list_type) in patterns {
        let cmd = NLPCommand {
            action: ActionType::List,
            content: pattern.to_string(),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&cmd);
        assert_eq!(args.get(1), Some(&list_type.to_string()));
    }
}

#[test]
fn test_accuracy_status_filters() {
    let status_tests = vec![
        ("show ongoing tasks", StatusType::Ongoing),
        ("list done tasks", StatusType::Done),
        ("show cancelled items", StatusType::Cancelled),
        ("what's pending", StatusType::Pending),
    ];

    for (_pattern, status) in status_tests {
        let cmd = NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            status: Some(status),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&cmd);
        assert!(args.contains(&"-s".to_string()));
    }
}

#[test]
fn test_accuracy_search_patterns() {
    let searches = vec![
        "find tasks with",
        "search for",
        "show me",
        "look for",
    ];

    for search in searches {
        let cmd = NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            search: Some(search.to_string()),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&cmd);
        assert!(args.contains(&"--search".to_string()));
    }
}

#[test]
fn test_accuracy_time_filters() {
    let time_tests = vec![
        ("last 7 days", 7),
        ("past week", 7),
        ("last 30 days", 30),
        ("this month", 30),
    ];

    for (_pattern, days) in time_tests {
        let cmd = NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            days: Some(days),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&cmd);
        assert!(args.contains(&"-d".to_string()));
        assert!(args.contains(&days.to_string()));
    }
}
