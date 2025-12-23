//! Help system for natural language commands
//!
//! This module provides comprehensive help documentation and examples
//! for natural language command usage in tascli.

use crate::nlp::{ActionType, QueryType, StatusType};

/// Help topics available
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HelpTopic {
    /// Overview and getting started
    Overview,
    /// Query types (overdue, upcoming, etc.)
    Queries,
    /// Compound commands
    Compound,
    /// Conditional execution
    Conditions,
    /// Natural language examples
    Examples,
    /// Available patterns
    Patterns,
    /// All topics
    All,
}

impl HelpTopic {
    /// Parse topic from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "overview" | "intro" | "introduction" | "getting-started" | "start" => Some(Self::Overview),
            "queries" | "query" | "search" | "filter" => Some(Self::Queries),
            "compound" | "sequential" | "multiple" | "batch" => Some(Self::Compound),
            "conditions" | "conditional" | "if" => Some(Self::Conditions),
            "examples" | "example" => Some(Self::Examples),
            "patterns" | "pattern" | "commands" => Some(Self::Patterns),
            "all" | "complete" | "full" => Some(Self::All),
            _ => None,
        }
    }

    /// Get description of topic
    pub fn description(&self) -> &'static str {
        match self {
            Self::Overview => "Getting started with natural language commands",
            Self::Queries => "Query types: overdue, upcoming, due today, etc.",
            Self::Compound => "Executing multiple commands in one input",
            Self::Conditions => "Conditional execution based on query results",
            Self::Examples => "Real-world examples organized by category",
            Self::Patterns => "All available command patterns",
            Self::All => "Complete help documentation",
        }
    }
}

/// Main help system
pub struct HelpSystem;

impl HelpSystem {
    /// Show overview (public for use in action handler)
    pub fn show_overview() {
        println!();
        println!("  Natural Language Commands for tascli");
        println!("  =====================================");
        println!();
        println!("  tascli's NLP feature lets you interact with your tasks using");
        println!("  natural language instead of memorizing command syntax.");
        println!();
        println!("  Quick Start:");
        println!("  ------------");
        println!("    tascli nlp \"add task buy groceries\"");
        println!("    tascli nlp \"show my tasks due today\"");
        println!("    tascli nlp \"mark task 1 as done\"");
        println!();
        println!("  Configuration:");
        println!("  -------------");
        println!("    tascli nlp config enable        - Enable NLP");
        println!("    tascli nlp config set-key <key> - Set OpenAI API key");
        println!("    tascli nlp config show          - Show current config");
        println!();
        println!("  Getting More Help:");
        println!("  -----------------");
        println!("    tascli nlp help queries      - Learn about query types");
        println!("    tascli nlp help compound     - Learn about multiple commands");
        println!("    tascli nlp help conditions   - Learn about conditional execution");
        println!("    tascli nlp help examples     - See real-world examples");
        println!("    tascli nlp help patterns     - See all command patterns");
        println!();
        println!("  Suggestions:");
        println!("  -----------");
        println!("    tascli nlp config suggest \"add t\"  - Get completion suggestions");
        println!();
    }

    /// Show help for a specific topic
    pub fn show_help(topic: HelpTopic) {
        match topic {
            HelpTopic::Overview => Self::show_overview(),
            HelpTopic::Queries => Self::show_queries(),
            HelpTopic::Compound => Self::show_compound(),
            HelpTopic::Conditions => Self::show_conditions(),
            HelpTopic::Examples => Self::show_examples(),
            HelpTopic::Patterns => Self::show_patterns(),
            HelpTopic::All => Self::show_all(),
        }
    }

