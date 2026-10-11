---
name: magician-shaders
description: Write GPU shaders in Rust with magician-rust's `#[shader]` macro and build.rs transpiler (Rust → WGSL via naga). Use when adding or editing a `#[shader("...", vertex|fragment)]` function, a `ShaderLayout`/`ShaderGroup` struct, setting up a shader crate's Cargo.toml and build.rs, or debugging a transpile error ("Failed to transpile wgsl", ERROR in shader_out/*.wgsl, missing SHADER_* const).
---

# Writing shaders in Rust with magician-rust

Shader functions are ordinary Rust that type-checks against `magician_rust`'s WGSL-mirroring math types. They are never run on the CPU. A build script finds every `#[shader]` function, pulls in what it depends on, transpiles it to WGSL, validates it with naga, and writes `<out>/<fn_name>.wgsl`. The `#[shader]` macro then embeds that file as `pub const SHADER_<fn_name>: &str`, or turns a failed transpile into a `compile_error!`.

Reference material:
- `references/translation.md`: the full Rust → WGSL type, constructor, and function mapping.
- `references/wgsl-composer.md`: the WGSL-side tools (`magician_ast`, `ShaderComposer`) for composing hand-written WGSL.

## 1. Crate setup

Shaders live in their own crate, because build.rs scans that crate's whole `src/` tree.

```toml
# Cargo.toml
[dependencies]
magician-rust   = { path = "../magician-rust" }     # or git/version
magician-macros = { path = "../magician-macros" }   # optional, also re-exported as magician_rust::macros
# only if the same structs also derive magician-vgpu's BindableObject / UniformBufferObject:
magician-vgpu = { path = "../magician-vgpu" }
wgpu = "29.0"          # the vgpu derives expand to plain `wgpu::` paths
bytemuck = "1"         # for #[derive(Pod, Zeroable)] on uniform structs

[build-dependencies]
magician-rust = { path = "../magician-rust" }
```

```rust
// build.rs
fn main() {
    magician_rust::build(
        "src".into(),               // crate source root to scan
        "shader_out".into(),        // where <fn>.wgsl files go
        Some("shader_dbg".into()),  // debug dumps, or None
    );
}
```

The first argument of `#[shader("./shader_out", ...)]` must point to the same folder as build.rs's `out_path`. Both are resolved relative to the crate's manifest dir. Add `shader_out/` and `shader_dbg/` to `.gitignore`.

## 2. Anatomy of a shader

```rust
use magician_rust::*;            // Vec*, Mat*, Texture2D, Sampler, math fns
use magician_rust::macros::*;    // shader, ShaderLayout, ShaderGroup

// Bind group: each field becomes a WGSL global at @group(N) @binding(field index).
#[derive(ShaderGroup)]
pub struct CameraInput {
    #[uniform] pub camera: Camera,   // -> var<uniform> camera: Camera;
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Camera {                  // plain struct -> WGSL struct
    pub view_pos: Vec4,
    pub view_proj: Mat4,
}

// Stage I/O: ShaderLayout is a marker. Only #[location]/#[builtin] matter.
#[derive(ShaderLayout)]
pub struct VertexInput  { #[location = 0] position: Vec3, #[location = 1] uv: Vec2 }

#[derive(ShaderLayout)]
pub struct VertexOutput { #[builtin = "position"] clip: Vec4, #[location = 0] uv: Vec2 }

#[shader("./shader_out", vertex)]
pub fn my_vs_main(#[group = 0] cam: CameraInput, model: VertexInput) -> VertexOutput {
    let clip = cam.camera.view_proj * Vec4::from_vec3_w(model.position, 1.0);
    return VertexOutput { clip, uv: model.uv };
}
```

Rules that follow from how the transpiler works:
- **Stage** is `vertex` or `fragment`. It becomes the `@vertex`/`@fragment` attribute, and the WGSL entry point keeps the Rust function's name. Pass that name as `ShaderSource::main_function`.
- **Group params** (`#[group = N] x: SomeShaderGroup`) are removed from the WGSL signature. Each struct field becomes a global, and `x.field` is rewritten to the bare global name `field`. As a result:
  - Field names must be unique across every group a shader uses.
  - A group param must not share a name with any of those fields.
  - Bindings are numbered in field declaration order, starting at 0.
- A group struct the entry point names without `#[group = N]` is silently dropped. Always give the index.
- `#[uniform]` gives `var<uniform>`. Fields with no attribute (`Texture2D`, `Sampler`, `BindlessArray<Texture2D>`) give a plain `var`. `#[read]`/`#[write]` are not implemented yet.
- Uniform structs must match WGSL alignment. Pad each `Vec3` to 16 bytes (see `Light { position: Vec3, _pad0: u32, ... }` in `test-learn-wgpu-12/shaders/src/common.rs`).
- Helpers: plain `fn`s, structs, and impls in the same crate are found by name resolution and emitted automatically, whichever module they live in. Code from other crates is not, apart from `magician_rust` builtins.
- If one shader's dependency graph pulls in two structs with the same name (for example `VertexOutput` from two modules), they collide. Keep per-shader I/O structs in that shader's module.

## 3. Supported Rust subset (verified)

Works (each item is covered by a test in `magician-rust/src/transpiler/mod.rs` or `test-shaders/tests/features.rs`):
- `let`/`let mut` (becomes `let`/`var`, optionally typed), arithmetic/comparison/logic/bit ops, compound assignment, `if`/`else if`/`else`, `while`, `loop`, `break`, `continue`, `return`, nested blocks.
- A trailing expression returns from a function: `fn half(v: f32) -> f32 { v * 0.5 }`.
- `match` on literals, with `|` patterns (`1 | 2 =>` becomes `case 1, 2:`), `_` (becomes `default`), and arms that are blocks or single expressions (`1 => x = 2.0,`).
- Literal suffixes: `1u32` becomes `1u`, `2i32` becomes `2i`, `1.5f32` becomes `1.5f`. Unsuffixed literals stay abstract.
- Casts: `x as f32` becomes `f32(x)`. `as usize`/`as isize` are dropped, so `arr[i as usize]` works.
- Constructors keep their scalar type: `UVec2::new(1, 2)` becomes `vec2<u32>(1, 2)`. `Mat4::identity()` and `MatN::zero()`/`VecN::zero()` work.
- Built-in functions with multi-word names become WGSL camelCase (`inverse_sqrt` becomes `inverseSqrt`, `count_one_bits_u32` becomes `countOneBits`). `equal_vec3`/`less_than_vec3`/... become the `==`/`<`/... operators.
- Struct literals including shorthand fields, array literals `[a, b, c]`, field access, indexing, fixed arrays `[T; N]`, calls to your own helper fns (snake_case helper names are kept as-is).
- The `xyzw` swizzle methods that `math.rs` defines and their `rgba` aliases (`.xyz()`, `.rgb()`, `.a()`, ...; see the reference).

**Does not work:**

| Write this | Not this | Why |
|---|---|---|
| `Mat4::new(c0, c1, c2, c3)` | `Mat4::from_cols_array(&[..])` | references are unsupported |
| `while i < n { ..; i += 1; }` | `for i in 0..n` | `for` is not supported |
| `normalize_vec3(v)`, `max_f32(a, b)` | `v.normalize()`, `a.max(b)` | non-swizzle methods are emitted as `v.normalize()`, which is not WGSL |
| `let x = if c { a } else { b }` → `select(b, a, c)` | `if`/`match` used as a value | only statement `if`/`match` are converted |
| `1u32` | `1u8`, `1i64`, ... | WGSL has no such type, so the transpiler reports an error |

Also unsupported: tuples, enums, `const`/`static` items, macros (hard limit), references and pointers, traits, closures, `impl Trait`, match guards and binding patterns.

`textureSample(t, s, uv)` is the texture builtin. It panics if called on the CPU, which is fine because shaders are never called.

## 4. Building and debugging

```bash
cargo build -p <shader-crate>
```

- A failure appears as `error: Failed to transpile wgsl: ParseError { message: ... }` at the `#[shader]` attribute. The same text is written into `shader_out/<fn>.wgsl` after an `ERROR` line.
- Read `shader_dbg/raw_wgsl_<fn>.wgsl` first. It is the transpiler output *before* naga, and naga's error span offsets point into it.
- `shader_dbg/extracted_<fn>.rs` shows exactly which Rust items the resolver pulled in. Check it when a helper or struct is missing or duplicated.
- `cannot find value SHADER_<fn>`: build.rs did not produce `<fn>.wgsl`. Check that build.rs exists, that the `#[shader]` path matches `out_path`, and that the function is a top-level `fn` in a file under `src/` (functions inside inline `mod {}` blocks are not collected as targets).
- build.rs re-runs whenever anything under `src/` changes. If outputs look stale, `touch build.rs`.

## 5. Using the output

```rust
// from the shader crate (the macro-generated const):
ShaderSource { source: shaders::primary::SHADER_primary_vs_main.into(), main_function: "primary_vs_main".into() }
// or straight from disk, as test-learn-wgpu-12 does:
ShaderSource { source: include_str!("../shaders/shader_out/primary_vs_main.wgsl").into(), main_function: "primary_vs_main".into() }
```

To wire these into pipelines, bind groups, and draws, see the `magician-vgpu` skill. A `#[derive(ShaderGroup, BindableObject)]` struct is the single definition both sides share.

Full working example: `test-learn-wgpu-12/shaders/src/{common,primary,light}.rs`.
