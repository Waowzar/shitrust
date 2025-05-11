use std::collections::HashMap;
use crate::ast::{Program, Stmt, Expr, Literal, BinOp, UnaryOp, Pattern, OptionalChainItem, Type};
use crate::error::ShitRustError;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Char(char),
    List(Vec<Value>),
    Dict(HashMap<String, Value>),
    Tuple(Vec<Value>),
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
        closure: Box<Environment>,
    },
    NativeFunction {
        name: String,
        func: fn(Vec<Value>) -> Result<Value, ShitRustError>,
    },
    None,
    Optional(Option<Box<Value>>),
    Trait(TraitDefinition),
    TraitImpl(TraitImplementation),
}

impl Value {
    pub fn type_name(&self) -> String {
        match self {
            Value::Int(_) => "int".to_string(),
            Value::Float(_) => "float".to_string(),
            Value::Bool(_) => "bool".to_string(),
            Value::String(_) => "string".to_string(),
            Value::Char(_) => "char".to_string(),
            Value::List(_) => "list".to_string(),
            Value::Dict(_) => "dict".to_string(),
            Value::Tuple(_) => "tuple".to_string(),
            Value::Function { .. } => "function".to_string(),
            Value::NativeFunction { .. } => "native function".to_string(),
            Value::None => "none".to_string(),
            Value::Optional(_) => "optional".to_string(),
            Value::Trait(_) => "trait".to_string(),
            Value::TraitImpl(_) => "trait implementation".to_string(),
        }
    }
    
    pub fn to_string(&self) -> String {
        match self {
            Value::Int(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::String(s) => s.clone(),
            Value::Char(c) => c.to_string(),
            Value::List(items) => {
                let items_str: Vec<String> = items.iter().map(|v| v.to_string()).collect();
                format!("[{}]", items_str.join(", "))
            },
            Value::Dict(map) => {
                let items: Vec<String> = map.iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_string()))
                    .collect();
                format!("{{{}}}", items.join(", "))
            },
            Value::Tuple(items) => {
                let items_str: Vec<String> = items.iter().map(|v| v.to_string()).collect();
                format!("({})", items_str.join(", "))
            },
            Value::Function { name, .. } => format!("<function {}>", name),
            Value::NativeFunction { name, .. } => format!("<native function {}>", name),
            Value::None => "none".to_string(),
            Value::Optional(opt) => {
                if let Some(value) = opt {
                    value.to_string()
                } else {
                    "None".to_string()
                }
            },
            Value::Trait(trait_def) => format!("<trait {}>", trait_def.name),
            Value::TraitImpl(impl_def) => format!("<trait implementation {}>", impl_def.type_name),
        }
    }
}

#[derive(Clone)]
pub struct Environment {
    values: HashMap<String, Value>,
    parent: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        let mut env = Environment {
            values: HashMap::new(),
            parent: None,
        };
        
        // Add built-in functions
        env.define("println".to_string(), Value::NativeFunction {
            name: "println".to_string(),
            func: |args| {
                let strings: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
                println!("{}", strings.join(" "));
                Ok(Value::None)
            },
        });
        
        env.define("print".to_string(), Value::NativeFunction {
            name: "print".to_string(),
            func: |args| {
                let strings: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
                print!("{}", strings.join(" "));
                Ok(Value::None)
            },
        });
        
        env
    }

    pub fn with_parent(parent: Environment) -> Self {
        Environment {
            values: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Result<Value, ShitRustError> {
        if let Some(value) = self.values.get(name) {
            Ok(value.clone())
        } else if let Some(parent) = &self.parent {
            parent.get(name)
        } else {
            Err(ShitRustError::UndefinedVariable(name.to_string()))
        }
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), ShitRustError> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            Ok(())
        } else if let Some(parent) = &mut self.parent {
            parent.assign(name, value)
        } else {
            Err(ShitRustError::UndefinedVariable(name.to_string()))
        }
    }
}

