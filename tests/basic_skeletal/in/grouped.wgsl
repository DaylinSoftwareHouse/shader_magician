#import textures
#import vertex

struct Material {
    texture_id: u32,
    _buffer: vec3<u32>
};

@group(1) @binding(0) var<uniform> materials: array<Material, 256>;

@main
fn fs_main(in: VertexOutput) -> vec4<f32> {
    let material = materials[in.mat_id];
    return texture_sample(material.texture_id, in.uvs) * in.color;
}