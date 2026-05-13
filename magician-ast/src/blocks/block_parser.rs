// block_parser.rs  –  parses WGSL block bodies into a typed AST.
//
// Meant to be called from your existing `Parser` after it has consumed
// the opening `{` of a function body (or any compound statement).
//
// Usage:
//   let block = BlockParser::new(raw_block_src).parse_block()?;

use std::error::Error;
use crate::{ShaderPreProcessorError, wgsl_ast::*};

pub struct BlockParser {
    input: Vec<char>,
    pos: usize,
}

// ── Public entry point ────────────────────────────────────────────────────────

impl BlockParser {
    /// `src` should be the raw block string **including** the surrounding `{ }`.
    pub fn new(src: &str) -> Self {
        Self { input: src.chars().collect(), pos: 0 }
    }

    /// Top-level: expects the entire input to be a single `{ … }` block.
    pub fn parse_block(&mut self) -> Result<Block, Box<dyn Error>> {
        self.skip_ws();
        self.expect('{')?;
        let block = self.parse_block_body()?;
        self.skip_ws();
        Ok(block)
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

impl BlockParser {
    // ── char-level ────────────────────────────────────────────────────────────

    fn peek(&self) -> Option<char> { self.input.get(self.pos).copied() }

    fn peek2(&self) -> Option<char> { self.input.get(self.pos + 1).copied() }

    fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += 1;
        Some(c)
    }

    fn expect(&mut self, ch: char) -> Result<(), Box<dyn Error>> {
        self.skip_ws();
        match self.advance() {
            Some(c) if c == ch => Ok(()),
            Some(c) => Err(self.err(&format!("expected '{ch}', got '{c}'"))),
            None    => Err(self.err(&format!("expected '{ch}', got EOF"))),
        }
    }

    fn err(&self, msg: &str) -> Box<dyn Error> {
        Box::new(ShaderPreProcessorError::ParseError(msg.to_string()))
    }

    // ── whitespace / comments ─────────────────────────────────────────────────

    fn skip_ws(&mut self) {
        loop {
            // whitespace
            while self.peek().map_or(false, |c| c.is_whitespace()) { self.advance(); }
            // line comment
            if self.peek() == Some('/') && self.peek2() == Some('/') {
                while self.peek().is_some() && self.peek() != Some('\n') { self.advance(); }
                continue;
            }
            // block comment
            if self.peek() == Some('/') && self.peek2() == Some('*') {
                self.advance(); self.advance();
                loop {
                    match self.advance() {
                        None => break,
                        Some('*') if self.peek() == Some('/') => { self.advance(); break; }
                        _ => {}
                    }
                }
                continue;
            }
            break;
        }
    }

    // ── identifier / keyword ──────────────────────────────────────────────────

    fn peek_word(&self) -> String {
        let mut p = self.pos;
        let mut s = String::new();
        while let Some(&c) = self.input.get(p) {
            if c.is_alphanumeric() || c == '_' { s.push(c); p += 1; }
            else { break; }
        }
        s
    }

    fn consume_ident(&mut self) -> Result<String, Box<dyn Error>> {
        let s = self.peek_word();
        if s.is_empty() {
            return Err(self.err(&format!("expected identifier, got {:?}", self.peek())));
        }
        self.pos += s.len();
        Ok(s)
    }

    /// Consume a WGSL type (may include `<…>` template args).
    fn consume_type(&mut self) -> Result<String, Box<dyn Error>> {
        let name = self.consume_ident()?;
        self.skip_ws();
        if self.peek() == Some('<') {
            let mut depth = 0usize;
            let mut raw = name;
            loop {
                match self.advance() {
                    None => return Err(self.err("unexpected EOF in type")),
                    Some('<') => { depth += 1; raw.push('<'); }
                    Some('>') => {
                        raw.push('>');
                        depth -= 1;
                        if depth == 0 { break; }
                    }
                    Some(c) => raw.push(c),
                }
            }
            Ok(raw)
        } else {
            Ok(name)
        }
    }

    /// Read a multi-character operator token, e.g. `+=`, `<<=`, `&&`, …
    fn consume_op_token(&mut self) -> String {
        let mut s = String::new();
        // Grab up to 3 chars that belong to operators
        for _ in 0..3 {
            match self.peek() {
                Some(c @ ('+' | '-' | '*' | '/' | '%' | '&' | '|' | '^'
                        | '!' | '~' | '<' | '>' | '=' | '?'))
                    => { s.push(c); self.advance(); }
                _ => break,
            }
        }
        s
    }

    /// Peek-check if the next non-whitespace chars are an assignment operator.
    fn peek_assign_op(&self) -> Option<AssignOp> {
        // Scan past any whitespace  (we're peeking, not mutating)
        let mut p = self.pos;
        while self.input.get(p).map_or(false, |c| c.is_whitespace()) { p += 1; }

        let a = self.input.get(p).copied();
        let b = self.input.get(p + 1).copied();
        let c = self.input.get(p + 2).copied();

        // Three-char: <<= >>=
        if a == Some('<') && b == Some('<') && c == Some('=') { return Some(AssignOp::Shl); }
        if a == Some('>') && b == Some('>') && c == Some('=') { return Some(AssignOp::Shr); }
        // Two-char
        let two: Option<AssignOp> = match (a, b) {
            (Some('+'), Some('=')) => Some(AssignOp::Add),
            (Some('-'), Some('=')) => Some(AssignOp::Sub),
            (Some('*'), Some('=')) => Some(AssignOp::Mul),
            (Some('/'), Some('=')) => Some(AssignOp::Div),
            (Some('%'), Some('=')) => Some(AssignOp::Mod),
            (Some('&'), Some('=')) => Some(AssignOp::And),
            (Some('|'), Some('=')) => Some(AssignOp::Or),
            (Some('^'), Some('=')) => Some(AssignOp::Xor),
            _ => None,
        };
        if two.is_some() { return two; }
        // Single `=` but not `==`
        if a == Some('=') && b != Some('=') { return Some(AssignOp::Simple); }
        None
    }

    fn consume_assign_op(&mut self) -> AssignOp {
        self.skip_ws();
        let op_str = self.consume_op_token();
        AssignOp::from_str(&op_str).expect("consume_assign_op called without peeking first")
    }

    // ── semicolon ─────────────────────────────────────────────────────────────

    fn expect_semi(&mut self) -> Result<(), Box<dyn Error>> {
        self.skip_ws();
        if self.peek() == Some(';') { self.advance(); Ok(()) }
        else { Err(self.err(&format!("expected ';', got {:?}", self.peek()))) }
    }

    // ── block body (after `{` has been consumed) ──────────────────────────────

    fn parse_block_body(&mut self) -> Result<Block, Box<dyn Error>> {
        let mut stmts = Vec::new();
        loop {
            self.skip_ws();
            if self.peek() == Some('}') { self.advance(); break; }
            if self.peek().is_none() { return Err(self.err("unexpected EOF in block")); }
            stmts.push(self.parse_statement()?);
        }
        Ok(Block { stmts })
    }

    fn parse_nested_block(&mut self) -> Result<Block, Box<dyn Error>> {
        self.skip_ws();
        self.expect('{')?;
        self.parse_block_body()
    }
}

// ── Statement parsing ─────────────────────────────────────────────────────────

impl BlockParser {
    fn parse_statement(&mut self) -> Result<Statement, Box<dyn Error>> {
        self.skip_ws();

        // Bare block
        if self.peek() == Some('{') {
            self.advance();
            let block = self.parse_block_body()?;
            return Ok(Statement::Block(block));
        }

        let word = self.peek_word();

        match word.as_str() {
            "var" | "let" | "const" => {
                let stmt = self.parse_var_decl()?;
                self.expect_semi()?;
                Ok(Statement::VarDecl(stmt))
            }
            "return" => {
                self.pos += "return".len();
                self.skip_ws();
                if self.peek() == Some(';') {
                    self.advance();
                    Ok(Statement::Return(None))
                } else {
                    let expr = self.parse_expression(0)?;
                    self.expect_semi()?;
                    Ok(Statement::Return(Some(expr)))
                }
            }
            "discard" => { self.pos += "discard".len(); self.expect_semi()?; Ok(Statement::Discard) }
            "break"   => {
                self.pos += "break".len();
                self.skip_ws();
                if self.peek_word() == "if" {
                    self.pos += "if".len();
                    self.skip_ws();
                    let cond = self.parse_expression(0)?;
                    self.expect_semi()?;
                    Ok(Statement::BreakIf(cond))
                } else {
                    self.expect_semi()?;
                    Ok(Statement::Break)
                }
            }
            "continue" => { self.pos += "continue".len(); self.expect_semi()?; Ok(Statement::Continue) }
            "if"       => self.parse_if(),
            "switch"   => self.parse_switch(),
            "loop"     => self.parse_loop(),
            "for"      => self.parse_for(),
            "while"    => self.parse_while(),
            _ => self.parse_assign_or_expr_stmt(),
        }
    }