    /// Show query types help
    fn show_queries() {
        println!();
        println!("  Query Types");
        println!("  ============");
        println!();
        println!("  Query types help you filter and find specific tasks.");
        println!();
        println!("  Time-Based Queries:");
        println!("  ------------------");
        println!("    overdue        - Tasks past their due date (not completed)");
        println!("    upcoming       - Tasks with future due dates");
        println!("    due today      - Tasks due today");
        println!("    due tomorrow   - Tasks due tomorrow");
        println!("    due this week  - Tasks due within 7 days");
        println!();
        println!("  Examples:");
        println!("    tascli nlp \"show overdue tasks\"");
        println!("    tascli nlp \"what's due today\"");
        println!("    tascli nlp \"list upcoming tasks\"");
        println!("    tascli nlp \"tasks due this week\"");
        println!();
        println!("  Status-Based Queries:");
        println!("  --------------------");
        println!("    done/completed     - Only completed tasks");
        println!("    pending/open       - Not yet completed");
        println!("    cancelled          - Cancelled tasks");
        println!("    all                - All tasks regardless of status");
        println!();
        println!("  Examples:");
        println!("    tascli nlp \"show completed tasks\"");
        println!("    tascli nlp \"list pending tasks\"");
        println!("    tascli nlp \"show all my tasks\"");
        println!();
        println!("  Priority-Based Queries:");
        println!("  ---------------------");
        println!("    high priority      - Tasks marked as high priority");
        println!("    urgent             - Urgent tasks");
        println!();
        println!("  Category-Based Queries:");
        println!("  ---------------------");
        println!("    <category> tasks  - Tasks in a specific category");
        println!();
        println!("  Examples:");
        println!("    tascli nlp \"show work tasks\"");
        println!("    tascli nlp \"list personal tasks\"");
        println!("    tascli nlp \"tasks for project X\"");
        println!();
        println!("  Search Queries:");
        println!("  --------------");
        println!("    search <term>      - Find tasks containing text");
        println!();
        println!("  Examples:");
        println!("    tascli nlp \"search for groceries\"");
        println!("    tascli nlp \"find tasks about meeting\"");
        println!();
    }

    /// Show compound commands help
    fn show_compound() {
        println!();
        println!("  Compound Commands");
        println!("  =================");
        println!();
        println!("  Execute multiple commands in a single natural language input.");
        println!();
        println!("  Syntax Patterns:");
        println!("  ---------------");
        println!("    and            - Connect two commands");
        println!("    then           - Sequential commands");
        println!("    after that     - Sequential commands");
        println!("    also           - Additional command");
        println!("    plus           - Add another command");
        println!();
        println!("  Examples:");
        println!("    tascli nlp \"add task buy milk and add task buy bread\"");
        println!("    tascli nlp \"add task call mom then add task schedule dentist\"");
        println!("    tascli nlp \"list work tasks and show overdue tasks\"");
        println!();
        println!("  Behavior:");
        println!("  ---------");
        println!("  - All commands are shown in a preview before execution");
        println!("  - If one command fails, others continue (continue-on-error mode)");
        println!("  - A summary shows success/failure for each command");
        println!();
        println!("  Example Output:");
        println!("  --------------");
        println!("  $ tascli nlp \"add task buy milk and add task buy bread\"");
        println!();
        println!("  NLP Interpretation");
        println!("  ==================");
        println!("  Input: \"add task buy milk and add task buy bread\"");
        println!("  Type: Compound command (2 actions)");
        println!("  Description: Add two tasks");
        println!();
        println!("  Command 1:");
        println!("    tascli task buy milk");
        println!();
        println!("  Command 2:");
        println!("    tascli task buy bread");
        println!();
        println!("  Execute these commands? [y/N]: y");
        println!("  Command 1: Success");
        println!("  Command 2: Success");
        println!("  Summary: 2 total, 2 successful, 0 failed");
        println!();
    }

