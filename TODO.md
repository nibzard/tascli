# Tascli Natural Language Integration - TODO List

## Phase 1: Core NLP Integration (Week 1-2)

### Project Setup
- [x] Analyze tascli codebase structure and functionality
- [x] Design the natural language processing architecture
- [x] Plan the OpenAI Responses API integration
- [x] Design the command parsing and mapping system
- [x] Plan implementation phases and architecture
- [x] Add OpenAI API dependency to Cargo.toml ✅
  - Added async-openai 0.24 to Cargo.toml dependencies
- [ ] Create NLP module structure in src/nlp/
- [ ] Set up configuration for NLP settings
- [ ] Create basic OpenAI client implementation

### Core NLP Functionality
- [ ] Implement NLP command parser with function calling
- [ ] Create command validation logic
- [ ] Build command mapper (NLP → tascli commands)
- [ ] Add `nlp` subcommand to existing CLI
- [ ] Integrate with existing tascli command execution

### Testing & Validation
- [x] Write unit tests for NLP parsing ✅
  - Implemented comprehensive unit tests in src/nlp/parser.rs
  - Tests cover NLP response parsing, command extraction, and function calling
  - Includes test cases for error handling and edge cases
- [x] Test command mapping accuracy ✅
  - Created mapper_tests.rs with 39 comprehensive mapping accuracy tests
  - All tests pass (309 total tests in suite)
  - Fixed describe_command inconsistency for "history" keyword
- [x] Create integration tests for end-to-end flow ✅
  - Added integration tests in src/nlp/integration_tests.rs
  - Tests cover command execution flow, error handling, and edge cases
  - Includes integration with OpenAI API mock for realistic testing
  - All 8 integration test scenarios passing
- [x] Validate with common natural language patterns

## Phase 2: Enhanced Natural Language Understanding (Week 3-4)

### Context Awareness
- [x] Implement context tracking for previous commands ✅
  - Created CommandContext in src/nlp/context.rs
  - Tracks command history with configurable depth
  - Integrated into NLPParser with context management methods
- [x] Add time context awareness (current time, day, etc.) ✅
  - Created TimeContext with today/tomorrow/weekday support
  - Automatically injects current date/time into parsing context
  - Handles relative date references
- [x] Create fuzzy matching for categories and existing tasks ✅
  - Implemented FuzzyMatcher using Levenshtein distance
  - Supports category and task name matching with configurable threshold
  - Integrated with NLPParser context-aware parsing
- [x] Build intelligent deadline inference ✅
  - Created DeadlineInference module with temporal expression parser
  - Supports relative dates (today, tomorrow, next week), day names, and date patterns
  - Automatic deadline detection from natural language task descriptions
  - Integrated with context-aware parsing for accurate deadline assignment

### Advanced Parsing
- [x] Handle complex queries ("show all overdue work tasks") ✅
  - Added QueryType enum with 9 variants (Overdue, Upcoming, Unscheduled, DueToday, DueTomorrow, DueThisWeek, DueThisMonth, Urgent, All)
  - Updated NLPCommand with query_type field
  - Updated OpenAI function schema to include query_type parameter
  - Enhanced system prompts with complex query examples
  - Updated CommandMapper with proper CLI flag mappings for each query type
  - Added comprehensive tests for all query types
- [x] Implement relative time parsing ("in 2 hours", "next week") ✅
  - Updated format_relative_deadline() to output tascli-compatible formats
  - +Xd for days (e.g., +7d for 7 days from now)
  - HH:MM for hours (e.g., 14:30 for 2 hours from now)
  - "today HH:MM" for minutes/seconds
  - Added regex patterns for "next <weekday>" and "for <weekday>"
  - Updated system prompts with relative time examples
  - Added 18 new tests (8 context.rs + 10 natural_language_patterns_tests.rs)
- [x] Add support for compound commands ✅
  - Implemented command chaining for multiple operations in single NLP input
  - Added CompoundCommand type to NLPCommand with commands vector
  - Updated OpenAI function schema to support compound operations
  - Enhanced system prompts with compound command examples
  - CommandMapper now handles sequential command execution
  - Allows users to chain operations like "add task X and list all tasks"
- [x] Create disambiguation for ambiguous inputs ✅
  - Implemented interactive disambiguation dialog in src/nlp/disambiguate.rs
  - Detects ambiguous task names, categories, and time references
  - Presents options to user for clarification
  - Integrates with NLPParser to request clarification before command execution
  - Added 15 tests covering disambiguation scenarios
  - All 636 tests passing

### Performance & Caching
- [x] Implement response caching system ✅
  - Added persistent SQLite-based cache in src/nlp/cache.rs
  - Integrated cache into OpenAI client with check-before-API-call flow
  - Configurable TTL (default 7 days), cache statistics, auto-cleanup
  - All 697 tests pass including 23 new cache tests
