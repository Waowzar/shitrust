use std::path::Path;
use std::process::Command;
use anyhow::{Result, Context};
use inkwell::context::Context;
use tempfile::NamedTempFile;
use std::fs;
use colored::Colorize;
use std::time::{Instant, Duration};

use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::interpreter::Interpreter;
use crate::code_gen::CodeGen;
use crate::error::ShitRustError;

/// Compiler configuration options
#[derive(Debug, Clone)]
pub struct CompilerOptions {
    pub verbose: bool,
    pub optimization_level: OptimizationLevel,
    pub emit_debug_info: bool,
    pub show_timings: bool,     // New option to display timing information
    pub emit_llvm_ir: bool,     // New option to save LLVM IR to a file
    pub color_output: bool,     // New option to control colored output
    pub strict_type_checking: bool,
}

/// LLVM optimization levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptimizationLevel {
    None,
    Less,
    Default,
    Aggressive,
}

impl Default for CompilerOptions {
    fn default() -> Self {
        Self {
            verbose: false,
            optimization_level: OptimizationLevel::Default,
            emit_debug_info: false,
            show_timings: false,
            emit_llvm_ir: false,
            color_output: true,
            strict_type_checking: false,
        }
    }
}

/// The main compiler for ShitRust language
pub struct Compiler {
    interpreter: Interpreter,
    options: CompilerOptions,
}

/// Timer struct to measure and display compilation phases
struct CompilationTimer {
    start_time: Instant,
    last_checkpoint: Instant,
    enabled: bool,
    color_output: bool,
}

