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