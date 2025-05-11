use std::collections::HashMap;
use crate::ast::{Type, Stmt};
use crate::error::{ShitRustError, Result};
use crate::interpreter::{Value, Interpreter};

/// Represents a trait definition
#[derive(Debug, Clone)]
pub struct Trait {
    /// Name of the trait
    pub name: String,
    
    /// Methods required by the trait
    pub methods: HashMap<String, TraitMethod>,
    
    /// Generic parameters for the trait
    pub generic_params: Vec<String>,
}

/// Represents a method signature in a trait
#[derive(Debug, Clone)]
pub struct TraitMethod {
    /// Name of the method
    pub name: String,
    
    /// Parameters of the method
    pub params: Vec<(String, Type)>,
    
    /// Return type of the method
    pub return_type: Type,
    
    /// Whether the method is async
    pub is_async: bool,
    
    /// Default implementation, if any
    pub default_impl: Option<Vec<Stmt>>,
}

/// Represents a trait implementation for a specific type
#[derive(Debug, Clone)]
pub struct TraitImpl {
    /// The trait being implemented
    pub trait_name: String,
    
    /// The type implementing the trait
    pub type_name: String,
    
    /// Methods implemented for the trait
    pub methods: HashMap<String, Value>,
    
    /// Generic parameters for the implementation
    pub generic_params: Vec<String>,
}

/// Registry for traits and their implementations
pub struct TraitRegistry {
    /// Map of trait name to trait definition
    traits: HashMap<String, Trait>,
    
    /// Map of (trait name, type name) to trait implementation
    impls: HashMap<(String, String), TraitImpl>,
}

impl TraitRegistry {
    /// Create a new trait registry
    pub fn new() -> Self {
        TraitRegistry {
            traits: HashMap::new(),
            impls: HashMap::new(),
        }
    }
    
    /// Register a trait
    pub fn register_trait(&mut self, trait_def: Trait) {
        self.traits.insert(trait_def.name.clone(), trait_def);
    }
    
    /// Register a trait implementation
    pub fn register_impl(&mut self, impl_def: TraitImpl) -> Result<()> {
        // Check if the trait exists
        if !self.traits.contains_key(&impl_def.trait_name) {
            return Err(ShitRustError::TypeError(
                format!("Cannot implement unknown trait '{}'", impl_def.trait_name)
            ));
        }
        
        // Check if all required methods are implemented
        let trait_def = &self.traits[&impl_def.trait_name];
        
        for (method_name, method_def) in &trait_def.methods {
            if !impl_def.methods.contains_key(method_name) && method_def.default_impl.is_none() {
                return Err(ShitRustError::TypeError(
                    format!("Missing implementation for required method '{}' in trait '{}'", 
                            method_name, impl_def.trait_name)
                ));
            }
        }
        
        // Register the implementation
        self.impls.insert((impl_def.trait_name.clone(), impl_def.type_name.clone()), impl_def);
        
        Ok(())
    }
    
    /// Check if a type implements a trait
    pub fn implements_trait(&self, trait_name: &str, type_name: &str) -> bool {
        self.impls.contains_key(&(trait_name.to_string(), type_name.to_string()))
    }
    
    /// Get a method from a trait implementation
    pub fn get_trait_method(&self, trait_name: &str, type_name: &str, method_name: &str) -> Option<Value> {
        let key = (trait_name.to_string(), type_name.to_string());
        
        if let Some(impl_def) = self.impls.get(&key) {
            if let Some(method) = impl_def.methods.get(method_name) {
                return Some(method.clone());
            }
        }
        
        // Check for default implementation
        if let Some(trait_def) = self.traits.get(trait_name) {
            if let Some(method_def) = trait_def.methods.get(method_name) {
                if let Some(default_impl) = &method_def.default_impl {
                    // TODO: Create a function value from the default implementation
                    // This would involve creating a closure with the appropriate environment
                    // return Some(Value::Function(...));
                }
            }
        }
        
        None
    }
    
    /// Create a trait implementation from AST nodes
    pub fn create_trait_impl_from_ast(&self, 
                                      trait_name: &str, 
                                      type_name: &str, 
                                      methods: &[Stmt],
                                      generic_params: &[String],
                                      interpreter: &mut Interpreter) -> Result<TraitImpl> {
        // Check if the trait exists
        if !self.traits.contains_key(trait_name) {
            return Err(ShitRustError::TypeError(
                format!("Cannot implement unknown trait '{}'", trait_name)
            ));
        }
        
        let mut impl_methods = HashMap::new();
        
        // Process each method declaration
        for method in methods {
            if let Stmt::Function { name, .. } = method {
                // Execute the function declaration to get the function value
                interpreter.execute_statement(method)?;
                
                // Retrieve the function value from the environment
                if let Some(func_value) = interpreter.get_value(name) {
                    impl_methods.insert(name.clone(), func_value);
                }
            }
        }
        
        // Create the implementation
        let impl_def = TraitImpl {
            trait_name: trait_name.to_string(),
            type_name: type_name.to_string(),
            methods: impl_methods,
            generic_params: generic_params.to_vec(),
        };
        
        Ok(impl_def)
    }
    
