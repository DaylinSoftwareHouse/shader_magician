//! Shaders exercising transpiler features, rendered by `tests/features.rs`.

use magician_rust::*;
use magician_rust::macros::*;

#[allow(dead_code)]
#[derive(ShaderLayout)]
pub struct FullscreenInput {
    #[builtin = "vertex_index"] index: u32
}

#[allow(dead_code)]
#[derive(ShaderLayout)]
pub struct FullscreenOutput {
    #[builtin = "position"] clip_position: Vec4
}

/// A triangle covering the whole screen, positioned by vertex index.
#[shader("./shader_out", vertex)]
pub fn fullscreen_vs_main(input: FullscreenInput) -> FullscreenOutput {
    let mut pos = Vec2::new(-1.0, -1.0);
    match input.index {
        1u32 => pos = Vec2::new(3.0, -1.0),
        2u32 => pos = Vec2::new(-1.0, 3.0),
        _ => {}
    }
    FullscreenOutput { clip_position: Vec4::from_vec2_zw(pos, 0.0, 1.0) }
}

#[allow(dead_code)]
#[derive(ShaderLayout)]
pub struct FeaturesOutput {
    #[location = 0] color: Vec4
}

/// 1.0 for channels 0 and 2, otherwise 0.0.
pub fn channel(i: u32) -> f32 {
    let mut v = 0.0;
    match i {
        0u32 | 2u32 => { v = 1.0; }
        _ => {}
    }
    v
}

/// Renders magenta, (1, 0, 1, 1), only if literal suffixes, casts, `|` match patterns,
/// trailing returns, identity matrices, comparison functions, camel case builtins and
/// color swizzles all transpile correctly.
#[shader("./shader_out", fragment)]
pub fn features_fs_main() -> FeaturesOutput {
    let m = Mat4::identity();
    let c = m * Vec4::new(channel(0u32), channel(1u32), channel(2 as u32), 1.0f32);
    let on = greater_than_vec3(c.rgb(), Vec3::splat(0.5));
    let alpha = c.a() * inverse_sqrt(1.0);
    FeaturesOutput {
        color: Vec4::new(select(0.0, 1.0, on.r()), select(0.0, 1.0, on.g()), select(0.0, 1.0, on.b()), alpha)
    }
}
