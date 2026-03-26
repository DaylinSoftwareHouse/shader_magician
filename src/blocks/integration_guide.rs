// ═══════════════════════════════════════════════════════════════════════════
// CHANGES TO YOUR EXISTING CODE  (shown as annotated diffs)
// ═══════════════════════════════════════════════════════════════════════════
//
// 1. lib.rs (or wherever ShaderElement is defined)
// ─────────────────────────────────────────────────
//
// Add the new modules and re-exports:
//
//   pub mod wgsl_ast;          // new
//   pub mod block_parser;      // new
//   pub mod ast_visitor;       // new
//
//   pub use wgsl_ast::*;
//   pub use ast_visitor::*;
//
// Change ShaderElement::Function  (remove `block: String` and
// `preprocessor_instructions: Vec<String>`, add `body: Block`):
//
//   BEFORE:
//     ShaderElement::Function {
//         attrs: Vec<Attr>,
//         name: String,
//         params: Vec<Param>,
//         ret_ty: Option<String>,
//         block: String,                          // raw source
//         preprocessor_instructions: Vec<String>, // extracted #directives
//     }
//
//   AFTER:
//     ShaderElement::Function {
//         attrs: Vec<Attr>,
//         name: String,
//         params: Vec<Param>,
//         ret_ty: Option<String>,
//         body: Block,           // fully parsed AST  ← new
//     }
//
// ─────────────────────────────────────────────────────────────────────────
// 2. parser.rs — parse_function
// ─────────────────────────────────────────────────────────────────────────
//
// BEFORE (bottom of parse_function):
//
//   let block = self.consume_block()?;
//   let preprocessor_instructions = Self::extract_preprocessor_instructions(&block);
//
//   Ok(ShaderElement::Function {
//       attrs,
//       name,
//       params,
//       block,
//       ret_ty,
//       preprocessor_instructions
//   })
//
// AFTER:
//
//   let raw_block = self.consume_block()?;   // still consumed as a raw string
//   let body = BlockParser::new(&raw_block)  // then parsed into a typed AST
//       .parse_block()?;
//
//   Ok(ShaderElement::Function {
//       attrs,
//       name,
//       params,
//       ret_ty,
//       body,
//   })
//
// Remove the `extract_preprocessor_instructions` method entirely — it is
// no longer needed; preprocessor instructions inside function bodies are
// preserved in `Statement::Expression(Expression::Identifier("#ident"))`
// or in the raw string of a `ShaderElement::PreprocessorInstruction` at
// the top level.
//
// ═══════════════════════════════════════════════════════════════════════════
// USAGE EXAMPLES
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod usage_examples {
    use crate::{Parser, ShaderElement, wgsl_ast::*, ast_visitor::*};

    const EXAMPLE_SHADER: &str = r#"
#define_import_path my_shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> @builtin(position) vec4<f32> {
    var out = in.position;
    let scale = 2.0;
    if out.z > 0.5 {
        out = out * scale;
    }
    for (var i: i32 = 0; i < 4; i++) {
        out[i] = clamp(out[i], 0.0, 1.0);
    }
    return vec4<f32>(out, 1.0);
}
"#;

    #[test]
    fn find_all_function_calls() {
        let mut parser = Parser::new(EXAMPLE_SHADER);
        let result = parser.parse_all_elements().unwrap();

        for elem in &result.elements {
            if let ShaderElement::Function { name, body, .. } = elem {
                let calls = collect_calls(body);
                println!("Function '{name}' calls: {calls:?}");
                // → [("clamp", 3)]
            }
        }
    }

    #[test]
    fn collect_all_local_vars() {
        let mut parser = Parser::new(EXAMPLE_SHADER);
        let result = parser.parse_all_elements().unwrap();

        for elem in &result.elements {
            if let ShaderElement::Function { name, body, .. } = elem {
                let vars = collect_local_vars(body);
                for v in vars {
                    println!("  {name}: {} {:?} {:?}", v.name, v.kind, v.ty);
                }
            }
        }
    }

    #[test]
    fn rename_variable() {
        let mut parser = Parser::new(EXAMPLE_SHADER);
        let mut result = parser.parse_all_elements().unwrap();

        for elem in &mut result.elements {
            if let ShaderElement::Function { body, .. } = elem {
                rename_identifier(body, "scale", "zoom_factor");
                println!("{}", emit_block(body, 0));
            }
        }
    }

    #[test]
    fn check_function_uses_builtin() {
        let mut parser = Parser::new(EXAMPLE_SHADER);
        let result = parser.parse_all_elements().unwrap();

        for elem in &result.elements {
            if let ShaderElement::Function { name, body, .. } = elem {
                if calls_function(body, "clamp") {
                    println!("{name} uses clamp()");
                }
            }
        }
    }

    #[test]
    fn custom_visitor_count_binary_ops() {
        use crate::ast_visitor::{Visitor, walk_expression};

        struct OpCounter { count: usize }
        impl Visitor for OpCounter {
            fn visit_expression(&mut self, expr: &Expression) {
                if matches!(expr, Expression::Binary(..)) { self.count += 1; }
                walk_expression(self, expr);
            }
        }

        let mut parser = Parser::new(EXAMPLE_SHADER);
        let result = parser.parse_all_elements().unwrap();

        for elem in &result.elements {
            if let ShaderElement::Function { name, body, .. } = elem {
                let mut counter = OpCounter { count: 0 };
                counter.visit_block(body);
                println!("{name}: {count} binary operations", count = counter.count);
            }
        }
    }

    #[test]
    fn round_trip_emit() {
        let mut parser = Parser::new(EXAMPLE_SHADER);
        let result = parser.parse_all_elements().unwrap();

        for elem in &result.elements {
            if let ShaderElement::Function { name, body, .. } = elem {
                let emitted = emit_block(body, 0);
                println!("// --- {name} ---\n{emitted}");
            }
        }
    }
}
