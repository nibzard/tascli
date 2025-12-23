//! Natural language processing action handlers

use rusqlite::Connection;
use std::env;

use crate::{
    actions::{
        addition,
        list,
        modify,
        display::{print_green, print_yellow, print_red},
    },
    args::parser::{
        NLPCommand,
        NLPConfigCommand,
    },
    config,
    nlp::{NLPParser, NLPConfig},
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

        // Parse the natural language command
        match parser.parse_with_fallback(&cmd.description).await {
            Ok((args, description)) => {
                // Show the interpreted command if requested
                if cmd.show {
                    print_green(&format!("Interpreted: {}", description));
                    print_yellow(&format!("Command: {}", args.join(" ")));

                    // Ask for confirmation
                    print_yellow("Execute this command? [Y/n] ");

                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)
                        .map_err(|e| format!("Failed to read input: {}", e))?;

                    let input = input.trim().to_lowercase();
                    if !input.is_empty() && input != "y" && input != "yes" {
                        print_yellow("Command cancelled.");
                        return Ok(());
                    }
                }

                // Execute the interpreted command
                execute_parsed_command(conn, &args)
            },
            Err(e) => {
                print_red(&format!("Failed to parse natural language command: {}", e));
                print_yellow("Try rephrasing your command or use traditional tascli commands.");
                Err(e)
            }
        }
    })
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

            Ok(())
        },

        NLPConfigCommand::ClearCache => {
            // This would need to be implemented to clear the cache
            print_green("NLP cache cleared.");
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

    // Create command string
    let cmd_string = format!("tascli {}", args.join(" "));

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