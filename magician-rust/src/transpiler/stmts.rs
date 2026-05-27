use magician_ast::*;
use syn::spanned::Spanned;

use crate::{Transpiler, expr, global::convert_ty};

pub fn convert_stmt(transpiler: &Transpiler, item: &syn::Stmt) -> Vec<Statement> {
    match &item {
        syn::Stmt::Local(local) => {
            // decode identifier, type, and mutability
            let (ident, ty, is_mut): (String, Option<String>, bool) = match &local.pat {
                syn::Pat::Ident(pat_ident) => {
                    let ident = pat_ident.ident.to_string();
                    let is_mut = pat_ident.mutability.is_some();
                    (ident, None, is_mut)
                },
                syn::Pat::Type(pat_type) => {
                    let syn::Pat::Ident(ident) = &*pat_type.pat
                        else { panic!("Typed pat had non ident sub pattern: {:?}", pat_type) };
                    let is_mut = ident.mutability.is_some();
                    let ty = convert_ty(&*pat_type.ty);
                    let ident = ident.ident.to_string();

                    (ident, Some(ty), is_mut)
                },
                _ => panic!("Unsupported let pattern {:?}", local.pat),
            };

            // decode initializer
            let initializer = local.init.as_ref().map(|init| {
                if init.diverge.is_some() { panic!("Diverging expression are not allowed in shader rust {:?} {:?}", local.span(), local) }

                expr::convert_expr(transpiler, &*init.expr)
            }).flatten();

            // compose variable declaration
            let decl = VarDecl {
                kind: if is_mut { VarKind::Var } else { VarKind::Let },
                template_args: None,
                name: ident,
                ty, initializer
            };

            vec![Statement::VarDecl(decl)]
        },

        syn::Stmt::Item(item) => todo!("Nested item support: {item:?}"),
        syn::Stmt::Expr(expr, _semi) => convert_stmt_expr(transpiler, expr),
        syn::Stmt::Macro(stmt_macro) => panic!("No macros allowed in shader functions {stmt_macro:?}"),
    }
}

