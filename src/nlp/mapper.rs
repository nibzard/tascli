//! Command mapper that converts NLP commands to tascli CLI arguments

use super::types::*;

pub struct CommandMapper;

impl CommandMapper {
    /// Convert an NLP command to tascli CLI arguments
    pub fn to_tascli_args(command: &NLPCommand) -> Vec<String> {
        let mut args = Vec::new();

        match command.action {
            ActionType::Task => {
                args.push("task".to_string());

                // Add category if specified
                if let Some(category) = &command.category {
                    args.push("-c".to_string());
                    args.push(category.clone());
                }

                // Add content
                args.push(command.content.clone());

                // Add deadline or schedule
                if let Some(deadline) = &command.deadline {
                    args.push(deadline.clone());
                } else if let Some(schedule) = &command.schedule {
                    args.push(schedule.clone());
                }
            },

            ActionType::Record => {
                args.push("record".to_string());

                if let Some(category) = &command.category {
                    args.push("-c".to_string());
                    args.push(category.clone());
                }

                args.push(command.content.clone());
            },

            ActionType::Done => {
                args.push("done".to_string());

                // Try to find the task by content or ID
                if !command.content.is_empty() {
                    args.push(command.content.clone());
                }
            },

            ActionType::List => {
                args.push("list".to_string());

                // Determine what to list (task or record)
                let list_type = if command.content.contains("record") ||
                                  command.content.contains("records") ||
                                  command.content.contains("history") {
                    "record"
                } else {
                    "task"
                };

                args.push(list_type.to_string());

                // Handle complex query types
                if let Some(query_type) = &command.query_type {
                    match query_type {
                        QueryType::Overdue => {
                            // Overdue: status=ongoing, target_time_max=now
                            args.push("-s".to_string());
                            args.push("ongoing".to_string());
                            args.push("--target-time-max".to_string());
                            args.push("now".to_string());
                        },
                        QueryType::Upcoming => {
                            // Upcoming: status=ongoing, target_time_min=now, target_time_max=+7days
                            args.push("-s".to_string());
                            args.push("ongoing".to_string());
                            args.push("--target-time-min".to_string());
                            args.push("now".to_string());
                            args.push("--target-time-max".to_string());
                            args.push("+7d".to_string());
                        },
                        QueryType::Unscheduled => {
                            // Unscheduled: target_time is null
                            args.push("--no-deadline".to_string());
                        },
                        QueryType::DueToday => {
                            // Due today: target_time_min=today, target_time_max=today
                            args.push("--target-time-min".to_string());
                            args.push("today".to_string());
                            args.push("--target-time-max".to_string());
                            args.push("today".to_string());
                        },
                        QueryType::DueTomorrow => {
                            // Due tomorrow: target_time_min=tomorrow, target_time_max=tomorrow
                            args.push("--target-time-min".to_string());
                            args.push("tomorrow".to_string());
                            args.push("--target-time-max".to_string());
                            args.push("tomorrow".to_string());
                        },
                        QueryType::DueThisWeek => {
                            // Due this week: target_time_min=now, target_time_max=+7d
                            args.push("--target-time-min".to_string());
                            args.push("now".to_string());
                            args.push("--target-time-max".to_string());
                            args.push("+7d".to_string());
                        },
                        QueryType::DueThisMonth => {
                            // Due this month: target_time_min=now, target_time-max=eom
                            args.push("--target-time-min".to_string());
                            args.push("now".to_string());
                            args.push("--target-time-max".to_string());
                            args.push("eom".to_string());
                        },
                        QueryType::Urgent => {
                            // Urgent: overdue or due very soon (today/tomorrow)
                            args.push("-s".to_string());
                            args.push("ongoing".to_string());
                            args.push("--target-time-max".to_string());
                            args.push("tomorrow".to_string());
                        },
                        QueryType::All => {
                            // No additional filters needed
                        },
                    }
                }

                // Add category filter
                if let Some(category) = &command.category {
                    args.push("-c".to_string());
                    args.push(category.clone());
                }

                // Add search filter
                if let Some(search) = &command.search {
                    args.push("--search".to_string());
                    args.push(search.clone());
                }

                // Add status filter (only if not already set by query_type)
                if let Some(status) = &command.status {
                    if command.query_type.is_none() || !matches!(command.query_type, Some(QueryType::Overdue | QueryType::Upcoming | QueryType::Urgent)) {
                        args.push("-s".to_string());
                        args.push(format!("{:?}", status).to_lowercase());
                    }
                }

                // Add days filter
                if let Some(days) = command.days {
                    args.push("-d".to_string());
                    args.push(days.to_string());
                }

                // Add limit
                if let Some(limit) = command.limit {
                    args.push("--limit".to_string());
                    args.push(limit.to_string());
                }
            },

            ActionType::Delete => {
                args.push("delete".to_string());

                // Add filters if specified
                if let Some(status) = &command.status {
                    args.push("--status".to_string());
                    args.push(format!("{:?}", status).to_lowercase());
                }

                // Otherwise try to delete by content
                if !command.content.is_empty() && command.content != "all" {
                    args.push(command.content.clone());
                }
            },

            ActionType::Update => {
                args.push("update".to_string());

                // Target to update (content or ID)
                if !command.content.is_empty() {
                    args.push(command.content.clone());
                }

                // Apply modifications
                for (key, value) in &command.modifications {
                    match key.as_str() {
                        "content" => {
                            args.push("--content".to_string());
                            args.push(value.clone());
                        },
                        "category" => {
                            args.push("--category".to_string());
                            args.push(value.clone());
                        },
                        "deadline" => {
                            args.push("--deadline".to_string());
                            args.push(value.clone());
                        },
                        "status" => {
                            args.push("--status".to_string());
                            args.push(value.clone());
                        },
                        _ => {}
                    }
                }
            },
        }

        args
    }