#[derive(Clone)]
pub struct Interpreter {
    environment: Environment,
    globals: Environment,
    current_source_file: String,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Environment::new();
        Interpreter {
            environment: globals.clone(),
            globals,
            current_source_file: String::new(),
        }
    }

    pub fn interpret(&mut self, program: &Program) -> Result<(), ShitRustError> {
        for stmt in &program.statements {
            self.execute_stmt(stmt)?;
        }
        
        Ok(())
    }
    
    fn execute_stmt(&mut self, stmt: &Stmt) -> Result<(), ShitRustError> {
        match stmt {
            Stmt::Expr(expr) => {
                self.evaluate_expr(expr)?;
                Ok(())
            },
            Stmt::Let { name, type_hint: _, value, mutable: _ } => {
                let evaluated = self.evaluate_expr(value)?;
                self.environment.define(name.clone(), evaluated);
                Ok(())
            },
            Stmt::Assign { target, value } => {
                if let Expr::Identifier(name) = target {
                    let evaluated = self.evaluate_expr(value)?;
                    self.environment.assign(name, evaluated)?;
                    Ok(())
                } else {
                    Err(ShitRustError::RuntimeError("Invalid assignment target".to_string()))
                }
            },
            Stmt::If { condition, then_block, else_block } => {
                let condition_value = self.evaluate_expr(condition)?;
                
                if let Value::Bool(true) = condition_value {
                    for stmt in then_block {
                        self.execute_stmt(stmt)?;
                    }
                } else if let Some(else_statements) = else_block {
                    for stmt in else_statements {
                        self.execute_stmt(stmt)?;
                    }
                }
                
                Ok(())
            },
            Stmt::While { condition, body } => {
                while let Value::Bool(true) = self.evaluate_expr(condition)? {
                    for stmt in body {
                        self.execute_stmt(stmt)?;
                    }
                }
                
                Ok(())
            },
            Stmt::For { var, iterator, body } => {
                // Get iterator value
                let iterator_value = self.evaluate_expr(iterator)?;
                
                // For now, only support iterating over lists
                if let Value::List(items) = iterator_value {
                    for item in items {
                        // Create new environment for each iteration
                        let previous_env = self.environment.clone();
                        self.environment = Environment::with_parent(previous_env);
                        
                        // Define loop variable
                        self.environment.define(var.clone(), item);
                        
                        // Execute body
                        for stmt in body {
                            self.execute_stmt(stmt)?;
                        }
                        
                        // Restore previous environment
                        self.environment = *self.environment.parent.unwrap();
                    }
                    
                    Ok(())
                } else {
                    Err(ShitRustError::TypeError(format!("Cannot iterate over {}", iterator_value.type_name())))
                }
            },
            Stmt::Return(value_opt) => {
                // For simplicity, we're not implementing return values yet
                if let Some(value) = value_opt {
                    let _ = self.evaluate_expr(value)?;
                }
                
                // In a real interpreter, we would handle return values properly
                Ok(())
            },
            Stmt::Function { name, params, return_type: _, body } => {
                // Convert params to just names for the function value
                let param_names: Vec<String> = params.iter().map(|(name, _)| name.clone()).collect();
                
                let function = Value::Function {
                    name: name.clone(),
                    params: param_names,
                    body: body.clone(),
                    closure: Box::new(self.environment.clone()),
                };
                
                self.environment.define(name.clone(), function);
                Ok(())
            },
            Stmt::Trait { name, methods, is_public: _, generic_params } => {
                let trait_methods = methods.iter().map(|m| {
                    (m.name.clone(), TraitMethod {
                        name: m.name.clone(),
                        params: m.params.clone(),
                        return_type: m.return_type.clone(),
                        body: m.body.clone(),
                        is_async: m.is_async,
                    })
                }).collect();
                
                let trait_def = TraitDefinition {
                    name: name.clone(),
                    methods: trait_methods,
                    generic_params: generic_params.clone(),
                };
                
                self.environment.define(name, Value::Trait(trait_def));
                Ok(())
            },
            Stmt::Impl { trait_name, type_name, methods, generic_params } => {
                // Create function values for all methods
                let mut method_map = HashMap::new();
                
                for method in methods {
                    if let Stmt::Function { name, params, return_type, body, is_async, .. } = method {
                        let func = FunctionValue {
                            name: name.clone(),
                            params: params.clone(),
                            body: body.clone(),
                            return_type: return_type.clone(),
                            closure_env: self.environment.clone(),
                            is_async: *is_async,
                        };
                        
                        method_map.insert(name.clone(), func);
                    }
                }
                
                let impl_def = TraitImplementation {
                    trait_name: trait_name.clone(),
                    type_name: type_name.clone(),
                    methods: method_map,
                    generic_params: generic_params.clone(),
                };
                
                // Store the implementation
                self.environment.define(&format!("impl:{}:{}", 
                    trait_name.as_deref().unwrap_or(""), type_name), 
                    Value::TraitImpl(impl_def));
                
                Ok(())
            },
            Stmt::TypeAlias { name, alias_type, is_public: _, generic_params: _ } => {
                // For now, we just store the type alias in the environment
                self.environment.define(&format!("type:{}", name), 
                    Value::String(alias_type.to_string()));
                Ok(())
            },
            Stmt::Use { path, as_name } => {
                // For now, we'll just register the module path in the environment
                let module_name = as_name.clone().unwrap_or_else(|| {
                    path.split('.').last().unwrap_or(path).to_string()
                });
                
                self.environment.define(&module_name, Value::String(path.clone()));
                Ok(())
            },
            Stmt::Loop { body } => {
                loop {
                    match self.execute_block(body, Environment::new_with_enclosing(self.environment.clone())) {
                        Ok(_) => (),
                        Err(ShitRustError::Break) => break,
                        Err(e) => return Err(e),
                    }
                }
                Ok(())
            },
            Stmt::Const { name, type_hint: _, value, is_public: _ } => {
                let value = self.evaluate_expr(value)?;
                self.environment.define(name, value);
                Ok(())
            },
            _ => {
                // Other statement types not yet implemented
                println!("Statement type not yet implemented: {:?}", stmt);
                Ok(())
            }
        }
    }
    
    fn evaluate_expr(&mut self, expr: &Expr) -> Result<Value, ShitRustError> {
        match expr {
            Expr::Literal(lit) => self.evaluate_literal(lit),
            Expr::Identifier(name) => self.environment.get(name),
            Expr::BinaryOp { left, op, right } => {
                let left_val = self.evaluate_expr(left)?;
                let right_val = self.evaluate_expr(right)?;
                
                match op {
                    BinOp::Add => self.add(left_val, right_val),
                    BinOp::Sub => self.subtract(left_val, right_val),
                    BinOp::Mul => self.multiply(left_val, right_val),
                    BinOp::Div => self.divide(left_val, right_val),
                    BinOp::Mod => self.modulo(left_val, right_val),
                    BinOp::Eq => self.equals(left_val, right_val),
                    BinOp::Ne => {
                        let result = self.equals(left_val, right_val)?;
                        if let Value::Bool(b) = result {
                            Ok(Value::Bool(!b))
                        } else {
                            Err(ShitRustError::RuntimeError("Equality check did not return boolean".to_string()))
                        }
                    },
                    BinOp::Lt => self.less_than(left_val, right_val),
                    BinOp::Le => self.less_equal(left_val, right_val),
                    BinOp::Gt => self.greater_than(left_val, right_val),
                    BinOp::Ge => self.greater_equal(left_val, right_val),
                    BinOp::And => {
                        if let Value::Bool(false) = left_val {
                            Ok(Value::Bool(false))
                        } else {
                            match right_val {
                                Value::Bool(b) => Ok(Value::Bool(b)),
                                _ => Err(ShitRustError::TypeError(format!("Expected boolean for '&&', got {}", right_val.type_name()))),
                            }
                        }
                    },
                    BinOp::Or => {
                        if let Value::Bool(true) = left_val {
                            Ok(Value::Bool(true))
                        } else {
                            match right_val {
                                Value::Bool(b) => Ok(Value::Bool(b)),
                                _ => Err(ShitRustError::TypeError(format!("Expected boolean for '||', got {}", right_val.type_name()))),
                            }
                        }
                    },
                }
            },
            Expr::UnaryOp { op, expr } => {
                let value = self.evaluate_expr(expr)?;
                
                match op {
                    UnaryOp::Neg => {
                        match value {
                            Value::Int(i) => Ok(Value::Int(-i)),
                            Value::Float(f) => Ok(Value::Float(-f)),
                            _ => Err(ShitRustError::TypeError(format!("Cannot negate {}", value.type_name()))),
                        }
                    },
                    UnaryOp::Not => {
                        match value {
                            Value::Bool(b) => Ok(Value::Bool(!b)),
                            _ => Err(ShitRustError::TypeError(format!("Cannot apply '!' to {}", value.type_name()))),
                        }
                    },
                }
            },
            Expr::Call { func, args } => {
                let callee = self.evaluate_expr(func)?;
                
                let mut evaluated_args = Vec::new();
                for arg in args {
                    evaluated_args.push(self.evaluate_expr(arg)?);
                }
                
                match callee {
                    Value::NativeFunction { func, .. } => {
                        func(evaluated_args)
                    },
                    Value::Function { name, params, body, closure } => {
                        if params.len() != evaluated_args.len() {
                            return Err(ShitRustError::RuntimeError(
                                format!("Expected {} arguments but got {}", params.len(), evaluated_args.len())
                            ));
                        }
                        
                        // Create new environment with function's closure as parent
                        let mut env = Environment::with_parent(*closure);
                        
                        // Add parameters to the environment
                        for (param, arg) in params.iter().zip(evaluated_args) {
                            env.define(param.clone(), arg);
                        }
                        
                        // Save current environment
                        let previous_env = self.environment.clone();
                        self.environment = env;
                        
                        // Execute function body
                        for stmt in &body {
                            // TODO: Handle return values properly
                            self.execute_stmt(stmt)?;
                        }
                        
                        // Restore previous environment
                        self.environment = previous_env;
                        
                        // For now, always return None
                        Ok(Value::None)
                    },
                    _ => Err(ShitRustError::RuntimeError(format!("Cannot call {}", callee.type_name()))),
                }
            },
            Expr::MethodCall { object, method, args } => {
                let obj_val = self.evaluate_expr(object)?;
                
                // For demonstration, let's implement a simple to_string method for all types
                if method == "to_string" {
                    return Ok(Value::String(obj_val.to_string()));
                }
                
                // Other methods would be implemented here
                
                Err(ShitRustError::RuntimeError(format!("Method '{}' not found on {}", method, obj_val.type_name())))
            },
            Expr::OptionalChain { expr, chain } => {
                let mut value = self.evaluate_expr(expr)?;
                
                // If the base value is None or null, return None
                if matches!(value, Value::None) {
                    return Ok(Value::None);
                }
                
                for item in chain {
                    match item {
                        OptionalChainItem::Field(field) => {
                            // Handle field access on optional values
                            match value {
                                Value::Object(ref object) => {
                                    if let Some(field_value) = object.get(field) {
                                        value = field_value.clone();
                                    } else {
                                        return Ok(Value::None);
                                    }
                                },
                                Value::None => return Ok(Value::None),
                                Value::Optional(opt) => {
                                    if let Some(inner) = opt {
                                        if let Value::Object(ref object) = **inner {
                                            if let Some(field_value) = object.get(field) {
                                                value = field_value.clone();
                                            } else {
                                                return Ok(Value::None);
                                            }
                                        } else {
                                            return Ok(Value::None);
                                        }
                                    } else {
                                        return Ok(Value::None);
                                    }
                                },
                                _ => return Err(ShitRustError::TypeError(
                                    format!("Cannot access property '{}' of non-object value", field)
                                )),
                            }
                        },
                        OptionalChainItem::Method(method, args) => {
                            // Handle method calls on optional values
                            let evaluated_args: Result<Vec<Value>, ShitRustError> = 
                                args.iter().map(|arg| self.evaluate_expr(arg)).collect();
                            
                            match value {
                                Value::Object(ref object) => {
                                    if let Some(method_value) = object.get(method) {
                                        match method_value {
                                            Value::Function(func) => {
                                                // Call the method with the object as 'this'
                                                value = self.call_function(func, &evaluated_args?, Some(Value::Object(object.clone())))?;
                                            },
                                            _ => return Err(ShitRustError::TypeError(
                                                format!("Property '{}' is not a method", method)
                                            )),
                                        }
                                    } else {
                                        return Ok(Value::None);
                                    }
                                },
                                Value::None => return Ok(Value::None),
                                Value::Optional(opt) => {
                                    if let Some(inner) = opt {
                                        if let Value::Object(ref object) = **inner {
                                            if let Some(method_value) = object.get(method) {
                                                match method_value {
                                                    Value::Function(func) => {
                                                        // Call the method with the object as 'this'
                                                        value = self.call_function(func, &evaluated_args?, Some(Value::Object(object.clone())))?;
                                                    },
                                                    _ => return Err(ShitRustError::TypeError(
                                                        format!("Property '{}' is not a method", method)
                                                    )),
                                                }
                                            } else {
                                                return Ok(Value::None);
                                            }
                                        } else {
                                            return Ok(Value::None);
                                        }
                                    } else {
                                        return Ok(Value::None);
                                    }
                                },
                                _ => return Err(ShitRustError::TypeError(
                                    format!("Cannot call method '{}' on non-object value", method)
                                )),
                            }
                        },
                        OptionalChainItem::Index(index_expr) => {
                            // Handle indexing on optional values
                            let index = self.evaluate_expr(index_expr)?;
                            
                            match value {
                                Value::List(list) => {
                                    if let Value::Int(idx) = index {
                                        if idx >= 0 && (idx as usize) < list.len() {
                                            value = list[idx as usize].clone();
                                        } else {
                                            return Ok(Value::None);
                                        }
                                    } else {
                                        return Err(ShitRustError::TypeError(
                                            format!("List index must be an integer, got {}", index)
                                        ));
                                    }
                                },
                                Value::None => return Ok(Value::None),
                                Value::Optional(opt) => {
                                    if let Some(inner) = opt {
                                        if let Value::List(list) = **inner {
                                            if let Value::Int(idx) = index {
                                                if idx >= 0 && (idx as usize) < list.len() {
                                                    value = list[idx as usize].clone();
                                                } else {
                                                    return Ok(Value::None);
                                                }
                                            } else {
                                                return Err(ShitRustError::TypeError(
                                                    format!("List index must be an integer, got {}", index)
                                                ));
                                            }
                                        } else {
                                            return Ok(Value::None);
                                        }
                                    } else {
                                        return Ok(Value::None);
                                    }
                                },
                                _ => return Err(ShitRustError::TypeError(
                                    format!("Cannot index non-list value")
                                )),
                            }
                        },
                    }
                }
                
                Ok(value)
            },
            Expr::PipelineChain { initial, chain } => {
                let mut value = self.evaluate_expr(initial)?;
                
                for step in chain {
                    // Each step in the pipeline takes the previous value as input
                    match &**step {
                        Expr::Call { func, args } => {
                            // For function calls, add the pipeline value as the first argument
                            let mut new_args = vec![value];
                            for arg in args {
                                new_args.push(self.evaluate_expr(arg)?);
                            }
                            
                            let function = self.evaluate_expr(func)?;
                            match function {
                                Value::Function(func) => {
                                    value = self.call_function(&func, &new_args, None)?;
                                },
                                _ => return Err(ShitRustError::TypeError(
                                    format!("Cannot call non-function value in pipeline")
                                )),
                            }
                        },
                        Expr::ListComprehension { expr, var_name, iterable: _, condition } => {
                            // For list comprehensions in a pipeline, use the pipeline value as the iterable
                            match value {
                                Value::List(list) => {
                                    let mut result = Vec::new();
                                    
                                    for item in list {
                                        // Bind the current item to the variable name
                                        self.environment.define(var_name, item.clone());
                                        
                                        // Check the condition if present
                                        if let Some(cond) = condition {
                                            match self.evaluate_expr(cond)? {
                                                Value::Bool(true) => {
                                                    // Evaluate the expression for each item
                                                    let item_result = self.evaluate_expr(expr)?;
                                                    result.push(item_result);
                                                },
                                                Value::Bool(false) => {},
                                                _ => return Err(ShitRustError::TypeError(
                                                    format!("Condition in list comprehension must be a boolean")
                                                )),
                                            }
                                        } else {
                                            // No condition, just evaluate the expression
                                            let item_result = self.evaluate_expr(expr)?;
                                            result.push(item_result);
                                        }
                                    }
                                    
                                    value = Value::List(result);
                                },
                                _ => return Err(ShitRustError::TypeError(
                                    format!("Pipeline value must be a list for list comprehension")
                                )),
                            }
                        },
                        _ => return Err(ShitRustError::TypeError(
                            format!("Unsupported expression in pipeline")
                        )),
                    }
                }
                
                Ok(value)
            },
            Expr::Match { expr, arms } => {
                let value = self.evaluate_expr(expr)?;
                
                for (pattern, result) in arms {
                    if self.pattern_matches(&value, pattern)? {
                        return self.evaluate_expr(result);
                    }
                }
                
                // No pattern matched
                Err(ShitRustError::PatternMatchError(
                    format!("No pattern matched value")
                ))
            },
            Expr::StructInit { name, fields } => {
                let mut obj = HashMap::new();
                
                // Evaluate all field expressions
                for (field_name, field_expr) in fields {
                    let field_value = self.evaluate_expr(field_expr)?;
                    obj.insert(field_name.clone(), field_value);
                }
                
                Ok(Value::Object(obj))
            },
            _ => {
                // Other expression types not yet implemented
                Err(ShitRustError::RuntimeError(format!("Expression type not yet implemented: {:?}", expr)))
            }
        }
    }
    
    fn evaluate_literal(&self, lit: &Literal) -> Result<Value, ShitRustError> {
        match lit {
            Literal::Int(i) => Ok(Value::Int(*i)),
            Literal::Float(f) => Ok(Value::Float(*f)),
            Literal::Bool(b) => Ok(Value::Bool(*b)),
            Literal::String(s) => Ok(Value::String(s.clone())),
            Literal::Char(c) => Ok(Value::Char(*c)),
            Literal::List(items) => {
                let mut values = Vec::new();
                for item in items {
                    values.push(self.evaluate_literal(item)?);
                }
                Ok(Value::List(values))
            },
            Literal::None => Ok(Value::None),
            _ => Err(ShitRustError::RuntimeError(format!("Literal type not yet implemented: {:?}", lit))),
        }
    }
    
    // Operator implementation
    
    fn add(&self, left: Value, right: Value) -> Result<Value, ShitRustError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 + b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + b as f64)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(a + &b)),
            (Value::String(a), b) => Ok(Value::String(a + &b.to_string())),
            (a, Value::String(b)) => Ok(Value::String(a.to_string() + &b)),
            _ => Err(ShitRustError::TypeError("Cannot add these types".to_string())),
        }
    }
    
    fn subtract(&self, left: Value, right: Value) -> Result<Value, ShitRustError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 - b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - b as f64)),
            _ => Err(ShitRustError::TypeError("Cannot subtract these types".to_string())),
        }
    }
    
    fn multiply(&self, left: Value, right: Value) -> Result<Value, ShitRustError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 * b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a * b as f64)),
            _ => Err(ShitRustError::TypeError("Cannot multiply these types".to_string())),
        }
    }
    
    fn divide(&self, left: Value, right: Value) -> Result<Value, ShitRustError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => {
                if b == 0 {
                    return Err(ShitRustError::RuntimeError("Division by zero".to_string()));
                }
                Ok(Value::Int(a / b))
            },
            (Value::Float(a), Value::Float(b)) => {
                if b == 0.0 {
                    return Err(ShitRustError::RuntimeError("Division by zero".to_string()));
                }
                Ok(Value::Float(a / b))
            },
            (Value::Int(a), Value::Float(b)) => {
                if b == 0.0 {
                    return Err(ShitRustError::RuntimeError("Division by zero".to_string()));
                }
                Ok(Value::Float(a as f64 / b))
            },
            (Value::Float(a), Value::Int(b)) => {
                if b == 0 {
                    return Err(ShitRustError::RuntimeError("Division by zero".to_string()));
                }
                Ok(Value::Float(a / b as f64))
            },
            _ => Err(ShitRustError::TypeError("Cannot divide these types".to_string())),
        }
    }
    
    fn modulo(&self, left: Value, right: Value) -> Result<Value, ShitRustError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => {
                if b == 0 {
                    return Err(ShitRustError::RuntimeError("Modulo by zero".to_string()));
                }
                Ok(Value::Int(a % b))
            },
            _ => Err(ShitRustError::TypeError("Modulo only works with integers".to_string())),
        }
    }
    
    fn equals(&self, left: Value, right: Value) -> Result<Value, ShitRustError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a == b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a == b)),
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a == b)),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a == b)),
            (Value::Char(a), Value::Char(b)) => Ok(Value::Bool(a == b)),
            (Value::None, Value::None) => Ok(Value::Bool(true)),
            _ => Ok(Value::Bool(false)),
        }
    }
    
    fn less_than(&self, left: Value, right: Value) -> Result<Value, ShitRustError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((a as f64) < b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(a < (b as f64))),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a < b)),
            (Value::Char(a), Value::Char(b)) => Ok(Value::Bool(a < b)),
            _ => Err(ShitRustError::TypeError("Cannot compare these types".to_string())),
        }
    }
    
    fn less_equal(&self, left: Value, right: Value) -> Result<Value, ShitRustError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((a as f64) <= b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(a <= (b as f64))),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a <= b)),
            (Value::Char(a), Value::Char(b)) => Ok(Value::Bool(a <= b)),
            _ => Err(ShitRustError::TypeError("Cannot compare these types".to_string())),
        }
    }
    
    fn greater_than(&self, left: Value, right: Value) -> Result<Value, ShitRustError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((a as f64) > b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(a > (b as f64))),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a > b)),
            (Value::Char(a), Value::Char(b)) => Ok(Value::Bool(a > b)),
            _ => Err(ShitRustError::TypeError("Cannot compare these types".to_string())),
        }
    }
    
    fn greater_equal(&self, left: Value, right: Value) -> Result<Value, ShitRustError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((a as f64) >= b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(a >= (b as f64))),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a >= b)),
            (Value::Char(a), Value::Char(b)) => Ok(Value::Bool(a >= b)),
            _ => Err(ShitRustError::TypeError("Cannot compare these types".to_string())),
        }
    }

    fn pattern_matches(&mut self, value: &Value, pattern: &Pattern) -> Result<bool, ShitRustError> {
        match pattern {
            Pattern::Wildcard => Ok(true),
            
            Pattern::Literal(lit) => {
                let pattern_value = self.evaluate_literal(lit)?;
                Ok(self.values_equal(value, &pattern_value))
            },
            
            Pattern::Identifier(name) => {
                // Bind the value to the identifier in the current environment
                self.environment.define(name, value.clone());
                Ok(true)
            },
            
            Pattern::EnumVariant { name, values } => {
                if let Value::Enum(variant, enum_values) = value {
                    if variant != name {
                        return Ok(false);
                    }
                    
                    if values.len() != enum_values.len() {
                        return Ok(false);
                    }
                    
                    // Check if nested patterns match the enum values
                    for (i, pattern) in values.iter().enumerate() {
                        if !self.pattern_matches(&enum_values[i], pattern)? {
                            return Ok(false);
                        }
                    }
                    
                    Ok(true)
                } else {
                    Ok(false)
                }
            },
            
            Pattern::Destructure { name, fields } => {
                if let Value::Object(obj) = value {
                    // First check the struct name if applicable
                    if !name.is_empty() {
                        if let Some(type_name) = obj.get("__type") {
                            if let Value::String(obj_type) = type_name {
                                if obj_type != name {
                                    return Ok(false);
                                }
                            }
                        }
                    }
                    
                    // Check if all fields in the pattern exist and match
                    for (field_name, field_pattern) in fields {
                        if let Some(field_value) = obj.get(field_name) {
                            if !self.pattern_matches(field_value, field_pattern)? {
                                return Ok(false);
                            }
                        } else {
                            return Ok(false);
                        }
                    }
                    
                    Ok(true)
                } else {
                    Ok(false)
                }
            },
            
            Pattern::Or(patterns) => {
                for p in patterns {
                    if self.pattern_matches(value, p)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            },
            
            Pattern::Range { start, end, inclusive } => {
                let start_value = self.evaluate_literal(start)?;
                let end_value = self.evaluate_literal(end)?;
                
                match (value, &start_value, &end_value) {
                    (Value::Int(v), Value::Int(s), Value::Int(e)) => {
                        if *inclusive {
                            Ok(*v >= *s && *v <= *e)
                        } else {
                            Ok(*v >= *s && *v < *e)
                        }
                    },
                    (Value::Float(v), Value::Float(s), Value::Float(e)) => {
                        if *inclusive {
                            Ok(*v >= *s && *v <= *e)
                        } else {
                            Ok(*v >= *s && *v < *e)
                        }
                    },
                    (Value::Char(v), Value::Char(s), Value::Char(e)) => {
                        if *inclusive {
                            Ok(*v >= *s && *v <= *e)
                        } else {
                            Ok(*v >= *s && *v < *e)
                        }
                    },
                    _ => Err(ShitRustError::TypeError(
                        format!("Cannot apply range pattern to incompatible types")
                    )),
                }
            },
        }
    }

    fn values_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => (a - b).abs() < 1e-9,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Char(a), Value::Char(b)) => a == b,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Dict(a), Value::Dict(b)) => a == b,
            (Value::Tuple(a), Value::Tuple(b)) => a == b,
            (Value::Function { .. }, Value::Function { .. }) => false,
            (Value::NativeFunction { .. }, Value::NativeFunction { .. }) => false,
            (Value::Optional(a), Value::Optional(b)) => {
                if let (Some(a), Some(b)) = (a, b) {
                    self.values_equal(a, b)
                } else {
                    a.is_none() && b.is_none()
                }
            },
            (Value::Trait(a), Value::Trait(b)) => a.name == b.name && a.methods == b.methods && a.generic_params == b.generic_params,
            (Value::TraitImpl(a), Value::TraitImpl(b)) => a.trait_name == b.trait_name && a.type_name == b.type_name && a.methods == b.methods && a.generic_params == b.generic_params,
            _ => false,
        }
    }

    fn call_function(&mut self, func: &FunctionValue, args: &[Value], this: Option<Value>) -> Result<Value, ShitRustError> {
        let mut env = Environment::with_parent(self.environment.clone());
        
        // Add parameters to the environment
        for (param, arg) in func.params.iter().zip(args) {
            env.define(param.clone(), arg.clone());
        }
        
        // Add 'this' to the environment if it's a method
        if let Some(this) = this {
            env.define("this", this);
        }
        
        // Save current environment
        let previous_env = self.environment.clone();
        self.environment = env;
        
        // Execute function body
        for stmt in &func.body {
            self.execute_stmt(stmt)?;
        }
        
        // Restore previous environment
        self.environment = previous_env;
        
        // For now, always return None
        Ok(Value::None)
    }

    /// Execute a program with async support
    pub fn execute_async(&mut self, program: &Program) -> Result<Value> {
        // Set the current source file
        self.current_source_file = program.source_file.clone();
        
        // Find the main function
        let main_function = self.find_main_function(program)?;
        
        // Create AsyncRuntime object
        let async_runtime_constructor = self.get_value("AsyncRuntime")
            .ok_or_else(|| ShitRustError::RuntimeError(
                "AsyncRuntime not found. Make sure stdlib::async_runtime is imported".to_string()
            ))?;
        
        // Instantiate AsyncRuntime
        let async_runtime = self.call_value(&async_runtime_constructor, &[])?;
        
        // Define block_on function
        let block_on_method = if let Value::Object(obj) = &async_runtime {
            // Try to find the block_on method
            obj.get_method("block_on")
                .ok_or_else(|| ShitRustError::RuntimeError(
                    "block_on method not found on AsyncRuntime".to_string()
                ))?
        } else {
            return Err(ShitRustError::RuntimeError(
                "AsyncRuntime is not an object".to_string()
            ));
        };
        
        // Call main function through the async runtime
        let main_result = self.call_value(&block_on_method, &[main_function])?;
        
        Ok(main_result)
    }
    
    /// Clone the interpreter
    pub fn clone(&self) -> Self {
        // Create a new environment
        let global_env = Environment::new();
        
        // Clone the current environment
        let current_env = self.environment.clone();
        
        Interpreter {
            environment: current_env,
            globals: global_env,
            current_source_file: self.current_source_file.clone(),
        }
    }
}

// Add TraitDefinition and TraitImplementation structs
#[derive(Debug, Clone)]
struct TraitDefinition {
    name: String,
    methods: HashMap<String, TraitMethod>,
    generic_params: Vec<String>,
}

#[derive(Debug, Clone)]
struct TraitMethod {
    name: String,
    params: Vec<(String, Type)>,
    return_type: Type,
    body: Option<Vec<Stmt>>,
    is_async: bool,
}

#[derive(Debug, Clone)]
struct TraitImplementation {
    trait_name: Option<String>,
    type_name: String,
    methods: HashMap<String, FunctionValue>,
    generic_params: Vec<String>,
}
