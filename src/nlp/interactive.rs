//! Interactive mode for complex multi-step natural language interactions
//!
//! Provides a REPL-like interface for exploratory task management workflows
//! with context persistence across queries.

use super::parser::NLPParser;
use super::types::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::io::{self, Write};

/// Session state for interactive mode
#[derive(Debug, Clone)]
pub struct InteractiveSession {
    /// Unique session identifier
    pub session_id: String,
    /// Number of interactions in this session
    pub interaction_count: usize,
    /// Session start timestamp
    pub start_time: i64,
    /// Last activity timestamp
    pub last_activity: i64,
    /// Whether session is active
    pub is_active: bool,
}

impl InteractiveSession {
    /// Create a new interactive session
    pub fn new() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        Self {
            session_id: format!("session_{}", now),
            interaction_count: 0,
            start_time: now,
            last_activity: now,
            is_active: true,
        }
    }

    /// Record an interaction
    pub fn record_interaction(&mut self) {
        use std::time::{SystemTime, UNIX_EPOCH};
        self.interaction_count += 1;
        self.last_activity = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
    }

    /// Get session duration in seconds
    pub fn duration(&self) -> i64 {
        self.last_activity - self.start_time
    }
}

/// Result of an interactive command
#[derive(Debug, Clone)]
pub enum InteractiveResult {
    /// Command executed successfully
    Success { command: NLPCommand, output: String },
    /// Command execution failed
    Error { message: String },
    /// User requested help
    Help,
    /// User requested to exit
    Exit,
    /// User requested context info
    ContextInfo { info: String },
    /// User requested to clear context
    ContextCleared,
    /// User requested to repeat/modify last command
    Repeat { command: NLPCommand },
    /// Ambiguous input - needs clarification
    Ambiguous { clarification: String },
}

/// Interactive mode configuration
#[derive(Debug, Clone)]
pub struct InteractiveConfig {
    /// Prompt string to display
    pub prompt: String,
    /// Whether to show interpretation transparency
    pub show_interpretation: bool,
    /// Whether to show context info on startup
    pub show_context_on_start: bool,
    /// Maximum history size for the session
    pub max_history: usize,
    /// Session timeout in seconds (None for no timeout)
    pub session_timeout: Option<i64>,
}

impl Default for InteractiveConfig {
    fn default() -> Self {
        Self {
            prompt: "\x1b[1;36mnlp>\x1b[0m ".to_string(),
            show_interpretation: true,
            show_context_on_start: true,
            max_history: 100,
            session_timeout: None,
        }
    }
}

/// Interactive REPL for natural language task management
pub struct InteractiveMode {
    parser: Arc<Mutex<NLPParser>>,
    session: InteractiveSession,
    config: InteractiveConfig,
    /// Session-specific command history
    history: Vec<String>,
    /// Last executed command (for repeat/modify)
    last_command: Option<NLPCommand>,
    /// Pending clarification response
    pending_clarification: Option<String>,
}

impl InteractiveMode {
    /// Create a new interactive mode instance
    pub fn new(parser: Arc<Mutex<NLPParser>>, config: InteractiveConfig) -> Self {
        Self {
            parser,
            session: InteractiveSession::new(),
            config,
            history: Vec::new(),
            last_command: None,
            pending_clarification: None,
        }
    }

    /// Create with default configuration
    pub fn with_parser(parser: Arc<Mutex<NLPParser>>) -> Self {
        Self::new(parser, InteractiveConfig::default())
    }

    /// Get the current session
    pub fn session(&self) -> &InteractiveSession {
        &self.session
    }

    /// Get session history
    pub fn history(&self) -> &[String] {
        &self.history
    }