    /// Show conditions help
    fn show_conditions() {
        println!();
        println!("  Conditional Execution");
        println!("  ====================");
        println!();
        println!("  Execute commands conditionally based on query results.");
        println!();
        println!("  Syntax:");
        println!("  ------");
        println!("    if <query> has tasks then <command>");
        println!("    if <query> is empty then <command>");
        println!("    if <query> has more than <n> tasks then <command>");
        println!("    if <query> has fewer than <n> tasks then <command>");
        println!();
        println!("  Operators:");
        println!("  ---------");
        println!("    has tasks / has items       - Query returns > 0 results");
        println!("    is empty                    - Query returns 0 results");
        println!("    has more than / >           - Greater than count");
        println!("    has fewer than / <          - Less than count");
        println!("    has exactly / =             - Exact count");
        println!();
        println!("  Examples:");
        println!("    tascli nlp \"if overdue has tasks then list overdue\"");
        println!("    tascli nlp \"if work tasks is empty then add task check email\"");
        println!("    tascli nlp \"if upcoming has more than 5 tasks then show upcoming\"");
        println!("    tascli nlp \"if pending has fewer than 3 tasks then list tasks\"");
        println!();
        println!("  Use Cases:");
        println!("  ---------");
        println!("  - Check for overdue tasks before adding new ones");
        println!("  - Ensure certain categories aren't empty");
        println!("  - Limit workload before adding more tasks");
        println!("  - Conditional notifications or summaries");
        println!();
        println!("  Advanced Example:");
        println!("  -----------------");
        println!("  $ tascli nlp \"if overdue has tasks then show overdue\"");
        println!();
        println!("  NLP Interpretation");
        println!("  ==================");
        println!("  Input: \"if overdue has tasks then show overdue\"");
        println!("  Type: Conditional command");
        println!();
        println!("  Condition: overdue has tasks");
        println!("    Query: overdue");
        println!("    Operator: has_tasks");
        println!();
        println!("  Then branch: show overdue");
        println!("    tascli list task --overdue");
        println!();
        println!("  Executing conditional command...");
        println!("  Condition met: Found 3 overdue tasks");
        println!("  Executing: show overdue");
        println!();
    }

    /// Show examples
    fn show_examples() {
        println!();
        println!("  Natural Language Examples");
        println!("  =========================");
        println!();
        println!("  Task Management:");
        println!("  ----------------");
        println!("    Add tasks:");
        println!("      tascli nlp \"add task buy groceries\"");
        println!("      tascli nlp \"create a task for calling mom\"");
        println!("      tascli nlp \"task: finish the report by Friday\"");
        println!();
        println!("    Complete tasks:");
        println!("      tascli nlp \"mark task 1 as done\"");
        println!("      tascli nlp \"complete task number 5\"");
        println!("      tascli nlp \"finish the first task\"");
        println!();
        println!("    Delete tasks:");
        println!("      tascli nlp \"delete task 3\"");
        println!("      tascli nlp \"remove task number 2\"");
        println!();
        println!("    Update tasks:");
        println!("      tascli nlp \"update task 1 to call dad instead\"");
        println!("      tascli nlp \"change task 2 content to buy eggs\"");
        println!();
        println!("  Queries & Filtering:");
        println!("  -------------------");
        println!("    Time-based:");
        println!("      tascli nlp \"show overdue tasks\"");
        println!("      tascli nlp \"what's due today\"");
        println!("      tascli nlp \"tasks due tomorrow\"");
        println!("      tascli nlp \"list upcoming tasks\"");
        println!();
        println!("    Status-based:");
        println!("      tascli nlp \"show completed tasks\"");
        println!("      tascli nlp \"list all pending tasks\"");
        println!("      tascli nlp \"show cancelled tasks\"");
        println!();
        println!("    Category-based:");
        println!("      tascli nlp \"show work tasks\"");
        println!("      tascli nlp \"list personal tasks\"");
        println!();
        println!("    Search:");
        println!("      tascli nlp \"search for meeting\"");
        println!("      tascli nlp \"find tasks with groceries\"");
        println!();
        println!("  Compound Commands:");
        println!("  -----------------");
        println!("    tascli nlp \"add task buy milk and add task buy bread\"");
        println!("    tascli nlp \"show work tasks then list personal tasks\"");
        println!("    tascli nlp \"complete task 1 and delete task 2\"");
        println!();
        println!("  Conditional:");
        println!("  -----------");
        println!("    tascli nlp \"if overdue has tasks then list overdue\"");
        println!("    tascli nlp \"if work tasks is empty then add task check email\"");
        println!();
        println!("  With Categories:");
        println!("  ---------------");
        println!("      tascli nlp \"add work task finish report\"");
        println!("      tascli nlp \"add personal task call mom\"");
        println!("      tascli nlp \"show home tasks\"");
        println!();
        println!("  With Deadlines:");
        println!("  ---------------");
        println!("      tascli nlp \"add task finish report by Friday\"");
        println!("      tascli nlp \"add task meeting tomorrow at 3pm\"");
        println!("      tascli nlp \"add task review project next week\"");
        println!();
        println!("  Records:");
        println!("  --------");
        println!("      tascli nlp \"add record had a productive meeting\"");
        println!("      tascli nlp \"record: completed phase 1 of project\"");
        println!("      tascli nlp \"show today's records\"");
        println!();
    }

