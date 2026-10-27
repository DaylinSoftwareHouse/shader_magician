use magician_ast::{BinaryOp, Expression, LValue, UnaryOp};

use crate::{Transpiler, global::{FunctionContext, convert_ty, translate_ty_name}};

pub fn convert_expr(transpiler: &Transpiler, func: &FunctionContext, item: &syn::Expr) -> Option<Expression> {
    match item {
        syn::Expr::Unary(expr_unary) => {
            // decode expression
            let expr = 
                convert_expr(transpiler, func, &*expr_unary.expr);

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
            let left = convert_expr(transpiler, func, &*expr_binary.left);
            let right = convert_expr(transpiler, func, &*expr_binary.right);

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
                .filter_map(|a| convert_expr(transpiler, func, a))
                .collect::<Vec<_>>();

            // decode vector and matrix constructors
            if let Some(constructor) = convert_constructor(&func_name, args.clone()) {
                return Some(constructor);
            }
            let Some(func_name) = func_name.last() else { return None };

            // functions defined in the shader keep their names
            if transpiler.is_user_fn(func_name) {
                return Some(Expression::Call(Box::new(Expression::Identifier(func_name.clone())), args));
            }

            // bitcasts are generic in WGSL, `bitcast_f32(x)` -> `bitcast<f32>(x)`
            if let Some(ty) = func_name.strip_prefix("bitcast_") {
                if !matches!(ty, "f32" | "u32" | "i32") { panic!("bitcast_{ty} is not a supported bitcast target") }
                return Some(Expression::Call(Box::new(Expression::Identifier(format!("bitcast<{ty}>"))), args));
            }

            // component-wise comparisons are operators in WGSL
            let func_name = convert_builtin_fn_name(func_name);
            if let Some(op) = comparison_op(&func_name) {
                let [left, right] = <[Expression; 2]>::try_from(args)
                    .unwrap_or_else(|_| panic!("{func_name} takes exactly 2 arguments"));
                return Some(Expression::Binary(Box::new(left), op, Box::new(right)));
            }

            Some(Expression::Call(Box::new(Expression::Identifier(func_name)), args))
        },

        syn::Expr::Field(expr_field) => {
            // convert parent and field
            let parent = convert_expr(transpiler, func, &*expr_field.base);
            let field = match &expr_field.member {
                syn::Member::Named(ident) => Some(ident.to_string()),
                syn::Member::Unnamed(_index) => None
            };

            if let Some(Expression::Identifier(identifier)) = &parent {
                if field.is_some() && func.global_params.contains_key(identifier) {
                    return Some(Expression::Identifier(field.unwrap()))
                }
            }

            // return expression if parent and field where found
            if parent.is_some() && field.is_some() {
                Some(Expression::Field(Box::new(parent.unwrap()), field.unwrap()))
            } else { None }
        },

        syn::Expr::Index(expr_index) => {
            let expr = convert_expr(transpiler, func, &*expr_index.expr);
            let index = convert_expr(transpiler, func, &*expr_index.index);

            if expr.is_some() && index.is_some() {
                Some(Expression::Index(Box::new(expr.unwrap()), Box::new(index.unwrap())))
            } else { None }
        },

        syn::Expr::Lit(expr_lit) => Some(convert_lit(&expr_lit.lit)),

        syn::Expr::MethodCall(expr_method_call) => {
            let Some(inner) = convert_expr(transpiler, func, &*expr_method_call.receiver)
                else { panic!("Failed to get method: {:?}", expr_method_call) };
            let args = expr_method_call.args.iter()
                .filter_map(|a| convert_expr(transpiler, func, a))
                .collect::<Vec<_>>();
            let method = expr_method_call.method.to_string();

            let is_swizzle = method.len() <= 4 && 
                method.chars().all(|a| a == 'x' || a == 'y' || a == 'z' || a == 'w' || a == 'r' || a == 'g' || a == 'b' || a == 'a');

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

                    convert_expr(transpiler, func, &field.expr).map(|a| (field_name, a))
                })
                .collect();
            fields.sort_by_key(|a| parent.fields.iter().position(|f| f.name == a.0));
            let fields = fields.into_iter().map(|a| a.1).collect();

            Some(Expression::TypeConstruct(ident, fields))
        },

        syn::Expr::Paren(paren) => {
            convert_expr(transpiler, func, &paren.expr)
                .map(|expr| Expression::Paren(Box::new(expr)))
        }

        syn::Expr::Array(expr_array) => {
            let elems = expr_array.elems.iter()
                .filter_map(|a| convert_expr(transpiler, func, a))
                .collect();
            Some(Expression::TypeConstruct("array".to_string(), elems))
        },

        syn::Expr::Tuple(expr_tuple) => todo!("Tuple support: {expr_tuple:?}"),
        syn::Expr::Infer(expr_infer) => todo!("Infered types: {expr_infer:?}"),
        syn::Expr::Cast(cast) => {
            let inner = convert_expr(transpiler, func, &cast.expr)?;
            let ty = convert_ty(&cast.ty);

            // usize/isize casts only exist to satisfy rust indexing, WGSL indexes with any integer
            if ty == "usize" || ty == "isize" { return Some(inner) }
            Some(Expression::TypeConstruct(ty, vec![inner]))
        },

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

