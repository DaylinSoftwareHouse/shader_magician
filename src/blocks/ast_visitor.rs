// ast_visitor.rs  –  walk and query the parsed WGSL AST.
//
// Two layers:
//   1. `Visitor` trait  –  override only the methods you care about.
//   2. Free functions   –  common one-shot queries (find calls, collect idents, …)

use crate::wgsl_ast::*;

// ── Visitor trait ─────────────────────────────────────────────────────────────

/// Implement this trait to walk any part of the AST.
/// Every `visit_*` method has a default no-op body so you only override
/// what you need.  The companion `walk_*` free functions drive child traversal;
/// call them from your override when you want to recurse.
#[allow(unused_variables)]
pub trait Visitor: Sized {
    fn visit_block(&mut self, block: &Block) { walk_block(self, block); }
    fn visit_statement(&mut self, stmt: &Statement) { walk_statement(self, stmt); }
    fn visit_expression(&mut self, expr: &Expression) { walk_expression(self, expr); }
    fn visit_var_decl(&mut self, decl: &VarDecl) { walk_var_decl(self, decl); }
    fn visit_if(&mut self, stmt: &IfStatement) { walk_if(self, stmt); }
    fn visit_switch(&mut self, stmt: &SwitchStatement) { walk_switch(self, stmt); }
    fn visit_loop(&mut self, stmt: &LoopStatement) { walk_loop(self, stmt); }
    fn visit_for(&mut self, stmt: &ForStatement) { walk_for(self, stmt); }
    fn visit_while(&mut self, stmt: &WhileStatement) { walk_while(self, stmt); }
    fn visit_lvalue(&mut self, lval: &LValue) { walk_lvalue(self, lval); }
}

// ── Walk functions ────────────────────────────────────────────────────────────

pub fn walk_block<V: Visitor>(v: &mut V, block: &Block) {
    for stmt in &block.stmts {
        v.visit_statement(stmt);
    }
}

pub fn walk_statement<V: Visitor>(v: &mut V, stmt: &Statement) {
    match stmt {
        Statement::VarDecl(d)       => v.visit_var_decl(d),
        Statement::Assign(a)        => {
            v.visit_lvalue(&a.target);
            v.visit_expression(&a.value);
        }
        Statement::Increment(lv, _) => v.visit_lvalue(lv),
        Statement::Return(Some(e))  => v.visit_expression(e),
        Statement::Return(None)     => {}
        Statement::BreakIf(e)       => v.visit_expression(e),
        Statement::If(s)            => v.visit_if(s),
        Statement::Switch(s)        => v.visit_switch(s),
        Statement::Loop(s)          => v.visit_loop(s),
        Statement::For(s)           => v.visit_for(s),
        Statement::While(s)         => v.visit_while(s),
        Statement::Expression(e)    => v.visit_expression(e),
        Statement::Block(b)         => v.visit_block(b),
        Statement::Discard | Statement::Break | Statement::Continue => {}
    }
}

pub fn walk_expression<V: Visitor>(v: &mut V, expr: &Expression) {
    match expr {
        Expression::Unary(_, e)        => v.visit_expression(e),
        Expression::Binary(l, _, r)    => { v.visit_expression(l); v.visit_expression(r); }
        Expression::Call(f, args)      => { v.visit_expression(f); args.iter().for_each(|a| v.visit_expression(a)); }
        Expression::Index(b, i)        => { v.visit_expression(b); v.visit_expression(i); }
        Expression::Field(b, _)        => v.visit_expression(b),
        Expression::Deref(e)           => v.visit_expression(e),
        Expression::AddrOf(e)          => v.visit_expression(e),
        Expression::TypeConstruct(_, a)=> a.iter().for_each(|a| v.visit_expression(a)),
        // leaf nodes
        Expression::IntLiteral(_)
        | Expression::FloatLiteral(_)
        | Expression::BoolLiteral(_)
        | Expression::Identifier(_)    => {}
    }
}

pub fn walk_var_decl<V: Visitor>(v: &mut V, decl: &VarDecl) {
    if let Some(init) = &decl.initializer { v.visit_expression(init); }
}

