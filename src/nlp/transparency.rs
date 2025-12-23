//! Transparency display for NLP command mapping
//!
//! This module provides functions to display how natural language input
//! was interpreted and mapped to tascli commands.

use super::types::*;
use super::mapper::CommandMapper;

/// Display the NLP interpretation transparency information
pub fn show_interpretation(input: &str, command: &NLPCommand, args: &[String]) {
    println!();
    println!("  NLP Interpretation");
    println!("  ==================");
    println!("  Input: \"{}\"", input);
    println!("  Interpreted: {}", CommandMapper::describe_command(command));
    println!();

    // Show confidence if available
    if let Some(conf) = command.confidence {
        let confidence_level = if conf >= 0.9 {
            "High"
        } else if conf >= 0.7 {
            "Medium"
        } else {
            "Low"
        };
        println!("  Confidence: {:.0}% ({})", conf * 100.0, confidence_level);
    }

    // Show source if available
    if let Some(ref source) = command.interpretation_source {
        println!("  Source: {}", source);
    }

    println!();

    // Show the mapped CLI command
    println!("  Mapped Command:");
    print!("    tascli");
    for arg in args {
        if arg.contains(' ') {
            print!(" \"{}\"", arg);
        } else {
            print!(" {}", arg);
        }
    }
    println!();
    println!();
}

/// Display transparency for compound commands
pub fn show_compound_interpretation(input: &str, commands: &[Vec<String>], description: &str) {
    println!();
    println!("  NLP Interpretation");
    println!("  ==================");
    println!("  Input: \"{}\"", input);
    println!("  Type: Compound command ({} actions)", commands.len());
    println!("  Description: {}", description);
    println!();

    // Show each command
    for (i, args) in commands.iter().enumerate() {
        println!("  Command {}:", i + 1);
        print!("    tascli");
        for arg in args {
            if arg.contains(' ') {
                print!(" \"{}\"", arg);
            } else {
                print!(" {}", arg);
            }
        }
        println!();
    }
    println!();
}

/// Display a simple one-line interpretation info
pub fn show_interpretation_compact(input: &str, command: &NLPCommand) {
    let confidence_str = command.confidence
        .map(|c| format!(" (confidence: {:.0}%)", c * 100.0))
        .unwrap_or_default();

    let source_str = command.interpretation_source
        .as_ref()
        .map(|s| format!(" [{}]", s))
        .unwrap_or_default();

    println!("NLP: \"{}\" -> {}{}{}", input, command.action, source_str, confidence_str);
}

/// Format interpretation as a string for programmatic use
pub fn format_interpretation(input: &str, command: &NLPCommand, args: &[String]) -> String {
    let mut result = format!("Input: \"{}\"\n", input);
    result.push_str(&format!("Action: {}\n", command.action));
    result.push_str(&format!("Description: {}\n", CommandMapper::describe_command(command)));

    if let Some(conf) = command.confidence {
        result.push_str(&format!("Confidence: {:.0}%\n", conf * 100.0));
    }

    if let Some(ref source) = command.interpretation_source {
        result.push_str(&format!("Source: {}\n", source));
    }

    result.push_str(&format!("Command: tascli {}\n", args.join(" ")));

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_interpretation_basic() {
        let command = NLPCommand {
            action: ActionType::Task,
            content: "test task".to_string(),
            confidence: Some(0.95),
            interpretation_source: Some("pattern".to_string()),
            ..Default::default()
        };

        let args = vec!["task".to_string(), "test task".to_string()];
        let result = format_interpretation("add test task", &command, &args);

        assert!(result.contains("Input: \"add test task\""));
        assert!(result.contains("Action: task"));
        assert!(result.contains("Confidence: 95%"));
        assert!(result.contains("Source: pattern"));
        assert!(result.contains("Command: tascli task"));
    }

    #[test]
    fn test_format_interpretation_no_confidence() {
        let command = NLPCommand {
            action: ActionType::List,
            content: "".to_string(),
            confidence: None,
            interpretation_source: None,
            ..Default::default()
        };

        let args = vec!["list".to_string(), "task".to_string()];
        let result = format_interpretation("show tasks", &command, &args);

        assert!(result.contains("Input: \"show tasks\""));
        assert!(result.contains("Action: list"));
        assert!(!result.contains("Confidence:"));
        assert!(!result.contains("Source:"));
    }

    #[test]
    fn test_format_interpretation_with_category() {
        let command = NLPCommand {
            action: ActionType::Task,
            content: "meeting".to_string(),
            category: Some("work".to_string()),
            confidence: Some(0.8),
            ..Default::default()
        };

        let args = vec!["task".to_string(), "-c".to_string(), "work".to_string(), "meeting".to_string()];
        let result = format_interpretation("add work meeting", &command, &args);

        assert!(result.contains("Input: \"add work meeting\""));
        assert!(result.contains("Description:"));
        assert!(result.contains("work"));
    }
}