    // ── var / let / const ─────────────────────────────────────────────────────

    fn parse_var_decl(&mut self) -> Result<VarDecl, Box<dyn Error>> {
        let kind_str = self.consume_ident()?;
        let kind = match kind_str.as_str() {
            "var"   => VarKind::Var,
            "let"   => VarKind::Let,
            "const" => VarKind::Const,
            other   => return Err(self.err(&format!("unknown var kind: {other}"))),
        };
        self.skip_ws();

        // Optional template args: var<uniform>
        let template_args = if self.peek() == Some('<') {
            self.advance();
            let mut inner = String::new();
            let mut depth = 1usize;
            loop {
                match self.advance() {
                    None => return Err(self.err("unexpected EOF in var template args")),
                    Some('<') => { depth += 1; inner.push('<'); }
                    Some('>') => {
                        depth -= 1;
                        if depth == 0 { break; }
                        inner.push('>');
                    }
                    Some(c) => inner.push(c),
                }
            }
            Some(inner)
        } else {
            None
        };

        self.skip_ws();
        let name = self.consume_ident()?;
        self.skip_ws();

        let ty = if self.peek() == Some(':') {
            self.advance();
            self.skip_ws();
            Some(self.consume_type()?)
        } else {
            None
        };

        self.skip_ws();
        let initializer = if self.peek() == Some('=') {
            self.advance();
            self.skip_ws();
            Some(self.parse_expression(0)?)
        } else {
            None
        };

        Ok(VarDecl { kind, template_args, name, ty, initializer })
    }