    /// Show all command patterns
    fn show_patterns() {
        println!();
        println!("  Available Command Patterns");
        println!("  ==========================");
        println!();
        println!("  Task Creation:");
        println!("  --------------");
        println!("    add task <description>              - Add a new task");
        println!("    add record <description>            - Add a new record");
        println!("    task: <description>                 - Quick add task");
        println!("    create task <description>           - Add a new task");
        println!();
        println!("  Task Completion:");
        println!("  ----------------");
        println!("    complete <number>                   - Mark task as done");
        println!("    done <number>                       - Mark task as done");
        println!("    finish <number>                     - Mark task as done");
        println!("    mark <number> as done               - Mark task as done");
        println!();
        println!("  Task Deletion:");
        println!("  -------------");
        println!("    delete <number>                     - Delete a task");
        println!("    remove <number>                     - Delete a task");
        println!();
        println!("  Task Updates:");
        println!("  ------------");
        println!("    update <number> to <content>        - Update task content");
        println!("    change <number> to <content>        - Update task content");
        println!("    modify <number>                     - Update a task");
        println!();
        println!("  Queries - Time:");
        println!("  -------------");
        println!("    overdue                            - Show overdue tasks");
        println!("    show overdue tasks                 - Show overdue tasks");
        println!("    upcoming                           - Show upcoming tasks");
        println!("    show upcoming tasks                - Show upcoming tasks");
        println!("    due today                          - Tasks due today");
        println!("    what's due today                   - Tasks due today");
        println!("    due tomorrow                       - Tasks due tomorrow");
        println!("    due this week                      - Tasks due this week");
        println!();
        println!("  Queries - Status:");
        println!("  ---------------");
        println!("    list                               - List all tasks");
        println!("    show tasks                         - List all tasks");
        println!("    show completed tasks               - Show done tasks");
        println!("    show pending tasks                 - Show open tasks");
        println!("    show cancelled tasks               - Show cancelled tasks");
        println!("    show all tasks                     - Show all tasks");
        println!();
        println!("  Queries - Category:");
        println!("  ------------------");
        println!("    show <category> tasks               - List by category");
        println!("    list <category> tasks               - List by category");
        println!("    <category> tasks                    - List by category");
        println!();
        println!("  Queries - Search:");
        println!("  ----------------");
        println!("    search <term>                      - Search for tasks");
        println!("    find <term>                        - Search for tasks");
        println!("    tasks containing <term>            - Search for tasks");
        println!();
        println!("  Compound Patterns:");
        println!("  -----------------");
        println!("    <cmd> and <cmd>                    - Execute both commands");
        println!("    <cmd> then <cmd>                   - Execute sequentially");
        println!("    <cmd> after that <cmd>             - Execute sequentially");
        println!("    <cmd> also <cmd>                   - Execute both commands");
        println!();
        println!("  Conditional Patterns:");
        println!("  --------------------");
        println!("    if <query> has tasks then <cmd>     - Conditional execution");
        println!("    if <query> is empty then <cmd>      - Conditional execution");
        println!("    if <query> has more than N then <cmd>  - Count-based condition");
        println!();
        println!("  Category Specification:");
        println!("  ----------------------");
        println!("    add <category> task <desc>          - Add with category");
        println!("    add task <desc> in <category>       - Add with category");
        println!();
        println!("  Time Specification:");
        println!("  ------------------");
        println!("    add task <desc> by <time>           - Add with deadline");
        println!("    add task <desc> on <date>           - Add with deadline");
        println!("    add task <desc> at <time>           - Add with deadline");
        println!();
        println!("  Priority:");
        println!("  ---------");
        println!("    show high priority tasks            - High priority only");
        println!("    show urgent tasks                   - Urgent tasks");
        println!();
    }

