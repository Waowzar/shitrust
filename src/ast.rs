#[derive(Debug, Clone)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
    Char,
    List(Box<Type>),
    Dict(Box<Type>, Box<Type>),
    Tuple(Vec<Type>),
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
    Future(Box<Type>),
    Range(Box<Type>),
    Void,
    Custom(String),
    Function(Vec<Type>, Box<Type>),
    Reference(Box<Type>, bool),
    Array(Box<Type>, Option<usize>),
    Generic(String, Vec<Type>),
    Trait(String),
    Union(Vec<Type>),
    Never,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Char(char),
    List(Vec<Expr>),
    Dict(Vec<(Expr, Expr)>),
    Tuple(Vec<Expr>),
    Range { start: Option<Box<Expr>>, end: Option<Box<Expr>>, inclusive: bool },
    None,
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    LeftShift,
    RightShift,
    NullishCoalescing,
    OptionalChaining,
    Pipeline,
    Exponent,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Neg,
    Not,
    BitNot,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Identifier(String),
    BinaryOp {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    UnaryOp {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
    },
    MethodCall {
        object: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    Index {
        target: Box<Expr>,
        index: Box<Expr>,
    },
    FieldAccess {
        object: Box<Expr>,
        field: String,
    },
    Lambda {
        params: Vec<(String, Option<Type>)>,
        body: Box<Expr>,
        return_type: Option<Type>,
    },
    Await {
        expr: Box<Expr>,
    },
    Try {
        expr: Box<Expr>,
    },
    TernaryIf {
        condition: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
    },
    RangeExpr {
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
        inclusive: bool,
    },
    OptionalChain {
        expr: Box<Expr>,
        chain: Vec<OptionalChainItem>,
    },
    TypeCast {
        expr: Box<Expr>,
        target_type: Type,
    },
    ListComprehension {
        expr: Box<Expr>,
        iterable: Box<Expr>,
        var_name: String,
        condition: Option<Box<Expr>>,
    },
    Match {
        expr: Box<Expr>,
        arms: Vec<(Pattern, Box<Expr>)>,
    },
    StructInit {
        name: String,
        fields: Vec<(String, Expr)>,
    },
    PipelineChain {
        initial: Box<Expr>,
        chain: Vec<Box<Expr>>,
    },
}

#[derive(Debug, Clone)]
pub enum OptionalChainItem {
    Field(String),
    Method(String, Vec<Expr>),
    Index(Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr),
    Let {
        name: String,
        type_hint: Option<Type>,
        value: Expr,
        mutable: bool,
    },
    Assign {
        target: Expr,
        value: Expr,
    },
    If {
        condition: Expr,
        then_block: Vec<Stmt>,
        else_block: Option<Vec<Stmt>>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    For {
        var: String,
        iterator: Expr,
        body: Vec<Stmt>,
    },
    Match {
        expr: Expr,
        arms: Vec<(Pattern, Vec<Stmt>)>,
    },
    Return(Option<Expr>),
    Break,
    Continue,
    Function {
        name: String,
        params: Vec<(String, Type)>,
        return_type: Type,
        body: Vec<Stmt>,
        is_async: bool,
        is_public: bool,
        generic_params: Vec<String>,
    },
    Struct {
        name: String,
        fields: Vec<(String, Type, bool)>,
        methods: Vec<Stmt>,
        is_public: bool,
        generic_params: Vec<String>,
    },
    Enum {
        name: String,
        variants: Vec<(String, Vec<Type>)>,
        is_public: bool,
        generic_params: Vec<String>,
    },
    Import {
        path: String,
        items: Vec<String>,
    },
    Try {
        block: Vec<Stmt>,
        catch_blocks: Vec<(Pattern, Vec<Stmt>)>,
        finally_block: Option<Vec<Stmt>>,
    },
    Async {
        block: Vec<Stmt>,
    },
    Loop {
        body: Vec<Stmt>,
    },
    Use {
        path: String,
        as_name: Option<String>,
    },
    Trait {
        name: String,
        methods: Vec<TraitMethod>,
        is_public: bool,
        generic_params: Vec<String>,
    },
    Impl {
        trait_name: Option<String>,
        type_name: String,
        methods: Vec<Stmt>,
        generic_params: Vec<String>,
    },
    Const {
        name: String,
        type_hint: Type,
        value: Expr,
        is_public: bool,
    },
    TypeAlias {
        name: String,
        alias_type: Type,
        is_public: bool,
        generic_params: Vec<String>,
    },
}

#[derive(Debug, Clone)]
pub struct TraitMethod {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_type: Type,
    pub body: Option<Vec<Stmt>>,
    pub is_async: bool,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Wildcard,
    Literal(Literal),
    Identifier(String),
    Destructure {
        name: String,
        fields: Vec<(String, Pattern)>,
    },
    EnumVariant {
        name: String,
        values: Vec<Pattern>,
    },
    Or(Vec<Pattern>),
    Range {
        start: Literal,
        end: Literal,
        inclusive: bool,
    },
}

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Stmt>,
    pub source_file: Option<String>,
}

impl Program {
    pub fn new(statements: Vec<Stmt>) -> Self {
        Self {
            statements,
            source_file: None,
        }
    }
    
    pub fn with_source(statements: Vec<Stmt>, source_file: String) -> Self {
        Self {
            statements,
            source_file: Some(source_file),
        }
    }
} 