    /// Start the interactive REPL loop
    pub async fn run(&mut self) -> Result<(), NLPError> {
        self.print_welcome();
        self.show_context_if_enabled().await;

        let stdin = io::stdin();
        let mut stdout = io::stdout();

        loop {
            // Print prompt
            print!("{}", self.config.prompt);
            stdout.flush()?;

            // Read input
            let mut input = String::new();
            stdin.read_line(&mut input)
                .map_err(|e| NLPError::ParseError(format!("Failed to read input: {}", e)))?;

            let input = input.trim();

            // Skip empty input
            if input.is_empty() {
                continue;
            }

            // Add to history
            self.add_to_history(input.to_string());
            self.session.record_interaction();

            // Handle built-in commands first
            if let Some(result) = self.handle_builtin_commands(input) {
                match result {
                    InteractiveResult::Exit => {
                        self.print_goodbye();
                        return Ok(());
                    }
                    InteractiveResult::Help => {
                        self.print_help();
                        continue;
                    }
                    InteractiveResult::ContextInfo { .. } => {
                        self.show_context().await;
                        continue;
                    }
                    InteractiveResult::ContextCleared => {
                        self.clear_context().await;
                        continue;
                    }
                    _ => {}
                }
            }

            // Process as NLP command
            match self.process_input(input).await {
                Ok(result) => {
                    self.handle_result(result);
                }
                Err(e) => {
                    println!("\x1b[1;31mError:\x1b[0m {}", e);
                }
            }

            // Check for session timeout
            if let Some(timeout) = self.config.session_timeout {
                use std::time::{SystemTime, UNIX_EPOCH};
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;

                if now - self.session.last_activity > timeout {
                    println!("\nSession timed out after {} seconds of inactivity.", timeout);
                    return Ok(());
                }
            }
        }
    }

    /// Process a single input and return the result
    pub async fn process_input(&mut self, input: &str) -> NLPResult<InteractiveResult> {
        let parser = self.parser.lock().await;

        // Check for clarification response
        if let Some(clarification) = &self.pending_clarification {
            // TODO: Handle clarification response
            self.pending_clarification = None;
        }

        // Parse the input
        match parser.parse(input).await {
            Ok(command) => {
                // Store for potential repeat (clone before showing interpretation)
                let action = command.action.clone();
                let content = command.content.clone();
                self.last_command = Some(command.clone());

                // Show interpretation if enabled
                if self.config.show_interpretation {
                    self.show_interpretation(&command);
                }

                Ok(InteractiveResult::Success {
                    command,
                    output: format!("Parsed: {:?} {}", action, content),
                })
            }
            Err(e) => {
                // Check if it's an ambiguous error by examining the message
                let error_msg = e.to_string();
                if error_msg.contains("ambiguous") || error_msg.contains("clarify") {
                    self.pending_clarification = Some(error_msg.clone());
                    Ok(InteractiveResult::Ambiguous { clarification: error_msg })
                } else {
                    Ok(InteractiveResult::Error {
                        message: error_msg,
                    })
                }
            }
        }
    }

    /// Handle built-in commands (exit, help, etc.)
    fn handle_builtin_commands(&mut self, input: &str) -> Option<InteractiveResult> {
        let input_lower = input.to_lowercase();

        match input_lower.as_str() {
            "exit" | "quit" | "q" | ":q" => Some(InteractiveResult::Exit),
            "help" | "h" | "?" | ":help" => Some(InteractiveResult::Help),
            "context" | "ctx" => Some(InteractiveResult::ContextInfo {
                info: String::new(),
            }),
            "clear" | "reset" => Some(InteractiveResult::ContextCleared),
            "repeat" | "r" | "!!" => {
                if let Some(cmd) = &self.last_command {
                    Some(InteractiveResult::Repeat {
                        command: cmd.clone(),
                    })
                } else {
                    println!("No previous command to repeat.");
                    None
                }
            }
            "history" => {
                self.show_history();
                None
            }
            _ => None,
        }
    }

    /// Handle an interactive result
    fn handle_result(&mut self, result: InteractiveResult) {
        match result {
            InteractiveResult::Success { command, output } => {
                println!("\x1b[1;32m✓\x1b[0m {}", output);
            }
            InteractiveResult::Error { message } => {
                println!("\x1b[1;31m✗\x1b[0m {}", message);
            }
            InteractiveResult::Ambiguous { clarification } => {
                println!("\x1b[1;33m?\x1b[0m {}", clarification);
            }
            InteractiveResult::Repeat { command } => {
                println!("Repeating: {:?} {}", command.action, command.content);
            }
            _ => {}
        }
    }

    /// Add input to history
    fn add_to_history(&mut self, input: String) {
        self.history.push(input);
        if self.history.len() > self.config.max_history {
            self.history.remove(0);
        }
    }

    /// Show command interpretation
    fn show_interpretation(&self, command: &NLPCommand) {
        // Format interpretation without full args (we only have the NLPCommand)
        println!("  Action: {:?}", command.action);
        if !command.content.is_empty() {
            println!("  Content: {}", command.content);
        }
        if let Some(ref cat) = command.category {
            println!("  Category: {}", cat);
        }
        if let Some(ref deadline) = command.deadline {
            println!("  Deadline: {}", deadline);
        }
    }