    // ── assignment or expression statement ────────────────────────────────────

    fn parse_assign_or_expr_stmt(&mut self) -> Result<Statement, Box<dyn Error>> {
        // Parse a primary expression — this covers identifiers with field/index
        // accesses, which is also what an lvalue can look like.
        let expr = self.parse_expression(0)?;

        self.skip_ws();

        // Check for ++ / --
        if self.peek() == Some('+') && self.peek2() == Some('+') {
            self.advance(); self.advance();
            self.expect_semi()?;
            let lval = expr_to_lvalue(expr)
                .ok_or_else(|| self.err("invalid lvalue for ++"))?;
            return Ok(Statement::Increment(lval, IncrOp::Inc));
        }
        if self.peek() == Some('-') && self.peek2() == Some('-') {
            self.advance(); self.advance();
            self.expect_semi()?;
            let lval = expr_to_lvalue(expr)
                .ok_or_else(|| self.err("invalid lvalue for --"))?;
            return Ok(Statement::Increment(lval, IncrOp::Dec));
        }

        // Check for assignment operators
        if let Some(op) = self.peek_assign_op() {
            // Only assign if expr is a valid lvalue
            if let Some(lval) = expr_to_lvalue(expr.clone()) {
                let _ = self.consume_assign_op(); // advance past the op
                self.skip_ws();
                // If `op` came from `expr_to_lvalue` on something like `_`, handle phony:
                let value = self.parse_expression(0)?;
                self.expect_semi()?;
                return Ok(Statement::Assign(AssignStatement { target: lval, op, value }));
            }
        }

        // Phony assignment: `_ = expr;`
        if let Expression::Identifier(ref s) = expr {
            if s == "_" {
                if let Some(op) = self.peek_assign_op() {
                    let _ = self.consume_assign_op();
                    self.skip_ws();
                    let value = self.parse_expression(0)?;
                    self.expect_semi()?;
                    return Ok(Statement::Assign(AssignStatement {
                        target: LValue::Phony,
                        op,
                        value,
                    }));
                }
            }
        }

        // Pure expression statement (e.g. a function call).
        self.expect_semi()?;
        Ok(Statement::Expression(expr))
    }

