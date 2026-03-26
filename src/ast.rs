use std::collections::HashMap;

use crate::WgslType;

/// Represents a block of statements in WGSL.
pub type Block = Vec<Statement>;

/// Represents a statement in WGSL.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Statement {
    /// Variable declaration: var x: T = expr; or let x = expr; or const x: T = expr;
    VarDecl {
        kind: VarKind,
        name: String,
        ty: Option<WgslType>,
        init: Option<Expression>,
    },
    /// Assignment: target = value;
    Assignment {
        target: Expression,
        value: Expression,
    },
    /// Return statement
    Return(Option<Expression>),
    /// If statement with optional else block
    If {
        condition: Expression,
        then_block: Block,
        else_block: Option<Block>,
    },
    /// For loop
    For {
        init: Option<Box<Statement>>,
        condition: Option<Expression>,
        update: Option<Expression>,
        body: Block,
    },
    /// Loop with optional continuing block
    Loop {
        body: Block,
        continuing: Option<Block>,
    },
    /// Break statement
    Break,
    /// Continue statement
    Continue,
    /// Expression as a statement
    ExprStatement(Expression),
    /// Block of statements
    Block(Block),
    /// Preprocessor placeholder for #id and #{value} syntax
    PreprocessorPlaceholder(String),
    /// Discard statement
    Discard,
}

/// Variable declaration kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VarKind {
    Var,
    Let,
    Const,
}

/// Represents an expression in WGSL.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expression {
    /// Literal value
    Literal(Literal),
    /// Identifier name
    Identifier(String),
    /// Field access: base.field
    FieldAccess {
        base: Box<Expression>,
        field: String,
    },
    /// Index access: base[index]
    IndexAccess {
        base: Box<Expression>,
        index: Box<Expression>,
    },
    /// Function call: name(args)
    Call {
        name: String,
        args: Vec<Expression>,
    },
    /// Type constructor: T(args)
    Constructor {
        ty: WgslType,
        args: Vec<Expression>,
    },
    /// Binary operation: left op right
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOp,
        right: Box<Expression>,
    },
    /// Unary operation: op expr
    UnaryOp {
        op: UnaryOp,
        expr: Box<Expression>,
    },
    /// Parenthesized expression
    Grouped(Box<Expression>),
}

/// Literal values in WGSL.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Literal {
    /// Integer literal: 42, -17
    Int(i64),
    /// Unsigned integer literal: 42u
    UInt(u64),
    /// Float literal: 1.0, 2.5
    Float(String), // Store as string to preserve exact representation
    /// Boolean literal: true, false
    Bool(bool),
}

/// Binary operators in WGSL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    Add,        // +
    Subtract,   // -
    Multiply,   // *
    Divide,     // /
    Modulo,     // %
    Equal,      // ==
    NotEqual,   // !=
    Less,       // <
    LessEqual,  // <=
    Greater,    // >
    GreaterEqual, // >=
    And,        // &&
    Or,         // ||
    BitAnd,     // &
    BitOr,      // |
    BitXor,     // ^
    ShiftLeft,  // <<
    ShiftRight, // >>
}

/// Unary operators in WGSL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    Negate,   // -
    Not,      // !
    BitNot,   // ~
    Reference, // &
}

