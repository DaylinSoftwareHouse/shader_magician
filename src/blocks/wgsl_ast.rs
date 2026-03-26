// wgsl_ast.rs  –  typed AST for WGSL function bodies

/// A fully-parsed WGSL function body / any `{ … }` block.
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Block {
    pub stmts: Vec<Statement>,
}

// ── Statements ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum Statement {
    /// `var/let/const <name> [: <ty>] [= <expr>];`
    VarDecl(VarDecl),
    /// `<lvalue> <op>= <expr>;`  or  `<lvalue> = <expr>;`
    Assign(AssignStatement),
    /// `<lvalue>++;`  or  `<lvalue>--;`
    Increment(LValue, IncrOp),
    /// `return [<expr>];`
    Return(Option<Expression>),
    /// `discard;`
    Discard,
    /// `break;`
    Break,
    /// `continue;`
    Continue,
    /// `break if <expr>;`  (WGSL continuing-block special form)
    BreakIf(Expression),
    /// `if <expr> { … } [else if … | else { … }]`
    If(IfStatement),
    /// `switch <expr> { <case>* }`
    Switch(SwitchStatement),
    /// `loop { … [continuing { … }] }`
    Loop(LoopStatement),
    /// `for (<init>; <cond>; <update>) { … }`
    For(ForStatement),
    /// `while <expr> { … }`
    While(WhileStatement),
    /// A bare expression used as a statement, e.g. a function call.
    Expression(Expression),
    /// `{ … }`  — a bare compound statement
    Block(Block),
}

// ── Variable declarations ─────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct VarDecl {
    /// `var`, `let`, or `const`
    pub kind: VarKind,
    /// Optional `<access, address_space>` template args on `var`
    pub template_args: Option<String>,
    pub name: String,
    pub ty: Option<String>,
    pub initializer: Option<Expression>,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum VarKind {
    Var,
    Let,
    Const,
}

// ── Assignments ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct AssignStatement {
    pub target: LValue,
    pub op: AssignOp,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum AssignOp {
    Simple,   // =
    Add,      // +=
    Sub,      // -=
    Mul,      // *=
    Div,      // /=
    Mod,      // %=
    And,      // &=
    Or,       // |=
    Xor,      // ^=
    Shl,      // <<=
    Shr,      // >>=
}

impl AssignOp {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "="   => Some(Self::Simple),
            "+="  => Some(Self::Add),
            "-="  => Some(Self::Sub),
            "*="  => Some(Self::Mul),
            "/="  => Some(Self::Div),
            "%="  => Some(Self::Mod),
            "&="  => Some(Self::And),
            "|="  => Some(Self::Or),
            "^="  => Some(Self::Xor),
            "<<=" => Some(Self::Shl),
            ">>=" => Some(Self::Shr),
            _     => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum IncrOp { Inc, Dec }

/// The left-hand side of an assignment.
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum LValue {
    Ident(String),
    /// `base[index]`
    Index(Box<LValue>, Box<Expression>),
    /// `base.field`
    Field(Box<LValue>, String),
    /// `*ptr`
    Deref(Box<LValue>),
    /// `_`  (phony assignment)
    Phony,
}

// ── Control flow ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct IfStatement {
    pub condition: Expression,
    pub body: Block,
    pub else_ifs: Vec<(Expression, Block)>,
    pub else_body: Option<Block>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct SwitchStatement {
    pub selector: Expression,
    pub cases: Vec<SwitchCase>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct SwitchCase {
    /// `None` == `default`
    pub selectors: Vec<Option<Expression>>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct LoopStatement {
    pub body: Block,
    pub continuing: Option<Block>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct ForStatement {
    /// The initialiser is either a var-decl or a simple assignment/call.
    pub init: Option<Box<Statement>>,
    pub condition: Option<Expression>,
    pub update: Option<Box<Statement>>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct WhileStatement {
    pub condition: Expression,
    pub body: Block,
}

// ── Expressions ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum Expression {
    /// Integer literal (stores raw text to preserve `u`/`i` suffixes)
    IntLiteral(String),
    /// Float literal (stores raw text to preserve `f`/`h` suffixes)
    FloatLiteral(String),
    BoolLiteral(bool),
    Identifier(String),

    // Unary
    Unary(UnaryOp, Box<Expression>),
    // Binary
    Binary(Box<Expression>, BinaryOp, Box<Expression>),

    // Postfix
    /// `expr(args…)`
    Call(Box<Expression>, Vec<Expression>),
    /// `expr[index]`
    Index(Box<Expression>, Box<Expression>),
    /// `expr.field`
    Field(Box<Expression>, String),
    /// `*expr`  (pointer dereference in expression position)
    Deref(Box<Expression>),
    /// `&expr`  (address-of)
    AddrOf(Box<Expression>),

    /// `type_name(args…)` — explicit type constructor / bitcast / etc.
    TypeConstruct(String, Vec<Expression>),
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum UnaryOp {
    Neg,        // -
    Not,        // !
    BitNot,     // ~
    Deref,      // *
    AddrOf,     // &
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub enum BinaryOp {
    // arithmetic
    Add, Sub, Mul, Div, Mod,
    // bitwise
    BitAnd, BitOr, BitXor, Shl, Shr,
    // logical
    And, Or,
    // comparison
    Eq, Ne, Lt, Le, Gt, Ge,
}

impl BinaryOp {
    /// Pratt precedence (higher = tighter binding).
    pub fn precedence(self) -> u8 {
        use BinaryOp::*;
        match self {
            Or           => 1,
            And          => 2,
            BitOr        => 3,
            BitXor       => 4,
            BitAnd       => 5,
            Eq | Ne      => 6,
            Lt | Le | Gt | Ge => 7,
            Shl | Shr    => 8,
            Add | Sub    => 9,
            Mul | Div | Mod => 10,
        }
    }
}