impl CompilationTimer {
    fn new(enabled: bool, color_output: bool) -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            last_checkpoint: now,
            enabled,
            color_output,
        }
    }
    
    fn checkpoint(&mut self, phase: &str) -> Duration {
        let now = Instant::now();
        let duration = now.duration_since(self.last_checkpoint);
        self.last_checkpoint = now;
        
        if self.enabled {
            let time_str = format!("{:.3}s", duration.as_secs_f64());
            if self.color_output {
                println!("  {} {} ({})", "✓".green().bold(), phase.cyan(), time_str.yellow());
            } else {
                println!("  ✓ {} ({})", phase, time_str);
            }
        }
        
        duration
    }
    
    fn total(&self) -> Duration {
        self.start_time.elapsed()
    }
    
    fn report_total(&self, success: bool) {
        if self.enabled {
            let total = self.total();
            let time_str = format!("{:.3}s", total.as_secs_f64());
            
            if self.color_output {
                if success {
                    println!("{} in {}", "Compilation completed successfully".green().bold(), time_str.yellow());
                } else {
                    println!("{} after {}", "Compilation failed".red().bold(), time_str.yellow());
                }
            } else {
                if success {
                    println!("Compilation completed successfully in {}", time_str);
                } else {
                    println!("Compilation failed after {}", time_str);
                }
            }
        }
    }
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            interpreter: Interpreter::new(),
            options: CompilerOptions::default(),
        }
    }

    /// Create a new compiler with verbose output
    pub fn with_verbose(verbose: bool) -> Self {
        let mut options = CompilerOptions::default();
        options.verbose = verbose;
        
        Compiler {
            interpreter: Interpreter::new(),
            options,
        }
    }
    
    /// Create a new compiler with custom options
    pub fn with_options(options: CompilerOptions) -> Self {
        Compiler {
            interpreter: Interpreter::new(),
            options,
        }
    }

    /// Compile the source code to a binary file
    pub fn compile(&self, source: &str, output_path: &Path) -> Result<()> {
        self.compile_with_filename(source, output_path, None)
    }
    
    /// Compile the source code with a filename for better error reporting
    pub fn compile_with_filename(&self, source: &str, output_path: &Path, filename: Option<String>) -> Result<()> {
        let mut timer = CompilationTimer::new(
            self.options.verbose || self.options.show_timings,
            self.options.color_output
        );
        
        let log_msg = |msg: &str, timer: &mut CompilationTimer| {
            if self.options.verbose {
                if self.options.color_output {
                    println!("{}", msg.blue());
                } else {
                    println!("{}", msg);
                }
            }
            timer.checkpoint(msg)
        };
        
        // Step 1: Tokenize the source code
        log_msg("Tokenizing source code...", &mut timer);
        
        let mut lexer = if let Some(filename) = &filename {
            Lexer::with_filename(source, filename.clone())
        } else {
            Lexer::new(source)
        };
        
        let tokens = lexer.scan_tokens()
            .context("Failed during lexical analysis")?;
        
        // Step 2: Parse the tokens into an AST
        log_msg("Parsing tokens into AST...", &mut timer);
        
        let mut parser = Parser::new(tokens);
        let program = if let Some(filename) = filename {
            let statements = parser.parse()
                .context("Failed during parsing")?
                .statements;
            crate::ast::Program::with_source(statements, filename)
        } else {
            parser.parse()
                .context("Failed during parsing")?
        };
        
        // Step 3: Generate LLVM IR code
        log_msg("Generating LLVM IR code...", &mut timer);
        
        let context = Context::create();
        let mut code_gen = CodeGen::new(&context, "shitrust_module");
        
        // Set code generator options
        code_gen.set_optimization_level(self.options.optimization_level);
        if self.options.emit_debug_info {
            code_gen.enable_debug_info();
        }
        
        code_gen.generate_code(&program)
            .context("Failed during code generation")?;
        
        // Step 4: Write IR to a temporary file
        log_msg("Writing LLVM IR to file...", &mut timer);
        
        let ir_file = NamedTempFile::new()
            .context("Failed to create temporary file for IR")?;
        let ir_path = ir_file.path();
        code_gen.write_to_file(ir_path)
            .context("Failed to write IR to file")?;
        
        if self.options.verbose {
            if self.options.color_output {
                println!("{} {}", "Generated LLVM IR saved to:".blue(), 
                       ir_path.display().to_string().cyan());
            } else {
                println!("Generated LLVM IR saved to: {}", ir_path.display());
            }
        }
        
        // Optionally save LLVM IR to a file with the same name as output but .ll extension
        if self.options.emit_llvm_ir {
            let mut ir_output = output_path.to_path_buf();
            ir_output.set_extension("ll");
            fs::copy(ir_path, &ir_output)
                .with_context(|| format!("Failed to save LLVM IR to {}", ir_output.display()))?;
            
            if self.options.verbose || self.options.show_timings {
                if self.options.color_output {
                    println!("{} {}", "LLVM IR saved to:".blue(), 
                           ir_output.display().to_string().cyan());
                } else {
                    println!("LLVM IR saved to: {}", ir_output.display());
                }
            }
        }
        
        // Step 5: Generate object file
        log_msg("Generating object file...", &mut timer);
        
        let obj_file = NamedTempFile::new()
            .context("Failed to create temporary file for object code")?;
        let obj_path = obj_file.path();
        code_gen.compile_to_object_file(obj_path)
            .context("Failed to compile IR to object file")?;
        
        if self.options.verbose {
            if self.options.color_output {
                println!("{} {}", "Generated object file:".blue(), 
                       obj_path.display().to_string().cyan());
            } else {
                println!("Generated object file: {}", obj_path.display());
            }
        }
        
        // Step 6: Link to create executable
        log_msg("Linking to create executable...", &mut timer);
        
        #[cfg(target_os = "windows")]
        let cc_cmd = "clang";
        #[cfg(not(target_os = "windows"))]
        let cc_cmd = "cc";
        
        let status = Command::new(cc_cmd)
            .arg(obj_path)
            .arg("-o")
            .arg(output_path)
            .status()
            .with_context(|| format!("Failed to run linker ({}). Is it installed on your system?", cc_cmd))?;
        
        if !status.success() {
            timer.report_total(false);
            return Err(anyhow::anyhow!("Linking failed with status: {}", status));
        }
        
        timer.checkpoint("Linking completed");
        
        if self.options.verbose || self.options.show_timings {
            if self.options.color_output {
                println!("{} {}", "Successfully compiled to:".green().bold(), 
                       output_path.display().to_string().cyan());
            } else {
                println!("Successfully compiled to: {}", output_path.display());
            }
        }
        
        // Report total compilation time
        timer.report_total(true);
        
        Ok(())
    }

    /// Run the source code using the interpreter
    pub fn run(&self, source: &str) -> Result<()> {
        self.run_with_filename(source, None)
    }
    
    /// Run the source code with a filename for better error reporting
    pub fn run_with_filename(&self, source: &str, filename: Option<String>) -> Result<()> {
        let mut timer = CompilationTimer::new(
            self.options.verbose || self.options.show_timings,
            self.options.color_output
        );
        
        // Step 1: Tokenize the source code
        if self.options.verbose {
            if self.options.color_output {
                println!("{}", "Tokenizing source code...".blue());
            } else {
                println!("Tokenizing source code...");
            }
        }
        
        let mut lexer = if let Some(filename) = &filename {
            Lexer::with_filename(source, filename.clone())
        } else {
            Lexer::new(source)
        };
        
        let tokens = lexer.scan_tokens()
            .context("Failed during lexical analysis")?;
        
        timer.checkpoint("Tokenization completed");
        
        // Step 2: Parse the tokens into an AST
        if self.options.verbose {
            if self.options.color_output {
                println!("{}", "Parsing tokens into AST...".blue());
            } else {
                println!("Parsing tokens into AST...");
            }
        }
        
        let mut parser = Parser::new(tokens);
        let program = if let Some(filename) = filename {
            let statements = parser.parse()
                .context("Failed during parsing")?
                .statements;
            crate::ast::Program::with_source(statements, filename)
        } else {
            parser.parse()
                .context("Failed during parsing")?
        };
        
        timer.checkpoint("Parsing completed");
        
        // Step 3: Interpret the program
        if self.options.verbose {
            if self.options.color_output {
                println!("{}", "Interpreting program...".blue());
            } else {
                println!("Interpreting program...");
            }
        }
        
        let mut interpreter = self.interpreter.clone();
        interpreter.interpret(&program)
            .context("Failed during interpretation")?;
        
        timer.checkpoint("Interpretation completed");
        
        // Report total execution time if enabled
        if self.options.show_timings {
            let total = timer.total();
            let time_str = format!("{:.3}s", total.as_secs_f64());
            
            if self.options.color_output {
                println!("{} in {}", "Program execution completed".green().bold(), time_str.yellow());
            } else {
                println!("Program execution completed in {}", time_str);
            }
        }
        
        Ok(())
    }

    /// Run a ShitRust program with asynchronous support from source string with an optional filename
    pub fn run_async_with_filename(&self, source: &str, filename: Option<String>) -> Result<()> {
        let start_time = std::time::Instant::now();
        
        // Create lexer
        let mut lexer = Lexer::with_filename(source, filename.unwrap_or_else(|| "unknown".to_string()));
        
        // Show timing if requested
        if self.options.show_timings {
            println!("{}: {:.2?}", "Lexing time".yellow().bold(), start_time.elapsed());
        }
        
        // Scan tokens
        let tokens = lexer.scan_tokens()?;
        
        // Create parser
        let mut parser = Parser::new(tokens);
        
        // Parse into AST
        let parse_start = std::time::Instant::now();
        let program = parser.parse()?;
        
        // Show timing if requested
        if self.options.show_timings {
            println!("{}: {:.2?}", "Parsing time".yellow().bold(), parse_start.elapsed());
        }
        
        // Execute with async runtime
        let execution_start = std::time::Instant::now();
        
        // Create interpreter
        let mut interpreter = Interpreter::new();
        
        // Add AsyncRuntime to the environment
        interpreter.load_module("stdlib::async_runtime")?;
        
        // Execute program
        interpreter.execute_async(&program)?;
        
        // Show timing if requested
        if self.options.show_timings {
            println!("{}: {:.2?}", "Execution time".yellow().bold(), execution_start.elapsed());
            println!("{}: {:.2?}", "Total time".yellow().bold(), start_time.elapsed());
        }
        
        Ok(())
    }
} 
