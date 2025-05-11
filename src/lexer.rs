use std::fmt;
use crate::error::{ShitRustError, Result, SourceLocation};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Identifiers & literals
    Identifier,
    IntLiteral,
    FloatLiteral,
    StringLiteral,
    CharLiteral,
    BoolLiteral,

    // Keywords
    Let,
    Mut,
    Fn,
    If,
    Else,
    While,
    For,
    In,
    Match,
    Return,
    Break,
    Continue,
    Struct,
    Enum,
    Import,
    From,
    As,
    Pub,
    True,
    False,
    None,
    // New keywords
    Async,
    Await,
    Try,
    Catch,
    Finally,
    Static,
    Type,
    Trait,
    Impl,
    Self_,
    This,
    Result,
    Ok,
    Err,
    Use,
    Const,
    Loop,

    // Types
    Int,
    Float,
    Bool,
    String,
    Char,
    Void,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    EqualEqual,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    And,
    Or,
    Not,
    PlusEqual,
    MinusEqual,
    StarEqual,
    SlashEqual,
    PercentEqual,

    // Delimiters
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Dot,
    Colon,
    Semicolon,
    Arrow,
    FatArrow,
    
    // Special
    EOF,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenType::Identifier => write!(f, "identifier"),
            TokenType::IntLiteral => write!(f, "integer literal"),
            TokenType::FloatLiteral => write!(f, "float literal"),
            TokenType::StringLiteral => write!(f, "string literal"),
            TokenType::CharLiteral => write!(f, "character literal"),
            TokenType::BoolLiteral => write!(f, "boolean literal"),
            // Keywords
            TokenType::Let => write!(f, "let"),
            TokenType::Mut => write!(f, "mut"),
            TokenType::Fn => write!(f, "fn"),
            TokenType::If => write!(f, "if"),
            TokenType::Else => write!(f, "else"),
            TokenType::While => write!(f, "while"),
            TokenType::For => write!(f, "for"),
            TokenType::In => write!(f, "in"),
            TokenType::Match => write!(f, "match"),
            TokenType::Return => write!(f, "return"),
            TokenType::Break => write!(f, "break"),
            TokenType::Continue => write!(f, "continue"),
            TokenType::Struct => write!(f, "struct"),
            TokenType::Enum => write!(f, "enum"),
            TokenType::Import => write!(f, "import"),
            TokenType::From => write!(f, "from"),
            TokenType::As => write!(f, "as"),
            TokenType::Pub => write!(f, "pub"),
            TokenType::True => write!(f, "true"),
            TokenType::False => write!(f, "false"),
            TokenType::None => write!(f, "none"),
            // New keywords
            TokenType::Async => write!(f, "async"),
            TokenType::Await => write!(f, "await"),
            TokenType::Try => write!(f, "try"),
            TokenType::Catch => write!(f, "catch"),
            TokenType::Finally => write!(f, "finally"),
            TokenType::Static => write!(f, "static"),
            TokenType::Type => write!(f, "type"),
            TokenType::Trait => write!(f, "trait"),
            TokenType::Impl => write!(f, "impl"),
            TokenType::Self_ => write!(f, "self"),
            TokenType::This => write!(f, "this"),
            TokenType::Result => write!(f, "result"),
            TokenType::Ok => write!(f, "ok"),
            TokenType::Err => write!(f, "err"),
            TokenType::Use => write!(f, "use"),
            TokenType::Const => write!(f, "const"),
            TokenType::Loop => write!(f, "loop"),
            // Types
            TokenType::Int => write!(f, "int"),
            TokenType::Float => write!(f, "float"),
            TokenType::Bool => write!(f, "bool"),
            TokenType::String => write!(f, "string"),
            TokenType::Char => write!(f, "char"),
            TokenType::Void => write!(f, "void"),
            // Operators and other symbols
            TokenType::Plus => write!(f, "+"),
            TokenType::Minus => write!(f, "-"),
            TokenType::Star => write!(f, "*"),
            TokenType::Slash => write!(f, "/"),
            TokenType::Percent => write!(f, "%"),
            TokenType::Equal => write!(f, "="),
            TokenType::EqualEqual => write!(f, "=="),
            TokenType::NotEqual => write!(f, "!="),
            TokenType::Greater => write!(f, ">"),
            TokenType::GreaterEqual => write!(f, ">="),
            TokenType::Less => write!(f, "<"),
            TokenType::LessEqual => write!(f, "<="),
            TokenType::And => write!(f, "&&"),
            TokenType::Or => write!(f, "||"),
            TokenType::Not => write!(f, "!"),
            TokenType::PlusEqual => write!(f, "+="),
            TokenType::MinusEqual => write!(f, "-="),
            TokenType::StarEqual => write!(f, "*="),
            TokenType::SlashEqual => write!(f, "/="),
            TokenType::PercentEqual => write!(f, "%="),
            TokenType::LeftParen => write!(f, "("),
            TokenType::RightParen => write!(f, ")"),
            TokenType::LeftBrace => write!(f, "{{"),
            TokenType::RightBrace => write!(f, "}}"),
            TokenType::LeftBracket => write!(f, "["),
            TokenType::RightBracket => write!(f, "]"),
            TokenType::Comma => write!(f, ","),
            TokenType::Dot => write!(f, "."),
            TokenType::Colon => write!(f, ":"),
            TokenType::Semicolon => write!(f, ";"),
            TokenType::Arrow => write!(f, "->"),
            TokenType::FatArrow => write!(f, "=>"),
            TokenType::EOF => write!(f, "end of file"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: usize, column: usize) -> Self {
        Token {
            token_type,
            lexeme,
            line,
            column,
        }
    }
    
    /// Get the source location of this token
    pub fn location(&self) -> SourceLocation {
        SourceLocation::new(self.line, self.column)
    }
}

