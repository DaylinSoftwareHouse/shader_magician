#import vertex

struct Material {
    texture_id: u32,
    _buffer: vec3<u32>
};

@group(1) @binding(0) var<uniform> materials: array<Material, 256>;

@group(2) @binding(0) var textures: binding_array<texture_2d<f32>>;
@group(2) @binding(1) var texture_sampler: sampler;

fn fs_main(in: VertexOutput) -> vec4<f32> {
    let material = materials[in.mat_id];
    return textureSample(textures[material.texture_id], texture_sampler, in.uvs) * in.color;
}