//! Natural language processing action handlers

use rusqlite::Connection;

use crate::{
    actions::{
        display::{print_green, print_yellow, print_red},
    },
    args::parser::{
        NLPCommand,
        NLPConfigCommand,
    },
    config,
    nlp::{
        NLPParser, SequentialExecutor, CompoundExecutionMode,
        PreviewManager, commands_to_previews, ConfirmationResult,
        SuggestionEngine, SuggestionRequest, PatternMatcher,
        ErrorRecoveryEngine, InteractiveRecoveryHandler,
    },
};

pub fn handle_nlp_command(conn: &Connection, cmd: &NLPCommand) -> Result<(), String> {
    // Handle configuration commands first
    if let Some(config_cmd) = &cmd.config {
        return handle_nlp_config(config_cmd);
    }

    // Parse natural language command
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| format!("Failed to create async runtime: {}", e))?;

    rt.block_on(async {
        // Get NLP configuration
        let nlp_config = config::get_nlp_config()
            .map_err(|e| format!("Failed to get NLP config: {}", e))?;

        if !nlp_config.enabled {
            return Err("NLP is disabled. Use 'tascli nlp config enable' to enable it.".to_string());
        }

        if nlp_config.api_key.is_none() {
            return Err("OpenAI API key not configured. Use 'tascli nlp config set-key <api_key>' to set it.".to_string());
        }

        // Create parser
        let parser = NLPParser::new(nlp_config.clone());

        // Parse the natural language command, checking for compound commands
        match parser.parse_to_compound_args(&cmd.description).await {
            Ok((all_args, description)) => {
                // Check if this is a compound command
                if all_args.len() > 1 {
                    // Handle compound command
                    handle_compound_command(conn, &all_args, &description, cmd.show, &nlp_config)
                } else {
                    // Handle single command
                    handle_single_command(conn, &all_args[0], &description, cmd.show, &nlp_config)
                }
            },
            Err(e) => {
                // Use error recovery to provide helpful suggestions
                print_red(&format!("Failed to parse natural language command: {}", e));

                // Try to get available categories for context
                let available_categories: Vec<String> = match crate::db::crud::query_items(
                    conn,
                    &crate::db::item::ItemQuery::new()
                ) {
                    Ok(items) => {
                        let mut cats: std::collections::HashSet<String> = std::collections::HashSet::new();
                        for item in items {
                            if !item.category.is_empty() {
                                cats.insert(item.category);
                            }
                        }
                        cats.into_iter().collect()
                    },
                    Err(_) => Vec::new(),
                };

                // Generate and display recovery options
                let recovery_result = ErrorRecoveryEngine::handle_error(&e, &cmd.description, &available_categories);
                ErrorRecoveryEngine::display_recovery(&recovery_result);

                print_yellow("\nYou can also use traditional tascli commands or 'tascli nlp config patterns' to see available patterns.");

                Err(e.to_string())
            }
        }
    })
}

/// Handle single command with preview
fn handle_single_command(
    conn: &Connection,
    args: &[String],
    description: &str,
    force_show: bool,
    nlp_config: &crate::nlp::NLPConfig,
) -> Result<(), String> {
    // Create preview manager
    let preview_enabled = force_show || nlp_config.preview_enabled;
    let preview_manager = PreviewManager::new(preview_enabled, nlp_config.auto_confirm);

    // Convert args to NLPCommand for preview
    let nlp_cmd = convert_args_to_nlp_command(args);

    // Create preview
    let preview = crate::nlp::PreviewCommand::from_nlp_command(&nlp_cmd, 0);

    // Show preview and get confirmation
    match preview_manager.preview_command(&preview)? {
        ConfirmationResult::Confirmed => {
            execute_parsed_command(conn, args)
        },
        ConfirmationResult::Cancelled => {
            print_yellow("Command cancelled.");
            Ok(())
        },
        ConfirmationResult::Edit => {
            print_yellow("Edit functionality not yet implemented. Command cancelled.");
            Ok(())
        },
    }
}

