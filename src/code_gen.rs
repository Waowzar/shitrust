use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{InitializationConfig, Target};
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};
use inkwell::types::BasicTypeEnum;
use inkwell::OptimizationLevel;
use std::collections::HashMap;
use std::path::Path;
use crate::ast::{Program, Stmt, Expr, Literal, Type as AstType, BinOp, UnaryOp};
use crate::error::ShitRustError;

pub struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    named_values: HashMap<String, PointerValue<'ctx>>,
    current_function: Option<FunctionValue<'ctx>>,
    printf_function: FunctionValue<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        
        // Initialize LLVM targets for the current machine
        Target::initialize_all(&InitializationConfig::default());
        
        // Add printf declaration for printing
        let i8_ptr_type = context.i8_type().ptr_type(Default::default());
        let printf_type = context.i32_type().fn_type(
            &[i8_ptr_type.into()], 
            true  // variadic
        );
        let printf_func = module.add_function("printf", printf_type, None);
        
        CodeGen {
            context,
            module,
            builder,
            named_values: HashMap::new(),
            current_function: None,
            printf_function: printf_func,
        }
    }
    
    pub fn generate_code(&mut self, program: &Program) -> Result<(), ShitRustError> {
        // First pass: register all function declarations
        for stmt in &program.statements {
            if let Stmt::Function { name, params, return_type, .. } = stmt {
                self.declare_function(name, params, return_type)?;
            }
        }
        
        // Generate main function if it doesn't exist
        let main_function = match self.module.get_function("main") {
            Some(func) => func,
            None => {
                // Create a minimal main function that just returns 0
                let main_type = self.context.i32_type().fn_type(&[], false);
                let main_func = self.module.add_function("main", main_type, None);
                let entry = self.context.append_basic_block(main_func, "entry");
                self.builder.position_at_end(entry);
                self.builder.build_return(Some(&self.context.i32_type().const_int(0, false)));
                main_func
            }
        };
        
        // Second pass: generate code for function bodies
        for stmt in &program.statements {
            match stmt {
                Stmt::Function { name, body, .. } => {
                    let function = self.module.get_function(name)
                        .ok_or_else(|| ShitRustError::RuntimeError(format!("No function named {}", name)))?;
                    
                    let entry = self.context.append_basic_block(function, "entry");
                    self.builder.position_at_end(entry);
                    
                    // Save current function
                    self.current_function = Some(function);
                    
                    // Generate code for function body
                    for stmt in body {
                        self.generate_stmt(stmt)?;
                    }
                    
                    // If we don't have a terminator (like return), add one
                    if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
                        if function.get_type().get_return_type().is_none() {
                            self.builder.build_return(None);
                        } else {
                            // Default return value based on return type
                            let ret_type = function.get_type().get_return_type().unwrap();
                            if ret_type == self.context.i32_type().into() {
                                self.builder.build_return(Some(&self.context.i32_type().const_int(0, false)));
                            } else if ret_type == self.context.f64_type().into() {
                                self.builder.build_return(Some(&self.context.f64_type().const_float(0.0)));
                            } else {
                                return Err(ShitRustError::RuntimeError(
                                    format!("Unsupported return type for function {}", name)
                                ));
                            }
                        }
                    }
                },
                _ => {
                    // Top-level statements are put in the main function
                    let main_block = main_function.get_first_basic_block().unwrap();
                    self.builder.position_at_end(main_block);
                    self.current_function = Some(main_function);
                    
                    // Handle non-function statements
                    if !matches!(stmt, Stmt::Function { .. }) {
                        self.generate_stmt(stmt)?;
                    }
                }
            }
        }
        
        // Verify the module
        if self.module.verify().is_err() {
            return Err(ShitRustError::RuntimeError("Generated LLVM IR is invalid".to_string()));
        }
        
        Ok(())
    }
    
    fn declare_function(
        &self, 
        name: &str, 
        params: &[(String, AstType)], 
        return_type: &AstType
    ) -> Result<FunctionValue<'ctx>, ShitRustError> {
        // Convert ShitRust types to LLVM types
        let param_types: Vec<BasicTypeEnum> = params
            .iter()
            .map(|(_, typ)| self.ast_type_to_llvm_type(typ))
            .collect::<Result<Vec<_>, _>>()?;
        
        let return_llvm_type = match return_type {
            AstType::Void => None,
            _ => Some(self.ast_type_to_llvm_type(return_type)?),
        };
        
        let fn_type = match return_llvm_type {
            Some(ret_type) => ret_type.fn_type(&param_types, false),
            None => self.context.void_type().fn_type(&param_types, false),
        };
        
        let function = self.module.add_function(name, fn_type, None);
        
        // Name parameters for debugging
        for (i, (param_name, _)) in params.iter().enumerate() {
            function.get_nth_param(i as u32)
                .unwrap()
                .set_name(param_name);
        }
        
        Ok(function)
    }
    
    fn ast_type_to_llvm_type(&self, typ: &AstType) -> Result<BasicTypeEnum<'ctx>, ShitRustError> {
        match typ {
            AstType::Int => Ok(self.context.i64_type().into()),
            AstType::Float => Ok(self.context.f64_type().into()),
            AstType::Bool => Ok(self.context.bool_type().into()),
            AstType::String => Ok(self.context.i8_type().ptr_type(Default::default()).into()),
            AstType::Char => Ok(self.context.i8_type().into()),
            _ => Err(ShitRustError::TypeError(format!("Unsupported type: {:?}", typ))),
        }
    }
    
    fn generate_stmt(&mut self, stmt: &Stmt) -> Result<(), ShitRustError> {
        match stmt {
            Stmt::Expr(expr) => {
                self.generate_expr(expr)?;
                Ok(())
            },
            Stmt::Let { name, type_hint: _, value, mutable: _ } => {
                let expr_value = self.generate_expr(value)?;
                
                // Allocate space on the stack
                let alloca = self.create_entry_block_alloca(name, expr_value.get_type());
                
                // Store the value
                self.builder.build_store(alloca, expr_value);
                
                // Add to our symbol table
                self.named_values.insert(name.clone(), alloca);
                
                Ok(())
            },
            Stmt::If { condition, then_block, else_block } => {
                let cond_value = self.generate_expr(condition)?;
                
                // Convert condition to i1 (boolean)
                let cond_val = match cond_value {
                    BasicValueEnum::IntValue(i) => i,
                    _ => return Err(ShitRustError::TypeError("Condition must be a boolean".to_string())),
                };
                
                let function = self.current_function.unwrap();
                let then_bb = self.context.append_basic_block(function, "then");
                let else_bb = self.context.append_basic_block(function, "else");
                let merge_bb = self.context.append_basic_block(function, "ifcont");
                
                self.builder.build_conditional_branch(cond_val, then_bb, else_bb);
                
                // Then block
                self.builder.position_at_end(then_bb);
                for stmt in then_block {
                    self.generate_stmt(stmt)?;
                }
                self.builder.build_unconditional_branch(merge_bb);
                
                // Else block
                self.builder.position_at_end(else_bb);
                if let Some(else_stmts) = else_block {
                    for stmt in else_stmts {
                        self.generate_stmt(stmt)?;
                    }
                }
                self.builder.build_unconditional_branch(merge_bb);
                
                // Continue in the merge block
                self.builder.position_at_end(merge_bb);
                
                Ok(())
            },
            Stmt::Return(value_opt) => {
                match value_opt {
                    Some(value) => {
                        let return_value = self.generate_expr(value)?;
                        self.builder.build_return(Some(&return_value));
                    },
                    None => {
                        self.builder.build_return(None);
                    }
                }
                
                Ok(())
            },
            // Other statement types would be handled here
            _ => Err(ShitRustError::RuntimeError(format!("Statement type not yet implemented: {:?}", stmt))),
        }
    }
    
    fn generate_expr(&mut self, expr: &Expr) -> Result<BasicValueEnum<'ctx>, ShitRustError> {
        match expr {
            Expr::Literal(lit) => self.generate_literal(lit),
            Expr::Identifier(name) => {
                if let Some(var) = self.named_values.get(name) {
                    Ok(self.builder.build_load(*var, name))
                } else {
                    Err(ShitRustError::UndefinedVariable(name.clone()))
                }
            },
            Expr::BinaryOp { left, op, right } => {
                let l_val = self.generate_expr(left)?;
                let r_val = self.generate_expr(right)?;
                
                match op {
                    BinOp::Add => self.generate_add(l_val, r_val),
                    BinOp::Sub => self.generate_sub(l_val, r_val),
                    BinOp::Mul => self.generate_mul(l_val, r_val),
                    BinOp::Div => self.generate_div(l_val, r_val),
                    BinOp::Eq => self.generate_eq(l_val, r_val),
                    BinOp::Ne => self.generate_ne(l_val, r_val),
                    BinOp::Lt => self.generate_lt(l_val, r_val),
                    BinOp::Le => self.generate_le(l_val, r_val),
                    BinOp::Gt => self.generate_gt(l_val, r_val),
                    BinOp::Ge => self.generate_ge(l_val, r_val),
                    // Other operators would be implemented here
                    _ => Err(ShitRustError::RuntimeError(format!("Binary operator not implemented: {:?}", op))),
                }
            },
            Expr::Call { func, args } => {
                if let Expr::Identifier(name) = &**func {
                    // Handle print/println as special cases
                    if name == "println" || name == "print" {
                        return self.generate_print_call(args, name == "println");
                    }
                    
                    // Get the function from the module
                    let function = self.module.get_function(name)
                        .ok_or_else(|| ShitRustError::UndefinedVariable(name.clone()))?;
                    
                    // Check that we have the right number of arguments
                    if function.count_params() as usize != args.len() {
                        return Err(ShitRustError::RuntimeError(
                            format!("Expected {} arguments but got {}", function.count_params(), args.len())
                        ));
                    }
                    
                    // Generate code for each argument
                    let mut arg_values = Vec::new();
                    for arg in args {
                        arg_values.push(self.generate_expr(arg)?);
                    }
                    
                    // Convert BasicValueEnum to dyn BasicValue
                    let args_refs: Vec<_> = arg_values.iter()
                        .map(|val| val as &dyn inkwell::values::BasicValue)
                        .collect();
                    
                    // Call the function
                    let call = self.builder.build_call(function, &args_refs, &format!("{}_call", name));
                    
                    // Get the return value if not void
                    match function.get_type().get_return_type() {
                        Some(_) => Ok(call.try_as_basic_value().left().unwrap()),
                        None => Err(ShitRustError::RuntimeError(
                            format!("Cannot use void function '{}' in an expression", name)
                        )),
                    }
                } else {
                    Err(ShitRustError::RuntimeError("Callee is not a function name".to_string()))
                }
            },
            // Other expression types would be handled here
            _ => Err(ShitRustError::RuntimeError(format!("Expression type not yet implemented: {:?}", expr))),
        }
    }
    
    fn generate_print_call(&self, args: &[Expr], add_newline: bool) -> Result<BasicValueEnum<'ctx>, ShitRustError> {
        if args.is_empty() {
            let format_str = if add_newline { "\n\0" } else { "\0" };
            let fmt_ptr = self.builder.build_global_string_ptr(format_str, "empty_fmt");
            
            let call = self.builder.build_call(
                self.printf_function, 
                &[fmt_ptr.as_pointer_value().into()], 
                "printf_call"
            );
            
            // printf returns an i32, but we'll return void for print calls
            Ok(self.context.i32_type().const_int(0, false).into())
        } else {
            // For now, we only support simple string printing
            let expr = &args[0];
            let value = self.generate_expr(expr)?;
            
            let format_str = match value.get_type() {
                BasicTypeEnum::IntType(_) => {
                    if add_newline { "%d\n\0" } else { "%d\0" }
                },
                BasicTypeEnum::FloatType(_) => {
                    if add_newline { "%f\n\0" } else { "%f\0" }
                },
                BasicTypeEnum::PointerType(_) => {
                    // Assuming this is a string
                    if add_newline { "%s\n\0" } else { "%s\0" }
                },
                _ => return Err(ShitRustError::TypeError("Unsupported print type".to_string())),
            };
            
            let fmt_ptr = self.builder.build_global_string_ptr(format_str, "fmt");
            
            let call = self.builder.build_call(
                self.printf_function, 
                &[fmt_ptr.as_pointer_value().into(), value.into()], 
                "printf_call"
            );
            
            // printf returns an i32, but we'll return void for print calls
            Ok(self.context.i32_type().const_int(0, false).into())
        }
    }
    
    fn generate_literal(&self, lit: &Literal) -> Result<BasicValueEnum<'ctx>, ShitRustError> {
        match lit {
            Literal::Int(value) => {
                Ok(self.context.i64_type().const_int(*value as u64, false).into())
            },
            Literal::Float(value) => {
                Ok(self.context.f64_type().const_float(*value).into())
            },
            Literal::Bool(value) => {
                Ok(self.context.bool_type().const_int(*value as u64, false).into())
            },
            Literal::String(value) => {
                // Add null terminator for C strings
                let value_with_null = format!("{}\0", value);
                let ptr = self.builder.build_global_string_ptr(&value_with_null, "str");
                Ok(ptr.as_pointer_value().into())
            },
            Literal::Char(value) => {
                Ok(self.context.i8_type().const_int(*value as u64, false).into())
            },
            // Other literal types would be handled here
            _ => Err(ShitRustError::RuntimeError(format!("Literal type not yet implemented: {:?}", lit))),
        }
    }
    
    // Helper methods for binary operations
    
    fn generate_add(&self, left: BasicValueEnum<'ctx>, right: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, ShitRustError> {
        match (left, right) {
            (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                Ok(self.builder.build_int_add(l, r, "addtmp").into())
            },
            (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                Ok(self.builder.build_float_add(l, r, "addtmp").into())
            },
            // String concatenation would be handled here with runtime function calls
            _ => Err(ShitRustError::TypeError("Incompatible types for addition".to_string())),
        }
    }
    
    fn generate_sub(&self, left: BasicValueEnum<'ctx>, right: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, ShitRustError> {
        match (left, right) {
            (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                Ok(self.builder.build_int_sub(l, r, "subtmp").into())
            },
            (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                Ok(self.builder.build_float_sub(l, r, "subtmp").into())
            },
            _ => Err(ShitRustError::TypeError("Incompatible types for subtraction".to_string())),
        }
    }
    
    fn generate_mul(&self, left: BasicValueEnum<'ctx>, right: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, ShitRustError> {
        match (left, right) {
            (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                Ok(self.builder.build_int_mul(l, r, "multmp").into())
            },
            (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                Ok(self.builder.build_float_mul(l, r, "multmp").into())
            },
            _ => Err(ShitRustError::TypeError("Incompatible types for multiplication".to_string())),
        }
    }
    
    fn generate_div(&self, left: BasicValueEnum<'ctx>, right: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, ShitRustError> {
        match (left, right) {
            (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                Ok(self.builder.build_int_signed_div(l, r, "divtmp").into())
            },
            (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                Ok(self.builder.build_float_div(l, r, "divtmp").into())
            },
            _ => Err(ShitRustError::TypeError("Incompatible types for division".to_string())),
        }
    }
    
    // Comparison operators
    
    fn generate_eq(&self, left: BasicValueEnum<'ctx>, right: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, ShitRustError> {
        match (left, right) {
            (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                Ok(self.builder.build_int_compare(inkwell::IntPredicate::EQ, l, r, "eqtmp").into())
            },
            (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                Ok(self.builder.build_float_compare(inkwell::FloatPredicate::OEQ, l, r, "eqtmp").into())
            },
            _ => Err(ShitRustError::TypeError("Incompatible types for equality comparison".to_string())),
        }
    }
    
    fn generate_ne(&self, left: BasicValueEnum<'ctx>, right: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, ShitRustError> {
        match (left, right) {
            (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                Ok(self.builder.build_int_compare(inkwell::IntPredicate::NE, l, r, "netmp").into())
            },
            (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                Ok(self.builder.build_float_compare(inkwell::FloatPredicate::ONE, l, r, "netmp").into())
            },
            _ => Err(ShitRustError::TypeError("Incompatible types for inequality comparison".to_string())),
        }
    }
    
    fn generate_lt(&self, left: BasicValueEnum<'ctx>, right: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, ShitRustError> {
        match (left, right) {
            (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                Ok(self.builder.build_int_compare(inkwell::IntPredicate::SLT, l, r, "lttmp").into())
            },
            (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                Ok(self.builder.build_float_compare(inkwell::FloatPredicate::OLT, l, r, "lttmp").into())
            },
            _ => Err(ShitRustError::TypeError("Incompatible types for less-than comparison".to_string())),
        }
    }
    
    fn generate_le(&self, left: BasicValueEnum<'ctx>, right: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, ShitRustError> {
        match (left, right) {
            (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                Ok(self.builder.build_int_compare(inkwell::IntPredicate::SLE, l, r, "letmp").into())
            },
            (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                Ok(self.builder.build_float_compare(inkwell::FloatPredicate::OLE, l, r, "letmp").into())
            },
            _ => Err(ShitRustError::TypeError("Incompatible types for less-equal comparison".to_string())),
        }
    }
    
    fn generate_gt(&self, left: BasicValueEnum<'ctx>, right: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, ShitRustError> {
        match (left, right) {
            (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                Ok(self.builder.build_int_compare(inkwell::IntPredicate::SGT, l, r, "gttmp").into())
            },
            (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                Ok(self.builder.build_float_compare(inkwell::FloatPredicate::OGT, l, r, "gttmp").into())
            },
            _ => Err(ShitRustError::TypeError("Incompatible types for greater-than comparison".to_string())),
        }
    }
    
    fn generate_ge(&self, left: BasicValueEnum<'ctx>, right: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, ShitRustError> {
        match (left, right) {
            (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                Ok(self.builder.build_int_compare(inkwell::IntPredicate::SGE, l, r, "getmp").into())
            },
            (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                Ok(self.builder.build_float_compare(inkwell::FloatPredicate::OGE, l, r, "getmp").into())
            },
            _ => Err(ShitRustError::TypeError("Incompatible types for greater-equal comparison".to_string())),
        }
    }
    
    fn create_entry_block_alloca(&self, name: &str, typ: BasicTypeEnum<'ctx>) -> PointerValue<'ctx> {
        let builder = self.context.create_builder();
        let entry = self.current_function.unwrap().get_first_basic_block().unwrap();
        
        match entry.get_first_instruction() {
            Some(first_instr) => builder.position_before(&first_instr),
            None => builder.position_at_end(entry),
        }
        
        builder.build_alloca(typ, name)
    }
    
    pub fn write_to_file(&self, path: &Path) -> Result<(), ShitRustError> {
        match self.module.print_to_file(path) {
            Ok(_) => Ok(()),
            Err(e) => Err(ShitRustError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other, 
                format!("Failed to write LLVM IR to file: {}", e)
            ))),
        }
    }
    
    pub fn compile_to_object_file(&self, path: &Path) -> Result<(), ShitRustError> {
        let target_triple = inkwell::targets::TargetMachine::get_default_triple();
        let target = inkwell::targets::Target::from_triple(&target_triple)
            .map_err(|e| ShitRustError::RuntimeError(format!("Failed to get target: {}", e)))?;
            
        let target_machine = target.create_target_machine(
            &target_triple,
            "generic",
            "",
            OptimizationLevel::Default,
            inkwell::targets::RelocMode::Default,
            inkwell::targets::CodeModel::Default,
        ).ok_or_else(|| ShitRustError::RuntimeError("Failed to create target machine".to_string()))?;
        
        target_machine.write_to_file(
            &self.module, 
            inkwell::targets::FileType::Object, 
            path
        ).map_err(|e| ShitRustError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other, 
            format!("Failed to write object file: {}", e)
        )))
    }
} 