    /// Generate a human-readable description of what the command will do
    pub fn describe_command(command: &NLPCommand) -> String {
        match command.action {
            ActionType::Task => {
                let mut desc = format!("Create task: {}", command.content);

                if let Some(category) = &command.category {
                    desc.push_str(&format!(" (category: {})", category));
                }

                if let Some(deadline) = &command.deadline {
                    desc.push_str(&format!(" (deadline: {})", deadline));
                } else if let Some(schedule) = &command.schedule {
                    desc.push_str(&format!(" (recurring: {})", schedule));
                }

                desc
            },

            ActionType::Record => {
                let mut desc = format!("Create record: {}", command.content);
                if let Some(category) = &command.category {
                    desc.push_str(&format!(" (category: {})", category));
                }
                desc
            },

            ActionType::Done => {
                format!("Mark task as done: {}", command.content)
            },

            ActionType::List => {
                let item_type = if command.content.contains("record") || command.content.contains("history") {
                    "records"
                } else {
                    "tasks"
                };
                let mut desc = format!("List {}", item_type);

                let mut filters = Vec::new();

                if let Some(query_type) = &command.query_type {
                    let query_desc = match query_type {
                        QueryType::Overdue => "overdue",
                        QueryType::Upcoming => "upcoming",
                        QueryType::Unscheduled => "unscheduled",
                        QueryType::DueToday => "due today",
                        QueryType::DueTomorrow => "due tomorrow",
                        QueryType::DueThisWeek => "due this week",
                        QueryType::DueThisMonth => "due this month",
                        QueryType::Urgent => "urgent",
                        QueryType::All => "all",
                    };
                    filters.push(query_desc.to_string());
                }

                if let Some(category) = &command.category {
                    filters.push(format!("category: {}", category));
                }

                if let Some(status) = &command.status {
                    filters.push(format!("status: {:?}", status));
                }

                if let Some(search) = &command.search {
                    filters.push(format!("search: {}", search));
                }

                if let Some(days) = command.days {
                    filters.push(format!("last {} days", days));
                }

                if !filters.is_empty() {
                    desc.push_str(&format!(" ({})", filters.join(", ")));
                }

                desc
            },

            ActionType::Delete => {
                if command.content == "all" {
                    "Delete all items".to_string()
                } else {
                    format!("Delete: {}", command.content)
                }
            },

            ActionType::Update => {
                format!("Update: {}", command.content)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_mapping() {
        let command = NLPCommand {
            action: ActionType::Task,
            content: "cleanup the trash".to_string(),
            category: Some("home".to_string()),
            deadline: Some("today".to_string()),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["task", "-c", "home", "cleanup the trash", "today"]);
    }

    #[test]
    fn test_recurring_task_mapping() {
        let command = NLPCommand {
            action: ActionType::Task,
            content: "write journal".to_string(),
            schedule: Some("daily".to_string()),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["task", "write journal", "daily"]);
    }

    #[test]
    fn test_list_mapping() {
        let command = NLPCommand {
            action: ActionType::List,
            content: "tasks".to_string(),
            category: Some("work".to_string()),
            status: Some(StatusType::Ongoing),
            days: Some(7),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["list", "task", "-c", "work", "-s", "ongoing", "-d", "7"]);
    }

    // === Task Mapping Tests ===

    #[test]
    fn test_task_mapping_minimal() {
        let command = NLPCommand {
            action: ActionType::Task,
            content: "buy groceries".to_string(),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["task", "buy groceries"]);
    }

    #[test]
    fn test_task_mapping_with_category() {
        let command = NLPCommand {
            action: ActionType::Task,
            content: "buy groceries".to_string(),
            category: Some("home".to_string()),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["task", "-c", "home", "buy groceries"]);
    }

    #[test]
    fn test_task_mapping_with_deadline() {
        let command = NLPCommand {
            action: ActionType::Task,
            content: "buy groceries".to_string(),
            deadline: Some("today".to_string()),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["task", "buy groceries", "today"]);
    }

    #[test]
    fn test_task_mapping_with_schedule() {
        let command = NLPCommand {
            action: ActionType::Task,
            content: "exercise".to_string(),
            schedule: Some("weekly".to_string()),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["task", "exercise", "weekly"]);
    }

    #[test]
    fn test_task_mapping_with_all_options() {
        let command = NLPCommand {
            action: ActionType::Task,
            content: "team meeting".to_string(),
            category: Some("work".to_string()),
            deadline: Some("tomorrow".to_string()),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["task", "-c", "work", "team meeting", "tomorrow"]);
    }

    #[test]
    fn test_task_mapping_deadline_takes_precedence_over_schedule() {
        let command = NLPCommand {
            action: ActionType::Task,
            content: "task".to_string(),
            deadline: Some("today".to_string()),
            schedule: Some("daily".to_string()),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        // Only deadline should be included
        assert_eq!(args, vec!["task", "task", "today"]);
        assert!(!args.contains(&"daily".to_string()));
    }

    // === Record Mapping Tests ===

    #[test]
    fn test_record_mapping_minimal() {
        let command = NLPCommand {
            action: ActionType::Record,
            content: "completed 5km run".to_string(),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["record", "completed 5km run"]);
    }

    #[test]
    fn test_record_mapping_with_category() {
        let command = NLPCommand {
            action: ActionType::Record,
            content: "logged work hours".to_string(),
            category: Some("work".to_string()),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["record", "-c", "work", "logged work hours"]);
    }

    // === Done Mapping Tests ===

    #[test]
    fn test_done_mapping() {
        let command = NLPCommand {
            action: ActionType::Done,
            content: "buy groceries".to_string(),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["done", "buy groceries"]);
    }

    #[test]
    fn test_done_mapping_with_id() {
        let command = NLPCommand {
            action: ActionType::Done,
            content: "42".to_string(),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["done", "42"]);
    }

    // === List Mapping Tests ===

    #[test]
    fn test_list_mapping_minimal() {
        let command = NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["list", "task"]);
    }

    #[test]
    fn test_list_mapping_with_category() {
        let command = NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            category: Some("work".to_string()),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["list", "task", "-c", "work"]);
    }

    #[test]
    fn test_list_mapping_with_status() {
        let command = NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            status: Some(StatusType::Done),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["list", "task", "-s", "done"]);
    }

    #[test]
    fn test_list_mapping_with_search() {
        let command = NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            search: Some("groceries".to_string()),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["list", "task", "--search", "groceries"]);
    }

