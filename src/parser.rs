use crate::ast::{Expr, Literal, Stmt, Program, BinOp, UnaryOp, Type};
use crate::error::ShitRustError;
use crate::lexer::{Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            current: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Program, ShitRustError> {
        let mut program = Program { statements: Vec::new() };
        
        while !self.is_at_end() {
            match self.declaration() {
                Ok(stmt) => program.statements.push(stmt),
                Err(e) => return Err(e),
            }
        }
        
        Ok(program)
    }
    
    fn declaration(&mut self) -> Result<Stmt, ShitRustError> {
        if self.match_token(&[TokenType::Let]) {
            return self.var_declaration();
        } else if self.match_token(&[TokenType::Fn]) {
            return self.function();
        } else if self.match_token(&[TokenType::Struct]) {
            return self.struct_declaration();
        } else if self.match_token(&[TokenType::Enum]) {
            return self.enum_declaration();
        } else if self.match_token(&[TokenType::Trait]) {
            return self.trait_declaration();
        } else if self.match_token(&[TokenType::Impl]) {
            return self.impl_declaration();
        } else if self.match_token(&[TokenType::Type]) {
            return self.type_alias();
        } else if self.match_token(&[TokenType::Const]) {
            return self.const_declaration();
        } else if self.match_token(&[TokenType::Async]) {
            if self.match_token(&[TokenType::Fn]) {
                return self.async_function();
            } else {
                return self.async_block();
            }
        } else if self.match_token(&[TokenType::Use]) {
            return self.use_declaration();
        }
        
        self.statement()
    }
    
    fn var_declaration(&mut self) -> Result<Stmt, ShitRustError> {
        // Parse variable name
        let name = self.consume(TokenType::Identifier, "Expected variable name")?;
        let name_str = name.lexeme.clone();
        
        // Check if mutable
        let mutable = self.match_token(&[TokenType::Mut]);
        
        // Parse optional type annotation
        let mut type_hint = None;
        if self.match_token(&[TokenType::Colon]) {
            type_hint = Some(self.parse_type()?);
        }
        
        // Parse initializer
        self.consume(TokenType::Equal, "Expected '=' after variable name")?;
        let initializer = self.expression()?;
        
        self.consume(TokenType::Semicolon, "Expected ';' after variable declaration")?;
        
        Ok(Stmt::Let {
            name: name_str,
            type_hint,
            value: initializer,
            mutable,
        })
    }
    
    fn function(&mut self) -> Result<Stmt, ShitRustError> {
        // Parse function name
        let name = self.consume(TokenType::Identifier, "Expected function name")?;
        let name_str = name.lexeme.clone();
        
        // Parse parameters
        self.consume(TokenType::LeftParen, "Expected '(' after function name")?;
        let mut params = Vec::new();
        
        if !self.check(TokenType::RightParen) {
            loop {
                let param_name = self.consume(TokenType::Identifier, "Expected parameter name")?;
                self.consume(TokenType::Colon, "Expected ':' after parameter name")?;
                let param_type = self.parse_type()?;
                
                params.push((param_name.lexeme.clone(), param_type));
                
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        
        self.consume(TokenType::RightParen, "Expected ')' after parameters")?;
        
        // Parse return type
        let return_type = if self.match_token(&[TokenType::Arrow]) {
            self.parse_type()?
        } else {
            Type::Void
        };
        
        // Parse function body
        self.consume(TokenType::LeftBrace, "Expected '{' before function body")?;
        let body = self.block()?;
        
        Ok(Stmt::Function {
            name: name_str,
            params,
            return_type,
            body,
        })
    }
    
    fn parse_type(&mut self) -> Result<Type, ShitRustError> {
        let type_token = self.advance();
        
        match type_token.token_type {
            TokenType::Int => Ok(Type::Int),
            TokenType::Float => Ok(Type::Float),
            TokenType::Bool => Ok(Type::Bool),
            TokenType::String => Ok(Type::String),
            TokenType::Char => Ok(Type::Char),
            TokenType::Void => Ok(Type::Void),
            TokenType::Identifier => Ok(Type::Custom(type_token.lexeme.clone())),
            _ => Err(ShitRustError::SyntaxError {
                line: type_token.line,
                column: type_token.column,
                message: format!("Expected type, got {}", type_token.lexeme),
            }),
        }
    }
    
    fn statement(&mut self) -> Result<Stmt, ShitRustError> {
        if self.match_token(&[TokenType::If]) {
            return self.if_statement();
        } else if self.match_token(&[TokenType::While]) {
            return self.while_statement();
        } else if self.match_token(&[TokenType::For]) {
            return self.for_statement();
        } else if self.match_token(&[TokenType::Return]) {
            return self.return_statement();
        } else if self.match_token(&[TokenType::LeftBrace]) {
            let statements = self.block()?;
            // In our AST we don't have a block statement, so we'll wrap it in a dummy if
            return Ok(Stmt::If {
                condition: Expr::Literal(Literal::Bool(true)),
                then_block: statements,
                else_block: None,
            });
        }
        
        // Default: expression statement
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after expression")?;
        Ok(Stmt::Expr(expr))
    }
    
    fn if_statement(&mut self) -> Result<Stmt, ShitRustError> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'if'")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expected ')' after if condition")?;
        
        self.consume(TokenType::LeftBrace, "Expected '{' before if body")?;
        let then_block = self.block()?;
        
        let else_block = if self.match_token(&[TokenType::Else]) {
            if self.match_token(&[TokenType::If]) {
                // Handle 'else if'
                let else_if_stmt = self.if_statement()?;
                Some(vec![else_if_stmt])
            } else {
                // Handle 'else'
                self.consume(TokenType::LeftBrace, "Expected '{' before else body")?;
                Some(self.block()?)
            }
        } else {
            None
        };
        
        Ok(Stmt::If {
            condition,
            then_block,
            else_block,
        })
    }
    
    fn while_statement(&mut self) -> Result<Stmt, ShitRustError> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'while'")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expected ')' after while condition")?;
        
        self.consume(TokenType::LeftBrace, "Expected '{' before while body")?;
        let body = self.block()?;
        
        Ok(Stmt::While {
            condition,
            body,
        })
    }
    
    fn for_statement(&mut self) -> Result<Stmt, ShitRustError> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'for'")?;
        
        // Parse the iterator variable
        let var_name = self.consume(TokenType::Identifier, "Expected variable name in for loop")?;
        
        // Expect 'in' keyword
        self.consume(TokenType::In, "Expected 'in' after variable name in for loop")?;
        
        // Parse the iterator expression
        let iterator_expr = self.expression()?;
        
        self.consume(TokenType::RightParen, "Expected ')' after for loop header")?;
        
        // Parse the body
        self.consume(TokenType::LeftBrace, "Expected '{' before for loop body")?;
        let body = self.block()?;
        
        Ok(Stmt::For {
            var: var_name.lexeme,
            iterator: iterator_expr,
            body,
        })
    }
    
    fn return_statement(&mut self) -> Result<Stmt, ShitRustError> {
        let value = if !self.check(TokenType::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };
        
        self.consume(TokenType::Semicolon, "Expected ';' after return value")?;
        
        Ok(Stmt::Return(value))
    }
    
    fn block(&mut self) -> Result<Vec<Stmt>, ShitRustError> {
        let mut statements = Vec::new();
        
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        
        self.consume(TokenType::RightBrace, "Expected '}' after block")?;
        
        Ok(statements)
    }
    
    fn expression(&mut self) -> Result<Expr, ShitRustError> {
        self.assignment()
    }
    
    fn assignment(&mut self) -> Result<Expr, ShitRustError> {
        let expr = self.logical_or()?;
        
        if self.match_token(&[TokenType::Equal]) {
            let value = self.assignment()?;
            
            match expr {
                Expr::Identifier(name) => {
                    return Ok(Expr::BinaryOp {
                        left: Box::new(Expr::Identifier(name)),
                        op: BinOp::Eq,
                        right: Box::new(value),
                    });
                },
                // For now, we only support assignment to variables
                _ => {
                    let token = self.previous();
                    return Err(ShitRustError::SyntaxError {
                        line: token.line,
                        column: token.column,
                        message: "Invalid assignment target".to_string(),
                    });
                },
            }
        }
        
        Ok(expr)
    }
    
    fn logical_or(&mut self) -> Result<Expr, ShitRustError> {
        let mut expr = self.logical_and()?;
        
        while self.match_token(&[TokenType::Or]) {
            let right = self.logical_and()?;
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                op: BinOp::Or,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn logical_and(&mut self) -> Result<Expr, ShitRustError> {
        let mut expr = self.equality()?;
        
        while self.match_token(&[TokenType::And]) {
            let right = self.equality()?;
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                op: BinOp::And,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn equality(&mut self) -> Result<Expr, ShitRustError> {
        let mut expr = self.comparison()?;
        
        while self.match_token(&[TokenType::EqualEqual, TokenType::NotEqual]) {
            let op = match self.previous().token_type {
                TokenType::EqualEqual => BinOp::Eq,
                TokenType::NotEqual => BinOp::Ne,
                _ => unreachable!(),
            };
            
            let right = self.comparison()?;
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn comparison(&mut self) -> Result<Expr, ShitRustError> {
        let mut expr = self.term()?;
        
        while self.match_token(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let op = match self.previous().token_type {
                TokenType::Greater => BinOp::Gt,
                TokenType::GreaterEqual => BinOp::Ge,
                TokenType::Less => BinOp::Lt,
                TokenType::LessEqual => BinOp::Le,
                _ => unreachable!(),
            };
            
            let right = self.term()?;
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn term(&mut self) -> Result<Expr, ShitRustError> {
        let mut expr = self.factor()?;
        
        while self.match_token(&[TokenType::Plus, TokenType::Minus]) {
            let op = match self.previous().token_type {
                TokenType::Plus => BinOp::Add,
                TokenType::Minus => BinOp::Sub,
                _ => unreachable!(),
            };
            
            let right = self.factor()?;
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn factor(&mut self) -> Result<Expr, ShitRustError> {
        let mut expr = self.unary()?;
        
        while self.match_token(&[TokenType::Star, TokenType::Slash, TokenType::Percent]) {
            let op = match self.previous().token_type {
                TokenType::Star => BinOp::Mul,
                TokenType::Slash => BinOp::Div,
                TokenType::Percent => BinOp::Mod,
                _ => unreachable!(),
            };
            
            let right = self.unary()?;
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn unary(&mut self) -> Result<Expr, ShitRustError> {
        if self.match_token(&[TokenType::Minus, TokenType::Not]) {
            let op = match self.previous().token_type {
                TokenType::Minus => UnaryOp::Neg,
                TokenType::Not => UnaryOp::Not,
                _ => unreachable!(),
            };
            
            let right = self.unary()?;
            return Ok(Expr::UnaryOp {
                op,
                expr: Box::new(right),
            });
        }
        
        self.call()
    }
    
    fn call(&mut self) -> Result<Expr, ShitRustError> {
        let mut expr = self.primary()?;
        
        loop {
            if self.match_token(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(&[TokenType::Dot]) {
                let name = self.consume(TokenType::Identifier, "Expected property name after '.'")?;
                expr = Expr::FieldAccess {
                    object: Box::new(expr),
                    field: name.lexeme.clone(),
                };
            } else {
                break;
            }
        }
        
        Ok(expr)
    }
    
    fn finish_call(&mut self, callee: Expr) -> Result<Expr, ShitRustError> {
        let mut arguments = Vec::new();
        
        if !self.check(TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    return Err(ShitRustError::SyntaxError {
                        line: self.peek().line,
                        column: self.peek().column,
                        message: "Cannot have more than 255 arguments".to_string(),
                    });
                }
                
                arguments.push(self.expression()?);
                
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        
        self.consume(TokenType::RightParen, "Expected ')' after arguments")?;
        
        Ok(Expr::Call {
            func: Box::new(callee),
            args: arguments,
        })
    }
    
    fn primary(&mut self) -> Result<Expr, ShitRustError> {
        if self.match_token(&[TokenType::True]) {
            return Ok(Expr::Literal(Literal::Bool(true)));
        }
        if self.match_token(&[TokenType::False]) {
            return Ok(Expr::Literal(Literal::Bool(false)));
        }
        if self.match_token(&[TokenType::None]) {
            return Ok(Expr::Literal(Literal::None));
        }
        
        if self.match_token(&[TokenType::IntLiteral]) {
            let value = self.previous().lexeme.parse::<i64>().unwrap_or(0);
            return Ok(Expr::Literal(Literal::Int(value)));
        }
        if self.match_token(&[TokenType::FloatLiteral]) {
            let value = self.previous().lexeme.parse::<f64>().unwrap_or(0.0);
            return Ok(Expr::Literal(Literal::Float(value)));
        }
        if self.match_token(&[TokenType::StringLiteral]) {
            let text = self.previous().lexeme.clone();
            // Remove quotes from string literal
            let content = if text.len() >= 2 {
                text[1..text.len()-1].to_string()
            } else {
                "".to_string()
            };
            
            return Ok(Expr::Literal(Literal::String(content)));
        }
        if self.match_token(&[TokenType::CharLiteral]) {
            let text = self.previous().lexeme.clone();
            // Extract character from 'c' format
            let ch = if text.len() >= 3 {
                text.chars().nth(1).unwrap_or('\0')
            } else {
                '\0'
            };
            
            return Ok(Expr::Literal(Literal::Char(ch)));
        }
        
        if self.match_token(&[TokenType::Identifier]) {
            return Ok(Expr::Identifier(self.previous().lexeme.clone()));
        }
        
        if self.match_token(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expected ')' after expression")?;
            return Ok(expr);
        }
        
        Err(ShitRustError::SyntaxError {
            line: self.peek().line,
            column: self.peek().column,
            message: format!("Expected expression, got '{}'", self.peek().lexeme),
        })
    }
    
    // Helper methods for token management
    
    fn match_token(&mut self, types: &[TokenType]) -> bool {
        for token_type in types {
            if self.check(*token_type) {
                self.advance();
                return true;
            }
        }
        
        false
    }
    
    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        
        self.peek().token_type == token_type
    }
    
    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        
        self.previous()
    }
    
    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::EOF
    }
    
    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }
    
    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }
    
    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token, ShitRustError> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(ShitRustError::SyntaxError {
                line: self.peek().line,
                column: self.peek().column,
                message: message.to_string(),
            })
        }
    }
    
    // New method for trait declarations
    fn trait_declaration(&mut self) -> Result<Stmt, ShitRustError> {
        // Check for public
        let is_public = self.previous_was(&[TokenType::Pub]);
        
        // Parse trait name
        let name = self.consume(TokenType::Identifier, "Expected trait name")?;
        let name_str = name.lexeme.clone();
        
        // Parse generic parameters if present
        let generic_params = self.parse_generic_params();
        
        // Parse trait body
        self.consume(TokenType::LeftBrace, "Expected '{' after trait name")?;
        let mut methods = Vec::new();
        
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            // Method signature within trait
            let method = self.trait_method()?;
            methods.push(method);
        }
        
        self.consume(TokenType::RightBrace, "Expected '}' after trait body")?;
        
        Ok(Stmt::Trait {
            name: name_str,
            methods,
            is_public,
            generic_params,
        })
    }
    
    // Helper for parsing trait methods
    fn trait_method(&mut self) -> Result<TraitMethod, ShitRustError> {
        // Check if method is async
        let is_async = self.match_token(&[TokenType::Async]);
        
        self.consume(TokenType::Fn, "Expected 'fn' for trait method")?;
        
        // Parse method name
        let name = self.consume(TokenType::Identifier, "Expected method name")?;
        let name_str = name.lexeme.clone();
        
        // Parse parameters
        self.consume(TokenType::LeftParen, "Expected '(' after method name")?;
        let mut params = Vec::new();
        
        // Skip 'self' parameter if present
        if self.match_token(&[TokenType::Self_]) {
            params.push(("self".to_string(), Type::Custom("Self".to_string())));
            
            if !self.check(TokenType::RightParen) {
                self.consume(TokenType::Comma, "Expected ',' after 'self' parameter")?;
            }
        }
        
        // Parse the rest of the parameters
        if !self.check(TokenType::RightParen) {
            loop {
                let param_name = self.consume(TokenType::Identifier, "Expected parameter name")?;
                self.consume(TokenType::Colon, "Expected ':' after parameter name")?;
                let param_type = self.parse_type()?;
                
                params.push((param_name.lexeme.clone(), param_type));
                
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        
        self.consume(TokenType::RightParen, "Expected ')' after parameters")?;
        
        // Parse return type
        let return_type = if self.match_token(&[TokenType::Arrow]) {
            self.parse_type()?
        } else {
            Type::Void
        };
        
        // Parse optional method body
        let body = if self.match_token(&[TokenType::LeftBrace]) {
            let block = self.block()?;
            Some(block)
        } else {
            self.consume(TokenType::Semicolon, "Expected ';' after trait method signature")?;
            None
        };
        
        Ok(TraitMethod {
            name: name_str,
            params,
            return_type,
            body,
            is_async,
        })
    }
    
    // New method for impl blocks
    fn impl_declaration(&mut self) -> Result<Stmt, ShitRustError> {
        // Parse generic parameters if present
        let generic_params = self.parse_generic_params();
        
        // Check for trait implementation
        let trait_name = if !self.check(TokenType::Identifier) {
            let trait_name = self.consume(TokenType::Identifier, "Expected trait name")?;
            let trait_name_str = trait_name.lexeme.clone();
            
            self.consume(TokenType::For, "Expected 'for' after trait name")?;
            
            Some(trait_name_str)
        } else {
            None
        };
        
        // Parse type name that we're implementing for
        let type_name = self.consume(TokenType::Identifier, "Expected type name")?;
        let type_name_str = type_name.lexeme.clone();
        
        // Parse impl body
        self.consume(TokenType::LeftBrace, "Expected '{' after type name")?;
        let mut methods = Vec::new();
        
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            // Method implementation within impl block
            if self.match_token(&[TokenType::Async]) {
                if self.match_token(&[TokenType::Fn]) {
                    methods.push(self.function_with_async(true)?);
                }
            } else if self.match_token(&[TokenType::Fn]) {
                methods.push(self.function_with_async(false)?);
            } else {
                return Err(ShitRustError::SyntaxError {
                    line: self.peek().line,
                    column: self.peek().column,
                    message: "Expected function declaration in impl block".to_string(),
                });
            }
        }
        
        self.consume(TokenType::RightBrace, "Expected '}' after impl body")?;
        
        Ok(Stmt::Impl {
            trait_name,
            type_name: type_name_str,
            methods,
            generic_params,
        })
    }
    
    fn parse_generic_params(&mut self) -> Vec<String> {
        let mut generic_params = Vec::new();
        
        if self.match_token(&[TokenType::Less]) {
            loop {
                if let TokenType::Identifier = self.peek().token_type {
                    let param = self.advance();
                    generic_params.push(param.lexeme.clone());
                }
                
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
            
            self.consume(TokenType::Greater, "Expected '>' after generic parameters")
                .unwrap_or_else(|_| Token::new(TokenType::Greater, ">".to_string(), 0, 0));
        }
        
        generic_params
    }
    
    fn type_alias(&mut self) -> Result<Stmt, ShitRustError> {
        // Check for public
        let is_public = self.previous_was(&[TokenType::Pub]);
        
        // Parse type name
        let name = self.consume(TokenType::Identifier, "Expected type name")?;
        let name_str = name.lexeme.clone();
        
        // Parse generic parameters if present
        let generic_params = self.parse_generic_params();
        
        self.consume(TokenType::Equal, "Expected '=' after type name")?;
        
        // Parse the type to alias
        let alias_type = self.parse_type()?;
        
        self.consume(TokenType::Semicolon, "Expected ';' after type alias")?;
        
        Ok(Stmt::TypeAlias {
            name: name_str,
            alias_type,
            is_public,
            generic_params,
        })
    }
    
    fn const_declaration(&mut self) -> Result<Stmt, ShitRustError> {
        // Check for public
        let is_public = self.previous_was(&[TokenType::Pub]);
        
        // Parse const name
        let name = self.consume(TokenType::Identifier, "Expected constant name")?;
        let name_str = name.lexeme.clone();
        
        // Type annotation is required for constants
        self.consume(TokenType::Colon, "Expected ':' after constant name")?;
        let type_hint = self.parse_type()?;
        
        // Parse initializer
        self.consume(TokenType::Equal, "Expected '=' after type annotation")?;
        let value = self.expression()?;
        
        self.consume(TokenType::Semicolon, "Expected ';' after constant declaration")?;
        
        Ok(Stmt::Const {
            name: name_str,
            type_hint,
            value,
            is_public,
        })
    }
    
    fn use_declaration(&mut self) -> Result<Stmt, ShitRustError> {
        // Parse module path
        let path = self.consume(TokenType::Identifier, "Expected module path")?;
        let mut path_str = path.lexeme.clone();
        
        // Handle dotted paths (e.g., std.io)
        while self.match_token(&[TokenType::Dot]) {
            let next = self.consume(TokenType::Identifier, "Expected identifier after '.'")?;
            path_str.push_str(&format!(".{}", next.lexeme));
        }
        
        // Handle 'as' renaming
        let as_name = if self.match_token(&[TokenType::As]) {
            let alias = self.consume(TokenType::Identifier, "Expected identifier after 'as'")?;
            Some(alias.lexeme.clone())
        } else {
            None
        };
        
        self.consume(TokenType::Semicolon, "Expected ';' after use declaration")?;
        
        Ok(Stmt::Use {
            path: path_str,
            as_name,
        })
    }
    
    fn async_block(&mut self) -> Result<Stmt, ShitRustError> {
        self.consume(TokenType::LeftBrace, "Expected '{' after 'async'")?;
        let block = self.block()?;
        
        Ok(Stmt::Async {
            block,
        })
    }
    
    fn async_function(&mut self) -> Result<Stmt, ShitRustError> {
        self.function_with_async(true)
    }
    
    fn function_with_async(&mut self, is_async: bool) -> Result<Stmt, ShitRustError> {
        // Check for public
        let is_public = self.previous_was(&[TokenType::Pub]);
        
        // Parse function name
        let name = self.consume(TokenType::Identifier, "Expected function name")?;
        let name_str = name.lexeme.clone();
        
        // Parse generic parameters if present
        let generic_params = self.parse_generic_params();
        
        // Parse parameters
        self.consume(TokenType::LeftParen, "Expected '(' after function name")?;
        let mut params = Vec::new();
        
        if !self.check(TokenType::RightParen) {
            loop {
                let param_name = self.consume(TokenType::Identifier, "Expected parameter name")?;
                self.consume(TokenType::Colon, "Expected ':' after parameter name")?;
                let param_type = self.parse_type()?;
                
                params.push((param_name.lexeme.clone(), param_type));
                
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        
        self.consume(TokenType::RightParen, "Expected ')' after parameters")?;
        
        // Parse return type
        let return_type = if self.match_token(&[TokenType::Arrow]) {
            self.parse_type()?
        } else {
            Type::Void
        };
        
        // Parse function body
        self.consume(TokenType::LeftBrace, "Expected '{' before function body")?;
        let body = self.block()?;
        
        Ok(Stmt::Function {
            name: name_str,
            params,
            return_type,
            body,
            is_async,
            is_public,
            generic_params,
        })
    }
    
    // Helper to check if the previous token was one of the given types
    fn previous_was(&self, types: &[TokenType]) -> bool {
        if self.current == 0 {
            return false;
        }
        
        let prev_token = &self.tokens[self.current - 2]; // -2 because we've already consumed a token
        types.iter().any(|t| *t == prev_token.token_type)
    }
}