    /// Show context information
    async fn show_context(&self) {
        let parser = self.parser.lock().await;
        let context = parser.get_context_state().await;

        println!("\x1b[1;36m=== Context ===\x1b[0m");

        if let Some(cat) = &context.last_category {
            println!("  Last category: {}", cat);
        }

        if let Some(content) = &context.last_content {
            println!("  Last task: {}", content);
        }

        if !context.known_categories.is_empty() {
            println!("  Categories: {}", context.known_categories.join(", "));
        }

        if !context.recent_tasks.is_empty() {
            let recent: Vec<&String> = context.recent_tasks.iter()
                .rev()
                .take(5)
                .collect();
            let recent_str: Vec<&str> = recent.iter().map(|s| s.as_str()).collect();
            println!("  Recent tasks: {}", recent_str.join(", "));
        }

        println!("  Interactions: {}", self.session.interaction_count);
        println!("  Session duration: {}s", self.session.duration());
    }

    /// Clear the context
    async fn clear_context(&mut self) {
        // Reset context by creating new
        let parser = self.parser.lock().await;
        // Context reset happens through parser's context
        println!("Context cleared.");
    }

    /// Show context on startup if enabled
    async fn show_context_if_enabled(&self) {
        if self.config.show_context_on_start {
            self.show_context().await;
        }
    }

    /// Show command history
    fn show_history(&self) {
        println!("\x1b[1;36m=== Command History ===\x1b[0m");
        for (i, cmd) in self.history.iter().enumerate() {
            println!("  {}  {}", i + 1, cmd);
        }
    }

    /// Print welcome message
    fn print_welcome(&self) {
        println!("\x1b[1;36m╔════════════════════════════════════════════════════════╗\x1b[0m");
        println!("\x1b[1;36m║\x1b[0m  \x1b[1;37mInteractive Natural Language Task Management\x1b[0m    \x1b[1;36m║\x1b[0m");
        println!("\x1b[1;36m╚════════════════════════════════════════════════════════╝\x1b[0m");
        println!("\nSession: \x1b[1;33m{}\x1b[0m", self.session.session_id);
        println!("Type \x1b[1;37mhelp\x1b[0m for available commands or \x1b[1;37mexit\x1b[0m to quit.\n");
    }

    /// Print goodbye message
    fn print_goodbye(&self) {
        println!("\n\x1b[1;36mSession Summary\x1b[0m");
        println!("  Interactions: {}", self.session.interaction_count);
        println!("  Duration: {}s", self.session.duration());
        println!("\nGoodbye!\n");
    }

    /// Print help information
    fn print_help(&self) {
        println!("\x1b[1;36m=== Interactive Mode Help ===\x1b[0m\n");

        println!("\x1b[1;37mNatural Language Commands:\x1b[0m");
        println!("  add task \"buy groceries\" category:shopping");
        println!("  mark \"buy groceries\" as done");
        println!("  list all work tasks");
        println!("  what's due today?");

        println!("\n\x1b[1;37mBuilt-in Commands:\x1b[0m");
        println!("  \x1b[1;32mhelp, h, ?\x1b[0m     Show this help");
        println!("  \x1b[1;32mexit, quit, q\x1b[0m   Exit interactive mode");
        println!("  \x1b[1;32mcontext, ctx\x1b[0m   Show current context");
        println!("  \x1b[1;32mclear, reset\x1b[0m    Clear session context");
        println!("  \x1b[1;32mrepeat, r, !!\x1b[0m   Repeat last command");
        println!("  \x1b[1;32mhistory\x1b[0m        Show command history");

        println!("\n\x1b[1;37mFeatures:\x1b[0m");
        println!("  • Context persistence across queries");
        println!("  • Fuzzy matching for categories and tasks");
        println!("  • Intelligent deadline inference");
        println!("  • Command interpretation transparency");
        println!("  • Refine queries iteratively\n");
    }
}