pub struct Lexer {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    column: usize,
    filename: Option<String>,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer {
            source: source.to_string(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            column: 1,
            filename: None,
        }
    }
    
    /// Create a new lexer with a filename for better error reporting
    pub fn with_filename(source: &str, filename: String) -> Self {
        Lexer {
            source: source.to_string(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            column: 1,
            filename: Some(filename),
        }
    }
    
    /// Return a source location at the current position
    fn current_location(&self) -> SourceLocation {
        if let Some(filename) = &self.filename {
            SourceLocation::with_file(self.line, self.column, filename.clone())
        } else {
            SourceLocation::new(self.line, self.column)
        }
    }
    
    /// Generate a syntax error at the current location
    fn error(&self, message: &str) -> ShitRustError {
        ShitRustError::SyntaxError {
            location: self.current_location(),
            message: message.to_string(),
        }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }

        // Add EOF token
        let eof_column = self.column;
        self.tokens.push(Token::new(
            TokenType::EOF,
            String::new(),
            self.line,
            eof_column,
        ));

        Ok(self.tokens.clone())
    }

    fn scan_token(&mut self) -> Result<()> {
        let c = self.advance();
        match c {
            // Single character tokens
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            '[' => self.add_token(TokenType::LeftBracket),
            ']' => self.add_token(TokenType::RightBracket),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            ':' => self.add_token(TokenType::Colon),
            ';' => self.add_token(TokenType::Semicolon),
            
            // Single or double character tokens
            '+' => {
                if self.match_char('=') {
                    self.add_token(TokenType::PlusEqual)
                } else {
                    self.add_token(TokenType::Plus)
                }
            },
            '-' => {
                if self.match_char('>') {
                    self.add_token(TokenType::Arrow)
                } else if self.match_char('=') {
                    self.add_token(TokenType::MinusEqual)
                } else {
                    self.add_token(TokenType::Minus)
                }
            },
            '*' => {
                if self.match_char('=') {
                    self.add_token(TokenType::StarEqual)
                } else {
                    self.add_token(TokenType::Star)
                }
            },
            '/' => {
                if self.match_char('/') {
                    // Comment until the end of the line
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else if self.match_char('*') {
                    // Block comment
                    self.block_comment()?;
                } else if self.match_char('=') {
                    self.add_token(TokenType::SlashEqual)
                } else {
                    self.add_token(TokenType::Slash)
                }
            },
            '%' => {
                if self.match_char('=') {
                    self.add_token(TokenType::PercentEqual)
                } else {
                    self.add_token(TokenType::Percent)
                }
            },
            '!' => {
                if self.match_char('=') {
                    self.add_token(TokenType::NotEqual)
                } else {
                    self.add_token(TokenType::Not)
                }
            },
            '=' => {
                if self.match_char('=') {
                    self.add_token(TokenType::EqualEqual)
                } else if self.match_char('>') {
                    self.add_token(TokenType::FatArrow)
                } else {
                    self.add_token(TokenType::Equal)
                }
            },
            '<' => {
                if self.match_char('=') {
                    self.add_token(TokenType::LessEqual)
                } else {
                    self.add_token(TokenType::Less)
                }
            },
            '>' => {
                if self.match_char('=') {
                    self.add_token(TokenType::GreaterEqual)
                } else {
                    self.add_token(TokenType::Greater)
                }
            },
            '&' => {
                if self.match_char('&') {
                    self.add_token(TokenType::And)
                } else {
                    return Err(self.error("Expected '&' after '&'"));
                }
            },
            '|' => {
                if self.match_char('|') {
                    self.add_token(TokenType::Or)
                } else {
                    return Err(self.error("Expected '|' after '|'"));
                }
            },
            
            // String literals
            '"' => self.string()?,
            '\'' => self.char_literal()?,
            
            // Whitespace
            ' ' | '\r' | '\t' => {},
            '\n' => {
                self.line += 1;
                self.column = 1;
            },
            
            // Numbers and identifiers
            _ => {
                if c.is_digit(10) {
                    self.number()?;
                } else if c.is_alphabetic() || c == '_' {
                    self.identifier();
                } else {
                    return Err(self.error(&format!("Unexpected character: '{}'", c)));
                }
            }
        }
        Ok(())
    }

    fn block_comment(&mut self) -> Result<()> {
        let mut nesting = 1;
        
        while nesting > 0 {
            if self.is_at_end() {
                return Err(self.error("Unterminated block comment"));
            }

            if self.peek() == '/' && self.peek_next() == '*' {
                self.advance();
                self.advance();
                nesting += 1;
            } else if self.peek() == '*' && self.peek_next() == '/' {
                self.advance();
                self.advance();
                nesting -= 1;
            } else if self.peek() == '\n' {
                self.advance();
                self.line += 1;
                self.column = 1;
            } else {
                self.advance();
            }
        }
        
        Ok(())
    }

    fn string(&mut self) -> Result<()> {
        let start_line = self.line;
        let start_column = self.column - 1;  // -1 for the opening "

        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
                self.column = 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err(self.error("Unterminated string"));
        }

        // Consume the closing "
        self.advance();

        // Extract string value without the quotes
        let value = self.source[self.start + 1..self.current - 1].to_string();
        self.add_token_with_lexeme(TokenType::StringLiteral, value);
        
        Ok(())
    }