    // ── if ────────────────────────────────────────────────────────────────────

    fn parse_if(&mut self) -> Result<Statement, Box<dyn Error>> {
        self.pos += "if".len();
        self.skip_ws();
        let condition = self.parse_expression(0)?;
        let body = self.parse_nested_block()?;

        let mut else_ifs = Vec::new();
        let mut else_body = None;

        loop {
            self.skip_ws();
            if self.peek_word() != "else" { break; }
            self.pos += "else".len();
            self.skip_ws();
            if self.peek_word() == "if" {
                self.pos += "if".len();
                self.skip_ws();
                let cond = self.parse_expression(0)?;
                let blk  = self.parse_nested_block()?;
                else_ifs.push((cond, blk));
            } else {
                else_body = Some(self.parse_nested_block()?);
                break;
            }
        }

        Ok(Statement::If(IfStatement { condition, body, else_ifs, else_body }))
    }

    // ── switch ────────────────────────────────────────────────────────────────

    fn parse_switch(&mut self) -> Result<Statement, Box<dyn Error>> {
        self.pos += "switch".len();
        self.skip_ws();
        let selector = self.parse_expression(0)?;
        self.skip_ws();
        self.expect('{')?;

        let mut cases = Vec::new();
        loop {
            self.skip_ws();
            if self.peek() == Some('}') { self.advance(); break; }

            let mut selectors: Vec<Option<Expression>> = Vec::new();

            // case X, Y: { … }   or   default { … }
            let word = self.peek_word();
            if word == "default" {
                self.pos += "default".len();
                self.skip_ws();
                // optional colon
                if self.peek() == Some(':') { self.advance(); }
                selectors.push(None);
            } else if word == "case" {
                self.pos += "case".len();
                self.skip_ws();
                loop {
                    self.skip_ws();
                    if self.peek_word() == "default" {
                        self.pos += "default".len();
                        selectors.push(None);
                    } else {
                        selectors.push(Some(self.parse_expression(0)?));
                    }
                    self.skip_ws();
                    if self.peek() == Some(',') { self.advance(); } else { break; }
                }
                self.skip_ws();
                if self.peek() == Some(':') { self.advance(); }
            } else {
                return Err(self.err(&format!("expected 'case' or 'default', got '{word}'")));
            }

            let body = self.parse_nested_block()?;
            cases.push(SwitchCase { selectors, body });
        }

        Ok(Statement::Switch(SwitchStatement { selector, cases }))
    }

    // ── loop ──────────────────────────────────────────────────────────────────