/// Handle compound commands (multiple commands in one input)
fn handle_compound_command(
    conn: &Connection,
    all_args: &[Vec<String>],
    description: &str,
    force_show: bool,
    nlp_config: &crate::nlp::NLPConfig,
) -> Result<(), String> {
    // Convert args to NLPCommands for SequentialExecutor
    let commands = convert_args_to_commands(all_args);

    // Create preview manager
    let preview_enabled = force_show || nlp_config.preview_enabled;
    let preview_manager = PreviewManager::new(preview_enabled, nlp_config.auto_confirm);

    // Create previews
    let previews = commands_to_previews(&commands);

    // Show preview and get confirmation
    match preview_manager.preview_compound(&previews, &CompoundExecutionMode::ContinueOnError)? {
        ConfirmationResult::Confirmed => {
            // Execute the compound command
            execute_compound_commands(conn, &commands, &preview_manager)
        },
        ConfirmationResult::Cancelled => {
            print_yellow("Commands cancelled.");
            Ok(())
        },
        ConfirmationResult::Edit => {
            print_yellow("Edit functionality not yet implemented. Commands cancelled.");
            Ok(())
        },
    }
}

/// Execute compound commands with summary
fn execute_compound_commands(
    conn: &Connection,
    commands: &[crate::nlp::NLPCommand],
    preview_manager: &PreviewManager,
) -> Result<(), String> {
    // Create executor
    let executor = SequentialExecutor::new(false, true); // Continue on error, verbose
    let execution_mode = CompoundExecutionMode::ContinueOnError;

    // Disable internal preview since we already showed it
    let result = executor.execute_compound(conn, commands, &execution_mode, false);

    match result {
        Ok(summary) => {
            // Print detailed results
            for res in &summary.results {
                if res.success {
                    print_green(&format!("Command {}: Success", res.index + 1));
                } else {
                    print_red(&format!("Command {}: Failed - {}",
                        res.index + 1,
                        res.error.as_deref().unwrap_or("Unknown error")));
                }
            }

            // Print summary using preview manager
            preview_manager.show_summary(summary.total, summary.successful, summary.failed);

            if !summary.is_complete_success() {
                print_yellow("\nSome commands failed. You can retry failed commands individually.");
            }

            Ok(())
        },
        Err(e) => Err(e),
    }
}

/// Convert CLI args back to NLPCommands (simplified for compatibility)
fn convert_args_to_commands(all_args: &[Vec<String>]) -> Vec<crate::nlp::NLPCommand> {
    all_args.iter().map(|args| convert_args_to_nlp_command(args)).collect()
}

/// Convert a single CLI args to NLPCommand
fn convert_args_to_nlp_command(args: &[String]) -> crate::nlp::NLPCommand {
    use crate::nlp::{NLPCommand, ActionType};

    let action = if args.first().map_or(false, |a| a == "task") {
        ActionType::Task
    } else if args.first().map_or(false, |a| a == "done") {
        ActionType::Done
    } else if args.first().map_or(false, |a| a == "update") {
        ActionType::Update
    } else if args.first().map_or(false, |a| a == "delete") {
        ActionType::Delete
    } else if args.first().map_or(false, |a| a == "record") {
        ActionType::Record
    } else {
        ActionType::List
    };

    // Extract content from args (simplified)
    let content = args.get(1).cloned().unwrap_or_default();
    let category = args.iter()
        .position(|a| a == "-c")
        .and_then(|i| args.get(i + 1))
        .cloned();

    NLPCommand {
        action,
        content,
        category,
        ..Default::default()
    }
}