pub fn walk_if<V: Visitor>(v: &mut V, stmt: &IfStatement) {
    v.visit_expression(&stmt.condition);
    v.visit_block(&stmt.body);
    for (cond, blk) in &stmt.else_ifs {
        v.visit_expression(cond);
        v.visit_block(blk);
    }
    if let Some(b) = &stmt.else_body { v.visit_block(b); }
}

pub fn walk_switch<V: Visitor>(v: &mut V, stmt: &SwitchStatement) {
    v.visit_expression(&stmt.selector);
    for case in &stmt.cases {
        for sel in &case.selectors {
            if let Some(e) = sel { v.visit_expression(e); }
        }
        v.visit_block(&case.body);
    }
}

pub fn walk_loop<V: Visitor>(v: &mut V, stmt: &LoopStatement) {
    v.visit_block(&stmt.body);
    if let Some(c) = &stmt.continuing { v.visit_block(c); }
}

pub fn walk_for<V: Visitor>(v: &mut V, stmt: &ForStatement) {
    if let Some(init) = &stmt.init   { v.visit_statement(init); }
    if let Some(cond) = &stmt.condition { v.visit_expression(cond); }
    if let Some(upd)  = &stmt.update { v.visit_statement(upd); }
    v.visit_block(&stmt.body);
}

pub fn walk_while<V: Visitor>(v: &mut V, stmt: &WhileStatement) {
    v.visit_expression(&stmt.condition);
    v.visit_block(&stmt.body);
}

pub fn walk_lvalue<V: Visitor>(v: &mut V, lval: &LValue) {
    match lval {
        LValue::Index(b, idx)  => { walk_lvalue(v, b); v.visit_expression(idx); }
        LValue::Field(b, _)    => walk_lvalue(v, b),
        LValue::Deref(b)       => walk_lvalue(v, b),
        LValue::Ident(_) | LValue::Phony => {}
    }
}

// ── Mutable visitor ───────────────────────────────────────────────────────────

/// Like `Visitor` but receives `&mut` references — use for transforms.
#[allow(unused_variables)]
pub trait VisitorMut: Sized {
    fn visit_block(&mut self, block: &mut Block) { walk_block_mut(self, block); }
    fn visit_statement(&mut self, stmt: &mut Statement) { walk_statement_mut(self, stmt); }
    fn visit_expression(&mut self, expr: &mut Expression) { walk_expression_mut(self, expr); }
    fn visit_var_decl(&mut self, decl: &mut VarDecl) { walk_var_decl_mut(self, decl); }
}

pub fn walk_block_mut<V: VisitorMut>(v: &mut V, block: &mut Block) {
    for stmt in &mut block.stmts { v.visit_statement(stmt); }
}

pub fn walk_statement_mut<V: VisitorMut>(v: &mut V, stmt: &mut Statement) {
    match stmt {
        Statement::VarDecl(d)       => v.visit_var_decl(d),
        Statement::Assign(a)        => v.visit_expression(&mut a.value),
        Statement::Return(Some(e))  => v.visit_expression(e),
        Statement::BreakIf(e)       => v.visit_expression(e),
        Statement::If(s)            => {
            v.visit_expression(&mut s.condition);
            v.visit_block(&mut s.body);
            for (c, b) in &mut s.else_ifs { v.visit_expression(c); v.visit_block(b); }
            if let Some(b) = &mut s.else_body { v.visit_block(b); }
        }
        Statement::Switch(s) => {
            v.visit_expression(&mut s.selector);
            for case in &mut s.cases {
                for sel in &mut case.selectors {
                    if let Some(e) = sel { v.visit_expression(e); }
                }
                v.visit_block(&mut case.body);
            }
        }
        Statement::Loop(s)  => {
            v.visit_block(&mut s.body);
            if let Some(c) = &mut s.continuing { v.visit_block(c); }
        }
        Statement::For(s)   => {
            if let Some(i) = &mut s.init      { v.visit_statement(i); }
            if let Some(c) = &mut s.condition { v.visit_expression(c); }
            if let Some(u) = &mut s.update    { v.visit_statement(u); }
            v.visit_block(&mut s.body);
        }
        Statement::While(s) => {
            v.visit_expression(&mut s.condition);
            v.visit_block(&mut s.body);
        }
        Statement::Expression(e) => v.visit_expression(e),
        Statement::Block(b)      => v.visit_block(b),
        _ => {}
    }
}

