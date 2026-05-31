use magician_macros::*;
use magician_rust::*;


#[derive(ShaderGroup)]
pub struct CameraInput {
    #[uniform] camera: Camera
}

pub struct Camera {
    pub view_pos: Vec4,
    pub view_proj: Mat4
}


#[derive(ShaderGroup)]
pub struct LightInput {
    #[uniform] light: Light
}

pub struct Light {
    position: Vec3,
    color: Vec3
}


#[derive(ShaderLayout)]
pub struct VertexInput {
    #[location = 0] position: Vec3
}

#[allow(dead_code)]
#[derive(ShaderLayout)]
pub struct VertexOutput {
    #[builtin = "position"] clip_position: Vec4,
    #[location = 0] color: Vec3
}

#[shader]
pub fn light_vs_main(
    cam_in: CameraInput,
    light_in: LightInput,
    model: VertexInput
) -> VertexOutput {
    let scale = 0.25;
    let clip_position = cam_in.camera.view_proj * Vec4::from_vec3_w(model.position * scale + light_in.light.position, 1.0);
    let color = light_in.light.color;
    return VertexOutput { clip_position, color };
}

#[allow(dead_code)]
#[derive(ShaderLayout)]
pub struct FragmentOutput {
    #[location = 0] color: Vec4
}

#[shader]
pub fn light_fs_main(input: VertexOutput) -> FragmentOutput {
    return FragmentOutput { color: Vec4::from_vec3_w(input.color, 1.0) }
}
