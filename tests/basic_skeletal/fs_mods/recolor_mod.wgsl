#import vertex

@main
fn recolor(in: VertexOutput, color: vec4<f32>) -> vec4<f32> {
    return vec4(color.r, color.b, color.g, color.a);
}