- [x] Add quick pattern matching for simple commands ✅
  - Two-tier LRU caching (hot cache: 100 entries, cold cache: 500 entries)
  - 5 new pattern matching patterns (search, priority, date quick, category setting)
  - All 763 tests passing
- [x] Create async API call handling with timeouts ✅
  - Added configurable timeout_seconds field to NLPConfig (default: 30 seconds)
  - Added Timeout(u64) error variant to NLPError
  - Updated OpenAIClient to use configurable timeout from config
  - Added timeout detection using reqwest's is_timeout() method
  - Updated parse_command and parse_command_with_context with timeout handling
  - Added timeout_seconds to NLPConfigSection in config/mod.rs
  - Comprehensive timeout tests (client.rs, types.rs)
  - All 763 tests pass
- [x] Optimize for reduced API usage ✅

## Phase 3: Advanced Features (Week 5-6)

### Multi-step Commands
- [x] Support for sequential operations ✅
  - Implemented SequentialExecutor in src/nlp/sequential.rs (521 lines)
  - Added ExecutionMode (StopOnError/ContinueOnError) with configurable error handling
  - Context sharing between commands in sequence (SharedContext for task IDs, categories, etc.)
  - Detailed execution summaries (ExecutionSummary, CommandResult with timing and status)
  - Comprehensive error tracking with SequentialError error type
  - All 763 tests passing
- [x] Implement command batching ✅
- [x] Add conditional logic support ✅
  - Implemented Condition, ConditionExpression, ComparisonOperator, ConditionalBranch types
  - Created conditional.rs module with ConditionEvaluator and ConditionalExecutor
  - Condition types: task exists, task count, category state, previous result, time-based, day-of-week
  - Added Conditional execution mode to CompoundExecutionMode
  - Integrated conditional executor with sequential executor
  - Added 8 conditional pattern matching patterns
  - All 846 tests passing
- [x] Create command preview and confirmation ✅
  - Created src/nlp/preview.rs module with PreviewCommand, PreviewFormatter, PreviewManager
  - Configuration options: preview_enabled and auto_confirm
  - CLI commands: enable/disable-preview, enable/disable-auto-confirm
  - Integration with SequentialExecutor and ConditionalExecutor
  - User confirmation prompts (Y/n/e) with support for Yes/No/Expand

### Smart Features
- [x] Auto-completion and suggestions ✅
  - SuggestionEngine with pattern matching in src/nlp/suggestions.rs
  - Typo correction support using Levenshtein distance
  - Context-aware suggestions based on command history
  - CLI commands: `nlp config suggest <input>` and `nlp config patterns`
  - AutoCompleter for shell integration
  - 23 unit tests covering all functionality
- [x] Error recovery and clarification requests ✅
  - Implemented intelligent error recovery with categorization, clarification requests, guided prompts, and suggestion strategies. Integrated into NLP command handler.
- [x] Learning from user corrections ✅
  - Implemented adaptive learning system with SQLite storage for corrections and patterns
  - Features: Jaro-Winkler fuzzy matching, pattern extraction, CLI commands (learning-stats, clear-learning, learn)
  - Integrated with NLP parser to apply learned corrections automatically
- [x] Personalized pattern recognition ✅
  - User-specific pattern recognition with SQLite-based persistent storage
  - Tracks individual command patterns, category preferences, and custom shortcuts
  - PersonalizationEngine adapts to user habits over time
  - CLI commands: personalization-status/reset/export, create/list/delete-shortcut
  - 21 unit tests

### Enhanced UX
- [ ] Add transparency in command mapping
- [ ] Show interpreted commands for verification
- [ ] Implement help system for natural language
- [ ] Add interactive mode for complex queries

## Phase 4: Integration & Polish (Week 7-8)

### Seamless Integration
- [ ] Make NLP the default interface (optional)
- [ ] Ensure backward compatibility
- [ ] Add configuration options for NLP features
- [ ] Implement graceful fallbacks

### Documentation & Examples
- [ ] Create comprehensive documentation
- [ ] Add natural language examples
- [ ] Write migration guide for existing users
- [ ] Create troubleshooting guide

### Performance & Optimization
- [ ] Benchmark NLP vs traditional commands
- [ ] Optimize API usage and caching
- [ ] Minimize binary size impact
- [ ] Ensure startup time remains fast

## Implementation Status

### Current Task: Phase 3 - Multi-step Commands
**Next Action**: Create command preview and confirmation

### Completed Tasks ✅
- Project analysis and architecture design
- Technical implementation plan
- OpenAI API integration strategy
- Command mapping system design
- Implementation roadmap
- ✅ Add OpenAI API dependency to Cargo.toml
- ✅ Create NLP module structure in src/nlp/
- ✅ Set up configuration for NLP settings
- ✅ Create basic OpenAI client implementation
- ✅ Implement NLP command parser with function calling
- ✅ Create command validation logic
- ✅ Build command mapper (NLP → tascli commands)
- ✅ Add nlp subcommand to existing CLI
- ✅ Write unit tests for NLP parsing
- ✅ Phase 2 Context Awareness (Commit 6ef1acb)
  - Context module with CommandContext, TimeContext, FuzzyMatcher
  - NLPParser context-aware integration
  - OpenAI client context support
  - 48 new tests (43 context.rs + 5 parser.rs)
  - All 527 tests passing