impl Statement {
    /// Convert this statement back to WGSL string representation.
    pub fn to_wgsl(&self, replacements: &HashMap<String, String>, indent: usize) -> String {
        let indent_str = "    ".repeat(indent);
        match self {
            Statement::VarDecl { kind, name, ty, init } => {
                let kind_str = match kind {
                    VarKind::Var => "var",
                    VarKind::Let => "let",
                    VarKind::Const => "const",
                };
                let ty_str = ty.as_ref().map(|t| format!(": {}", t.to_wgsl())).unwrap_or_default();
                let init_str = init.as_ref().map(|e| format!(" = {}", e.to_wgsl(replacements))).unwrap_or_default();
                format!("{}{} {}{}{};\n", indent_str, kind_str, name, ty_str, init_str)
            }
            Statement::Assignment { target, value } => {
                format!("{}{} = {};\n", indent_str, target.to_wgsl(replacements), value.to_wgsl(replacements))
            }
            Statement::Return(expr) => {
                match expr {
                    Some(e) => format!("{}return {};\n", indent_str, e.to_wgsl(replacements)),
                    None => format!("{}return;\n", indent_str),
                }
            }
            Statement::If { condition, then_block, else_block } => {
                let mut result = format!("{}if {} {{\n", indent_str, condition.to_wgsl(replacements));
                for stmt in then_block {
                    result.push_str(&stmt.to_wgsl(replacements, indent + 1));
                }
                result.push_str(&indent_str);
                result.push_str("}");
                if let Some(else_blk) = else_block {
                    if else_blk.len() == 1 && matches!(else_blk[0], Statement::If { .. }) {
                        result.push_str(" else ");
                        result.push_str(&else_blk[0].to_wgsl(replacements, indent).trim_start());
                    } else {
                        result.push_str(" else {\n");
                        for stmt in else_blk {
                            result.push_str(&stmt.to_wgsl(replacements, indent + 1));
                        }
                        result.push_str(&indent_str);
                        result.push_str("}");
                    }
                }
                result.push('\n');
                result
            }
            Statement::For { init, condition, update, body } => {
                let init_str = init.as_ref().map(|s| {
                    let s = s.to_wgsl(replacements, 0);
                    s.trim_end_matches('\n').trim_end_matches(';').to_string()
                }).unwrap_or_default();
                let cond_str = condition.as_ref().map(|e| e.to_wgsl(replacements)).unwrap_or_default();
                let update_str = update.as_ref().map(|e| e.to_wgsl(replacements)).unwrap_or_default();
                let mut result = format!("{}for ({}; {}; {}) {{\n", indent_str, init_str, cond_str, update_str);
                for stmt in body {
                    result.push_str(&stmt.to_wgsl(replacements, indent + 1));
                }
                result.push_str(&indent_str);
                result.push_str("}\n");
                result
            }
            Statement::Loop { body, continuing } => {
                let mut result = format!("{}loop {{\n", indent_str);
                for stmt in body {
                    result.push_str(&stmt.to_wgsl(replacements, indent + 1));
                }
                if let Some(cont_blk) = continuing {
                    result.push_str(&format!("{}continuing {{\n", "    ".repeat(indent + 1)));
                    for stmt in cont_blk {
                        result.push_str(&stmt.to_wgsl(replacements, indent + 2));
                    }
                    result.push_str(&format!("{}}}\n", "    ".repeat(indent + 1)));
                }
                result.push_str(&indent_str);
                result.push_str("}\n");
                result
            }
            Statement::Break => format!("{}break;\n", indent_str),
            Statement::Continue => format!("{}continue;\n", indent_str),
            Statement::ExprStatement(expr) => {
                format!("{}{};\n", indent_str, expr.to_wgsl(replacements))
            }
            Statement::Block(block) => {
                let mut result = format!("{}{{\n", indent_str);
                for stmt in block {
                    result.push_str(&stmt.to_wgsl(replacements, indent + 1));
                }
                result.push_str(&indent_str);
                result.push_str("}\n");
                result
            }
            Statement::PreprocessorPlaceholder(raw) => {
                // Apply replacements if available
                let replaced = replacements.get(raw).cloned().unwrap_or_else(|| raw.clone());
                format!("{}{}\n", indent_str, replaced)
            }
            Statement::Discard => format!("{}discard;\n", indent_str),
        }
    }
}

impl Expression {
    /// Convert this expression back to WGSL string representation.
    pub fn to_wgsl(&self, replacements: &HashMap<String, String>) -> String {
        match self {
            Expression::Literal(lit) => lit.to_wgsl(),
            Expression::Identifier(name) => {
                // Check for preprocessor replacements
                replacements.get(name).cloned().unwrap_or_else(|| name.clone())
            }
            Expression::FieldAccess { base, field } => {
                format!("{}.{}", base.to_wgsl(replacements), field)
            }
            Expression::IndexAccess { base, index } => {
                format!("{}[{}]", base.to_wgsl(replacements), index.to_wgsl(replacements))
            }
            Expression::Call { name, args } => {
                let args_str: Vec<String> = args.iter().map(|a| a.to_wgsl(replacements)).collect();
                format!("{}({})", name, args_str.join(", "))
            }
            Expression::Constructor { ty, args } => {
                let args_str: Vec<String> = args.iter().map(|a| a.to_wgsl(replacements)).collect();
                format!("{}({})", ty.to_wgsl(), args_str.join(", "))
            }
            Expression::BinaryOp { left, op, right } => {
                format!("{} {} {}", left.to_wgsl(replacements), op.to_wgsl(), right.to_wgsl(replacements))
            }
            Expression::UnaryOp { op, expr } => {
                format!("{}{}", op.to_wgsl(), expr.to_wgsl(replacements))
            }
            Expression::Grouped(expr) => {
                format!("({})", expr.to_wgsl(replacements))
            }
        }
    }
}

