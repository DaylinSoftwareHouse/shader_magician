use magician_ast::{BinaryOp, Expression, LValue, UnaryOp};

use crate::Transpiler;

pub fn convert_expr(transpiler: &Transpiler, item: &syn::Expr) -> Option<Expression> {
    match item {
        syn::Expr::Unary(expr_unary) => {
            // decode expression
            let expr = 
                convert_expr(transpiler, &*expr_unary.expr);

            // decode unary operator
            let unary = match &expr_unary.op {
                syn::UnOp::Deref(_star) => UnaryOp::Deref,
                syn::UnOp::Not(_not) => UnaryOp::Not,
                syn::UnOp::Neg(_minus) => UnaryOp::Neg,
                _ => panic!("Unary op: {expr_unary:?}"),
            };

            // optionally encode expression
            if let Some(expr) = expr {
                Some(Expression::Unary(unary, Box::new(expr)))
            } else { None }
        },

        syn::Expr::Binary(expr_binary) => {
            // convert left and right side
            let left = convert_expr(transpiler, &*expr_binary.left);
            let right = convert_expr(transpiler, &*expr_binary.left);

            // convert binary operation
            let binop = match &expr_binary.op {
                syn::BinOp::Add(_plus) => BinaryOp::Add,
                syn::BinOp::Sub(_minus) => BinaryOp::Sub,
                syn::BinOp::Mul(_star) => BinaryOp::Mul,
                syn::BinOp::Div(_slash) => BinaryOp::Div,
                syn::BinOp::Rem(_percent) => BinaryOp::Mod,
                syn::BinOp::And(_and_and) => BinaryOp::And,
                syn::BinOp::Or(_or_or) => BinaryOp::Or,
                syn::BinOp::BitXor(_caret) => BinaryOp::BitXor,
                syn::BinOp::BitAnd(_and) => BinaryOp::BitAnd,
                syn::BinOp::BitOr(_or) => BinaryOp::BitOr,
                syn::BinOp::Shl(_shl) => BinaryOp::Shl,
                syn::BinOp::Shr(_shr) => BinaryOp::Shr,
                syn::BinOp::Eq(_eq_eq) => BinaryOp::Eq,
                syn::BinOp::Lt(_lt) => BinaryOp::Lt,
                syn::BinOp::Le(_le) => BinaryOp::Le,
                syn::BinOp::Ne(_ne) => BinaryOp::Ne,
                syn::BinOp::Ge(_ge) => BinaryOp::Ge,
                syn::BinOp::Gt(_gt) => BinaryOp::Gt,
                _ => panic!("Unsupported binary operation: {:?}", expr_binary.op),
            };

            // optionally convert to final expression
            if left.is_some() && right.is_some() {
                Some(Expression::Binary(Box::new(left.unwrap()), binop, Box::new(right.unwrap())))
            } else { None }
        },
        
        syn::Expr::Call(expr_call) => {
            // decode function name
            let syn::Expr::Path(func_name) = &*expr_call.func else { panic!("ExprCall has no function path") };
            let func_name = func_name.path.segments.iter().map(|a| a.ident.to_string()).collect::<Vec<_>>();

            // decode arguments
            let args = expr_call.args.iter()
                .filter_map(|a| convert_expr(transpiler, a))
                .collect::<Vec<_>>();

            // decode function call
            if let Some(constructor) = convert_constructor_name(&func_name) {
                Some(Expression::TypeConstruct(constructor, args))
            } else if func_name.len() > 0 {
                let func_name = func_name[func_name.len() - 1].clone();
                Some(Expression::Call(Box::new(Expression::Identifier(func_name)), args))
            } else {
                None
            }
        },

        syn::Expr::Field(expr_field) => {
            // convert parent and field
            let parent = convert_expr(transpiler, &*expr_field.base);
            let field = match &expr_field.member {
                syn::Member::Named(ident) => Some(ident.to_string()),
                syn::Member::Unnamed(_index) => None
            };

            // return expression if parent and field where found
            if parent.is_some() && field.is_some() {
                Some(Expression::Field(Box::new(parent.unwrap()), field.unwrap()))
            } else { None }
        },

        syn::Expr::Index(expr_index) => {
            let expr = convert_expr(transpiler, &*expr_index.expr);
            let index = convert_expr(transpiler, &*expr_index.index);

            if expr.is_some() && index.is_some() {
                Some(Expression::Index(Box::new(expr.unwrap()), Box::new(index.unwrap())))
            } else { None }
        },

        syn::Expr::Lit(expr_lit) => {
            match &expr_lit.lit {
                syn::Lit::Byte(lit_byte) => Some(Expression::IntLiteral(lit_byte.value().to_string())),
                syn::Lit::Int(lit_int) => Some(Expression::IntLiteral(lit_int.to_string())),
                syn::Lit::Float(lit_float) => Some(Expression::FloatLiteral(lit_float.to_string())),
                syn::Lit::Bool(lit_bool) => Some(Expression::BoolLiteral(lit_bool.value)),
                _ => panic!("Invalid literal {:?}", expr_lit.lit),
            }
        },

        syn::Expr::MethodCall(expr_method_call) => {
            let Some(inner) = convert_expr(transpiler, &*expr_method_call.receiver)
                else { panic!("Failed to get method: {:?}", expr_method_call) };
            let args = expr_method_call.args.iter()
                .filter_map(|a| convert_expr(transpiler, a))
                .collect::<Vec<_>>();
            let method = expr_method_call.method.to_string();

            let is_swizzle = method.len() <= 4 && 
                method.chars().all(|a| a == 'x' || a == 'y' || a == 'z' || a == 'r' || a == 'g' || a == 'b');

            if is_swizzle { Some(Expression::Field(Box::new(inner), method)) }
            else { Some(Expression::Call(Box::new(Expression::Field(Box::new(inner), method)), args)) }
        },

        syn::Expr::Path(expr_path) => {
            let segments = expr_path.path.segments.iter().collect::<Vec<_>>();

            if segments.len() == 1 {
                let ident = segments.get(0).unwrap().ident.to_string();
                Some(Expression::Identifier(ident))
            } else { None }
        },

        syn::Expr::Struct(expr_struct) => {
            // get struct identifier
            let ident = expr_struct.path.segments.iter().last()
                .expect("No identifier for expr struct provided!");
            let ident = ident.ident.to_string();

            // get parent struct
            let Some(parent) = transpiler.structs.get(&ident)
                else { panic!("Failed to find parent struct for {ident:?}") };

            // create and sort fields
            let mut fields: Vec<(String, Expression)> = expr_struct.fields.iter()
                .filter_map(|field| {
                    let syn::Member::Named(field_name) = &field.member 
                        else { panic!("Only named fields allow in struct builder!") };
                    let field_name = field_name.to_string();

                    convert_expr(transpiler, &field.expr).map(|a| (field_name, a))
                })
                .collect();
            fields.sort_by_key(|a| parent.fields.iter().position(|f| f.name == a.0));
            let fields = fields.into_iter().map(|a| a.1).collect();

            Some(Expression::TypeConstruct(ident, fields))
        },

        syn::Expr::Tuple(expr_tuple) => todo!("Tuple support: {expr_tuple:?}"),
        syn::Expr::Infer(expr_infer) => todo!("Infered types: {expr_infer:?}"),

        _ => panic!("Unsupported expression: {:?}", item)
    }
}