    /// Create a trait definition from AST nodes
    pub fn create_trait_from_ast(&self,
                                name: &str,
                                methods: &[TraitMethod],
                                generic_params: &[String]) -> Trait {
        let mut trait_methods = HashMap::new();
        
        // Process each method
        for method in methods {
            trait_methods.insert(method.name.clone(), method.clone());
        }
        
        // Create the trait definition
        Trait {
            name: name.to_string(),
            methods: trait_methods,
            generic_params: generic_params.to_vec(),
        }
    }
}

// Extension trait for Interpreter to handle traits
pub trait TraitSupport {
    /// Create a trait definition from a statement
    fn define_trait(&mut self, stmt: &Stmt) -> Result<()>;
    
    /// Create a trait implementation from a statement
    fn implement_trait(&mut self, stmt: &Stmt) -> Result<()>;
    
    /// Call a trait method
    fn call_trait_method(&mut self, object: &Value, trait_name: &str, method_name: &str, args: &[Value]) -> Result<Value>;
}

impl TraitSupport for Interpreter {
    fn define_trait(&mut self, stmt: &Stmt) -> Result<()> {
        if let Stmt::Trait { name, methods, generic_params, .. } = stmt {
            // Create trait methods from AST methods
            let mut trait_methods = Vec::new();
            
            // Process each method in the trait
            for method in methods {
                if let Stmt::Function { name: method_name, params, return_type, body, is_async, .. } = method {
                    let trait_method = TraitMethod {
                        name: method_name.clone(),
                        params: params.clone(),
                        return_type: return_type.clone(),
                        is_async: *is_async,
                        default_impl: if body.is_empty() { None } else { Some(body.clone()) },
                    };
                    trait_methods.push(trait_method);
                }
            }
            
            // Create the trait
            let trait_def = self.trait_registry.create_trait_from_ast(
                name, 
                &trait_methods,
                generic_params
            );
            
            // Register the trait
            self.trait_registry.register_trait(trait_def);
            
            Ok(())
        } else {
            Err(ShitRustError::TypeError("Expected trait definition".to_string()))
        }
    }
    
    fn implement_trait(&mut self, stmt: &Stmt) -> Result<()> {
        if let Stmt::Impl { trait_name, type_name, methods, generic_params, .. } = stmt {
            // Create and register the trait implementation
            let impl_def = self.trait_registry.create_trait_impl_from_ast(
                trait_name,
                type_name,
                methods,
                generic_params,
                self
            )?;
            
            self.trait_registry.register_impl(impl_def)?;
            
            Ok(())
        } else {
            Err(ShitRustError::TypeError("Expected trait implementation".to_string()))
        }
    }
    
    fn call_trait_method(&mut self, object: &Value, trait_name: &str, method_name: &str, args: &[Value]) -> Result<Value> {
        // Get the object's type
        let type_name = match object {
            Value::Object(obj) => {
                if let Some(Value::String(type_name)) = obj.get("__type") {
                    type_name.clone()
                } else {
                    return Err(ShitRustError::TypeError("Object has no type information".to_string()));
                }
            },
            Value::Int(_) => "int".to_string(),
            Value::Float(_) => "float".to_string(),
            Value::Bool(_) => "bool".to_string(),
            Value::String(_) => "string".to_string(),
            Value::List(_) => "list".to_string(),
            Value::Dict(_) => "map".to_string(),
            Value::Set(_) => "set".to_string(),
            Value::Function { .. } => "function".to_string(),
            Value::NativeFunction { .. } => "function".to_string(),
            Value::None => return Err(ShitRustError::TypeError("Cannot call trait method on None".to_string())),
            Value::Range { .. } => "range".to_string(),
        };
        
        // Check if the type implements the trait
        if !self.trait_registry.implements_trait(trait_name, &type_name) {
            return Err(ShitRustError::TypeError(
                format!("Type '{}' does not implement trait '{}'", type_name, trait_name)
            ));
        }
        
        // Get the trait method
        if let Some(method) = self.trait_registry.get_trait_method(trait_name, &type_name, method_name) {
            // Create new args with the object as 'this'
            let mut method_args = vec![object.clone()];
            method_args.extend_from_slice(args);
            
            // Call the method
            match method {
                Value::Function { func, arity, .. } => {
                    // Handle function arity
                    if arity >= 0 && method_args.len() != arity as usize {
                        return Err(ShitRustError::RuntimeError(
                            format!("Wrong number of arguments for trait method: expected {}, got {}", 
                                    arity, method_args.len())
                        ));
                    }
                    
                    // Call the function
                    self.call_function(&func, &method_args)
                },
                Value::NativeFunction { func, arity, .. } => {
                    // Handle function arity
                    if arity >= 0 && method_args.len() != arity as usize {
                        return Err(ShitRustError::RuntimeError(
                            format!("Wrong number of arguments for trait method: expected {}, got {}", 
                                    arity, method_args.len())
                        ));
                    }
                    
                    // Call the native function
                    func(&method_args)
                },
                _ => Err(ShitRustError::TypeError("Trait method is not callable".to_string())),
            }
        } else {
            Err(ShitRustError::RuntimeError(
                format!("Trait '{}' has no method '{}'", trait_name, method_name)
            ))
        }
    }
} 