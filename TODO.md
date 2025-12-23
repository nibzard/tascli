# Tascli Natural Language Integration - TODO List

## Phase 1: Core NLP Integration (Week 1-2)

### Project Setup
- [x] Analyze tascli codebase structure and functionality
- [x] Design the natural language processing architecture
- [x] Plan the OpenAI Responses API integration
- [x] Design the command parsing and mapping system
- [x] Plan implementation phases and architecture
- [ ] Add OpenAI API dependency to Cargo.toml
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
- [ ] Add support for compound commands
- [ ] Create disambiguation for ambiguous inputs

### Performance & Caching
- [ ] Implement response caching system
- [ ] Add quick pattern matching for simple commands
- [ ] Create async API call handling with timeouts
- [ ] Optimize for reduced API usage

## Phase 3: Advanced Features (Week 5-6)

### Multi-step Commands
- [ ] Support for sequential operations
- [ ] Implement command batching
- [ ] Add conditional logic support
- [ ] Create command preview and confirmation

### Smart Features
- [ ] Auto-completion and suggestions
- [ ] Error recovery and clarification requests
- [ ] Learning from user corrections
- [ ] Personalized pattern recognition

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

### Current Task: Phase 2 - Advanced Parsing
**Next Action**: Add support for compound commands

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