pub fn walk_expression_mut<V: VisitorMut>(v: &mut V, expr: &mut Expression) {
    match expr {
        Expression::Unary(_, e)         => v.visit_expression(e),
        Expression::Binary(l, _, r)     => { v.visit_expression(l); v.visit_expression(r); }
        Expression::Call(f, args)       => { v.visit_expression(f); for a in args { v.visit_expression(a); } }
        Expression::Index(b, i)         => { v.visit_expression(b); v.visit_expression(i); }
        Expression::Field(b, _)         => v.visit_expression(b),
        Expression::Deref(e)            => v.visit_expression(e),
        Expression::AddrOf(e)           => v.visit_expression(e),
        Expression::TypeConstruct(_, a) => for x in a { v.visit_expression(x); },
        _ => {}
    }
}

pub fn walk_var_decl_mut<V: VisitorMut>(v: &mut V, decl: &mut VarDecl) {
    if let Some(i) = &mut decl.initializer { v.visit_expression(i); }
}

// ── High-level query helpers ──────────────────────────────────────────────────

/// Collect all function-call sites in a block: `(callee_name, arg_count)`.
pub fn collect_calls(block: &Block) -> Vec<(String, usize)> {
    struct CallCollector(Vec<(String, usize)>);
    impl Visitor for CallCollector {
        fn visit_expression(&mut self, expr: &Expression) {
            // if let Expression::Call(f, args) | Expression::TypeConstruct(_, _) = expr {
                if let Expression::Call(f, args) = expr {
                    let name = callee_name(f);
                    self.0.push((name, args.len()));
                } else if let Expression::TypeConstruct(f, args) = expr {
                    self.0.push((f.clone(), args.len()));
                }
            // }
            walk_expression(self, expr);
        }
    }
    let mut v = CallCollector(Vec::new());
    v.visit_block(block);
    v.0
}

fn callee_name(expr: &Expression) -> String {
    match expr {
        Expression::Identifier(s) => s.clone(),
        Expression::Field(_, f)   => f.clone(),
        other => format!("{other:?}"),
    }
}

/// Collect every identifier referenced in a block.
pub fn collect_identifiers(block: &Block) -> Vec<String> {
    struct IdentCollector(Vec<String>);
    impl Visitor for IdentCollector {
        fn visit_expression(&mut self, expr: &Expression) {
            if let Expression::Identifier(s) = expr { self.0.push(s.clone()); }
            walk_expression(self, expr);
        }
    }
    let mut v = IdentCollector(Vec::new());
    v.visit_block(block);
    v.0
}

/// Collect all local variable names declared with `var`/`let`/`const`.
pub fn collect_local_vars(block: &Block) -> Vec<VarDecl> {
    struct VarCollector(Vec<VarDecl>);
    impl Visitor for VarCollector {
        fn visit_var_decl(&mut self, d: &VarDecl) { self.0.push(d.clone()); }
    }
    let mut v = VarCollector(Vec::new());
    v.visit_block(block);
    v.0
}

/// Return `true` if the block contains any call to `fn_name`.
pub fn calls_function(block: &Block, fn_name: &str) -> bool {
    collect_calls(block).iter().any(|(n, _)| n == fn_name)
}

/// Rename all occurrences of identifier `from` → `to` throughout a block.
/// Touches both expression positions and lvalue positions.
pub fn rename_identifier(block: &mut Block, from: &str, to: &str) {
    struct Renamer<'a> { from: &'a str, to: &'a str }
    impl VisitorMut for Renamer<'_> {
        fn visit_expression(&mut self, expr: &mut Expression) {
            if let Expression::Identifier(s) = expr {
                if s == self.from { *s = self.to.to_string(); }
            }
            walk_expression_mut(self, expr);
        }
    }
    Renamer { from, to }.visit_block(block);
}