    fn parse_loop(&mut self) -> Result<Statement, Box<dyn Error>> {
        self.pos += "loop".len();
        self.skip_ws();
        self.expect('{')?;

        let mut stmts = Vec::new();
        let mut continuing: Option<Block> = None;

        loop {
            self.skip_ws();
            if self.peek() == Some('}') { self.advance(); break; }
            if self.peek_word() == "continuing" {
                self.pos += "continuing".len();
                self.skip_ws();
                continuing = Some(self.parse_nested_block()?);
                self.skip_ws();
                self.expect('}')?; // closing brace of loop
                break;
            }
            stmts.push(self.parse_statement()?);
        }

        Ok(Statement::Loop(LoopStatement { body: Block { stmts }, continuing }))
    }

    // ── for ───────────────────────────────────────────────────────────────────

    fn parse_for(&mut self) -> Result<Statement, Box<dyn Error>> {
        self.pos += "for".len();
        self.skip_ws();
        self.expect('(')?;
        self.skip_ws();

        // init (optional)
        let init = if self.peek() == Some(';') {
            self.advance(); None
        } else {
            let word = self.peek_word();
            let s = if word == "var" || word == "let" {
                let d = self.parse_var_decl()?;
                self.expect_semi()?;
                Statement::VarDecl(d)
            } else {
                self.parse_assign_or_expr_stmt()?
            };
            Some(Box::new(s))
        };

        self.skip_ws();

        // condition (optional)
        let condition = if self.peek() == Some(';') {
            self.advance(); None
        } else {
            let e = self.parse_expression(0)?;
            self.expect_semi()?;
            Some(e)
        };

        self.skip_ws();

        // update (optional) — no trailing semicolon inside for
        let update = if self.peek() == Some(')') {
            None
        } else {
            // Parse like an assign_or_expr but don't consume the semicolon at end.
            // We use the same helper but it consumes a ';'; that's fine because
            // WGSL for-update also accepts an optional ';'.
            let saved = self.pos;
            let s = self.parse_assign_or_expr_stmt();
            match s {
                Ok(stmt) => Some(Box::new(stmt)),
                Err(_) => {
                    // No trailing semi — try expression only
                    self.pos = saved;
                    let expr = self.parse_expression(0)?;
                    Some(Box::new(Statement::Expression(expr)))
                }
            }
        };

        self.skip_ws();
        self.expect(')')?;

        let body = self.parse_nested_block()?;
        Ok(Statement::For(ForStatement { init, condition, update, body }))
    }

    // ── while ─────────────────────────────────────────────────────────────────

    fn parse_while(&mut self) -> Result<Statement, Box<dyn Error>> {
        self.pos += "while".len();
        self.skip_ws();
        let condition = self.parse_expression(0)?;
        let body = self.parse_nested_block()?;
        Ok(Statement::While(WhileStatement { condition, body }))
    }
}

// ── Expression parsing (Pratt) ────────────────────────────────────────────────

impl BlockParser {
    /// Entry: parse an expression with minimum precedence `min_prec`.
    fn parse_expression(&mut self, min_prec: u8) -> Result<Expression, Box<dyn Error>> {
        let mut lhs = self.parse_unary()?;

        loop {
            self.skip_ws();
            let Some(op) = self.peek_binary_op() else { break };
            if op.precedence() <= min_prec { break; }
            self.consume_binary_op(op);
            self.skip_ws();
            let rhs = self.parse_expression(op.precedence())?;
            lhs = Expression::Binary(Box::new(lhs), op, Box::new(rhs));
        }

        Ok(lhs)
    }