impl Literal {
    /// Convert this literal to WGSL string representation.
    pub fn to_wgsl(&self) -> String {
        match self {
            Literal::Int(n) => n.to_string(),
            Literal::UInt(n) => format!("{}u", n),
            Literal::Float(f) => f.clone(),
            Literal::Bool(b) => b.to_string(),
        }
    }
}

impl BinaryOp {
    /// Convert this operator to WGSL string representation.
    pub fn to_wgsl(&self) -> &'static str {
        match self {
            BinaryOp::Add => "+",
            BinaryOp::Subtract => "-",
            BinaryOp::Multiply => "*",
            BinaryOp::Divide => "/",
            BinaryOp::Modulo => "%",
            BinaryOp::Equal => "==",
            BinaryOp::NotEqual => "!=",
            BinaryOp::Less => "<",
            BinaryOp::LessEqual => "<=",
            BinaryOp::Greater => ">",
            BinaryOp::GreaterEqual => ">=",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
            BinaryOp::BitAnd => "&",
            BinaryOp::BitOr => "|",
            BinaryOp::BitXor => "^",
            BinaryOp::ShiftLeft => "<<",
            BinaryOp::ShiftRight => ">>",
        }
    }
}

impl UnaryOp {
    /// Convert this operator to WGSL string representation.
    pub fn to_wgsl(&self) -> &'static str {
        match self {
            UnaryOp::Negate => "-",
            UnaryOp::Not => "!",
            UnaryOp::BitNot => "~",
            UnaryOp::Reference => "&",
        }
    }
}

/// Convert a block of statements to WGSL string representation.
pub fn block_to_wgsl(block: &Block, replacements: &HashMap<String, String>) -> String {
    let mut result = String::from("{\n");
    for stmt in block {
        result.push_str(&stmt.to_wgsl(replacements, 1));
    }
    result.push_str("}\n");
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal_to_wgsl() {
        assert_eq!(Literal::Int(42).to_wgsl(), "42");
        assert_eq!(Literal::Int(-17).to_wgsl(), "-17");
        assert_eq!(Literal::UInt(100).to_wgsl(), "100u");
        assert_eq!(Literal::Float("1.5".to_string()).to_wgsl(), "1.5");
        assert_eq!(Literal::Bool(true).to_wgsl(), "true");
    }

    #[test]
    fn test_simple_statement() {
        let stmt = Statement::Return(Some(Expression::Identifier("x".to_string())));
        let result = stmt.to_wgsl(&HashMap::new(), 0);
        assert_eq!(result, "return x;\n");
    }

    #[test]
    fn test_var_decl() {
        let stmt = Statement::VarDecl {
            kind: VarKind::Var,
            name: "x".to_string(),
            ty: Some(WgslType::Primitive("f32".to_string())),
            init: Some(Expression::Literal(Literal::Float("1.0".to_string()))),
        };
        let result = stmt.to_wgsl(&HashMap::new(), 0);
        assert_eq!(result, "var x: f32 = 1.0;\n");
    }

    #[test]
    fn test_if_statement() {
        let stmt = Statement::If {
            condition: Expression::Identifier("flag".to_string()),
            then_block: vec![Statement::Return(Some(Expression::Identifier("x".to_string())))],
            else_block: Some(vec![Statement::Return(Some(Expression::Identifier("y".to_string())))]),
        };
        let result = stmt.to_wgsl(&HashMap::new(), 0);
        assert!(result.contains("if flag {"));
        assert!(result.contains("return x;"));
        assert!(result.contains("else {"));
        assert!(result.contains("return y;"));
    }

    #[test]
    fn test_expression() {
        let expr = Expression::BinaryOp {
            left: Box::new(Expression::Identifier("a".to_string())),
            op: BinaryOp::Add,
            right: Box::new(Expression::Identifier("b".to_string())),
        };
        assert_eq!(expr.to_wgsl(&HashMap::new()), "a + b");
    }
}