pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod interpreter;
pub mod formatter;
pub mod compiler;
pub mod code_gen;

// New modules for language improvements
pub mod stdlib;
pub mod module_system;
pub mod type_system;
pub mod traits;

// Standard library modules
pub mod stdlib {
    pub mod io;
    pub mod collections;
    pub mod time;
    pub mod string;
    pub mod math;
    pub mod fs;
    pub mod net;
    pub mod async_runtime;
}

// Re-export common items
pub use error::ShitRustError;
pub use error::Result;
pub use ast::{Program, Stmt, Expr, Literal, Type};
pub use lexer::Lexer;
pub use parser::Parser;
pub use interpreter::Interpreter;
pub use formatter::Formatter;
pub use compiler::Compiler; 