pub fn convert_expr_to_lvalue(transpiler: &Transpiler, expr: &Expression) -> Option<LValue> {
    match expr {
        Expression::Identifier(ident) => Some(LValue::Ident(ident.clone())),
        Expression::Index(expr, index) => 
            convert_expr_to_lvalue(transpiler, expr).map(|a| {
                LValue::Index(Box::new(a), Box::new(*index.clone()))
            }),
        Expression::Field(expr, field) =>
            convert_expr_to_lvalue(transpiler, expr).map(|a| {
                LValue::Field(Box::new(a), field.clone())
            }),
        Expression::Deref(expr) =>
            convert_expr_to_lvalue(transpiler, expr).map(|a| {
                LValue::Deref(Box::new(a))
            }),
        _ => None
    }
}

pub fn convert_pat_to_expr(transpiler: &Transpiler, pat: &syn::Pat) -> Option<Expression> {
    match pat {
        syn::Pat::Ident(pat_ident) => 
            Some(Expression::Identifier(pat_ident.ident.to_string())),

        syn::Pat::Lit(expr_lit) => 
            match &expr_lit.lit {
                syn::Lit::Byte(lit_byte) => Some(Expression::IntLiteral(lit_byte.value().to_string())),
                syn::Lit::Int(lit_int) => Some(Expression::IntLiteral(lit_int.to_string())),
                syn::Lit::Float(lit_float) => Some(Expression::FloatLiteral(lit_float.to_string())),
                syn::Lit::Bool(lit_bool) => Some(Expression::BoolLiteral(lit_bool.value)),
                _ => panic!("Invalid literal {:?}", expr_lit.lit),
            },

        syn::Pat::Or(pat_or) => {
            pat_or.cases.iter()
                .fold(None, |prev, pat| {
                    if let Some(prev) = prev {
                        let new = convert_pat_to_expr(transpiler, pat)
                            .expect("Failed or creation in pat conversion");
                        Some(Expression::Binary(Box::new(prev), BinaryOp::Or, Box::new(new)))
                    } else {
                        convert_pat_to_expr(transpiler, pat)
                    }
                })
        },

        _ => None
    }
}

fn convert_constructor_name(segments: &[String]) -> Option<String> {
    // handle two segment constructors (i.e. Vec2::new or Mat4::from_cols)
    if segments.len() >= 2 {
        // convert built-in constructors
        let result = match segments[segments.len() - 2].as_str() {
            "Vec2" | "DVec2" | "UVec2" | "IVec2" | "BVec2" => Some("vec2"), 
            "Vec3" | "DVec3" | "UVec3" | "IVec3" | "BVec3" => Some("vec3"), 
            "Vec4" | "DVec4" | "UVec4" | "IVec4" | "BVec4" => Some("vec4"),
            "Mat2" | "DMat2" | "UMat2" | "IMat2" | "BMat2" => Some("mat2x2"), 
            "Mat3" | "DMat3" | "UMat3" | "IMat3" | "BMat3" => Some("mat3x3"), 
            "Mat4" | "DMat4" | "UMat4" | "IMat4" | "BMat4" => Some("mat4x4"),
            _ => None
        };

        // return built-in if found
        if let Some(result) = result { return Some(result.to_string()) }
    }

    None
}
