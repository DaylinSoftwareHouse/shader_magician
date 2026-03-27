#import textures
#import vertex

struct Material {
    texture_id: u32,
    _buffer: vec3<u32>
};

@group(1) @binding(0) var<uniform> materials: array<Material, 256>;

@public
struct FragmentOutput {
    color: vec4<f32>
};

@public
@default(FragmentOutput)
fn def_fragment_output() -> FragmentOutput {
    var output: FragmentOutput;
    return output;
}

@main
fn fs_main(in: VertexOutput, out: FragmentOutput) -> FragmentOutput {
    let material = materials[in.mat_id];
    out.color = texture_sample(material.texture_id, in.uvs) * in.color;
    return out;
}