/// Converts a `match` arm pattern into `switch` case selectors, `None` being `default`.
pub fn convert_pat_to_selectors(pat: &syn::Pat) -> Vec<Option<Expression>> {
    match pat {
        syn::Pat::Lit(expr_lit) => vec![Some(convert_lit(&expr_lit.lit))],
        syn::Pat::Wild(_) => vec![None],
        syn::Pat::Or(pat_or) => pat_or.cases.iter().flat_map(convert_pat_to_selectors).collect(),
        syn::Pat::Paren(paren) => convert_pat_to_selectors(&paren.pat),
        syn::Pat::Ident(pat_ident) if pat_ident.subpat.is_none() =>
            panic!("Binding match patterns like `{}` are not supported, use `_`", pat_ident.ident),
        other => panic!("Unsupported match pattern, only literals, `|` and `_` are allowed: {other:?}")
    }
}

/// Converts a rust literal to a WGSL literal, mapping type suffixes (`1u32` -> `1u`).
pub fn convert_lit(lit: &syn::Lit) -> Expression {
    match lit {
        syn::Lit::Byte(lit_byte) => Expression::IntLiteral(lit_byte.value().to_string()),
        syn::Lit::Int(lit_int) => {
            let digits = lit_int.base10_digits();
            match lit_int.suffix() {
                "" | "usize" | "isize" => Expression::IntLiteral(digits.to_string()),
                "u32" => Expression::IntLiteral(format!("{digits}u")),
                "i32" => Expression::IntLiteral(format!("{digits}i")),
                "f32" => Expression::FloatLiteral(format!("{digits}f")),
                "f16" => Expression::FloatLiteral(format!("{digits}h")),
                "f64" => Expression::FloatLiteral(format!("{digits}.0")),
                other => panic!("WGSL has no {other} type, found literal {lit_int}")
            }
        },
        syn::Lit::Float(lit_float) => {
            let digits = lit_float.base10_digits();
            match lit_float.suffix() {
                "" | "f64" => Expression::FloatLiteral(digits.to_string()),
                "f32" => Expression::FloatLiteral(format!("{digits}f")),
                "f16" => Expression::FloatLiteral(format!("{digits}h")),
                other => panic!("WGSL has no {other} type, found literal {lit_float}")
            }
        },
        syn::Lit::Bool(lit_bool) => Expression::BoolLiteral(lit_bool.value),
        other => panic!("Invalid literal {other:?}"),
    }
}

