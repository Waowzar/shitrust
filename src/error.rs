use thiserror::Error;
use std::path::PathBuf;
use colored::*;

/// Represents a location in source code
#[derive(Debug, Clone, Copy)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

impl SourceLocation {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

impl std::fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

/// Errors that can occur in the ShitRust compiler and interpreter
#[derive(Debug, thiserror::Error)]
pub enum ShitRustError {
    #[error("Syntax error at line {line}, column {column}: {message}")]
    SyntaxError {
        line: usize,
        column: usize,
        message: String,
    },
    
    #[error("Type error: {0}")]
    TypeError(String),
    
    #[error("Runtime error: {0}")]
    RuntimeError(String),
    
    #[error("Value error: {0}")]
    ValueError(String),
    
    #[error("IO error: {0}")]
    IOException(String),
    
    #[error("Not implemented: {0}")]
    NotImplemented(String),
    
    #[error("Break statement outside of loop")]
    Break,
    
    #[error("Continue statement outside of loop")]
    Continue,
    
    #[error("Return statement outside of function")]
    Return,
    
    // Add new error types
    #[error("Module error: {0}")]
    ModuleError(String),
    
    #[error("Module not found: {0}")]
    ModuleNotFound(String),
    
    #[error("Trait error: {0}")]
    TraitError(String),
    
    #[error("No pattern matched the value: {0}")]
    PatternMatchError(String),
    
    #[error("Async error: {0}")]
    AsyncError(String),
    
    #[error("Concurrent operation error: {0}")]
    ConcurrencyError(String),
    
    #[error("Optional access error: {0}")]
    OptionalError(String),
    
    #[error("Timeout error: {0}")]
    TimeoutError(String),
}

impl ShitRustError {
    /// Format the error with colored output
    pub fn format_error(&self) -> String {
        match self {
            ShitRustError::SyntaxError { line, column, message } => {
                format!("{} at line {}, column {}: {}", 
                    "Syntax Error".red().bold(),
                    line,
                    column,
                    message)
            },
            ShitRustError::TypeError(message) => {
                format!("{}: {}", 
                    "Type Error".red().bold(),
                    message)
            },
            _ => format!("{}", self.to_string().red())
        }
    }
}

/// Result type for ShitRust operations
pub type Result<T> = std::result::Result<T, ShitRustError>; 