/// Pretty-print a block back to WGSL source (round-trip).
pub fn emit_block(block: &Block, indent: usize) -> String {
    let mut out = String::new();
    emit_block_inner(block, indent, &mut out);
    out
}

fn pad(n: usize) -> String { "    ".repeat(n) }

fn emit_block_inner(block: &Block, indent: usize, out: &mut String) {
    out.push_str("{\n");
    for stmt in &block.stmts {
        emit_stmt(stmt, indent + 1, out);
    }
    out.push_str(&pad(indent));
    out.push('}');
}

fn emit_stmt(stmt: &Statement, indent: usize, out: &mut String) {
    out.push_str(&pad(indent));
    match stmt {
        Statement::VarDecl(d) => {
            let kw = match d.kind { VarKind::Var => "var", VarKind::Let => "let", VarKind::Const => "const" };
            out.push_str(kw);
            if let Some(t) = &d.template_args { out.push('<'); out.push_str(t); out.push('>'); }
            out.push(' ');
            out.push_str(&d.name);
            if let Some(ty) = &d.ty { out.push_str(": "); out.push_str(ty); }
            if let Some(init) = &d.initializer { out.push_str(" = "); out.push_str(&emit_expr(init)); }
            out.push_str(";\n");
        }
        Statement::Assign(a) => {
            out.push_str(&emit_lvalue(&a.target));
            out.push(' ');
            out.push_str(assign_op_str(&a.op));
            out.push(' ');
            out.push_str(&emit_expr(&a.value));
            out.push_str(";\n");
        }
        Statement::Increment(lv, op) => {
            out.push_str(&emit_lvalue(lv));
            out.push_str(match op { IncrOp::Inc => "++", IncrOp::Dec => "--" });
            out.push_str(";\n");
        }
        Statement::Return(None)    => out.push_str("return;\n"),
        Statement::Return(Some(e)) => { out.push_str("return "); out.push_str(&emit_expr(e)); out.push_str(";\n"); }
        Statement::Discard  => out.push_str("discard;\n"),
        Statement::Break    => out.push_str("break;\n"),
        Statement::Continue => out.push_str("continue;\n"),
        Statement::BreakIf(e) => { out.push_str("break if "); out.push_str(&emit_expr(e)); out.push_str(";\n"); }
        Statement::If(s) => {
            out.push_str("if ");
            out.push_str(&emit_expr(&s.condition));
            out.push(' ');
            emit_block_inner(&s.body, indent, out);
            for (cond, blk) in &s.else_ifs {
                out.push_str(" else if ");
                out.push_str(&emit_expr(cond));
                out.push(' ');
                emit_block_inner(blk, indent, out);
            }
            if let Some(b) = &s.else_body {
                out.push_str(" else ");
                emit_block_inner(b, indent, out);
            }
            out.push('\n');
        }
        Statement::Switch(s) => {
            out.push_str("switch ");
            out.push_str(&emit_expr(&s.selector));
            out.push_str(" {\n");
            for case in &s.cases {
                for sel in &case.selectors {
                    out.push_str(&pad(indent + 1));
                    match sel {
                        None    => out.push_str("default"),
                        Some(e) => { out.push_str("case "); out.push_str(&emit_expr(e)); }
                    }
                    out.push_str(": ");
                }
                emit_block_inner(&case.body, indent + 1, out);
                out.push('\n');
            }
            out.push_str(&pad(indent));
            out.push_str("}\n");
        }
        Statement::Loop(s) => {
            out.push_str("loop ");
            emit_block_inner(&s.body, indent, out);
            if let Some(c) = &s.continuing {
                out.push_str(" continuing ");
                emit_block_inner(c, indent, out);
            }
            out.push('\n');
        }
        Statement::For(s) => {
            out.push_str("for (");
            if let Some(i) = &s.init {
                let mut tmp = String::new();
                emit_stmt(i, 0, &mut tmp);
                out.push_str(tmp.trim());
            }
            out.push_str("; ");
            if let Some(c) = &s.condition { out.push_str(&emit_expr(c)); }
            out.push_str("; ");
            if let Some(u) = &s.update {
                let mut tmp = String::new();
                emit_stmt(u, 0, &mut tmp);
                let trimmed = tmp.trim().trim_end_matches(';');
                out.push_str(trimmed);
            }
            out.push_str(") ");
            emit_block_inner(&s.body, indent, out);
            out.push('\n');
        }
        Statement::While(s) => {
            out.push_str("while ");
            out.push_str(&emit_expr(&s.condition));
            out.push(' ');
            emit_block_inner(&s.body, indent, out);
            out.push('\n');
        }
        Statement::Expression(e) => { out.push_str(&emit_expr(e)); out.push_str(";\n"); }
        Statement::Block(b) => { emit_block_inner(b, indent, out); out.push('\n'); }
    }
}