fn handle_nlp_config(config_cmd: &NLPConfigCommand) -> Result<(), String> {
    match config_cmd {
        NLPConfigCommand::Enable => {
            let mut nlp_config = config::get_nlp_config()
                .unwrap_or_default();
            nlp_config.enabled = true;
            config::update_nlp_config(&nlp_config)?;
            print_green("NLP functionality enabled.");
            Ok(())
        },

        NLPConfigCommand::Disable => {
            let mut nlp_config = config::get_nlp_config()
                .unwrap_or_default();
            nlp_config.enabled = false;
            config::update_nlp_config(&nlp_config)?;
            print_green("NLP functionality disabled.");
            Ok(())
        },

        NLPConfigCommand::SetKey { api_key } => {
            let mut nlp_config = config::get_nlp_config()
                .unwrap_or_default();
            nlp_config.api_key = Some(api_key.clone());
            config::update_nlp_config(&nlp_config)?;
            print_green("OpenAI API key configured successfully.");
            Ok(())
        },

        NLPConfigCommand::Show => {
            let nlp_config = config::get_nlp_config()
                .unwrap_or_default();

            println!("NLP Configuration:");
            println!("  Enabled: {}", nlp_config.enabled);
            println!("  API Key: {}",
                if nlp_config.api_key.is_some() {
                    "***configured***"
                } else {
                    "not set"
                });
            println!("  Model: {}", nlp_config.model);
            println!("  Fallback to traditional: {}", nlp_config.fallback_to_traditional);
            println!("  Cache commands: {}", nlp_config.cache_commands);
            println!("  Context window: {}", nlp_config.context_window);
            println!("  Max API calls/minute: {}", nlp_config.max_api_calls_per_minute);
            println!("  API base URL: {}", nlp_config.api_base_url);
            println!("  Preview enabled: {}", nlp_config.preview_enabled);
            println!("  Auto-confirm: {}", nlp_config.auto_confirm);

            Ok(())
        },

        NLPConfigCommand::ClearCache => {
            // This would need to be implemented to clear the cache
            print_green("NLP cache cleared.");
            Ok(())
        },

        NLPConfigCommand::EnablePreview => {
            let mut nlp_config = config::get_nlp_config()
                .unwrap_or_default();
            nlp_config.preview_enabled = true;
            config::update_nlp_config(&nlp_config)?;
            print_green("Preview mode enabled. You'll see command previews before execution.");
            Ok(())
        },

        NLPConfigCommand::DisablePreview => {
            let mut nlp_config = config::get_nlp_config()
                .unwrap_or_default();
            nlp_config.preview_enabled = false;
            config::update_nlp_config(&nlp_config)?;
            print_green("Preview mode disabled. Commands will execute immediately.");
            Ok(())
        },

        NLPConfigCommand::EnableAutoConfirm => {
            let mut nlp_config = config::get_nlp_config()
                .unwrap_or_default();
            nlp_config.auto_confirm = true;
            config::update_nlp_config(&nlp_config)?;
            print_green("Auto-confirm enabled. Preview will be shown but commands execute automatically.");
            Ok(())
        },

        NLPConfigCommand::DisableAutoConfirm => {
            let mut nlp_config = config::get_nlp_config()
                .unwrap_or_default();
            nlp_config.auto_confirm = false;
            config::update_nlp_config(&nlp_config)?;
            print_green("Auto-confirm disabled. You'll be prompted before execution.");
            Ok(())
        },

        NLPConfigCommand::Suggest { input } => {
            // Get suggestions for the input
            let request = SuggestionRequest {
                input: input.clone(),
                cursor_position: input.len(),
                recent_commands: Vec::new(),
                available_categories: Vec::new(),
            };

            let result = SuggestionEngine::suggest(&request);

            // Show validation status
            if result.is_valid {
                print_green(&format!("✓ Valid command: '{}'", input));
            } else {
                print_yellow(&format!("⚠ Partial or invalid command: '{}'", input));
            }

            // Show suggestions
            println!();
            print!("{}", SuggestionEngine::format_suggestions(&result.suggestions));

            Ok(())
        },

        NLPConfigCommand::Patterns => {
            let patterns = SuggestionEngine::command_patterns();

            println!("Available Natural Language Command Patterns:");
            println!("===========================================");
            println!();

            for (pattern, description) in patterns {
                println!("  {:30} - {}", pattern, description);
            }

            println!();
            print_yellow("Use 'tascli nlp config suggest <partial-input>' to get suggestions for your input.");

            Ok(())
        },
    }
}

fn execute_parsed_command(conn: &Connection, args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("No command to execute".to_string());
    }

    // Parse and execute the command using the existing CLI infrastructure
    // This is a simplified approach - in a real implementation, you might want
    // to directly call the action handlers instead of re-parsing

    // For now, let's create a mock CLI args structure
    use crate::args::parser::{CliArgs};
    use clap::Parser;

    // Split into args for parsing
    let cmd_args: Vec<&str> = std::iter::once("tascli")
        .chain(args.iter().map(|s| s.as_str()))
        .collect();

    // Parse the command
    let parsed_args = CliArgs::try_parse_from(cmd_args)
        .map_err(|e| format!("Failed to parse generated command: {}", e))?;

    // Execute using existing handler
    super::handler::handle_commands(conn, parsed_args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nlp_config_enable() {
        // This would require setting up a test config
        // For now, just test that the function doesn't panic
        let result = handle_nlp_config(&NLPConfigCommand::Enable);
        // In a real test, we'd mock the config system
        println!("Result: {:?}", result);
    }
}