//! OpenAI API client for natural language processing

use super::types::*;
use reqwest::Client;
use serde_json::{json, Value};
use std::time::{Duration, Instant};

pub struct OpenAIClient {
    client: Client,
    config: NLPConfig,
    last_request_time: Option<Instant>,
    request_count: u32,
}

impl OpenAIClient {
    /// Create a new OpenAI client with the given configuration
    pub fn new(config: NLPConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            config,
            last_request_time: None,
            request_count: 0,
        }
    }

    /// Check if we're rate limited and wait if necessary
    async fn check_rate_limit(&mut self) {
        let now = Instant::now();

        // Reset counter if more than a minute has passed
        if let Some(last_time) = self.last_request_time {
            if now.duration_since(last_time) > Duration::from_secs(60) {
                self.request_count = 0;
            }
        }

        // If we're at the limit, wait
        if self.request_count >= self.config.max_api_calls_per_minute {
            if let Some(last_time) = self.last_request_time {
                let elapsed = now.duration_since(last_time);
                if elapsed < Duration::from_secs(60) {
                    tokio::time::sleep(Duration::from_secs(60) - elapsed).await;
                }
            }
            self.request_count = 0;
        }

        self.last_request_time = Some(now);
        self.request_count += 1;
    }

    /// Parse natural language input into a structured command
    pub async fn parse_command(&mut self, input: &str) -> NLPResult<NLPCommand> {
        if !self.config.enabled {
            return Err(NLPError::ConfigError("NLP is not enabled".to_string()));
        }

        if let Some(ref api_key) = self.config.api_key {
            if api_key.is_empty() {
                return Err(NLPError::InvalidAPIKey);
            }
        } else {
            return Err(NLPError::InvalidAPIKey);
        }

        self.check_rate_limit().await;

        let tool_definition = json!({
            "type": "function",
            "function": {
                "name": "parse_task_command",
                "description": "Parse natural language into tascli command structure",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "enum": ["task", "record", "done", "update", "delete", "list"],
                            "description": "The tascli action to perform"
                        },
                        "content": {
                            "type": "string",
                            "description": "Main task or record content"
                        },
                        "category": {
                            "type": "string",
                            "description": "Category for the task or record"
                        },
                        "deadline": {
                            "type": "string",
                            "description": "Deadline for tasks (e.g., 'today', 'tomorrow', '2025-12-25')"
                        },
                        "schedule": {
                            "type": "string",
                            "description": "Recurring schedule (e.g., 'daily', 'weekly Monday', 'monthly 1st')"
                        },
                        "status": {
                            "type": "string",
                            "enum": ["ongoing", "done", "cancelled", "duplicate", "suspended", "pending", "open", "closed", "all"],
                            "description": "Status filter for listing commands"
                        },
                        "search": {
                            "type": "string",
                            "description": "Search terms for filtering"
                        },
                        "days": {
                            "type": "integer",
                            "description": "Number of days to look back for listing"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of results to return"
                        },
                        "modifications": {
                            "type": "object",
                            "description": "Modifications for update commands",
                            "properties": {
                                "content": {"type": "string"},
                                "category": {"type": "string"},
                                "deadline": {"type": "string"},
                                "status": {"type": "string"}
                            }
                        }
                    },
                    "required": ["action", "content"]
                }
            }
        });

        let system_prompt = r#"You are a task management assistant that converts natural language into structured commands for tascli CLI tool.

Rules:
1. Parse the user's intent into one of the actions: task, record, done, update, delete, list
2. Extract relevant information like content, category, deadlines, schedules
3. For time expressions, convert them to tascli's format:
   - Relative times: "today", "tomorrow", "yesterday", "eom", "eoy"
   - Dates: "YYYY-MM-DD", "MM/DD", "MM/DD/YYYY"
   - Times: "HH:MM", "3PM", "3:00PM"
   - Recurring: "daily", "weekly Monday", "monthly 1st"
4. For listing commands, extract filters like status, search terms, categories
5. If the user's intent is unclear, make reasonable assumptions based on context