fn emit_expr(expr: &Expression) -> String {
    match expr {
        Expression::IntLiteral(s)   => s.clone(),
        Expression::FloatLiteral(s) => s.clone(),
        Expression::BoolLiteral(b)  => b.to_string(),
        Expression::Identifier(s)   => s.clone(),
        Expression::Unary(op, e) => {
            let op_s = match op { UnaryOp::Neg => "-", UnaryOp::Not => "!", UnaryOp::BitNot => "~", UnaryOp::Deref => "*", UnaryOp::AddrOf => "&" };
            format!("{}{}", op_s, emit_expr(e))
        }
        Expression::Binary(l, op, r) => {
            format!("({} {} {})", emit_expr(l), binary_op_str(*op), emit_expr(r))
        }
        Expression::Call(f, args) => {
            let arg_str: Vec<_> = args.iter().map(emit_expr).collect();
            format!("{}({})", emit_expr(f), arg_str.join(", "))
        }
        Expression::Index(b, i) => format!("{}[{}]", emit_expr(b), emit_expr(i)),
        Expression::Field(b, f) => format!("{}.{}", emit_expr(b), f),
        Expression::Deref(e)    => format!("(*{})", emit_expr(e)),
        Expression::AddrOf(e)   => format!("(&{})", emit_expr(e)),
        Expression::TypeConstruct(ty, args) => {
            let arg_str: Vec<_> = args.iter().map(emit_expr).collect();
            format!("{}({})", ty, arg_str.join(", "))
        }
    }
}

fn emit_lvalue(lv: &LValue) -> String {
    match lv {
        LValue::Ident(s)     => s.clone(),
        LValue::Phony        => "_".to_string(),
        LValue::Index(b, i)  => format!("{}[{}]", emit_lvalue(b), emit_expr(i)),
        LValue::Field(b, f)  => format!("{}.{}", emit_lvalue(b), f),
        LValue::Deref(b)     => format!("(*{})", emit_lvalue(b)),
    }
}

fn assign_op_str(op: &AssignOp) -> &'static str {
    match op {
        AssignOp::Simple => "=",  AssignOp::Add => "+=", AssignOp::Sub => "-=",
        AssignOp::Mul    => "*=", AssignOp::Div => "/=", AssignOp::Mod => "%=",
        AssignOp::And    => "&=", AssignOp::Or  => "|=", AssignOp::Xor => "^=",
        AssignOp::Shl    => "<<=",AssignOp::Shr => ">>=",
    }
}

fn binary_op_str(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+", BinaryOp::Sub => "-", BinaryOp::Mul => "*",
        BinaryOp::Div => "/", BinaryOp::Mod => "%",
        BinaryOp::BitAnd => "&", BinaryOp::BitOr => "|", BinaryOp::BitXor => "^",
        BinaryOp::Shl => "<<", BinaryOp::Shr => ">>",
        BinaryOp::And => "&&", BinaryOp::Or  => "||",
        BinaryOp::Eq  => "==", BinaryOp::Ne  => "!=",
        BinaryOp::Lt  => "<",  BinaryOp::Le  => "<=",
        BinaryOp::Gt  => ">",  BinaryOp::Ge  => ">=",
    }
}