    /// Show all help topics
    fn show_all() {
        Self::show_overview();
        println!();
        println!("  {}", "=".repeat(40));
        println!();
        Self::show_queries();
        println!();
        println!("  {}", "=".repeat(40));
        println!();
        Self::show_compound();
        println!();
        println!("  {}", "=".repeat(40));
        println!();
        Self::show_conditions();
        println!();
        println!("  {}", "=".repeat(40));
        println!();
        Self::show_examples();
        println!();
        println!("  {}", "=".repeat(40));
        println!();
        Self::show_patterns();
    }

    /// List all available help topics
    pub fn list_topics() {
        println!();
        println!("  Available Help Topics");
        println!("  =====================");
        println!();
        let topics = [
            (HelpTopic::Overview, "Getting started guide"),
            (HelpTopic::Queries, "Query types explained"),
            (HelpTopic::Compound, "Multiple commands in one input"),
            (HelpTopic::Conditions, "Conditional execution"),
            (HelpTopic::Examples, "Real-world examples"),
            (HelpTopic::Patterns, "All command patterns"),
            (HelpTopic::All, "Complete documentation"),
        ];

        for (topic, description) in topics {
            let topic_str = format!("{:?}", topic).to_lowercase();
            println!("    {:20} - {}", topic_str, description);
        }
        println!();
        println!("  Usage: tascli nlp help <topic>");
        println!();
    }

    /// Get suggestions for similar topics when input doesn't match
    pub fn suggest_topic(input: &str) -> Vec<String> {
        let input_lower = input.to_lowercase();
        let mut suggestions = Vec::new();

        let topics = [
            ("overview", HelpTopic::Overview),
            ("queries", HelpTopic::Queries),
            ("query", HelpTopic::Queries),
            ("search", HelpTopic::Queries),
            ("compound", HelpTopic::Compound),
            ("sequential", HelpTopic::Compound),
            ("multiple", HelpTopic::Compound),
            ("conditions", HelpTopic::Conditions),
            ("conditional", HelpTopic::Conditions),
            ("if", HelpTopic::Conditions),
            ("examples", HelpTopic::Examples),
            ("example", HelpTopic::Examples),
            ("patterns", HelpTopic::Patterns),
            ("commands", HelpTopic::Patterns),
        ];

        for (name, _topic) in topics {
            let name_lower = name.to_lowercase();
            // Check if input starts with topic name
            if name_lower.starts_with(&input_lower) || input_lower.starts_with(&name_lower) {
                suggestions.push(name.to_string());
            }
            // Check for partial matches (fuzzy)
            else if input_lower.len() >= 3 && name_lower.contains(&input_lower[..3]) {
                suggestions.push(name.to_string());
            }
        }

        // Deduplicate while preserving order
        let mut seen = std::collections::HashSet::new();
        suggestions.retain(|s| seen.insert(s.clone()));

        suggestions
    }

    /// Get context-aware help suggestions based on partial input
    pub fn suggest_for_input(input: &str) -> Vec<HelpSuggestion> {
        let input_lower = input.to_lowercase();
        let mut suggestions = Vec::new();

        // Empty input - suggest overview
        if input_lower.is_empty() {
            suggestions.push(HelpSuggestion {
                topic: HelpTopic::Overview,
                reason: "New to NLP commands? Start here.".to_string(),
            });
            return suggestions;
        }

        // Task-related input
        if input_lower.contains("task") || input_lower.contains("add") {
            suggestions.push(HelpSuggestion {
                topic: HelpTopic::Examples,
                reason: "See task creation examples".to_string(),
            });
        }

        // Query-related input
        if input_lower.contains("show") || input_lower.contains("list") || input_lower.contains("due") {
            suggestions.push(HelpSuggestion {
                topic: HelpTopic::Queries,
                reason: "Learn about query types".to_string(),
            });
        }

        // Conditional input
        if input_lower.contains("if") || input_lower.contains("when") {
            suggestions.push(HelpSuggestion {
                topic: HelpTopic::Conditions,
                reason: "Learn about conditional execution".to_string(),
            });
        }

        // Multiple commands
        if input_lower.contains(" and ") || input_lower.contains(" then ") {
            suggestions.push(HelpSuggestion {
                topic: HelpTopic::Compound,
                reason: "Learn about compound commands".to_string(),
            });
        }

        // Search input
        if input_lower.contains("search") || input_lower.contains("find") {
            suggestions.push(HelpSuggestion {
                topic: HelpTopic::Queries,
                reason: "Learn about search queries".to_string(),
            });
        }

        // Default: suggest patterns for syntax reference
        if suggestions.is_empty() {
            suggestions.push(HelpSuggestion {
                topic: HelpTopic::Patterns,
                reason: "See all available command patterns".to_string(),
            });
        }

        suggestions
    }
}

