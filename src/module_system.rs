use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use crate::error::{ShitRustError, Result};
use crate::interpreter::{Interpreter, Value};
use crate::ast::{Stmt, Program};
use crate::lexer::Lexer;
use crate::parser::Parser;

/// Represents a module in the ShitRust language
pub struct Module {
    /// Name of the module
    pub name: String,
    
    /// Path to the module file
    pub path: PathBuf,
    
    /// Exported values from the module
    pub exports: HashMap<String, Value>,
    
    /// Whether the module has been loaded
    pub loaded: bool,
}

impl Module {
    /// Create a new module
    pub fn new(name: &str, path: PathBuf) -> Self {
        Module {
            name: name.to_string(),
            path,
            exports: HashMap::new(),
            loaded: false,
        }
    }
    
    /// Load a module from a file
    pub fn load(&mut self, interpreter: &mut Interpreter) -> Result<()> {
        if self.loaded {
            return Ok(());
        }
        
        // Read the file content
        let content = fs::read_to_string(&self.path)
            .map_err(|e| ShitRustError::IOException(format!("Error reading module '{}': {}", self.name, e)))?;
        
        // Parse the file
        let mut lexer = Lexer::with_filename(&content, self.path.to_string_lossy().into_owned());
        let tokens = lexer.scan_tokens()?;
        
        let mut parser = Parser::new(tokens);
        let program = parser.parse()?;
        
        // Execute the module code
        self.execute_module(program, interpreter)?;
        
        self.loaded = true;
        Ok(())
    }
    
    /// Execute module code and collect exports
    fn execute_module(&mut self, program: Program, interpreter: &mut Interpreter) -> Result<()> {
        // Create a new environment for the module
        let previous_env = interpreter.get_environment();
        let module_env = interpreter.create_module_environment();
        
        // Execute all statements in the module
        for stmt in program.statements {
            // Execute the statement
            interpreter.execute_statement(&stmt)?;
            
            // Check for exports
            if let Stmt::Function { name, is_public, .. } = &stmt {
                if *is_public {
                    // Add to exports if it's public
                    if let Some(value) = interpreter.get_value(name) {
                        self.exports.insert(name.clone(), value);
                    }
                }
            } else if let Stmt::Struct { name, is_public, .. } = &stmt {
                if *is_public {
                    // Add to exports if it's public
                    if let Some(value) = interpreter.get_value(name) {
                        self.exports.insert(name.clone(), value);
                    }
                }
            } else if let Stmt::Enum { name, is_public, .. } = &stmt {
                if *is_public {
                    // Add to exports if it's public
                    if let Some(value) = interpreter.get_value(name) {
                        self.exports.insert(name.clone(), value);
                    }
                }
            } else if let Stmt::Trait { name, is_public, .. } = &stmt {
                if *is_public {
                    // Add to exports if it's public
                    if let Some(value) = interpreter.get_value(name) {
                        self.exports.insert(name.clone(), value);
                    }
                }
            } else if let Stmt::Const { name, is_public, .. } = &stmt {
                if *is_public {
                    // Add to exports if it's public
                    if let Some(value) = interpreter.get_value(name) {
                        self.exports.insert(name.clone(), value);
                    }
                }
            } else if let Stmt::TypeAlias { name, is_public, .. } = &stmt {
                if *is_public {
                    // Add to exports if it's public
                    if let Some(value) = interpreter.get_value(&format!("type:{}", name)) {
                        self.exports.insert(name.clone(), value);
                    }
                }
            }
        }
        
        // Restore the previous environment
        interpreter.set_environment(previous_env);
        
        Ok(())
    }
}

/// Module registry for managing modules
pub struct ModuleRegistry {
    /// Map of module name to module
    modules: HashMap<String, Module>,
    
    /// Standard library modules
    stdlib_modules: HashMap<String, HashMap<String, Value>>,
    
    /// Search paths for modules
    search_paths: Vec<PathBuf>,
}

impl ModuleRegistry {
    /// Create a new module registry
    pub fn new() -> Self {
        let mut registry = ModuleRegistry {
            modules: HashMap::new(),
            stdlib_modules: HashMap::new(),
            search_paths: Vec::new(),
        };
        
        // Add current directory to search paths
        registry.add_search_path(".");
        
        // Initialize standard library modules
        registry.init_stdlib();
        
        registry
    }
    
    /// Add a search path for modules
    pub fn add_search_path<P: AsRef<Path>>(&mut self, path: P) {
        self.search_paths.push(PathBuf::from(path.as_ref()));
    }
    
    /// Initialize standard library modules
    fn init_stdlib(&mut self) {
        // Initialize standard library modules
        use crate::stdlib::collections;
        use crate::stdlib::io;
        use crate::stdlib::time;
        
        // Register standard library modules
        let collections_module: HashMap<String, Value> = collections::init_collections_module()
            .into_iter()
            .collect();
        self.stdlib_modules.insert("collections".to_string(), collections_module);
        
        let io_module: HashMap<String, Value> = io::init_io_module()
            .into_iter()
            .collect();
        self.stdlib_modules.insert("io".to_string(), io_module);
        
        let time_module: HashMap<String, Value> = time::init_time_module()
            .into_iter()
            .collect();
        self.stdlib_modules.insert("time".to_string(), time_module);
        
        // Additional modules can be added here as they are implemented
    }
    
    /// Import a module
    pub fn import_module(&mut self, name: &str, interpreter: &mut Interpreter) -> Result<HashMap<String, Value>> {
        // Check if it's a standard library module
        if let Some(stdlib) = self.stdlib_modules.get(name) {
            return Ok(stdlib.clone());
        }
        
        // Check if we've already loaded this module
        if let Some(module) = self.modules.get_mut(name) {
            if !module.loaded {
                module.load(interpreter)?;
            }
            return Ok(module.exports.clone());
        }
        
        // Try to find the module file
        let module_path = self.find_module_file(name)?;
        
        // Create and load the module
        let mut module = Module::new(name, module_path);
        module.load(interpreter)?;
        
        // Store the loaded module
        let exports = module.exports.clone();
        self.modules.insert(name.to_string(), module);
        
        Ok(exports)
    }
    
    /// Find a module file in the search paths
    fn find_module_file(&self, name: &str) -> Result<PathBuf> {
        // Replace dots with path separators
        let path_name = name.replace('.', "/");
        
        // Try to find the module in search paths
        for search_path in &self.search_paths {
            // Try .sr extension first
            let file_path = search_path.join(&path_name).with_extension("sr");
            if file_path.exists() {
                return Ok(file_path);
            }
            
            // Try directory with __init__.sr
            let dir_path = search_path.join(&path_name);
            let init_path = dir_path.join("__init__.sr");
            if init_path.exists() {
                return Ok(init_path);
            }
        }
        
        Err(ShitRustError::ModuleNotFound(name.to_string()))
    }
    
    /// Register a built-in module
    pub fn register_stdlib_module(&mut self, name: &str, exports: HashMap<String, Value>) {
        self.stdlib_modules.insert(name.to_string(), exports);
    }
} 