    fn char_literal(&mut self) -> Result<()> {
        let start_line = self.line;
        let start_column = self.column - 1;  // -1 for the opening '

        // Allow escaped characters
        if self.peek() == '\\' {
            self.advance();
            if self.is_at_end() {
                return Err(self.error("Unterminated character literal"));
            }
            self.advance();
        } else if self.peek() != '\'' {
            self.advance();
        } else {
            return Err(self.error("Empty character literal"));
        }

        if self.peek() != '\'' {
            return Err(self.error("Character literal may contain only one character"));
        }

        // Consume the closing '
        self.advance();

        // Extract char value without the quotes
        let value = self.source[self.start + 1..self.current - 1].to_string();
        self.add_token_with_lexeme(TokenType::CharLiteral, value);
        
        Ok(())
    }

    fn number(&mut self) -> Result<()> {
        while self.peek().is_digit(10) {
            self.advance();
        }

        // Look for a decimal part
        if self.peek() == '.' && self.peek_next().is_digit(10) {
            // Consume the "."
            self.advance();

            while self.peek().is_digit(10) {
                self.advance();
            }
            
            // Check for exponent
            if self.peek() == 'e' || self.peek() == 'E' {
                self.advance();
                if self.peek() == '+' || self.peek() == '-' {
                    self.advance();
                }
                if !self.peek().is_digit(10) {
                    return Err(self.error("Invalid exponent in float literal"));
                }
                while self.peek().is_digit(10) {
                    self.advance();
                }
            }

            self.add_token_with_lexeme(
                TokenType::FloatLiteral,
                self.source[self.start..self.current].to_string(),
            );
        } else {
            self.add_token_with_lexeme(
                TokenType::IntLiteral,
                self.source[self.start..self.current].to_string(),
            );
        }
        
        Ok(())
    }

