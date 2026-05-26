use magician_ast::Statement;

use crate::expr;

pub fn convert_stmt(item: &syn::Stmt) -> Vec<Statement> {
    match &item {
        // syn::Stmt::Local(local) => todo!(),
        // syn::Stmt::Item(item) => todo!(),
        syn::Stmt::Expr(expr, _semi) => convert_stmt_expr(expr),
        syn::Stmt::Macro(stmt_macro) => panic!("No macros allowed in shader functions {stmt_macro:?}"),
        _ => vec![]
    }
}

pub fn convert_stmt_expr(item: &syn::Expr) -> Vec<Statement> {
    match item {
        syn::Expr::Array(expr_array) => todo!(),
        syn::Expr::Assign(expr_assign) => todo!(),
        syn::Expr::Async(expr_async) => todo!(),
        syn::Expr::Await(expr_await) => todo!(),
        syn::Expr::Binary(expr_binary) => todo!(),
        syn::Expr::Block(expr_block) => todo!(),
        syn::Expr::Break(expr_break) => vec![Statement::Break],
        syn::Expr::Call(expr_call) => todo!(),
        syn::Expr::Cast(expr_cast) => todo!(),
        syn::Expr::Closure(expr_closure) => todo!(),
        syn::Expr::Const(expr_const) => todo!(),
        syn::Expr::Continue(expr_continue) => vec![Statement::Continue],
        syn::Expr::Field(expr_field) => todo!(),
        syn::Expr::ForLoop(expr_for_loop) => todo!(),
        syn::Expr::Group(expr_group) => todo!(),
        syn::Expr::If(expr_if) => todo!(),
        syn::Expr::Index(expr_index) => todo!(),
        syn::Expr::Infer(expr_infer) => todo!(),
        syn::Expr::Let(expr_let) => todo!(),
        syn::Expr::Lit(expr_lit) => todo!(),
        syn::Expr::Loop(expr_loop) => todo!(),
        syn::Expr::Macro(expr_macro) => todo!(),
        syn::Expr::Match(expr_match) => todo!(),
        syn::Expr::MethodCall(expr_method_call) => todo!(),
        syn::Expr::Paren(expr_paren) => todo!(),
        syn::Expr::Path(expr_path) => todo!(),
        syn::Expr::Range(expr_range) => todo!(),
        syn::Expr::RawAddr(expr_raw_addr) => todo!(),
        syn::Expr::Reference(expr_reference) => todo!(),
        syn::Expr::Repeat(expr_repeat) => todo!(),
        syn::Expr::Return(expr_return) => 
            vec![Statement::Return(expr_return.expr.as_ref().map(|a| expr::convert_expr(&*a)).flatten())],
        syn::Expr::Struct(expr_struct) => todo!(),
        syn::Expr::Try(expr_try) => todo!(),
        syn::Expr::TryBlock(expr_try_block) => todo!(),
        syn::Expr::Tuple(expr_tuple) => todo!(),
        syn::Expr::Unary(expr_unary) => todo!(),
        syn::Expr::Unsafe(expr_unsafe) => todo!(),
        syn::Expr::Verbatim(token_stream) => todo!(),
        syn::Expr::While(expr_while) => todo!(),
        syn::Expr::Yield(expr_yield) => todo!(),
        _ => todo!(),
    }
}