/// A help suggestion with context
#[derive(Debug, Clone)]
pub struct HelpSuggestion {
    /// The suggested topic
    pub topic: HelpTopic,
    /// Why this topic is suggested
    pub reason: String,
}

/// Format help suggestions for display
pub fn format_help_suggestions(suggestions: &[HelpSuggestion]) -> String {
    if suggestions.is_empty() {
        return "No help suggestions available".to_string();
    }

    let mut output = String::from("Help Suggestions:\n");

    for (i, suggestion) in suggestions.iter().enumerate() {
        let topic_str = format!("{:?}", suggestion.topic).to_lowercase();
        output.push_str(&format!("  {}. tascli nlp help {} - {}\n",
            i + 1, topic_str, suggestion.reason));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_topic_from_str() {
        assert_eq!(HelpTopic::from_str("overview"), Some(HelpTopic::Overview));
        assert_eq!(HelpTopic::from_str("OVERVIEW"), Some(HelpTopic::Overview));
        assert_eq!(HelpTopic::from_str("queries"), Some(HelpTopic::Queries));
        assert_eq!(HelpTopic::from_str("query"), Some(HelpTopic::Queries));
        assert_eq!(HelpTopic::from_str("compound"), Some(HelpTopic::Compound));
        assert_eq!(HelpTopic::from_str("conditions"), Some(HelpTopic::Conditions));
        assert_eq!(HelpTopic::from_str("examples"), Some(HelpTopic::Examples));
        assert_eq!(HelpTopic::from_str("patterns"), Some(HelpTopic::Patterns));
        assert_eq!(HelpTopic::from_str("all"), Some(HelpTopic::All));
        assert_eq!(HelpTopic::from_str("invalid"), None);
    }

    #[test]
    fn test_help_topic_description() {
        assert!(HelpTopic::Overview.description().contains("Getting started"));
        assert!(HelpTopic::Queries.description().contains("Query types"));
        assert!(HelpTopic::Compound.description().contains("multiple commands"));
    }

    #[test]
    fn test_suggest_topic() {
        let suggestions = HelpSystem::suggest_topic("over");
        assert!(suggestions.contains(&"overview".to_string()));

        let suggestions = HelpSystem::suggest_topic("quer");
        assert!(suggestions.contains(&"queries".to_string()));
        assert!(suggestions.contains(&"query".to_string()));
    }

    #[test]
    fn test_suggest_for_input() {
        let suggestions = HelpSystem::suggest_for_input("");
        assert!(!suggestions.is_empty());
        assert_eq!(suggestions[0].topic, HelpTopic::Overview);

        let suggestions = HelpSystem::suggest_for_input("add task");
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.topic == HelpTopic::Examples));

        let suggestions = HelpSystem::suggest_for_input("if overdue has tasks");
        assert!(suggestions.iter().any(|s| s.topic == HelpTopic::Conditions));
    }

    #[test]
    fn test_format_help_suggestions() {
        let suggestions = vec![
            HelpSuggestion {
                topic: HelpTopic::Overview,
                reason: "Start here".to_string(),
            },
        ];
        let formatted = format_help_suggestions(&suggestions);
        assert!(formatted.contains("overview"));
        assert!(formatted.contains("Start here"));
    }

    #[test]
    fn test_help_suggestion_clone() {
        let suggestion = HelpSuggestion {
            topic: HelpTopic::Examples,
            reason: "Test".to_string(),
        };
        let cloned = suggestion.clone();
        assert_eq!(suggestion.topic, cloned.topic);
        assert_eq!(suggestion.reason, cloned.reason);
    }
}
