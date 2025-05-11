# ShitRust Language Grammar

This document defines the formal grammar for the ShitRust programming language.

## Lexical Grammar

### Keywords

```
let, mut, fn, if, else, while, for, in, match, return, break, continue
struct, enum, trait, impl, pub, where, as, import, from, self, true, false, none
```

### Operators

```
+    -    *    /    %    =    ==   !=   <    >    <=   >=   !    &&   ||   
+=   -=   *=   /=   %=   ++   --   ->   =>   ..   ...  ::   ?
```

### Delimiters

```
(    )    {    }    [    ]    ,    .    :    ;    |    $
```

### Literals

```
IntLiteral    ::= Digit+
FloatLiteral  ::= Digit+ "." Digit+ [("e"|"E") ["+"|"-"] Digit+]
StringLiteral ::= "\"" [^"]* "\""
CharLiteral   ::= "'" . "'"
BoolLiteral   ::= "true" | "false"
NoneLiteral   ::= "none"

Digit         ::= "0"..."9"
Letter        ::= "a"..."z" | "A"..."Z"
Identifier    ::= (Letter | "_") (Letter | Digit | "_")*
```

## Syntactic Grammar

### Program

```
Program        ::= Declaration*
```

### Declarations

```
Declaration    ::= VarDeclaration
                 | FunctionDeclaration
                 | StructDeclaration
                 | EnumDeclaration
                 | TraitDeclaration
                 | ImplDeclaration
                 | ImportDeclaration

VarDeclaration ::= "let" ["mut"] Identifier [":" Type] "=" Expression ";"

FunctionDeclaration ::= ["pub"] "fn" Identifier "(" [ParameterList] ")" ["->" Type] Block

ParameterList  ::= Parameter ("," Parameter)*
Parameter      ::= Identifier ":" Type ["=" Expression]

StructDeclaration ::= ["pub"] "struct" Identifier ["<" GenericParams ">"] "{" StructField* [MethodDeclaration*] "}"
StructField    ::= ["pub"] Identifier ":" Type ","

EnumDeclaration ::= ["pub"] "enum" Identifier ["<" GenericParams ">"] "{" EnumVariant* "}"
EnumVariant    ::= Identifier ["(" [TypeList] ")"] ","

TraitDeclaration ::= ["pub"] "trait" Identifier ["<" GenericParams ">"] "{" TraitMember* "}"
TraitMember    ::= FunctionSignature [";" | Block]

ImplDeclaration ::= "impl" ["<" GenericParams ">"] [TraitName "for"] TypeName "{" MethodDeclaration* "}"
MethodDeclaration ::= ["pub"] "fn" Identifier "(" [ParameterList] ")" ["->" Type] Block

ImportDeclaration ::= "import" ( "{" ImportedItems "}" "from" StringLiteral | "*" "as" Identifier "from" StringLiteral | ImportPath ) ";"
ImportedItems  ::= Identifier ("," Identifier)*
ImportPath     ::= StringLiteral
```

### Types

```
Type           ::= PrimaryType
                 | ListType
                 | DictType
                 | TupleType
                 | OptionType
                 | ResultType
                 | FunctionType
                 | ReferenceType

PrimaryType    ::= Identifier ["<" TypeList ">"]
ListType       ::= "[" Type "]"
DictType       ::= "{" Type ":" Type "}"
TupleType      ::= "(" Type ("," Type)* ")"
OptionType     ::= "Option" "<" Type ">"
ResultType     ::= "Result" "<" Type "," Type ">"
FunctionType   ::= "fn" "(" [TypeList] ")" "->" Type
ReferenceType  ::= "&" ["mut"] Type

TypeList       ::= Type ("," Type)*
GenericParams  ::= Identifier [":" TraitBound] ("," Identifier [":" TraitBound])*
TraitBound     ::= Identifier ["+" Identifier]*
```

### Statements

```
Statement      ::= ExprStatement
                 | VarDeclaration
                 | Block
                 | IfStatement
                 | WhileStatement
                 | ForStatement
                 | MatchStatement
                 | ReturnStatement
                 | BreakStatement
                 | ContinueStatement

ExprStatement  ::= Expression ";"
Block          ::= "{" Statement* "}"

IfStatement    ::= "if" Expression Block ["else" (IfStatement | Block)]
WhileStatement ::= "while" Expression Block
ForStatement   ::= "for" Identifier "in" Expression Block

MatchStatement ::= "match" Expression "{" MatchArm* "}"
MatchArm       ::= Pattern "=>" (Block | Expression ",")

ReturnStatement ::= "return" [Expression] ";"
BreakStatement ::= "break" ";"
ContinueStatement ::= "continue" ";"
```

