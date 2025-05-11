use std::collections::HashMap;
use crate::ast::{Type, Expr, Stmt, Pattern, Literal};
use crate::error::{ShitRustError, Result};

/// Represents a type environment for type checking
pub struct TypeEnvironment {
    /// Variables and their types
    variables: HashMap<String, Type>,
    
    /// Type aliases
    type_aliases: HashMap<String, Type>,
    
    /// Generic type parameters
    generic_params: Vec<String>,
    
    /// Parent environment (for nested scopes)
    parent: Option<Box<TypeEnvironment>>,
}

impl TypeEnvironment {
    /// Create a new type environment
    pub fn new() -> Self {
        TypeEnvironment {
            variables: HashMap::new(),
            type_aliases: HashMap::new(),
            generic_params: Vec::new(),
            parent: None,
        }
    }
    
    /// Create a new child environment
    pub fn new_child(&self) -> Self {
        TypeEnvironment {
            variables: HashMap::new(),
            type_aliases: HashMap::new(),
            generic_params: Vec::new(),
            parent: Some(Box::new(self.clone())),
        }
    }
    
    /// Define a variable type
    pub fn define(&mut self, name: String, typ: Type) {
        self.variables.insert(name, typ);
    }
    
    /// Get a variable's type
    pub fn get(&self, name: &str) -> Option<Type> {
        if let Some(typ) = self.variables.get(name) {
            Some(typ.clone())
        } else if let Some(parent) = &self.parent {
            parent.get(name)
        } else {
            None
        }
    }
    
    /// Define a type alias
    pub fn define_alias(&mut self, name: String, typ: Type) {
        self.type_aliases.insert(name, typ);
    }
    
    /// Resolve a type alias
    pub fn resolve_alias(&self, name: &str) -> Option<Type> {
        if let Some(typ) = self.type_aliases.get(name) {
            Some(typ.clone())
        } else if let Some(parent) = &self.parent {
            parent.resolve_alias(name)
        } else {
            None
        }
    }
    
    /// Add generic type parameters
    pub fn add_generic_params(&mut self, params: Vec<String>) {
        self.generic_params.extend(params);
    }
    
    /// Check if a type is a generic parameter
    pub fn is_generic_param(&self, name: &str) -> bool {
        self.generic_params.contains(&name.to_string()) || 
            if let Some(parent) = &self.parent {
                parent.is_generic_param(name)
            } else {
                false
            }
    }
}

impl Clone for TypeEnvironment {
    fn clone(&self) -> Self {
        TypeEnvironment {
            variables: self.variables.clone(),
            type_aliases: self.type_aliases.clone(),
            generic_params: self.generic_params.clone(),
            parent: self.parent.clone(),
        }
    }
}

/// Type checker for ShitRust
pub struct TypeChecker {
    /// Current type environment
    env: TypeEnvironment,
    
    /// Type constraints for generics
    constraints: HashMap<String, Vec<Type>>,
}

impl TypeChecker {
    /// Create a new type checker
    pub fn new() -> Self {
        TypeChecker {
            env: TypeEnvironment::new(),
            constraints: HashMap::new(),
        }
    }
    
    /// Type check a program
    pub fn check_program(&mut self, program: &[Stmt]) -> Result<()> {
        for stmt in program {
            self.check_statement(stmt)?;
        }
        Ok(())
    }
    
