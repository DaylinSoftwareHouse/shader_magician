# Hand-written WGSL: magician-ast and ShaderComposer

Use these when you work with WGSL source directly instead of transpiling Rust.

## magician-ast (`magician_rust::ast` or the `magician-ast` crate)

A small syn-like parser and serializer for WGSL.

```rust
use magician_ast::*;

// Top-level items plus preprocessor lines
let file = ShaderFile::parse("lighting", src)?;      // name, elements, imports
for el in &file.elements {
    match el {
        ShaderElement::Function { attrs, name, params, ret_ty, block, .. } => { /* block: Block */ }
        ShaderElement::Struct { attrs, name, params } => {}
        ShaderElement::Global { attrs, declared_as, name, ty } => {}
        _ => {}
    }
}
let wgsl = ShaderElement::to_wgsl(&file.elements, &HashMap::new(), /*only_public*/ false);

// Function bodies
let mut block = BlockParser::new("{ let x = 1.0; return x; }").parse_block()?;
rename_identifier(&mut block, "x", "y");
let calls = collect_calls(&block);              // Vec<(fn_name, arg_count)>
let text  = emit_block(&block, 0);
```

The body AST lives in `magician-ast/src/blocks/wgsl_ast.rs` (`Block`, `Statement`, `Expression`, `LValue`, ...). Implement `Visitor`/`VisitorMut` from `ast_visitor.rs` to walk it. The Rust transpiler emits this same AST.

Preprocessor lines recognised at the top level:
- `#define_import_path <name>`: overrides the file's name.
- `#import <name>` or `#import <name>::item`: records a dependency on `<name>`.

## ShaderComposer (`magician_rust::ShaderComposer`)

Stitches a base shader, the files it imports, and "modifier" functions into one WGSL module that has a generated entry point.

```rust
let mut composer = ShaderComposer::new();
composer.load_file_from_src("base", base_src)?;
composer.load_file_from_src("tint", tint_src)?;

let build = BuildInstructions {
    main_attribute: "@fragment",
    main_fn_name: "fs_final_main",
    input_types: &["VertexOutput"],   // generated main's params, passed to each modifier
    output_type: "vec4<f32>",
};
let wgsl: &String = composer.compile(CompilationInstructions {
    shaders: &["base".into(), "tint".into()],
    defs: &[("TINT".into(), "0.5".into())],   // textual replacements
    instructions: &build,
    ..Default::default()
});
```

Conventions inside the WGSL files (custom attributes that the composer consumes):
- `@public @default(<output_type>) fn ...() -> <output_type>`: produces the initial `result`. Exactly one must be reachable.
- `@main fn m(v0: In0, ..., prev: <output_type>) -> <output_type>`: a modifier. Modifiers from the listed `shaders` run in order: `result = m(v0, .., result);`.
- Imported files (`#import`) contribute only their `@public` items, and their `@main` is skipped.
- Results are cached by a hash of the instructions. An unknown import panics.