    fn peek_binary_op(&self) -> Option<BinaryOp> {
        let a = self.peek()?;
        let b = self.peek2();
        match (a, b) {
            // two-char ops — check before single-char to avoid false positives
            ('&', Some('&')) => Some(BinaryOp::And),
            ('|', Some('|')) => Some(BinaryOp::Or),
            ('=', Some('=')) => Some(BinaryOp::Eq),
            ('!', Some('=')) => Some(BinaryOp::Ne),
            ('<', Some('=')) => Some(BinaryOp::Le),
            ('>', Some('=')) => Some(BinaryOp::Ge),
            ('<', Some('<')) => Some(BinaryOp::Shl),
            ('>', Some('>')) => Some(BinaryOp::Shr),
            // single-char — must not be followed by '=' (that would be assign op)
            ('+', b) if b != Some('=') && b != Some('+') => Some(BinaryOp::Add),
            ('-', b) if b != Some('=') && b != Some('-') && b != Some('>') => Some(BinaryOp::Sub),
            ('*', b) if b != Some('=') => Some(BinaryOp::Mul),
            ('/', b) if b != Some('=') && b != Some('/') && b != Some('*') => Some(BinaryOp::Div),
            ('%', b) if b != Some('=') => Some(BinaryOp::Mod),
            ('&', b) if b != Some('=') && b != Some('&') => Some(BinaryOp::BitAnd),
            ('|', b) if b != Some('=') && b != Some('|') => Some(BinaryOp::BitOr),
            ('^', b) if b != Some('=') => Some(BinaryOp::BitXor),
            ('<', b) if b != Some('=') && b != Some('<') => Some(BinaryOp::Lt),
            ('>', b) if b != Some('=') && b != Some('>') => Some(BinaryOp::Gt),
            _ => None,
        }
    }

    fn consume_binary_op(&mut self, op: BinaryOp) {
        use BinaryOp::*;
        let n = match op {
            And | Or | Eq | Ne | Le | Ge | Shl | Shr => 2,
            _ => 1,
        };
        for _ in 0..n { self.advance(); }
    }