    #[test]
    fn test_list_mapping_with_days() {
        let command = NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            days: Some(14),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["list", "task", "-d", "14"]);
    }

    #[test]
    fn test_list_mapping_with_limit() {
        let command = NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            limit: Some(10),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["list", "task", "--limit", "10"]);
    }

    #[test]
    fn test_list_mapping_records() {
        let command = NLPCommand {
            action: ActionType::List,
            content: "show me my records".to_string(),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["list", "record"]);
    }

    #[test]
    fn test_list_mapping_history() {
        let command = NLPCommand {
            action: ActionType::List,
            content: "show my history".to_string(),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["list", "record"]);
    }

    #[test]
    fn test_list_mapping_all_filters() {
        let command = NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            category: Some("work".to_string()),
            status: Some(StatusType::Ongoing),
            search: Some("meeting".to_string()),
            days: Some(7),
            limit: Some(20),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["list", "task", "-c", "work", "--search", "meeting", "-s", "ongoing", "-d", "7", "--limit", "20"]);
    }

    #[test]
    fn test_list_mapping_status_cancelled() {
        let command = NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            status: Some(StatusType::Cancelled),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["list", "task", "-s", "cancelled"]);
    }

    #[test]
    fn test_list_mapping_status_pending() {
        let command = NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            status: Some(StatusType::Pending),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["list", "task", "-s", "pending"]);
    }

    // === Complex Query Mapping Tests ===

    #[test]
    fn test_list_mapping_overdue() {
        let command = NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            query_type: Some(QueryType::Overdue),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["list", "task", "-s", "ongoing", "--target-time-max", "now"]);
    }

    #[test]
    fn test_list_mapping_overdue_with_category() {
        let command = NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            query_type: Some(QueryType::Overdue),
            category: Some("work".to_string()),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["list", "task", "-s", "ongoing", "--target-time-max", "now", "-c", "work"]);
    }

    #[test]
    fn test_list_mapping_upcoming() {
        let command = NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            query_type: Some(QueryType::Upcoming),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["list", "task", "-s", "ongoing", "--target-time-min", "now", "--target-time-max", "+7d"]);
    }

    #[test]
    fn test_list_mapping_unscheduled() {
        let command = NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            query_type: Some(QueryType::Unscheduled),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["list", "task", "--no-deadline"]);
    }

    #[test]
    fn test_list_mapping_due_today() {
        let command = NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            query_type: Some(QueryType::DueToday),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["list", "task", "--target-time-min", "today", "--target-time-max", "today"]);
    }

    #[test]
    fn test_list_mapping_due_tomorrow() {
        let command = NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            query_type: Some(QueryType::DueTomorrow),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["list", "task", "--target-time-min", "tomorrow", "--target-time-max", "tomorrow"]);
    }

    #[test]
    fn test_list_mapping_due_this_week() {
        let command = NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            query_type: Some(QueryType::DueThisWeek),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["list", "task", "--target-time-min", "now", "--target-time-max", "+7d"]);
    }

    #[test]
    fn test_list_mapping_due_this_month() {
        let command = NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            query_type: Some(QueryType::DueThisMonth),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["list", "task", "--target-time-min", "now", "--target-time-max", "eom"]);
    }

    #[test]
    fn test_list_mapping_urgent() {
        let command = NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            query_type: Some(QueryType::Urgent),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["list", "task", "-s", "ongoing", "--target-time-max", "tomorrow"]);
    }

    // === Delete Mapping Tests ===

    #[test]
    fn test_delete_mapping_with_content() {
        let command = NLPCommand {
            action: ActionType::Delete,
            content: "buy groceries".to_string(),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["delete", "buy groceries"]);
    }

    #[test]
    fn test_delete_mapping_with_status() {
        let command = NLPCommand {
            action: ActionType::Delete,
            content: "".to_string(),
            status: Some(StatusType::Done),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["delete", "--status", "done"]);
    }

    #[test]
    fn test_delete_mapping_all() {
        let command = NLPCommand {
            action: ActionType::Delete,
            content: "all".to_string(),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["delete"]);
    }

    #[test]
    fn test_delete_mapping_with_content_and_status() {
        let command = NLPCommand {
            action: ActionType::Delete,
            content: "old task".to_string(),
            status: Some(StatusType::Cancelled),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["delete", "--status", "cancelled", "old task"]);
    }

    // === Update Mapping Tests ===

    #[test]
    fn test_update_mapping_with_content_modification() {
        let mut command = NLPCommand {
            action: ActionType::Update,
            content: "old task".to_string(),
            ..Default::default()
        };
        command.modifications.insert("content".to_string(), "new task description".to_string());

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["update", "old task", "--content", "new task description"]);
    }

    #[test]
    fn test_update_mapping_with_category_modification() {
        let mut command = NLPCommand {
            action: ActionType::Update,
            content: "my task".to_string(),
            ..Default::default()
        };
        command.modifications.insert("category".to_string(), "work".to_string());

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["update", "my task", "--category", "work"]);
    }

    #[test]
    fn test_update_mapping_with_deadline_modification() {
        let mut command = NLPCommand {
            action: ActionType::Update,
            content: "my task".to_string(),
            ..Default::default()
        };
        command.modifications.insert("deadline".to_string(), "tomorrow".to_string());

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["update", "my task", "--deadline", "tomorrow"]);
    }

    #[test]
    fn test_update_mapping_with_status_modification() {
        let mut command = NLPCommand {
            action: ActionType::Update,
            content: "my task".to_string(),
            ..Default::default()
        };
        command.modifications.insert("status".to_string(), "cancelled".to_string());

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["update", "my task", "--status", "cancelled"]);
    }

    #[test]
    fn test_update_mapping_multiple_modifications() {
        let mut command = NLPCommand {
            action: ActionType::Update,
            content: "my task".to_string(),
            ..Default::default()
        };
        command.modifications.insert("content".to_string(), "updated task".to_string());
        command.modifications.insert("category".to_string(), "urgent".to_string());
        command.modifications.insert("deadline".to_string(), "today".to_string());

        let args = CommandMapper::to_tascli_args(&command);
        // Order is preserved in HashMap iteration for small maps in Rust
        assert!(args.starts_with(&["update".to_string(), "my task".to_string()]));
        assert!(args.contains(&"--content".to_string()));
        assert!(args.contains(&"updated task".to_string()));
        assert!(args.contains(&"--category".to_string()));
        assert!(args.contains(&"urgent".to_string()));
        assert!(args.contains(&"--deadline".to_string()));
        assert!(args.contains(&"today".to_string()));
    }

    #[test]
    fn test_update_mapping_unknown_modification_ignored() {
        let mut command = NLPCommand {
            action: ActionType::Update,
            content: "my task".to_string(),
            ..Default::default()
        };
        command.modifications.insert("unknown_field".to_string(), "value".to_string());

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["update", "my task"]);
    }

    // === Describe Command Tests ===

    #[test]
    fn test_describe_task_minimal() {
        let command = NLPCommand {
            action: ActionType::Task,
            content: "buy groceries".to_string(),
            ..Default::default()
        };

        let desc = CommandMapper::describe_command(&command);
        assert_eq!(desc, "Create task: buy groceries");
    }

    #[test]
    fn test_describe_task_with_category() {
        let command = NLPCommand {
            action: ActionType::Task,
            content: "buy groceries".to_string(),
            category: Some("home".to_string()),
            ..Default::default()
        };

        let desc = CommandMapper::describe_command(&command);
        assert_eq!(desc, "Create task: buy groceries (category: home)");
    }

    #[test]
    fn test_describe_task_with_deadline() {
        let command = NLPCommand {
            action: ActionType::Task,
            content: "buy groceries".to_string(),
            deadline: Some("today".to_string()),
            ..Default::default()
        };

        let desc = CommandMapper::describe_command(&command);
        assert_eq!(desc, "Create task: buy groceries (deadline: today)");
    }

    #[test]
    fn test_describe_task_with_schedule() {
        let command = NLPCommand {
            action: ActionType::Task,
            content: "exercise".to_string(),
            schedule: Some("daily".to_string()),
            ..Default::default()
        };

        let desc = CommandMapper::describe_command(&command);
        assert_eq!(desc, "Create task: exercise (recurring: daily)");
    }

    #[test]
    fn test_describe_task_with_all() {
        let command = NLPCommand {
            action: ActionType::Task,
            content: "meeting".to_string(),
            category: Some("work".to_string()),
            deadline: Some("tomorrow".to_string()),
            ..Default::default()
        };

        let desc = CommandMapper::describe_command(&command);
        assert_eq!(desc, "Create task: meeting (category: work) (deadline: tomorrow)");
    }

    #[test]
    fn test_describe_record_minimal() {
        let command = NLPCommand {
            action: ActionType::Record,
            content: "completed workout".to_string(),
            ..Default::default()
        };

        let desc = CommandMapper::describe_command(&command);
        assert_eq!(desc, "Create record: completed workout");
    }

    #[test]
    fn test_describe_record_with_category() {
        let command = NLPCommand {
            action: ActionType::Record,
            content: "completed workout".to_string(),
            category: Some("health".to_string()),
            ..Default::default()
        };

        let desc = CommandMapper::describe_command(&command);
        assert_eq!(desc, "Create record: completed workout (category: health)");
    }

    #[test]
    fn test_describe_done() {
        let command = NLPCommand {
            action: ActionType::Done,
            content: "buy groceries".to_string(),
            ..Default::default()
        };

        let desc = CommandMapper::describe_command(&command);
        assert_eq!(desc, "Mark task as done: buy groceries");
    }

    #[test]
    fn test_describe_list_tasks() {
        let command = NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            ..Default::default()
        };

        let desc = CommandMapper::describe_command(&command);
        assert_eq!(desc, "List tasks");
    }

    #[test]
    fn test_describe_list_records() {
        let command = NLPCommand {
            action: ActionType::List,
            content: "show my records".to_string(),
            ..Default::default()
        };

        let desc = CommandMapper::describe_command(&command);
        assert_eq!(desc, "List records");
    }

    #[test]
    fn test_describe_list_with_filters() {
        let command = NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            category: Some("work".to_string()),
            status: Some(StatusType::Ongoing),
            ..Default::default()
        };

        let desc = CommandMapper::describe_command(&command);
        assert!(desc.contains("List tasks"));
        assert!(desc.contains("category: work"));
        assert!(desc.contains("status: Ongoing"));
    }

    #[test]
    fn test_describe_list_with_search() {
        let command = NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            search: Some("meeting".to_string()),
            ..Default::default()
        };

        let desc = CommandMapper::describe_command(&command);
        assert!(desc.contains("search: meeting"));
    }

    #[test]
    fn test_describe_list_with_days() {
        let command = NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            days: Some(7),
            ..Default::default()
        };

        let desc = CommandMapper::describe_command(&command);
        assert!(desc.contains("last 7 days"));
    }

    #[test]
    fn test_describe_list_with_query_type_overdue() {
        let command = NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            query_type: Some(QueryType::Overdue),
            ..Default::default()
        };

        let desc = CommandMapper::describe_command(&command);
        assert!(desc.contains("overdue"));
    }

    #[test]
    fn test_describe_list_with_query_type_upcoming() {
        let command = NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            query_type: Some(QueryType::Upcoming),
            ..Default::default()
        };

        let desc = CommandMapper::describe_command(&command);
        assert!(desc.contains("upcoming"));
    }

    #[test]
    fn test_describe_list_with_query_type_and_category() {
        let command = NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            query_type: Some(QueryType::Overdue),
            category: Some("work".to_string()),
            ..Default::default()
        };

        let desc = CommandMapper::describe_command(&command);
        assert!(desc.contains("overdue"));
        assert!(desc.contains("category: work"));
    }

    #[test]
    fn test_describe_delete_specific() {
        let command = NLPCommand {
            action: ActionType::Delete,
            content: "old task".to_string(),
            ..Default::default()
        };

        let desc = CommandMapper::describe_command(&command);
        assert_eq!(desc, "Delete: old task");
    }

    #[test]
    fn test_describe_delete_all() {
        let command = NLPCommand {
            action: ActionType::Delete,
            content: "all".to_string(),
            ..Default::default()
        };

        let desc = CommandMapper::describe_command(&command);
        assert_eq!(desc, "Delete all items");
    }

    #[test]
    fn test_describe_update() {
        let command = NLPCommand {
            action: ActionType::Update,
            content: "my task".to_string(),
            ..Default::default()
        };

        let desc = CommandMapper::describe_command(&command);
        assert_eq!(desc, "Update: my task");
    }

    // === Edge Cases ===

    #[test]
    fn test_task_with_empty_content() {
        let command = NLPCommand {
            action: ActionType::Task,
            content: "".to_string(),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["task", ""]);
    }

    #[test]
    fn test_task_with_special_characters() {
        let command = NLPCommand {
            action: ActionType::Task,
            content: "task with \"quotes\" and 'apostrophes'".to_string(),
            category: Some("test & demo".to_string()),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["task", "-c", "test & demo", "task with \"quotes\" and 'apostrophes'"]);
    }

    #[test]
    fn test_list_with_unicode_status() {
        let command = NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            status: Some(StatusType::Suspended),
            ..Default::default()
        };

        let args = CommandMapper::to_tascli_args(&command);
        assert_eq!(args, vec!["list", "task", "-s", "suspended"]);
    }
}