/// Converts a call like `Vec3::new(..)` or `Mat4::identity()` on a built-in vector,
/// matrix, texture or sampler type to a WGSL constructor.  Returns `None` for any
/// other call.
fn convert_constructor(segments: &[String], args: Vec<Expression>) -> Option<Expression> {
    // handle two segment constructors (i.e. Vec2::new or Mat4::identity)
    if segments.len() < 2 { return None }
    let ty = translate_ty_name(&segments[segments.len() - 2]);
    let method = segments[segments.len() - 1].as_str();
    let is_builtin = ty.starts_with("vec") || ty.starts_with("mat") || ty == "texture_2d<f32>" || ty == "sampler";
    if !is_builtin { return None }

    match method {
        // zero value constructor
        "zero" => Some(Expression::TypeConstruct(ty.to_string(), vec![])),

        // build identity matrices column by column
        "identity" if ty.starts_with("mat") => {
            let size = ty[3..4].parse::<usize>().expect("Matrix type has no size");
            let scalar = &ty[ty.find('<')? + 1 .. ty.len() - 1];
            let columns = (0 .. size)
                .map(|col| {
                    let rows = (0 .. size)
                        .map(|row| Expression::FloatLiteral(if row == col { "1.0" } else { "0.0" }.to_string()))
                        .collect();
                    Expression::TypeConstruct(format!("vec{size}<{scalar}>"), rows)
                })
                .collect();
            Some(Expression::TypeConstruct(ty.to_string(), columns))
        },

        // all other constructors (new, splat, from_vec3_w, ...) map their arguments directly
        _ => Some(Expression::TypeConstruct(ty.to_string(), args))
    }
}

/// Type suffixes used to name the per-type variants of built-in functions.
const TYPE_SUFFIXES: &[&str] = &[
    "vec2", "vec3", "vec4", "dvec2", "dvec3", "dvec4",
    "ivec2", "ivec3", "ivec4", "uvec2", "uvec3", "uvec4",
    "bvec2", "bvec3", "bvec4", "mat2", "mat3", "mat4",
    "dmat2", "dmat3", "dmat4", "f32", "u32", "i32"
];

/// Converts the rust name of a built-in function to its WGSL name, by dropping the
/// type suffix and variant markers and converting it to camel case
/// (`inverse_sqrt_vec3` -> `inverseSqrt`, `mix_vec3_t` -> `mix`).
fn convert_builtin_fn_name(name: &str) -> String {
    let mut tokens = name.split('_').collect::<Vec<_>>();

    // drop variant markers that follow a type suffix (mix_vec3_t, pow_vec3_cw) and round_wgsl
    if tokens.len() > 2 && matches!(tokens[tokens.len() - 1], "t" | "cw") && TYPE_SUFFIXES.contains(&tokens[tokens.len() - 2]) {
        tokens.pop();
    }
    if tokens.len() > 1 && tokens[tokens.len() - 1] == "wgsl" { tokens.pop(); }

    // drop the type suffix
    if tokens.len() > 1 && TYPE_SUFFIXES.contains(&tokens[tokens.len() - 1]) { tokens.pop(); }

    // convert to camel case
    let mut result = tokens[0].to_string();
    for token in &tokens[1..] {
        let mut chars = token.chars();
        if let Some(first) = chars.next() {
            result.extend(first.to_uppercase());
            result.push_str(chars.as_str());
        }
    }
    return result;
}

/// The operator for a component-wise comparison function, `equal` -> `==`.
fn comparison_op(wgsl_name: &str) -> Option<BinaryOp> {
    match wgsl_name {
        "equal" => Some(BinaryOp::Eq),
        "notEqual" => Some(BinaryOp::Ne),
        "lessThan" => Some(BinaryOp::Lt),
        "lessThanEqual" => Some(BinaryOp::Le),
        "greaterThan" => Some(BinaryOp::Gt),
        "greaterThanEqual" => Some(BinaryOp::Ge),
        _ => None
    }
}