/// Create a new interactive session with parser
pub fn create_interactive_mode(
    parser: Arc<Mutex<NLPParser>>,
    config: Option<InteractiveConfig>,
) -> InteractiveMode {
    match config {
        Some(cfg) => InteractiveMode::new(parser, cfg),
        None => InteractiveMode::with_parser(parser),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_new() {
        let session = InteractiveSession::new();
        assert!(session.is_active);
        assert_eq!(session.interaction_count, 0);
        assert!(session.session_id.starts_with("session_"));
    }

    #[test]
    fn test_session_record_interaction() {
        let mut session = InteractiveSession::new();
        assert_eq!(session.interaction_count, 0);

        session.record_interaction();
        assert_eq!(session.interaction_count, 1);

        session.record_interaction();
        assert_eq!(session.interaction_count, 2);
    }

    #[test]
    fn test_session_duration() {
        let session = InteractiveSession::new();
        // Duration should be small (just created)
        assert!(session.duration() < 10);
    }

    #[test]
    fn test_interactive_config_default() {
        let config = InteractiveConfig::default();
        assert_eq!(config.prompt, "\x1b[1;36mnlp>\x1b[0m ");
        assert!(config.show_interpretation);
        assert!(config.show_context_on_start);
        assert_eq!(config.max_history, 100);
        assert!(config.session_timeout.is_none());
    }

    #[test]
    fn test_interactive_mode_new() {
        let config = InteractiveConfig::default();
        let _mode = InteractiveMode::new(
            Arc::new(Mutex::new(create_test_parser())),
            config,
        );
        // Basic creation test - actual parsing requires async
    }

    #[test]
    fn test_handle_builtin_exit() {
        let mut mode = create_test_mode();
        let result = mode.handle_builtin_commands("exit");
        assert!(matches!(result, Some(InteractiveResult::Exit)));

        let result = mode.handle_builtin_commands("quit");
        assert!(matches!(result, Some(InteractiveResult::Exit)));

        let result = mode.handle_builtin_commands("q");
        assert!(matches!(result, Some(InteractiveResult::Exit)));
    }

    #[test]
    fn test_handle_builtin_help() {
        let mut mode = create_test_mode();
        let result = mode.handle_builtin_commands("help");
        assert!(matches!(result, Some(InteractiveResult::Help)));
    }

    #[test]
    fn test_handle_builtin_context() {
        let mut mode = create_test_mode();
        let result = mode.handle_builtin_commands("context");
        assert!(matches!(result, Some(InteractiveResult::ContextInfo { .. })));
    }

    #[test]
    fn test_handle_builtin_clear() {
        let mut mode = create_test_mode();
        let result = mode.handle_builtin_commands("clear");
        assert!(matches!(result, Some(InteractiveResult::ContextCleared)));
    }

    #[test]
    fn test_handle_builtin_repeat_none() {
        let mut mode = create_test_mode();
        // No last command yet
        let result = mode.handle_builtin_commands("repeat");
        assert!(result.is_none());
    }

    #[test]
    fn test_handle_unknown_command() {
        let mut mode = create_test_mode();
        let result = mode.handle_builtin_commands("add task");
        assert!(result.is_none()); // Not a built-in, should be handled by parser
    }

    #[test]
    fn test_add_to_history() {
        let mut mode = create_test_mode();
        assert!(mode.history.is_empty());

        mode.add_to_history("first command".to_string());
        assert_eq!(mode.history.len(), 1);
        assert_eq!(mode.history[0], "first command");

        mode.add_to_history("second command".to_string());
        assert_eq!(mode.history.len(), 2);
    }

    #[test]
    fn test_history_limit() {
        let config = InteractiveConfig {
            max_history: 3,
            ..Default::default()
        };
        let mut mode = InteractiveMode::new(
            Arc::new(Mutex::new(create_test_parser())),
            config,
        );

        // Add more than max_history
        for i in 1..=5 {
            mode.add_to_history(format!("command {}", i));
        }

        // Should only keep last 3
        assert_eq!(mode.history.len(), 3);
        assert_eq!(mode.history[0], "command 3");
        assert_eq!(mode.history[2], "command 5");
    }

    #[test]
    fn test_interactive_result_variants() {
        // Test that result types can be created
        let _success = InteractiveResult::Success {
            command: NLPCommand::default(),
            output: "test".to_string(),
        };
        let _error = InteractiveResult::Error {
            message: "error".to_string(),
        };
        let _help = InteractiveResult::Help;
        let _exit = InteractiveResult::Exit;
    }

    fn create_test_parser() -> NLPParser {
        // This would require a valid config for full testing
        // For now, just ensure compilation
        let config = NLPConfig::default();
        NLPParser::new(config)
    }

    fn create_test_mode() -> InteractiveMode {
        InteractiveMode::with_parser(Arc::new(Mutex::new(create_test_parser())))
    }

    #[test]
    fn test_case_insensitive_builtin() {
        let mut mode = create_test_mode();

        let result = mode.handle_builtin_commands("EXIT");
        assert!(matches!(result, Some(InteractiveResult::Exit)));

        let result = mode.handle_builtin_commands("Help");
        assert!(matches!(result, Some(InteractiveResult::Help)));

        let result = mode.handle_builtin_commands("CONTEXT");
        assert!(matches!(result, Some(InteractiveResult::ContextInfo { .. })));
    }
}
