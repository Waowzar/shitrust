use clap::{Parser, Subcommand, ValueEnum};
use colored::*;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::io::Write;
use anyhow::{Result, Context};
use shitrust::compiler::{Compiler, CompilerOptions, OptimizationLevel};
use shitrust::error::ShitRustError;
use shitrust::formatter::Formatter;
use shitrust::type_system::TypeChecker;

/// ShitRust programming language compiler and runtime
#[derive(Parser)]
#[command(name = "shitrust")]
#[command(about = "ShitRust programming language compiler and runtime", long_about = None)]
#[command(version, author)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
    
    /// Set optimization level
    #[arg(short, long, value_enum, default_value_t = OptLevel::Default)]
    optimization: OptLevel,
    
    /// Enable debug information in output
    #[arg(short, long)]
    debug: bool,

    /// Show compilation timing information
    #[arg(short = 't', long)]
    timings: bool,

    /// Emit LLVM IR to a file with .ll extension
    #[arg(long)]
    emit_llvm: bool,

    /// Disable colored output
    #[arg(long)]
    no_color: bool,

    /// Enable strict type checking
    #[arg(long)]
    strict_types: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Debug, ValueEnum)]
enum OptLevel {
    None,
    Less,
    Default,
    Aggressive,
}

impl From<OptLevel> for OptimizationLevel {
    fn from(level: OptLevel) -> Self {
        match level {
            OptLevel::None => OptimizationLevel::None,
            OptLevel::Less => OptimizationLevel::Less,
            OptLevel::Default => OptimizationLevel::Default,
            OptLevel::Aggressive => OptimizationLevel::Aggressive,
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a ShitRust program
    Compile {
        /// Input file
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output file
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
    },
    /// Run a ShitRust program
    Run {
        /// Input file
        #[arg(value_name = "FILE")]
        input: PathBuf,
    },
    /// Format a ShitRust program
    Format {
        /// Input file
        #[arg(value_name = "FILE")]
        input: PathBuf,
        
        /// Apply changes in-place
        #[arg(short, long)]
        in_place: bool,
    },
    /// Check a ShitRust program for type errors
    Check {
        /// Input file
        #[arg(value_name = "FILE")]
        input: PathBuf,
    },
    /// Run a ShitRust program in async mode
    RunAsync {
        /// Input file
        #[arg(value_name = "FILE")]
        input: PathBuf,
    },
    /// Show information about ShitRust
    Info,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Apply no-color setting if provided
    if cli.no_color {
        colored::control::set_override(false);
    }
    
    // Create compiler options
    let options = CompilerOptions {
        verbose: cli.verbose,
        optimization_level: cli.optimization.into(),
        emit_debug_info: cli.debug,
        show_timings: cli.timings,
        emit_llvm_ir: cli.emit_llvm,
        color_output: !cli.no_color,
        strict_type_checking: cli.strict_types,
    };
    
    let compiler = Compiler::with_options(options);

    let result = match &cli.command {
        Commands::Compile { input, output } => {
            let output = output.clone().unwrap_or_else(|| {
                let mut path = input.clone();
                path.set_extension("exe");
                path
            });
            
            println!("{} {} to {}", "Compiling".green().bold(), 
                input.display().to_string().cyan(),
                output.display().to_string().cyan());
            
            let source = fs::read_to_string(input)
                .with_context(|| format!("Failed to read file: {}", input.display()))?;
            
            let filename = input.to_string_lossy().to_string();
            
            // Type check if strict types are enabled
            if cli.strict_types {
                println!("{} {}", "Type checking".green().bold(),
                    input.display().to_string().cyan());
                
                let mut lexer = shitrust::lexer::Lexer::with_filename(&source, filename.clone());
                let tokens = lexer.scan_tokens()?;
                
                let mut parser = shitrust::parser::Parser::new(tokens);
                let program = parser.parse()?;
                
                let mut type_checker = TypeChecker::new();
                type_checker.check_program(&program.statements)?;
                
                println!("{}", "Type check passed".green().bold());
            }
            
            match compiler.compile_with_filename(&source, &output, Some(filename)) {
                Ok(_) => {
                    if !cli.verbose && !cli.timings {
                        println!("{} {}", "Successfully compiled".green().bold(), 
                            output.display().to_string().cyan());
                    }
                    Ok(())
                },
                Err(e) => {
                    if let Some(sr_err) = e.downcast_ref::<ShitRustError>() {
                        eprintln!("{}", sr_err.format_error());
                    } else {
                        eprintln!("{}: {}", "Error".red().bold(), e);
                    }
                    Err(e)
                }
            }
        }
        Commands::Run { input } => {
            println!("{} {}", "Running".green().bold(), 
                input.display().to_string().cyan());
            
            let source = fs::read_to_string(input)
                .with_context(|| format!("Failed to read file: {}", input.display()))?;
            
            let filename = input.to_string_lossy().to_string();
            
            // Type check if strict types are enabled
            if cli.strict_types {
                println!("{} {}", "Type checking".green().bold(),
                    input.display().to_string().cyan());
                
                let mut lexer = shitrust::lexer::Lexer::with_filename(&source, filename.clone());
                let tokens = lexer.scan_tokens()?;
                
                let mut parser = shitrust::parser::Parser::new(tokens);
                let program = parser.parse()?;
                
                let mut type_checker = TypeChecker::new();
                type_checker.check_program(&program.statements)?;
                
                println!("{}", "Type check passed".green().bold());
            }
            
            match compiler.run_with_filename(&source, Some(filename)) {
                Ok(_) => Ok(()),
                Err(e) => {
                    if let Some(sr_err) = e.downcast_ref::<ShitRustError>() {
                        eprintln!("{}", sr_err.format_error());
                    } else {
                        eprintln!("{}: {}", "Error".red().bold(), e);
                    }
                    Err(e)
                }
            }
        }
        Commands::RunAsync { input } => {
            println!("{} {} in async mode", "Running".green().bold(), 
                input.display().to_string().cyan());
            
            let source = fs::read_to_string(input)
                .with_context(|| format!("Failed to read file: {}", input.display()))?;
            
            let filename = input.to_string_lossy().to_string();
            
            // Type check if strict types are enabled
            if cli.strict_types {
                let mut lexer = shitrust::lexer::Lexer::with_filename(&source, filename.clone());
                let tokens = lexer.scan_tokens()?;
                
                let mut parser = shitrust::parser::Parser::new(tokens);
                let program = parser.parse()?;
                
                let mut type_checker = TypeChecker::new();
                type_checker.check_program(&program.statements)?;
            }
            
            // Run with async runtime
            match compiler.run_async_with_filename(&source, Some(filename)) {
                Ok(_) => Ok(()),
                Err(e) => {
                    if let Some(sr_err) = e.downcast_ref::<ShitRustError>() {
                        eprintln!("{}", sr_err.format_error());
                    } else {
                        eprintln!("{}: {}", "Error".red().bold(), e);
                    }
                    Err(e)
                }
            }
        }
        Commands::Check { input } => {
            println!("{} {}", "Type checking".green().bold(), 
                input.display().to_string().cyan());
            
            let source = fs::read_to_string(input)
                .with_context(|| format!("Failed to read file: {}", input.display()))?;
            
            let filename = input.to_string_lossy().to_string();
            
            let mut lexer = shitrust::lexer::Lexer::with_filename(&source, filename);
            let tokens = lexer.scan_tokens()?;
            
            let mut parser = shitrust::parser::Parser::new(tokens);
            let program = parser.parse()?;
            
            let mut type_checker = TypeChecker::new();
            match type_checker.check_program(&program.statements) {
                Ok(_) => {
                    println!("{}", "Type check passed. No errors found.".green().bold());
                    Ok(())
                },
                Err(e) => {
                    if let Some(sr_err) = e.downcast_ref::<ShitRustError>() {
                        eprintln!("{}", sr_err.format_error());
                    } else {
                        eprintln!("{}: {}", "Type Error".red().bold(), e);
                    }
                    Err(e)
                }
            }
        }
        Commands::Format { input, in_place } => {
            println!("{} {}", "Formatting".green().bold(), 
                input.display().to_string().cyan());
            
            // Read the source file
            let source = fs::read_to_string(input)
                .with_context(|| format!("Failed to read file: {}", input.display()))?;
            
            // Format the source
            let mut formatter = Formatter::new();
            match formatter.format(&source) {
                Ok(formatted) => {
                    if *in_place {
                        // Write the formatted source back to the file
                        fs::write(input, &formatted)
                            .with_context(|| format!("Failed to write to file: {}", input.display()))?;
                        
                        println!("{} {}", "Successfully formatted".green().bold(), 
                            input.display().to_string().cyan());
                    } else {
                        // Write the formatted source to stdout
                        std::io::stdout().write_all(formatted.as_bytes())
                            .context("Failed to write to stdout")?;
                    }
                    Ok(())
                },
                Err(e) => {
                    eprintln!("{}: {}", "Formatting error".red().bold(), e);
                    Err(e)
                }
            }
        },
        Commands::Info => {
            println!("{}", "ShitRust Programming Language".green().bold());
            println!("Version: {}", env!("CARGO_PKG_VERSION").cyan());
            println!("Authors: {}", env!("CARGO_PKG_AUTHORS").cyan());
            
            println!("\n{}", "Compiler Features:".yellow().bold());
            println!("  • LLVM-based optimizing compiler");
            println!("  • Static type checker");
            println!("  • Interactive interpreter");
            println!("  • Source formatter");
            println!("  • Detailed error reporting");
            
            println!("\n{}", "Language Features:".yellow().bold());
            println!("  • Strong typing with type inference");
            println!("  • Memory safety mechanisms");
            println!("  • First-class functions");
            println!("  • Advanced pattern matching");
            println!("  • Structs and traits");
            println!("  • Generic types and functions");
            println!("  • Asynchronous programming");
            println!("  • Concurrency with threads and locks");
            println!("  • Cryptography functions");
            println!("  • Pipeline operator");
            
            println!("\n{}", "Usage Examples:".yellow().bold());
            println!("  Compile:    {} examples/hello.sr", "shitrust compile".cyan());
            println!("  Run:        {} examples/hello.sr", "shitrust run".cyan());
            println!("  Type check: {} examples/hello.sr", "shitrust check".cyan());
            println!("  Run async:  {} examples/async.sr", "shitrust run-async".cyan());
            println!("  Format:     {} -i examples/hello.sr", "shitrust format".cyan());
            
            println!("\n{}:", "More Information".yellow().bold());
            println!("  Website: {}", "https://shitrust-lang.org".cyan());
            println!("  GitHub:  {}", "https://github.com/Waowzar/shitrust".cyan());
            println!("  Docs:    {}", "https://docs.shitrust-lang.org".cyan());
            
            Ok(())
        }
    };

    // Return Ok if we succeed, otherwise exit with a non-zero status
    match result {
        Ok(_) => Ok(()),
        Err(_) => {
            // If we've already printed the error, just exit with code 1
            std::process::exit(1);
        }
    }
} 