### Patterns

```
Pattern        ::= LiteralPattern
                 | IdentifierPattern
                 | WildcardPattern
                 | StructPattern
                 | EnumPattern
                 | TuplePattern
                 | ListPattern

LiteralPattern ::= Literal
IdentifierPattern ::= Identifier
WildcardPattern ::= "_"
StructPattern  ::= TypeName "{" [FieldPattern ("," FieldPattern)*] "}"
FieldPattern   ::= Identifier ":" Pattern
EnumPattern    ::= EnumName "::" Identifier ["(" [Pattern ("," Pattern)*] ")"]
TuplePattern   ::= "(" [Pattern ("," Pattern)*] ")"
ListPattern    ::= "[" [Pattern ("," Pattern)*] "]"
```

### Expressions

```
Expression     ::= AssignmentExpr

AssignmentExpr ::= LogicalOrExpr ["=" AssignmentExpr]
                 | LogicalOrExpr ("+=" | "-=" | "*=" | "/=" | "%=") AssignmentExpr

LogicalOrExpr  ::= LogicalAndExpr ["||" LogicalOrExpr]
LogicalAndExpr ::= EqualityExpr ["&&" LogicalAndExpr]
EqualityExpr   ::= RelationalExpr [("==" | "!=") EqualityExpr]
RelationalExpr ::= AdditiveExpr [("<" | ">" | "<=" | ">=") RelationalExpr]
AdditiveExpr   ::= MultiplicativeExpr [("+" | "-") AdditiveExpr]
MultiplicativeExpr ::= UnaryExpr [("*" | "/" | "%") MultiplicativeExpr]

UnaryExpr      ::= ("!" | "-") UnaryExpr | PostfixExpr
PostfixExpr    ::= PrimaryExpr
                 | PostfixExpr "." Identifier
                 | PostfixExpr "[" Expression "]"
                 | PostfixExpr "(" [Arguments] ")"
                 | PostfixExpr "++"
                 | PostfixExpr "--"

PrimaryExpr    ::= Literal
                 | Identifier
                 | "(" Expression ")"
                 | ListExpr
                 | DictExpr
                 | TupleExpr
                 | StructExpr
                 | LambdaExpr
                 | IfExpr
                 | MatchExpr
                 | ListComprehensionExpr

Literal        ::= IntLiteral
                 | FloatLiteral
                 | StringLiteral
                 | CharLiteral
                 | BoolLiteral
                 | NoneLiteral

ListExpr       ::= "[" [ExpressionList] "]"
DictExpr       ::= "{" [KeyValuePair ("," KeyValuePair)*] "}"
KeyValuePair   ::= Expression ":" Expression
TupleExpr      ::= "(" [Expression ("," Expression)*] ")"
StructExpr     ::= TypeName "{" [FieldExpr ("," FieldExpr)*] "}"
FieldExpr      ::= Identifier ":" Expression

LambdaExpr     ::= "|" [ParameterList] "|" ["->" Type] (Expression | Block)
IfExpr         ::= "if" Expression Block "else" (IfExpr | Block)
MatchExpr      ::= "match" Expression "{" [MatchExprArm+] "}"
MatchExprArm   ::= Pattern "=>" Expression ","

Arguments      ::= Expression ("," Expression)*
ExpressionList ::= Expression ("," Expression)*

ListComprehensionExpr ::= "[" Expression "for" Identifier "in" Expression ["if" Expression] "]"
```

## Precedence and Associativity

In order of decreasing precedence:

1. Grouping, member access, method call, subscript, postfix (`()`, `.`, `()`, `[]`, `++`, `--`)
2. Unary operators (`!`, `-`)
3. Multiplication, division, remainder (`*`, `/`, `%`)
4. Addition, subtraction (`+`, `-`)
5. Relational operators (`<`, `>`, `<=`, `>=`)
6. Equality operators (`==`, `!=`)
7. Logical AND (`&&`)
8. Logical OR (`||`)
9. Assignment operators (`=`, `+=`, `-=`, `*=`, `/=`, `%=`)

## Contextual Keywords

Some identifiers have special meaning in specific contexts:

- `self`: Refers to the current object instance within methods
- `where`: Used in generic type constraints
- `as`: Used for type casting and module imports
- `pub`: Marks items as publicly accessible

## Comments

```
// Single-line comment
/* Multi-line comment */
/// Documentation comment
```