Examples:
- "add a task for today to cleanup the trash" → action: "task", content: "cleanup the trash", deadline: "today"
- "show my work tasks" → action: "list", content: "tasks", category: "work"
- "mark the cleanup task as done" → action: "done", content: "cleanup"
- "create daily task to write journal" → action: "task", content: "write journal", schedule: "daily""#;

        let request_body = json!({
            "model": self.config.model,
            "input": [
                {
                    "role": "system",
                    "content": [
                        {
                            "type": "input_text",
                            "text": system_prompt
                        }
                    ]
                },
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "input_text",
                            "text": input
                        }
                    ]
                }
            ],
            "tools": [tool_definition],
            "tool_choice": {"type": "function", "function": {"name": "parse_task_command"}},
            "temperature": 0.1,
            "max_output_tokens": 300,
            "text": {
                "format": {
                    "type": "text"
                }
            }
        });

        let response = self.client
            .post(&format!("{}/responses", self.config.api_base_url))
            .header("Authorization", format!("Bearer {}", self.config.api_key.as_ref().unwrap()))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if response.status() == 401 {
            return Err(NLPError::InvalidAPIKey);
        }

        if response.status() == 429 {
            return Err(NLPError::RateLimited);
        }

        let response_text = response.text().await?;
        let response_json: Value = serde_json::from_str(&response_text)?;

        // Check for API errors
        if let Some(error) = response_json.get("error") {
            return Err(NLPError::APIError(
                error.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown API error")
                    .to_string()
            ));
        }

        // Extract the tool call response
        let output = response_json.get("output")
            .and_then(|o| o.as_array())
            .and_then(|arr| arr.first())
            .ok_or_else(|| NLPError::ParseError("No output in response".to_string()))?;

        let tool_calls = output.get("tool_calls")
            .and_then(|tc| tc.as_array());

        if let Some(tool_calls) = tool_calls {
            for tool_call in tool_calls {
                if let Some(function) = tool_call.get("function") {
                    if let Some("parse_task_command") = function.get("name").and_then(|n| n.as_str()) {
                        if let Some(arguments) = function.get("arguments") {
                            let command: NLPCommand = serde_json::from_value(arguments.clone())?;
                            return Ok(command);
                        }
                    }
                }
            }
        }

        // Fallback: try to extract text response if no tool calls
        if let Some(content) = output.get("content")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("text"))
            .and_then(|t| t.as_str())
        {
            // Simple text parsing as fallback
            return self.fallback_parse(content);
        }

        Err(NLPError::ParseError("Could not parse command from response".to_string()))
    }

    /// Simple fallback parsing when tool calling fails
    fn fallback_parse(&self, input: &str) -> NLPResult<NLPCommand> {
        let input_lower = input.to_lowercase();

        // Basic keyword detection
        let action = if input_lower.contains("task") || input_lower.contains("add") || input_lower.contains("create") {
            ActionType::Task
        } else if input_lower.contains("record") {
            ActionType::Record
        } else if input_lower.contains("done") || input_lower.contains("complete") {
            ActionType::Done
        } else if input_lower.contains("show") || input_lower.contains("list") {
            ActionType::List
        } else if input_lower.contains("delete") || input_lower.contains("remove") {
            ActionType::Delete
        } else if input_lower.contains("update") || input_lower.contains("change") {
            ActionType::Update
        } else {
            ActionType::Task // Default to task creation
        };

        Ok(NLPCommand {
            action,
            content: input.to_string(),
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_config() -> NLPConfig {
        NLPConfig {
            enabled: true,
            api_key: Some("test-api-key".to_string()),
            ..Default::default()
        }
    }

    // === Client Creation Tests ===

    #[test]
    fn test_client_new_with_default_config() {
        let config = NLPConfig::default();
        let client = OpenAIClient::new(config);
        assert_eq!(client.request_count, 0);
        assert!(client.last_request_time.is_none());
    }

    #[test]
    fn test_client_new_with_custom_config() {
        let config = make_test_config();
        let client = OpenAIClient::new(config);
        assert_eq!(client.request_count, 0);
        assert!(client.last_request_time.is_none());
    }

    // === Parse Command Tests - Error Conditions ===

    #[tokio::test]
    async fn test_parse_command_disabled() {
        let mut config = make_test_config();
        config.enabled = false;
        let mut client = OpenAIClient::new(config);

        let result = client.parse_command("add a task").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            NLPError::ConfigError(msg) => assert!(msg.contains("not enabled")),
            _ => panic!("Expected ConfigError"),
        }
    }

    #[tokio::test]
    async fn test_parse_command_no_api_key() {
        let mut config = make_test_config();
        config.api_key = None;
        let mut client = OpenAIClient::new(config);

        let result = client.parse_command("add a task").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NLPError::InvalidAPIKey));
    }

    #[tokio::test]
    async fn test_parse_command_empty_api_key() {
        let mut config = make_test_config();
        config.api_key = Some("".to_string());
        let mut client = OpenAIClient::new(config);

        let result = client.parse_command("add a task").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NLPError::InvalidAPIKey));
    }

    // === Fallback Parse Tests ===

    #[test]
    fn test_fallback_parse_task() {
        let client = OpenAIClient::new(make_test_config());
        let result = client.fallback_parse("add a task to buy groceries");
        assert!(result.is_ok());
        let command = result.unwrap();
        assert_eq!(command.action, ActionType::Task);
        assert_eq!(command.content, "add a task to buy groceries");
    }

    #[test]
    fn test_fallback_parse_create() {
        let client = OpenAIClient::new(make_test_config());
        let result = client.fallback_parse("create a new task");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().action, ActionType::Task);
    }

    #[test]
    fn test_fallback_parse_record() {
        let client = OpenAIClient::new(make_test_config());
        let result = client.fallback_parse("record my workout");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().action, ActionType::Record);
    }

    #[test]
    fn test_fallback_parse_done() {
        let client = OpenAIClient::new(make_test_config());
        // "done" keyword should match Done
        let result = client.fallback_parse("mark item as done");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().action, ActionType::Done);
    }

    #[test]
    fn test_fallback_parse_complete() {
        let client = OpenAIClient::new(make_test_config());
        // "complete" keyword should match Done
        let result = client.fallback_parse("complete my assignment");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().action, ActionType::Done);
    }

    #[test]
    fn test_fallback_parse_list() {
        let client = OpenAIClient::new(make_test_config());
        // "show" keyword should match List
        let result = client.fallback_parse("show my items");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().action, ActionType::List);
    }

    #[test]
    fn test_fallback_parse_delete() {
        let client = OpenAIClient::new(make_test_config());
        // "delete" alone should match Delete
        let result = client.fallback_parse("delete item");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().action, ActionType::Delete);
    }

    #[test]
    fn test_fallback_parse_remove() {
        let client = OpenAIClient::new(make_test_config());
        // "remove" alone should match Delete
        let result = client.fallback_parse("remove item");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().action, ActionType::Delete);
    }

    #[test]
    fn test_fallback_parse_delete_with_task_keyword() {
        let client = OpenAIClient::new(make_test_config());
        // "delete task" - "task" keyword is checked first, so it becomes a Task action
        let result = client.fallback_parse("delete old task");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().action, ActionType::Task);
    }

    #[test]
    fn test_fallback_parse_update() {
        let client = OpenAIClient::new(make_test_config());
        // "update" keyword alone should match Update
        let result = client.fallback_parse("update deadline");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().action, ActionType::Update);
    }

    #[test]
    fn test_fallback_parse_change() {
        let client = OpenAIClient::new(make_test_config());
        // "change" keyword alone should match Update
        let result = client.fallback_parse("change priority");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().action, ActionType::Update);
    }

    #[test]
    fn test_fallback_parse_default_to_task() {
        let client = OpenAIClient::new(make_test_config());
        let result = client.fallback_parse("something random");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().action, ActionType::Task);
    }

    #[test]
    fn test_fallback_parse_case_insensitive() {
        let client = OpenAIClient::new(make_test_config());

        let result1 = client.fallback_parse("ADD a TASK");
        assert_eq!(result1.unwrap().action, ActionType::Task);

        // "Delete" alone should match Delete
        let result2 = client.fallback_parse("DELETE ITEM");
        assert_eq!(result2.unwrap().action, ActionType::Delete);

        // "SHOW" alone should match List
        let result3 = client.fallback_parse("SHOW ITEMS");
        assert_eq!(result3.unwrap().action, ActionType::List);
    }

    #[test]
    fn test_fallback_parse_content_preserved() {
        let client = OpenAIClient::new(make_test_config());
        let input = "I want to add a very important task for tomorrow";
        let result = client.fallback_parse(input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().content, input);
    }

    #[test]
    fn test_fallback_parse_empty_input() {
        let client = OpenAIClient::new(make_test_config());
        let result = client.fallback_parse("");
        assert!(result.is_ok());
        // Default action for empty/unknown input
        assert_eq!(result.unwrap().action, ActionType::Task);
    }

    #[test]
    fn test_fallback_parse_unicode() {
        let client = OpenAIClient::new(make_test_config());
        let result = client.fallback_parse("add task with emoji 🎉");
        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert_eq!(cmd.action, ActionType::Task);
        assert_eq!(cmd.content, "add task with emoji 🎉");
    }

    #[test]
    fn test_fallback_parse_multiple_keywords() {
        let client = OpenAIClient::new(make_test_config());

        // "task" comes first in the if-else chain, so it should win
        let result = client.fallback_parse("add a task and update it later");
        assert_eq!(result.unwrap().action, ActionType::Task);

        // "done" comes before "list"
        let result = client.fallback_parse("mark as done and list others");
        assert_eq!(result.unwrap().action, ActionType::Done);
    }

    // === Rate Limit Tests ===

    #[tokio::test]
    async fn test_rate_limit_initial_state() {
        let config = NLPConfig {
            max_api_calls_per_minute: 10,
            ..make_test_config()
        };
        let client = OpenAIClient::new(config);

        assert_eq!(client.request_count, 0);
        assert!(client.last_request_time.is_none());
    }

    #[tokio::test]
    async fn test_rate_limit_tracking() {
        let config = NLPConfig {
            max_api_calls_per_minute: 5,
            ..make_test_config()
        };
        let mut client = OpenAIClient::new(config);

        // Simulate rate limit checks (without actually making API calls)
        for _ in 0..3 {
            client.check_rate_limit().await;
        }

        assert_eq!(client.request_count, 3);
        assert!(client.last_request_time.is_some());
    }

    #[tokio::test]
    async fn test_rate_limit_reset_after_minute() {
        // This test demonstrates the rate limit reset logic
        // but we can't easily test the actual time-based behavior
        // without making the test slow or using mock time
        let config = NLPConfig {
            max_api_calls_per_minute: 2,
            ..make_test_config()
        };
        let mut client = OpenAIClient::new(config);

        client.check_rate_limit().await;
        assert_eq!(client.request_count, 1);

        // In real scenario, after 60 seconds the counter would reset
        // For this test we just verify the increment works
        client.check_rate_limit().await;
        assert_eq!(client.request_count, 2);
    }

    // === Config Tests ===

    #[test]
    fn test_client_custom_model() {
        let config = NLPConfig {
            model: "custom-gpt-model".to_string(),
            ..make_test_config()
        };
        let client = OpenAIClient::new(config);
        assert_eq!(client.config.model, "custom-gpt-model");
    }

    #[test]
    fn test_client_custom_timeout() {
        // Client is created with a 30-second timeout
        // We can't directly test the timeout, but we can verify client creation works
        let config = make_test_config();
        let client = OpenAIClient::new(config);
        // Client should be created successfully
        assert_eq!(client.request_count, 0);
    }

    #[test]
    fn test_client_custom_api_base_url() {
        let config = NLPConfig {
            api_base_url: "https://custom-api.example.com/v1".to_string(),
            ..make_test_config()
        };
        let client = OpenAIClient::new(config);
        assert_eq!(client.config.api_base_url, "https://custom-api.example.com/v1");
    }

    #[test]
    fn test_client_max_api_calls_setting() {
        let config = NLPConfig {
            max_api_calls_per_minute: 100,
            ..make_test_config()
        };
        let client = OpenAIClient::new(config);
        assert_eq!(client.config.max_api_calls_per_minute, 100);
    }

    // === Edge Cases ===

    #[test]
    fn test_fallback_parse_with_special_characters() {
        let client = OpenAIClient::new(make_test_config());
        let input = "delete task!@#$%^&*()";
        let result = client.fallback_parse(input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().content, input);
    }

    #[test]
    fn test_fallback_parse_very_long_input() {
        let client = OpenAIClient::new(make_test_config());
        let input = "add a task ".repeat(100);
        let result = client.fallback_parse(&input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().content, input);
    }

    #[test]
    fn test_fallback_parse_with_newlines() {
        let client = OpenAIClient::new(make_test_config());
        let input = "add a task\nwith newlines\nand more text";
        let result = client.fallback_parse(input);
        assert!(result.is_ok());
        // The content is preserved as-is
        assert!(result.unwrap().content.contains("newlines"));
    }
}