use magician_ast::Expression;

pub fn convert_expr(item: &syn::Expr) -> Option<Expression> {
    match item {
        syn::Expr::Array(expr_array) => todo!(),
        syn::Expr::Assign(expr_assign) => todo!(),
        syn::Expr::Async(expr_async) => todo!(),
        syn::Expr::Await(expr_await) => todo!(),
        syn::Expr::Binary(expr_binary) => todo!(),
        syn::Expr::Block(expr_block) => todo!(),
        syn::Expr::Break(expr_break) => todo!(),
        syn::Expr::Call(expr_call) => {
            let syn::Expr::Path(func_name) = &*expr_call.func else { panic!("ExprCall has no function path") };
            let func_name = func_name.path.segments.iter().map(|a| a.ident.to_string()).collect::<Vec<_>>();
            let func_name = convert_constructor_name(&func_name)
                .expect("Failed to find constructor function name");
            let args = expr_call.args.iter().filter_map(convert_expr).collect::<Vec<_>>();
            Some(Expression::TypeConstruct(func_name, args))
        },
        syn::Expr::Cast(expr_cast) => todo!(),
        syn::Expr::Closure(expr_closure) => todo!(),
        syn::Expr::Const(expr_const) => todo!(),
        syn::Expr::Continue(expr_continue) => todo!(),
        syn::Expr::Field(expr_field) => todo!(),
        syn::Expr::ForLoop(expr_for_loop) => todo!(),
        syn::Expr::Group(expr_group) => todo!(),
        syn::Expr::If(expr_if) => todo!(),
        syn::Expr::Index(expr_index) => todo!(),
        syn::Expr::Infer(expr_infer) => todo!(),
        syn::Expr::Let(expr_let) => todo!(),
        syn::Expr::Lit(expr_lit) => {
            match &expr_lit.lit {
                syn::Lit::Byte(lit_byte) => Some(Expression::IntLiteral(lit_byte.value().to_string())),
                syn::Lit::Int(lit_int) => Some(Expression::IntLiteral(lit_int.to_string())),
                syn::Lit::Float(lit_float) => Some(Expression::FloatLiteral(lit_float.to_string())),
                syn::Lit::Bool(lit_bool) => Some(Expression::BoolLiteral(lit_bool.value)),
                _ => panic!("Invalid literal {:?}", expr_lit.lit),
            }
        },
        syn::Expr::Loop(expr_loop) => todo!(),
        syn::Expr::Macro(expr_macro) => todo!(),
        syn::Expr::Match(expr_match) => todo!(),
        syn::Expr::MethodCall(expr_method_call) => {
            let syn::Expr::Path(ident) = &*expr_method_call.receiver else { panic!("Failed to get method identifier") };
            let ident = ident.path.segments.iter().last().map(|a| a.ident.to_string())
                .expect("Failed to get method identifier");
            let field = expr_method_call.method.to_string();
            let inner = Expression::Field(Box::new(Expression::Identifier(ident)), field);
            let args = expr_method_call.args.iter().filter_map(convert_expr).collect::<Vec<_>>();
            Some(Expression::Call(Box::new(inner), args))
        },
        syn::Expr::Paren(expr_paren) => todo!(),
        syn::Expr::Path(expr_path) => todo!(),
        syn::Expr::Range(expr_range) => todo!(),
        syn::Expr::RawAddr(expr_raw_addr) => todo!(),
        syn::Expr::Reference(expr_reference) => todo!(),
        syn::Expr::Repeat(expr_repeat) => todo!(),
        syn::Expr::Return(expr_return) => todo!(),
        syn::Expr::Struct(expr_struct) => {
            todo!("Expr struct {expr_struct:?}")
        },
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