    fn identifier(&mut self) {
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.advance();
        }

        let text = &self.source[self.start..self.current];
        let token_type = match text {
            // Keywords
            "let" => TokenType::Let,
            "mut" => TokenType::Mut,
            "fn" => TokenType::Fn,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "while" => TokenType::While,
            "for" => TokenType::For,
            "in" => TokenType::In,
            "match" => TokenType::Match,
            "return" => TokenType::Return,
            "break" => TokenType::Break,
            "continue" => TokenType::Continue,
            "struct" => TokenType::Struct,
            "enum" => TokenType::Enum,
            "import" => TokenType::Import,
            "from" => TokenType::From,
            "as" => TokenType::As,
            "pub" => TokenType::Pub,
            "true" => TokenType::True,
            "false" => TokenType::False,
            "none" => TokenType::None,
            // New keywords
            "async" => TokenType::Async,
            "await" => TokenType::Await,
            "try" => TokenType::Try,
            "catch" => TokenType::Catch,
            "finally" => TokenType::Finally,
            "static" => TokenType::Static,
            "type" => TokenType::Type,
            "trait" => TokenType::Trait,
            "impl" => TokenType::Impl,
            "self" => TokenType::Self_,
            "this" => TokenType::This,
            "result" => TokenType::Result,
            "ok" => TokenType::Ok,
            "err" => TokenType::Err,
            "use" => TokenType::Use,
            "const" => TokenType::Const,
            "loop" => TokenType::Loop,
            // Types
            "int" => TokenType::Int,
            "float" => TokenType::Float,
            "bool" => TokenType::Bool,
            "string" => TokenType::String,
            "char" => TokenType::Char,
            "void" => TokenType::Void,
            // Identifier
            _ => TokenType::Identifier,
        };

        self.add_token(token_type);
    }

    fn advance(&mut self) -> char {
        let c = self.source.chars().nth(self.current).unwrap_or('\0');
        self.current += 1;
        self.column += 1;
        c
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.source.chars().nth(self.current).unwrap_or('\0') != expected {
            return false;
        }
        
        self.current += 1;
        self.column += 1;
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source.chars().nth(self.current).unwrap_or('\0')
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source.chars().nth(self.current + 1).unwrap_or('\0')
        }
    }

    fn add_token(&mut self, token_type: TokenType) {
        let lexeme = self.source[self.start..self.current].to_string();
        self.add_token_with_lexeme(token_type, lexeme);
    }

    fn add_token_with_lexeme(&mut self, token_type: TokenType, lexeme: String) {
        self.tokens.push(Token::new(
            token_type,
            lexeme,
            self.line,
            self.column - (self.current - self.start),
        ));
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
} 