    /// Type check a statement
    pub fn check_statement(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Let { name, type_hint, value, mutable: _ } => {
                let value_type = self.infer_expr(value)?;
                
                if let Some(hint) = type_hint {
                    if !self.types_compatible(hint, &value_type)? {
                        return Err(ShitRustError::TypeError(
                            format!("Type mismatch: expected {:?}, found {:?}", hint, value_type)
                        ));
                    }
                    self.env.define(name.clone(), hint.clone());
                } else {
                    self.env.define(name.clone(), value_type);
                }
            },
            Stmt::Assign { target, value } => {
                let target_type = self.infer_expr(target)?;
                let value_type = self.infer_expr(value)?;
                
                if !self.types_compatible(&target_type, &value_type)? {
                    return Err(ShitRustError::TypeError(
                        format!("Type mismatch in assignment: expected {:?}, found {:?}", target_type, value_type)
                    ));
                }
            },
            Stmt::If { condition, then_block, else_block } => {
                let cond_type = self.infer_expr(condition)?;
                if !matches!(cond_type, Type::Bool) {
                    return Err(ShitRustError::TypeError(
                        format!("Condition must be a boolean, found {:?}", cond_type)
                    ));
                }
                
                let child_env = self.env.new_child();
                let old_env = std::mem::replace(&mut self.env, child_env);
                
                for stmt in then_block {
                    self.check_statement(stmt)?;
                }
                
                if let Some(else_block) = else_block {
                    // Reset environment for else block
                    self.env = old_env.new_child();
                    
                    for stmt in else_block {
                        self.check_statement(stmt)?;
                    }
                }
                
                self.env = old_env;
            },
            Stmt::Function { name, params, return_type, body, is_async, is_public: _, generic_params } => {
                // Create a new environment for the function
                let old_env = std::mem::replace(&mut self.env, self.env.new_child());
                
                // Add generic parameters
                self.env.add_generic_params(generic_params.clone());
                
                // Add parameters to environment
                for (param_name, param_type) in params {
                    self.env.define(param_name.clone(), param_type.clone());
                }
                
                // Check function body
                for stmt in body {
                    self.check_statement(stmt)?;
                }
                
                // Create function type
                let param_types: Vec<Type> = params.iter().map(|(_, t)| t.clone()).collect();
                let func_type = Type::Function(param_types, Box::new(return_type.clone()));
                
                // Add function to parent environment
                self.env = old_env;
                self.env.define(name.clone(), if *is_async {
                    Type::Future(Box::new(func_type))
                } else {
                    func_type
                });
            },
            Stmt::Struct { name, fields, methods, is_public: _, generic_params } => {
                // Add generic parameters
                let old_env = self.env.clone();
                self.env.add_generic_params(generic_params.clone());
                
                // Check field types
                for (_, field_type, _) in fields {
                    self.check_type(field_type)?;
                }
                
                // Check methods
                for method in methods {
                    if let Stmt::Function { .. } = method {
                        self.check_statement(method)?;
                    } else {
                        return Err(ShitRustError::TypeError(
                            "Only function definitions are allowed in structs".to_string()
                        ));
                    }
                }
                
                // Define the struct type
                self.env = old_env;
                self.env.define(name.clone(), Type::Custom(name.clone()));
            },
            Stmt::Enum { name, variants, is_public: _, generic_params } => {
                // Add generic parameters
                let old_env = self.env.clone();
                self.env.add_generic_params(generic_params.clone());
                
                // Check variant types
                for (_, variant_types) in variants {
                    for typ in variant_types {
                        self.check_type(typ)?;
                    }
                }
                
                // Define the enum type
                self.env = old_env;
                self.env.define(name.clone(), Type::Custom(name.clone()));
            },
            Stmt::Trait { name, methods, is_public: _, generic_params } => {
                // Add generic parameters
                let old_env = self.env.clone();
                self.env.add_generic_params(generic_params.clone());
                
                // Check method signatures
                for method in methods {
                    for (_, param_type) in &method.params {
                        self.check_type(param_type)?;
                    }
                    self.check_type(&method.return_type)?;
                }
                
                // Define the trait type
                self.env = old_env;
                self.env.define(name.clone(), Type::Trait(name.clone()));
            },
            // Add more statement types as needed
            _ => (), // Handle other statement types
        }
        