- ✅ Phase 2 Deadline Inference
  - DeadlineInference module with temporal expression parsing
  - Support for relative dates, day names, date patterns
  - Automatic deadline detection from task descriptions
  - Context-aware integration for accurate assignment
- ✅ Phase 2 Complex Query Handling
  - QueryType enum with 9 variants for filtering tasks
  - Enhanced NLPCommand with query_type field
  - Updated OpenAI function schema and system prompts
  - CommandMapper mappings for all query types
  - Comprehensive test coverage
- ✅ Phase 2 Relative Time Parsing
  - format_relative_deadline() outputs tascli-compatible formats
  - +Xd for days, HH:MM for hours, "today HH:MM" for minutes/seconds
  - Regex patterns for "next <weekday>" and "for <weekday>"
  - Updated system prompts with relative time examples
  - 18 new tests (8 context.rs + 10 natural_language_patterns_tests.rs)
  - All 621 tests passing (Commit a2e708d)
- ✅ Phase 2 Disambiguation System
  - Interactive disambiguation dialog in src/nlp/disambiguate.rs
  - Detects ambiguous task names, categories, and time references
  - Presents options to user for clarification
  - Integrates with NLPParser to request clarification before command execution
  - 15 new tests covering disambiguation scenarios
  - All 636 tests passing
- ✅ Phase 2 Compound Command Support (Commit 52dbaac)
  - Implemented command chaining for multiple operations in single NLP input
  - Added CompoundCommand type to NLPCommand with commands vector
  - Updated OpenAI function schema to support compound operations
  - Enhanced system prompts with compound command examples
  - CommandMapper now handles sequential command execution
  - Allows users to chain operations like "add task X and list all tasks"
- ✅ Phase 2 Response Caching System
  - Added persistent SQLite-based cache in src/nlp/cache.rs
  - Integrated cache into OpenAI client with check-before-API-call flow
  - Configurable TTL (default 7 days), cache statistics, auto-cleanup
  - All 697 tests pass including 23 new cache tests
- ✅ Phase 2 API Usage Optimizations
  - Two-tier LRU caching (hot cache: 100 entries, cold cache: 500 entries)
  - 5 new pattern matching patterns for common commands
  - Configurable timeout for NLP API calls (default: 30 seconds)
  - Timeout detection using reqwest's is_timeout() method
  - All 763 tests passing
- ✅ Phase 3 Pattern Matching & Performance (Commit 11e3498, 038325f, 310c35f)
  - Quick pattern matching for simple commands (search, priority, date, category)
  - Hot and cold cache implementation for reduced API calls
  - Configurable timeout with Timeout error variant
  - Comprehensive timeout tests in client.rs and types.rs
- ✅ Phase 3 Sequential Operations (Commit 849fcca)
  - SequentialExecutor in src/nlp/sequential.rs (521 lines)
  - ExecutionMode with StopOnError/ContinueOnError options
  - SharedContext for passing data between commands in sequence
  - ExecutionSummary with detailed timing and status for each command
  - SequentialError for comprehensive error tracking
  - All 763 tests passing
- ✅ Phase 3 Command Batching
  - Batching system for grouping related commands together
  - Improved efficiency for multi-command operations
  - Context preservation across batched commands
- ✅ Phase 3 Conditional Logic Support (Commit d948225)
  - Condition, ConditionExpression, ComparisonOperator, ConditionalBranch types
  - conditional.rs module with ConditionEvaluator and ConditionalExecutor
  - Condition types: task exists, task count, category state, previous result, time-based, day-of-week
  - Conditional execution mode in CompoundExecutionMode
  - Integration with sequential executor
  - 8 conditional pattern matching patterns
  - All 846 tests passing
- ✅ Phase 3 Personalized Pattern Recognition (Commit 79919bd)
  - User-specific pattern recognition with SQLite storage
  - UserProfile, PatternFrequencyTracker, PersonalizedPatternMatcher, PersonalizationEngine
  - Tracks command patterns, category preferences, and shortcuts per user
  - CLI: personalization-status/reset/export, create/list/delete-shortcut
  - Integrated with NLP parser for automatic personalized pattern application
  - 21 unit tests

### Key Decisions Made
- Use OpenAI Responses API with gpt-5-nano
- Implement as additional NLP module without disrupting existing code
- Maintain full backward compatibility
- Focus on high-accuracy command mapping (>95%)
- Implement smart caching to manage costs and performance

### Target Metrics
- Command parsing accuracy: >95%
- API response time: <500ms (cached: <10ms)
- Cost per user: <$0.05/month for moderate usage
- Performance impact: <20% slower than traditional commands
- Binary size increase: <500KB