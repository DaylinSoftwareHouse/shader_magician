@public @group(2) @binding(0) var textures: binding_array<texture_2d<f32>>;
@public @group(2) @binding(1) var texture_sampler: sampler;

@public
fn sample_texture(index: u32, uv: vec2<f32>) -> vec4<f32> {
    return textureSample(textures[index], texture_sampler, uv);
}
