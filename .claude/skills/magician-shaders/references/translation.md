# Rust → WGSL translation reference

The source of truth is `magician-rust/src/transpiler/{global,expr,stmts}.rs` and `magician-rust/src/math.rs`. If this file disagrees with the code, the code is right.

## Types (`translate_ty_name` in global.rs)

| Rust | WGSL |
|---|---|
| `f32`, `i32`, `u32`, `bool` | unchanged |
| `Vec2/3/4`, `DVec*`, `IVec*`, `UVec*`, `BVec*` | `vecN<f32/f64/i32/u32/bool>` |
| `Mat2/3/4`, `DMat*` (`IMat*`, `UMat*`, `BMat*` are mapped but have no Rust type) | `matNxN<f32/f64/...>` |
| `Texture2D` | `texture_2d<f32>` |
| `Sampler` | `sampler` |
| `BindlessArray<T>` | `binding_array<T>` (group field only, 128 slots at runtime) |
| `[T; N]` (N must be an integer literal) | `array<T, N>` |
| any other path | its last segment, unchanged, so your own structs keep their names |

References, slices, tuples, pointers, `impl Trait`, and fn types all panic with `todo!`.

## Constructors (`convert_constructor` in expr.rs)

A call to `X::anything(args)`, where `X` is one of the vector or matrix types above, becomes the typed constructor `vecN<T>(args)` or `matNxN<T>(args)`. `identity` and `zero` are special-cased. For every other method the name is thrown away, so the arguments must be a valid WGSL constructor call:

| Rust | WGSL | OK? |
|---|---|---|
| `Vec3::new(x, y, z)` | `vec3<f32>(x, y, z)` | yes |
| `UVec2::new(1, 2)` | `vec2<u32>(1, 2)` | yes |
| `Vec3::splat(s)` | `vec3<f32>(s)` | yes |
| `Vec4::from_vec3_w(v, w)` | `vec4<f32>(v, w)` | yes |
| `Vec3::from_vec2_z(v, z)`, `Vec4::from_vec2_zw(v, z, w)` | `vec3<f32>(v, z)`, `vec4<f32>(v, z, w)` | yes |
| `Mat3::new(c0, c1, c2)` | `mat3x3<f32>(c0, c1, c2)` | yes (columns) |
| `Mat4::identity()` | `mat4x4<f32>(vec4<f32>(1.0, 0.0, 0.0, 0.0), ...)` | yes |
| `Mat4::zero()` (or any `X::zero()`) | `mat4x4<f32>()` | yes |
| `Mat4::from_cols_array(&[..])` | n/a | **no**: references are unsupported |

Struct literals `S { a, b: expr }` become `S(a, b)`, with fields reordered to declaration order.

## Swizzles

A method call whose name is 4 or fewer characters, all from `xyzwrgba`, is emitted as a field (`v.xyz()` becomes `v.xyz`). The Rust type still has to define the method so the code type-checks. `math.rs` currently defines:

- `Vec2`: `x y xx xy yx yy`
- `Vec3`: `x y z xy xz yz xyz zyx`
- `Vec4`: `x y z w xy zw xyz xyzw wzyx`
- `DVec*`/`IVec*`/`UVec*`: `x y z w` plus `xy` (and `xyz` on 4-wide)
- `BVec*`: `x y z w`

Every type also has `rgba` aliases of those swizzles (`r g b a rg rgb rgba ba bgr abgr ...`, added by `impl_rgba_swizzles!` in `math.rs`), which emit WGSL's `.rgb` and so on. To add a swizzle, add the method to `math.rs`.

Any other method call is emitted literally as `recv.method(args)`, which is invalid WGSL.

## Free functions (`convert_builtin_fn_name` in expr.rs)

Calls to functions defined in the shader's own crate keep their names. For anything else, only the last path segment is kept (`magician_rust::sqrt` becomes `sqrt`), then the name is converted:
1. A trailing `_t`/`_cw` variant marker after a type suffix is dropped (`mix_vec3_t`, `pow_vec3_cw`), as is a trailing `_wgsl` (`round_wgsl`).
2. A trailing type suffix (`vec2..4`, `dvec*`, `ivec*`, `uvec*`, `bvec*`, `mat2..4`, `dmat*`, `f32`, `u32`, `i32`) is dropped.
3. The remaining words are joined in camelCase (`inverse_sqrt` becomes `inverseSqrt`, `quantize_to_f16` becomes `quantizeToF16`).

Every built-in in `math.rs` translates this way, for example `abs_* acos atan2 clamp_* count_one_bits_* cross determinant_* dot_* extract_bits_* face_forward_* first_leading_bit_* inverse_sqrt* length_* max_* min_* mix* normalize_* pow* quantize_to_f16 reflect_* reverse_bits_* round* select* smoothstep* transpose_* pack*/unpack* textureSample`.

`equal_* not_equal_* less_than_* less_than_equal_* greater_than_* greater_than_equal_*` become the `== != < <= > >=` operators, which give a `vecN<bool>`.

`frexp` and `modf` return tuples, so they can't be used.

## Statements (stmts.rs)

| Rust | WGSL |
|---|---|
| `let x = e;` / `let x: T = e;` | `let x = e;` / `let x: T = e;` |
| `let mut x = e;` | `var x = e;` |
| `x = e;`, `x += e;` (all compound ops) | same |
| `if c {..} else if d {..} else {..}` | same |
| `while c {..}` | `while (c) {..}` |
| `loop {..}` | `loop {..}` |
| `match e { 1 \| 2 => x = 1.0, _ => {..} }` | `switch e { case 1, 2: { x = 1.0; } default: {..} }` (literal, `\|` and `_` patterns only; no guards) |
| trailing expression `v * 2.0` in a fn with a return type | `return (v * 2.0);` |
| `x as f32` / `x as usize` | `f32(x)` / `x` |
| `1u32`, `2i32`, `1.5f32`, `1f16` | `1u`, `2i`, `1.5f`, `1h` |
| `[a, b, c]` | `array(a, b, c)` |
| `return e;` / `break;` / `continue;` | same |
| `{ .. }` | nested block |
| `for`, `let else`, `if let`, nested items, macros, `if`/`match` as values | unsupported |

## Attributes

| Rust | WGSL |
|---|---|
| `#[location = N]` on a field or param | `@location(N)` |
| `#[builtin = "position"]` (any builtin name) | `@builtin(position)` |
| `#[group = N]` on an entry-point param of a `ShaderGroup` type | fields become `@group(N) @binding(i)` globals |
| `#[uniform]` on a group field | `var<uniform>` |
