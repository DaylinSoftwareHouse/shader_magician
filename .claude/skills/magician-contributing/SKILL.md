---
name: magician-contributing
description: Work on the shader_magician workspace itself (magician-ast, magician-macros, magician-rust's build pipeline and Rust→WGSL transpiler, magician-vgpu and its derive macros). Use when fixing or extending the transpiler (new types, builtins, expressions, statements), changing build.rs target discovery or dependency resolution, editing the WGSL parser or composer, changing VirtualGpu, buffer, pipeline, or frame code, or verifying changes against the example crates.
---

# Contributing to shader_magician

A Cargo workspace on edition 2024, wgpu 29, naga 29, syn 2. For *using* the crates, see the `magician-shaders` and `magician-vgpu` skills. This skill covers changing them.

## Crate map

| Crate | Role | Key files |
|---|---|---|
| `magician-ast` | WGSL parser and serializer: top-level items plus a full body AST | `elements.rs` (`ShaderElement`, `to_wgsl`), `parser.rs` (top level, `#import`), `blocks/wgsl_ast.rs` (body AST), `blocks/block_parser.rs`, `blocks/ast_visitor.rs` |
| `magician-macros` | proc macros used in shader crates | `#[shader]` (embeds `<out>/<fn>.wgsl` as `SHADER_<fn>`, or `compile_error!` on `ERROR\n`), `#[group]`, and `ShaderLayout`/`ShaderGroup`, which are **marker derives that emit nothing** and exist only so the attributes parse |
| `magician-rust` | CPU-side WGSL math mirror plus the build-time transpiler | `build/` (pipeline), `transpiler/` (syn → magician-ast), `math.rs`, `textures.rs`, `types.rs` (`BindlessArray`), `composer.rs` |
| `magician-vgpu` (+ `macros/`) | wgpu wrapper | `lib.rs` (`VirtualGpu`), `bindable.rs`, `pipeline.rs`, `frame.rs`, `pass.rs`, `attachments.rs`, `buffers/`, `textures/`; derive macros `BindableObject`, `UniformBufferObject` |
| `test-shaders` | scratch shader crate | mostly commented out |
| `test-learn-wgpu-12` + `/shaders` | end-to-end example (learn-wgpu tutorial 12, normal mapping) | the real integration test |

## How a shader is built (`magician_rust::build`)

1. **Discover targets** (`build/mod.rs::collect_shader_fns`): parse every `.rs` under `src`, take top-level `fn`s with `#[shader("path", stage)]`, and name them by file path (`crate::primary::primary_vs_main`).
2. **Index** (`build/index.rs::ProjectIndex`): every named item gets a `crate::module::Name` key, impls are bucketed by self type, and `use` statements are recorded as `name -> path` (paths starting with `magician_` are skipped). Glob `use` is ignored, and `use x as y` hits a `todo!`.
3. **Resolve** (`build/resolver.rs`): BFS over identifiers (`build/visit.rs::IdentCollector`), qualifying them by current module or `use` map, then a Kahn topological sort. Unknown names, such as math builtins, drop out silently.
4. **Stitch** (`build/stitch.rs`): the ordered items become one `syn::File`.
5. **Transpile** (`transpiler/`): `Transpiler::transpile_raw` turns items into `ShaderElement`s. `#[derive(ShaderGroup)]` structs are parked in `unfinished_globals` until an entry-point param with `#[group = N]` turns them into globals. `transpile_wgsl` reparses with naga, validates, and re-emits WGSL.
6. **Write**: success goes to `<out>/<fn>.wgsl`. A panic anywhere is caught by `catch_unwind` and written as `ERROR\n<msg>`, which the `#[shader]` macro surfaces as a compile error. With a debug dir, the build also writes `extracted_<fn>.rs`, `extracted_<fn>.syntree`, and `raw_wgsl_<fn>.wgsl`.

Unsupported syntax therefore panics with `todo!`/`panic!`. That is the intended error channel, so write clear panic messages, because they become the user's compile error.

## Common extension points

- **New Rust type → WGSL type**: `translate_ty_name` (and generics such as `BindlessArray` in `convert_ty`) in `transpiler/global.rs`, plus the Rust type in `math.rs`/`textures.rs`/`types.rs`. If it can be bound, add a `BindGroupPart` impl in `magician-vgpu/src/bindable.rs`.
- **Constructor**: `convert_constructor` in `transpiler/expr.rs`. `identity` and `zero` are special-cased; every other method name is dropped and its arguments go to the typed WGSL constructor.
- **Builtin function**: add the Rust stub to `math.rs` named `wgsl_name_in_snake_case_<typesuffix>`. `convert_builtin_fn_name` in `expr.rs` drops the suffix and camelCases the rest, so most new builtins need no transpiler change. Functions defined in the shader crate are never renamed (`Transpiler::is_user_fn`). Add a test to `transpiler/mod.rs` for anything unusual.
- **Expression/statement**: `convert_expr` in `expr.rs` and `convert_stmt`/`convert_stmt_expr` in `stmts.rs`, emitting nodes from `magician_ast` (`Expression`, `Statement`). If the AST lacks a node, add it to `wgsl_ast.rs` with its emitter in `ast_visitor.rs` (`emit_block_inner` and the expression emitter next to it) and the `walk_*` visitor functions.
- **Group attribute** (`#[read]`/`#[write]` storage buffers are still TODO): `convert_global` in `global.rs` turns any path attribute into `var<attr>`. Also update the `ShaderGroup` derive's `attributes(...)` in `magician-macros` and the `BindableObject` derive in `magician-vgpu/macros`.
- **Pipeline options** (topology, blend, cull, multiple targets): `PipelineBuilder` in `magician-vgpu/src/pipeline.rs`. These values are currently hard-coded.

Known gaps worth knowing before you change related code (see also `todo` at the repo root): non-swizzle method calls are emitted verbatim, `for` loops, tuples, enums, and `if`/`match` as values are unsupported, struct names are global per shader (bare idents), group field names become global WGSL names, and the order of top-level WGSL items comes from `AHashMap` iteration, so it can change between builds (compare outputs order-independently).

## Verifying changes

```bash
cargo check --workspace
```

```bash
cargo build -p shaders
```

```bash
cargo test -p magician-vgpu
```

- `cargo build -p shaders` regenerates `test-learn-wgpu-12/shaders/shader_out/*.wgsl` and `shader_dbg/`. These directories are gitignored, so diff them against a pre-change copy (`cp -r test-learn-wgpu-12/shaders/shader_out <scratch>/before`) to check a transpiler change. Shaders that still validate and produce equivalent output are the regression test.
- To test a new language feature, add a temporary `#[shader]` fn in `test-shaders/src/` (with `pub mod` in `lib.rs`), build `-p test-shaders`, and read `shader_out/<fn>.wgsl` plus `shader_dbg/raw_wgsl_<fn>.wgsl`. Remove it afterwards or turn it into a kept example.
- The vgpu tests include a headless GPU round-trip that skips itself when no adapter exists.
- `cargo run -p tutorial12-camera` opens the example window, which is the visual end-to-end check. Use the `run` skill to drive and screenshot it.
- Transpiler unit tests live at the bottom of `magician-rust/src/transpiler/mod.rs`. Each one transpiles a small shader with `transpile(src)`, which also runs naga validation, and asserts on the raw WGSL. Add one for every transpiler change: `cargo test -p magician-rust`.
- `cargo test -p test-shaders` renders `test-shaders/src/features.rs` headless and checks the pixels, which proves the generated WGSL computes the right values on a GPU. Extend `features_fs_main` when a feature affects values rather than just syntax.
- Downstream check: `../Gearbox` (its `shaders` crate is transpiled by this code, plus `cargo test` and headless screenshots through `GEARBOX_HEADLESS=640x480 GEARBOX_SCREENSHOT=out.png cargo run --example shapes`) and `../iso_ui` (renders through Gearbox; `scripts/pngdiff.py before.png after.png 0` compares screenshots). Take baseline screenshots before changing anything.

## Code conventions (match the surrounding code)

- Explicit `return x;` at function ends is common in this codebase. Short `// lowercase comment` lines above logical steps, and `///` docs on public items.
- `getset` (`#[getset(get = "pub", get_mut = "pub")]`) for struct accessors in vgpu. Builders take `mut self` and return `Self`.
- `anyhow::Result` for fallible vgpu APIs. The build pipeline panics with a descriptive message instead (see above).
- `ahash::AHashMap` for maps in hot or internal code.
- Keep the WGSL mapping for every public `math.rs` function valid. If a Rust helper can't be transpiled, document that or fix the mapping.
