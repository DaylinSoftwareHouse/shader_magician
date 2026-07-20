use crate::*;

#[derive(Clone, Copy, Debug)]
pub struct Texture2D;

#[derive(Clone, Copy, Debug)]
pub struct Sampler;

#[allow(unused, nonstandard_style)]
pub fn textureSample(texture: Texture2D, sampler: Sampler, uv: Vec2) -> Vec4 {
    todo!("Texture sampling is not implemented yet")
}