pub fn convert_stmt_expr(transpiler: &Transpiler, item: &syn::Expr) -> Vec<Statement> {
    match item {
        syn::Expr::Break(_expr_break) => vec![Statement::Break],
        syn::Expr::Continue(_expr_continue) => vec![Statement::Continue],
        syn::Expr::Return(expr_return) => 
            vec![Statement::Return(expr_return.expr.as_ref().map(|a| expr::convert_expr(transpiler, &*a)).flatten())],
        
        syn::Expr::Assign(expr_assign) => {
            // convert left and right sides of expression
            let left = expr::convert_expr(transpiler, &expr_assign.left)
                .map(|expr| expr::convert_expr_to_lvalue(transpiler, &expr))
                .flatten();
            let right = expr::convert_expr(transpiler, &expr_assign.right);

            // optionally build final statement
            if left.is_some() && right.is_some() {
                vec![Statement::Assign(AssignStatement { 
                    target: left.unwrap(), 
                    op: AssignOp::Simple, 
                    value: right.unwrap() 
                })]
            } else { vec![] }
        },

        syn::Expr::Binary(expr_binary) => {
            // convert left and right sides of expression
            let left = expr::convert_expr(transpiler, &expr_binary.left)
                .map(|expr| expr::convert_expr_to_lvalue(transpiler, &expr))
                .flatten();
            let right = expr::convert_expr(transpiler, &expr_binary.right);

            // decode operation
            let binop = match &expr_binary.op {
                syn::BinOp::AddAssign(_plus_eq) => AssignOp::Add,
                syn::BinOp::SubAssign(_minus_eq) => AssignOp::Sub,
                syn::BinOp::MulAssign(_star_eq) => AssignOp::Mul,
                syn::BinOp::DivAssign(_slash_eq) => AssignOp::Div,
                syn::BinOp::RemAssign(_percent_eq) => AssignOp::Mod,
                syn::BinOp::BitXorAssign(_caret_eq) => AssignOp::Xor,
                syn::BinOp::BitAndAssign(_and_eq) => AssignOp::And,
                syn::BinOp::BitOrAssign(_or_eq) => AssignOp::Or,
                syn::BinOp::ShlAssign(_shl_eq) => AssignOp::Shl,
                syn::BinOp::ShrAssign(_shr_eq) => AssignOp::Shr,
                _ => panic!("Unsupported statement operation: {:?}", expr_binary.op),
            };

            // optionally build final statement
            if left.is_some() && right.is_some() {
                vec![Statement::Assign(AssignStatement { 
                    target: left.unwrap(), 
                    op: binop, 
                    value: right.unwrap() 
                })]
            } else { vec![] }
        },

        syn::Expr::Block(expr_block) => {
            let mut block = Vec::new();

            for stmt in &expr_block.block.stmts {
                let stmts = convert_stmt(transpiler, stmt);
                block.extend(stmts);
            }

            vec![Statement::Block(Block { stmts: block })]
        },
        
        syn::Expr::Let(expr_let) => {
            // decode identifier, type, and mutability
            let (ident, ty, is_mut): (String, Option<String>, bool) = match &*expr_let.pat {
                syn::Pat::Ident(pat_ident) => {
                    let ident = pat_ident.ident.to_string();
                    let is_mut = pat_ident.mutability.is_some();
                    (ident, None, is_mut)
                },
                syn::Pat::Type(pat_type) => {
                    let syn::Pat::Ident(ident) = &*pat_type.pat
                        else { panic!("Typed pat had non ident sub pattern: {:?}", pat_type) };
                    let is_mut = ident.mutability.is_some();
                    let ty = convert_ty(&*pat_type.ty);
                    let ident = ident.ident.to_string();

                    (ident, Some(ty), is_mut)
                },
                _ => panic!("Unsupported let pattern {:?}", expr_let.pat),
            };

            // decode initializer
            let expr = expr::convert_expr(transpiler, &expr_let.expr);

            // compose variable declaration
            let decl = VarDecl {
                kind: if is_mut { VarKind::Var } else { VarKind::Let },
                template_args: None,
                name: ident,
                ty, 
                initializer: expr
            };

            vec![Statement::VarDecl(decl)]
        },

        syn::Expr::If(expr_if) => {
            // decode condition and body
            let condition = expr::convert_expr(transpiler, &expr_if.cond)
                .expect("Missing condition in if statement");
            let body = Block {
                stmts: expr_if.then_branch.stmts.iter()
                    .flat_map(|stmt| convert_stmt(transpiler, stmt))
                    .collect()
            };

            let mut else_ifs = Vec::new();
            let mut else_body = None;

            // recursively find else ifs and else body
            fn recr_elses(
                else_ifs: &mut Vec<(Expression, Block)>,
                else_body: &mut Option<Block>,
                transpiler: &Transpiler, 
                expr: &syn::Expr
            ) {
                match expr {
                    // find else ifs
                    syn::Expr::If(expr_if) => {
                        let condition = expr::convert_expr(transpiler, &expr_if.cond)
                            .expect("Missing condition in if statement");
                        let body = Block {
                            stmts: expr_if.then_branch.stmts.iter()
                                .flat_map(|stmt| convert_stmt(transpiler, stmt))
                                .collect()
                        };
                        if let Some((_, else_expr)) = &expr_if.else_branch {
                            recr_elses(else_ifs, else_body, transpiler, &*else_expr);
                        }
                        else_ifs.push((condition, body));
                    }

                    // find else block
                    syn::Expr::Block(block) => {
                        let body = Block {
                            stmts: block.block.stmts.iter()
                                .flat_map(|stmt| convert_stmt(transpiler, stmt))
                                .collect()
                        };
                        *else_body = Some(body);
                    }

                    // this shouldn't happen according to syn docs
                    other => panic!("Unsupported if else condition/body: {other:?}")
                }
            }

            // start recr else searching
            if let Some((_, else_expr)) = &expr_if.else_branch {
                recr_elses(&mut else_ifs, &mut else_body, transpiler, &*else_expr);
            }

            // compile final if statement
            vec![Statement::If(IfStatement { condition, body, else_ifs, else_body })]
        },

        syn::Expr::While(expr_while) => {
            let condition = expr::convert_expr(transpiler, &expr_while.cond)
                .expect("Missing condition in if statement");
            let body = Block {
                stmts: expr_while.body.stmts.iter()
                    .flat_map(|stmt| convert_stmt(transpiler, stmt))
                    .collect()
            };
            vec![Statement::While(WhileStatement { condition, body })]
        },

        syn::Expr::Loop(expr_loop) => {
            let body = Block {
                stmts: expr_loop.body.stmts.iter()
                    .flat_map(|stmt| convert_stmt(transpiler, stmt))
                    .collect()
            };
            vec![Statement::Loop(LoopStatement { body, continuing: None })]
        },

        syn::Expr::Match(expr_match) => {
            let Some(selector) = expr::convert_expr(transpiler, &*expr_match.expr)
                else { return vec![] };

            let cases = expr_match.arms.iter()
                .filter_map(|case| {
                    let selector = expr::convert_pat_to_expr(transpiler, &case.pat);
                    let syn::Expr::Block(block) = &*case.body else { return None };
                    let body = Block {
                        stmts: block.block.stmts.iter()
                            .flat_map(|stmt| convert_stmt(transpiler, stmt))
                            .collect()
                    };
                    Some(SwitchCase { selectors: vec![selector], body })
                })
                .collect::<Vec<SwitchCase>>();

            vec![Statement::Switch(SwitchStatement { selector, cases })]
        },
        
        other => {
            let Some(expression) = expr::convert_expr(transpiler, other)
                else { panic!("Unexported stmt expr: {item:?}") };
            vec![Statement::Expression(expression)]
        }
    }
}
