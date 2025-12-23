use rusqlite::Connection;
use clap::Parser;

use crate::{
    actions::{
        addition,
        list,
        modify,
        nlp,
        display::{print_yellow},
    },
    args::parser::{
        Action,
        CliArgs,
        ListCommand,
    },
};

pub fn handle_commands(conn: &Connection, args: CliArgs) -> Result<(), String> {
    // If we have a subcommand, handle it traditionally
    if let Some(action) = args.arguments {
        return match action {
            Action::Task(cmd) => addition::handle_taskcmd(conn, &cmd),
            Action::Record(cmd) => addition::handle_recordcmd(conn, &cmd),
            Action::Done(cmd) => modify::handle_donecmd(conn, &cmd),
            Action::Delete(cmd) => modify::handle_deletecmd(conn, &cmd),
            Action::Update(cmd) => modify::handle_updatecmd(conn, &cmd),
            Action::List(list_cmd) => match list_cmd {
                ListCommand::Task(cmd) => list::handle_listtasks(conn, cmd),
                ListCommand::Record(cmd) => list::handle_listrecords(conn, cmd),
                ListCommand::Show(cmd) => list::handle_showcontent(conn, cmd),
            },
            Action::NLP(cmd) => nlp::handle_nlp_command(conn, &cmd),
        };
    }

    // No subcommand provided - check if we have raw input
    if args.raw_input.is_empty() {
        // No input at all - show help
        print_usage();
        return Ok(());
    }

    // Join raw input into a single string
    let input = args.raw_input.join(" ");

    // If --no-nlp flag is set, try to parse as traditional command
    if args.no_nlp {
        return try_traditional_parse(conn, &input);
    }

    // Check if input looks like a traditional command
    // If so, try traditional first, fall back to NLP
    if looks_like_traditional_command(&input) {
        match try_traditional_parse(conn, &input) {
            Ok(_) => return Ok(()),
            Err(e) => {
                // Traditional parsing failed, fall through to NLP
                print_yellow(&format!("Traditional parsing failed: {}, trying NLP...", e));
            }
        }
    }

    // Route through NLP by default
    route_through_nlp(conn, &input)
}

/// Route input through NLP parser
fn route_through_nlp(conn: &Connection, input: &str) -> Result<(), String> {
    let nlp_cmd = crate::args::parser::NLPCommand {
        description: input.to_string(),
        show: false,
        config: None,
    };

    nlp::handle_nlp_command(conn, &nlp_cmd)
}

/// Check if input looks like a traditional command
fn looks_like_traditional_command(input: &str) -> bool {
    let lower = input.trim().to_lowercase();
    let first_word = lower.split_whitespace().next();

    matches!(first_word, Some("task") | Some("record") | Some("done") | Some("update") | Some("delete") | Some("list"))
}

/// Try to parse input as a traditional command
fn try_traditional_parse(conn: &Connection, input: &str) -> Result<(), String> {
    // Prepend "tascli" to simulate command invocation
    let cmd_args: Vec<&str> = std::iter::once("tascli")
        .chain(input.split_whitespace())
        .collect();

    match CliArgs::try_parse_from(cmd_args) {
        Ok(parsed_args) => {
            // Recursively handle the parsed arguments
            handle_commands(conn, parsed_args)
        },
        Err(e) => {
            Err(format!("Failed to parse as traditional command: {}", e))
        }
    }
}

/// Print usage information
fn print_usage() {
    println!("tascli - A simple CLI tool for tracking tasks and records");
    println!();
    println!("Natural Language Mode (default):");
    println!("  tascli add task to review PRs today");
    println!("  tascli show my work tasks");
    println!("  tascli complete task 1");
    println!();
    println!("Traditional Commands (also work):");
    println!("  tascli task \"Review PRs\" today");
    println!("  tascli list task -c work");
    println!("  tascli done 1");
    println!();
    println!("Options:");
    println!("  --no-nlp    Disable NLP mode, force traditional parsing");
    println!("  -h, --help  Show detailed help");
}
