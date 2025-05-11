use std::io::{self, Write};
use anyhow::{Result, Context};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::ast::{Program, Stmt, Expr};

/// Formatter for ShitRust code
pub struct Formatter {
    indent_size: usize,
    current_indent: usize,
}

impl Formatter {
    /// Create a new formatter with default settings
    pub fn new() -> Self {
        Self {
            indent_size: 4,
            current_indent: 0,
        }
    }
    
    /// Create a new formatter with custom indent size
    pub fn with_indent_size(indent_size: usize) -> Self {
        Self {
            indent_size,
            current_indent: 0,
        }
    }
    
    /// Format ShitRust source code and return the formatted code
    pub fn format(&mut self, source: &str) -> Result<String> {
        // Parse the source code
        let mut lexer = Lexer::new(source);
        let tokens = lexer.scan_tokens().context("Failed to tokenize source")?;
        
        let mut parser = Parser::new(tokens);
        let program = parser.parse().context("Failed to parse source")?;
        
        // Format the AST
        let mut output = Vec::new();
        self.format_program(&program, &mut output)?;
        
        // Convert to string
        let formatted = String::from_utf8(output).context("Failed to convert to UTF-8")?;
        Ok(formatted)
    }
    
    /// Format a program and write to the output
    fn format_program(&mut self, program: &Program, output: &mut Vec<u8>) -> io::Result<()> {
        for stmt in &program.statements {
            self.format_stmt(stmt, output)?;
            writeln!(output)?;
        }
        
        Ok(())
    }
    
    /// Format a statement and write to the output
    fn format_stmt(&mut self, stmt: &Stmt, output: &mut Vec<u8>) -> io::Result<()> {
        self.write_indent(output)?;
        
        match stmt {
            Stmt::Expr(expr) => {
                self.format_expr(expr, output)?;
                writeln!(output, ";")?;
            }
            Stmt::Let { name, type_hint, value, mutable } => {
                if *mutable {
                    write!(output, "let mut {} ", name)?;
                } else {
                    write!(output, "let {} ", name)?;
                }
                
                if let Some(ty) = type_hint {
                    write!(output, ": {} ", format!("{:?}", ty).to_lowercase())?;
                }
                
                write!(output, "= ")?;
                self.format_expr(value, output)?;
                writeln!(output, ";")?;
            }
            Stmt::Function { name, params, return_type, body, is_async, is_public } => {
                if *is_public {
                    write!(output, "pub ")?;
                }
                
                if *is_async {
                    write!(output, "async ")?;
                }
                
                write!(output, "fn {}(", name)?;
                
                // Format parameters
                for (i, (param_name, param_type)) in params.iter().enumerate() {
                    if i > 0 {
                        write!(output, ", ")?;
                    }
                    write!(output, "{}: {}", param_name, format!("{:?}", param_type).to_lowercase())?;
                }
                
                write!(output, ") -> {} ", format!("{:?}", return_type).to_lowercase())?;
                writeln!(output, "{{")?;
                
                // Format function body
                self.current_indent += 1;
                for stmt in body {
                    self.format_stmt(stmt, output)?;
                }
                self.current_indent -= 1;
                
                self.write_indent(output)?;
                writeln!(output, "}}")?;
            }
            Stmt::If { condition, then_block, else_block } => {
                write!(output, "if ")?;
                self.format_expr(condition, output)?;
                writeln!(output, " {{")?;
                
                // Format then block
                self.current_indent += 1;
                for stmt in then_block {
                    self.format_stmt(stmt, output)?;
                }
                self.current_indent -= 1;
                
                self.write_indent(output)?;
                
                // Format else block if it exists
                if let Some(else_block) = else_block {
                    writeln!(output, "}} else {{")?;
                    
                    self.current_indent += 1;
                    for stmt in else_block {
                        self.format_stmt(stmt, output)?;
                    }
                    self.current_indent -= 1;
                    
                    self.write_indent(output)?;
                    writeln!(output, "}}")?;
                } else {
                    writeln!(output, "}}")?;
                }
            }
            // Handle other statement types...
            _ => {
                // Fallback for unimplemented statement types
                write!(output, "/* Unformatted: {:?} */", stmt)?;
            }
        }
        
        Ok(())
    }
    
    /// Format an expression and write to the output
    fn format_expr(&mut self, expr: &Expr, output: &mut Vec<u8>) -> io::Result<()> {
        match expr {
            Expr::Literal(lit) => {
                write!(output, "{:?}", lit)?;
            }
            Expr::Identifier(name) => {
                write!(output, "{}", name)?;
            }
            Expr::BinaryOp { left, op, right } => {
                self.format_expr(left, output)?;
                write!(output, " {:?} ", op)?;
                self.format_expr(right, output)?;
            }
            // Handle other expression types...
            _ => {
                // Fallback for unimplemented expression types
                write!(output, "/* Unformatted: {:?} */", expr)?;
            }
        }
        
        Ok(())
    }
    
    /// Write the current indentation to the output
    fn write_indent(&self, output: &mut Vec<u8>) -> io::Result<()> {
        for _ in 0..self.current_indent * self.indent_size {
            write!(output, " ")?;
        }
        Ok(())
    }
} 