    fn parse_unary(&mut self) -> Result<Expression, Box<dyn Error>> {
        self.skip_ws();
        match self.peek() {
            Some('-') => { self.advance(); let e = self.parse_unary()?; Ok(Expression::Unary(UnaryOp::Neg, Box::new(e))) }
            Some('!') => { self.advance(); let e = self.parse_unary()?; Ok(Expression::Unary(UnaryOp::Not, Box::new(e))) }
            Some('~') => { self.advance(); let e = self.parse_unary()?; Ok(Expression::Unary(UnaryOp::BitNot, Box::new(e))) }
            Some('*') => { self.advance(); let e = self.parse_unary()?; Ok(Expression::Unary(UnaryOp::Deref, Box::new(e))) }
            Some('&') => { self.advance(); let e = self.parse_unary()?; Ok(Expression::Unary(UnaryOp::AddrOf, Box::new(e))) }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expression, Box<dyn Error>> {
        let mut expr = self.parse_primary()?;

        loop {
            self.skip_ws();
            match self.peek() {
                Some('.') => {
                    self.advance();
                    self.skip_ws();
                    let field = self.consume_ident()?;
                    expr = Expression::Field(Box::new(expr), field);
                }
                Some('[') => {
                    self.advance();
                    self.skip_ws();
                    let idx = self.parse_expression(0)?;
                    self.skip_ws();
                    self.expect(']')?;
                    expr = Expression::Index(Box::new(expr), Box::new(idx));
                }
                Some('(') => {
                    // Call — but only if this follows an identifier/field/index,
                    // not an arbitrary expression (to avoid mis-parsing `a(b)(c)`
                    // when `a(b)` returns a non-callable type). In WGSL this is
                    // fine since calls are always direct or through postfix.
                    let args = self.parse_call_args()?;
                    expr = Expression::Call(Box::new(expr), args);
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_call_args(&mut self) -> Result<Vec<Expression>, Box<dyn Error>> {
        self.expect('(')?;
        let mut args = Vec::new();
        self.skip_ws();
        if self.peek() == Some(')') { self.advance(); return Ok(args); }
        loop {
            self.skip_ws();
            args.push(self.parse_expression(0)?);
            self.skip_ws();
            if self.peek() == Some(',') { 
                self.advance();
                self.skip_ws();
                if self.peek() == Some(')') { break; }
            }
            else { break; }
        }
        self.expect(')')?;
        Ok(args)
    }

    fn parse_primary(&mut self) -> Result<Expression, Box<dyn Error>> {
        self.skip_ws();

        // Grouped expression
        if self.peek() == Some('(') {
            self.advance();
            let e = self.parse_expression(0)?;
            self.skip_ws();
            self.expect(')')?;
            return Ok(e);
        }

        // Numeric literal
        if self.peek().map_or(false, |c| c.is_ascii_digit()) {
            return self.parse_numeric_literal();
        }

        // Negative numeric literal (already handled by unary, but `0x…` etc.)
        let word = self.peek_word();

        // Bool literals
        if word == "true"  { self.pos += 4; return Ok(Expression::BoolLiteral(true));  }
        if word == "false" { self.pos += 5; return Ok(Expression::BoolLiteral(false)); }

        if !word.is_empty() {
            self.pos += word.len();
            self.skip_ws();
            // Type constructor or call: `vec3<f32>(…)` or `MyStruct { … }`
            // Consume template args if present
            let full_name = if self.peek() == Some('<') {
                let mut n = word.clone();
                self.advance(); n.push('<');
                let mut depth = 1usize;
                loop {
                    match self.advance() {
                        None => return Err(self.err("EOF in type template")),
                        Some('<') => { depth += 1; n.push('<'); }
                        Some('>') => {
                            n.push('>'); depth -= 1;
                            if depth == 0 { break; }
                        }
                        Some(c) => n.push(c),
                    }
                }
                n
            } else {
                word
            };

            self.skip_ws();
            if self.peek() == Some('(') {
                let args = self.parse_call_args()?;
                return Ok(Expression::TypeConstruct(full_name, args));
            }

            return Ok(Expression::Identifier(full_name));
        }

        Err(self.err(&format!("unexpected token: {:?} at position {} (word: {word})", self.peek(), self.pos)))
    }

    fn parse_numeric_literal(&mut self) -> Result<Expression, Box<dyn Error>> {
        let mut s = String::new();
        // Hex
        if self.peek() == Some('0') && self.peek2() == Some('x') {
            s.push(self.advance().unwrap());
            s.push(self.advance().unwrap());
            while self.peek().map_or(false, |c| c.is_ascii_hexdigit()) {
                s.push(self.advance().unwrap());
            }
        } else {
            while self.peek().map_or(false, |c| c.is_ascii_digit()) {
                s.push(self.advance().unwrap());
            }
            if self.peek() == Some('.') {
                s.push(self.advance().unwrap());
                while self.peek().map_or(false, |c| c.is_ascii_digit()) {
                    s.push(self.advance().unwrap());
                }
            }
        }
        // suffix (u, i, f, h)
        if self.peek().map_or(false, |c| matches!(c, 'u' | 'i' | 'f' | 'h')) {
            s.push(self.advance().unwrap());
        }

        if s.contains('.') || s.ends_with('f') || s.ends_with('h') {
            Ok(Expression::FloatLiteral(s))
        } else {
            Ok(Expression::IntLiteral(s))
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Convert an expression to an lvalue if possible.  Returns `None` if the
/// expression cannot be the target of an assignment.
pub fn expr_to_lvalue(expr: Expression) -> Option<LValue> {
    match expr {
        Expression::Identifier(s) => {
            if s == "_" { Some(LValue::Phony) } else { Some(LValue::Ident(s)) }
        }
        Expression::Index(base, idx) => {
            let lv = expr_to_lvalue(*base)?;
            Some(LValue::Index(Box::new(lv), idx))
        }
        Expression::Field(base, field) => {
            let lv = expr_to_lvalue(*base)?;
            Some(LValue::Field(Box::new(lv), field))
        }
        Expression::Unary(UnaryOp::Deref, inner) => {
            let lv = expr_to_lvalue(*inner)?;
            Some(LValue::Deref(Box::new(lv)))
        }
        _ => None,
    }
}