        Ok(())
    }
    
    /// Infer the type of an expression
    pub fn infer_expr(&mut self, expr: &Expr) -> Result<Type> {
        match expr {
            Expr::Literal(lit) => self.infer_literal(lit),
            Expr::Identifier(name) => {
                if let Some(typ) = self.env.get(name) {
                    Ok(typ)
                } else {
                    Err(ShitRustError::TypeError(format!("Undefined variable: {}", name)))
                }
            },
            Expr::BinaryOp { left, op, right } => {
                let left_type = self.infer_expr(left)?;
                let right_type = self.infer_expr(right)?;
                
                match op {
                    crate::ast::BinOp::Add | 
                    crate::ast::BinOp::Sub | 
                    crate::ast::BinOp::Mul | 
                    crate::ast::BinOp::Div | 
                    crate::ast::BinOp::Mod => {
                        if self.types_compatible(&left_type, &right_type)? {
                            Ok(left_type)
                        } else {
                            Err(ShitRustError::TypeError(
                                format!("Incompatible types for binary operation: {:?} and {:?}", left_type, right_type)
                            ))
                        }
                    },
                    crate::ast::BinOp::Eq | 
                    crate::ast::BinOp::Ne | 
                    crate::ast::BinOp::Lt | 
                    crate::ast::BinOp::Le | 
                    crate::ast::BinOp::Gt | 
                    crate::ast::BinOp::Ge => {
                        if self.types_compatible(&left_type, &right_type)? {
                            Ok(Type::Bool)
                        } else {
                            Err(ShitRustError::TypeError(
                                format!("Incompatible types for comparison: {:?} and {:?}", left_type, right_type)
                            ))
                        }
                    },
                    crate::ast::BinOp::And | 
                    crate::ast::BinOp::Or => {
                        if matches!(left_type, Type::Bool) && matches!(right_type, Type::Bool) {
                            Ok(Type::Bool)
                        } else {
                            Err(ShitRustError::TypeError(
                                format!("Boolean operations require boolean operands, found {:?} and {:?}", left_type, right_type)
                            ))
                        }
                    },
                    _ => Ok(left_type), // Handle other operators as needed
                }
            },
            // Add more expression types as needed
            _ => Ok(Type::Custom("any".to_string())), // Default case, should be replaced with proper handling
        }
    }
    
    /// Infer the type of a literal
    fn infer_literal(&self, lit: &Literal) -> Result<Type> {
        match lit {
            Literal::Int(_) => Ok(Type::Int),
            Literal::Float(_) => Ok(Type::Float),
            Literal::Bool(_) => Ok(Type::Bool),
            Literal::String(_) => Ok(Type::String),
            Literal::Char(_) => Ok(Type::Char),
            Literal::List(elements) => {
                if elements.is_empty() {
                    // Empty list, can't infer element type
                    Ok(Type::List(Box::new(Type::Custom("any".to_string()))))
                } else {
                    // Infer type from first element
                    let first_type = self.infer_expr(&elements[0])?;
                    
                    // Check that all elements have compatible types
                    for element in elements.iter().skip(1) {
                        let element_type = self.infer_expr(element)?;
                        if !self.types_compatible(&first_type, &element_type)? {
                            return Err(ShitRustError::TypeError(
                                format!("Inconsistent element types in list: found both {:?} and {:?}", 
                                       first_type, element_type)
                            ));
                        }
                    }
                    
                    Ok(Type::List(Box::new(first_type)))
                }
            },
            // Add more literal types as needed
            _ => Ok(Type::Custom("any".to_string())), // Default case, should be replaced with proper handling
        }
    }
    
    /// Check if two types are compatible
    pub fn types_compatible(&self, expected: &Type, actual: &Type) -> Result<bool> {
        match (expected, actual) {
            // Same types are always compatible
            (a, b) if a == b => Ok(true),
            
            // Generic type parameters can match any type
            (Type::Custom(name), _) if self.env.is_generic_param(name) => Ok(true),
            (_, Type::Custom(name)) if self.env.is_generic_param(name) => Ok(true),
            
            // Container types compatibility
            (Type::List(a), Type::List(b)) => self.types_compatible(a, b),
            (Type::Option(a), Type::Option(b)) => self.types_compatible(a, b),
            (Type::Result(a1, a2), Type::Result(b1, b2)) => {
                Ok(self.types_compatible(a1, b1)? && self.types_compatible(a2, b2)?)
            },
            
            // Function types compatibility
            (Type::Function(a_params, a_ret), Type::Function(b_params, b_ret)) => {
                if a_params.len() != b_params.len() {
                    return Ok(false);
                }
                
                for (a_param, b_param) in a_params.iter().zip(b_params.iter()) {
                    if !self.types_compatible(a_param, b_param)? {
                        return Ok(false);
                    }
                }
                
                self.types_compatible(a_ret, b_ret)
            },
            
            // Type aliases
            (Type::Custom(name), other) => {
                if let Some(aliased) = self.env.resolve_alias(name) {
                    self.types_compatible(&aliased, other)
                } else {
                    Ok(false)
                }
            },
            (other, Type::Custom(name)) => {
                if let Some(aliased) = self.env.resolve_alias(name) {
                    self.types_compatible(other, &aliased)
                } else {
                    Ok(false)
                }
            },
            
            // Other cases are not compatible
            _ => Ok(false),
        }
    }
    
    /// Check that a type is valid
    fn check_type(&self, typ: &Type) -> Result<()> {
        match typ {
            Type::Custom(name) => {
                if !self.env.is_generic_param(name) && self.env.resolve_alias(name).is_none() && self.env.get(name).is_none() {
                    return Err(ShitRustError::TypeError(format!("Undefined type: {}", name)));
                }
            },
            Type::List(element_type) => self.check_type(element_type)?,
            Type::Option(inner_type) => self.check_type(inner_type)?,
            Type::Result(ok_type, err_type) => {
                self.check_type(ok_type)?;
                self.check_type(err_type)?;
            },
            Type::Function(param_types, return_type) => {
                for param_type in param_types {
                    self.check_type(param_type)?;
                }
                self.check_type(return_type)?;
            },
            // Add more type checks as needed
            _ => (), // Primitive types are always valid
        }
        
        Ok(())
    }
} 
