//! Natural Language Processing module for tascli
//!
//! This module provides natural language parsing capabilities using OpenAI's Responses API,
//! allowing users to interact with tascli using natural language commands.

pub mod cache;
pub mod client;
pub mod parser;
pub mod mapper;
pub mod validator;
pub mod types;
pub mod context;
pub mod pattern_matcher;
pub mod sequential;
pub mod batching;
pub mod conditional;
pub mod preview;
pub mod suggestions;
pub mod error_recovery;
pub mod learning;
pub mod personalization;
pub mod transparency;
pub mod help;

pub use help::{
    HelpSystem,
    HelpTopic,
    HelpSuggestion,
    format_help_suggestions,
};

#[cfg(test)]
mod mapper_tests;

#[cfg(test)]
mod integration_tests;

#[cfg(test)]
mod natural_language_patterns_tests;

pub use cache::{ResponseCache, CacheStats};
pub use client::OpenAIClient;
pub use types::*;
pub use parser::NLPParser;
pub use mapper::CommandMapper;
pub use validator::CommandValidator;
pub use context::{CommandContext, ContextualCommand, TimeContext, FuzzyMatcher, DeadlineInference, InferredDeadline, DisambiguationHelper};
pub use pattern_matcher::{PatternMatcher, PatternMatch};

// Sequential execution support
pub use types::{
    SequentialContext,
    CommandExecutionResult,
    CommandOutput,
    ExecutionSummary,
};
pub use sequential::SequentialExecutor;

// Batching support
pub use batching::{
    BatchAnalyzer,
    BatchExecutor,
    CommandBatch,
    BatchType,
    BatchResult,
    BatchExecutionSummary,
};

// Conditional execution support
pub use conditional::{
    ConditionEvaluator,
    ConditionalExecutor,
    ConditionBuilder,
    BranchExecutionResult,
};
pub use types::{
    Condition,
    ConditionExpression,
    ComparisonOperator,
    ConditionalBranch,
};
pub use preview::{
    PreviewManager,
    PreviewCommand,
    PreviewFormatter,
    ConfirmationResult,
    commands_to_previews,
};
pub use suggestions::{
    Suggestion,
    SuggestionType,
    SuggestionRequest,
    SuggestionResult,
    SuggestionEngine,
    AutoCompleter,
};
pub use error_recovery::{
    ErrorCategory,
    RecoveryStrategy,
    ClarificationRequest,
    ClarificationOption,
    ParameterChange,
    GuidedPrompt,
    RecoveryResult,
    ErrorRecoveryEngine,
    RecoveryContext,
    InteractiveRecoveryHandler,
};
pub use learning::{
    LearningEngine,
    LearningDB,
    LearnedCorrection,
    LearnedPattern,
    PatternType,
    PatternCorrection,
    LearningStats,
    format_action,
};
pub use personalization::{
    UserProfile,
    PhrasePattern,
    PatternFrequency,
    PersonalizedShortcut,
    PersonalizationDB,
    PersonalizationEngine,
    PersonalizedPatternMatcher,
    PersonalizationStats,
    get_user_id,
};
pub use transparency::{
    show_interpretation,
    show_compound_interpretation,
    show_interpretation_compact,
    format_interpretation,
};