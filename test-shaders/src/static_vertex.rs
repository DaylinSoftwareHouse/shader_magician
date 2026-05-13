use magician_core::*;
use magician_macros::shader;

use crate::textures::test_sample;


pub struct CameraInput {
    pub camera_vp: Uniform<Mat4>
}

pub struct VertexInput {
    pub position: Location<0, Vec3>,
    pub uvs: Location<1, Vec2>,
    pub normal: Location<2, Vec3>
}

pub struct InstanceInput {
    pub model_matrix_0: Location<5, Vec4>,
    pub model_matrix_1: Location<6, Vec4>,
    pub model_matrix_2: Location<7, Vec4>,
    pub model_matrix_3: Location<8, Vec4>,
    pub mat_id: Location<9, u32>
}

pub struct VertexOutput {
    pub clip_position: BuiltIn<{ BuiltInTy::Position as u32 }, Vec4>,
    pub uvs: Location<0, Vec2>,
    pub color: Location<1, Vec4>,
    pub world_normal: Location<2, Vec3>,
    pub world_position: Location<3, Vec3>,
    pub mat_id: Location<4, u32>
}

#[shader]
pub fn static_vertex_main(
    camera: Group<CameraInput>,
    model: VertexInput,
    instance: InstanceInput
) -> VertexOutput {
    let model_matrix: Mat4 = Mat4::new(
        *instance.model_matrix_0, 
        *instance.model_matrix_1, 
        *instance.model_matrix_2, 
        *instance.model_matrix_3
    );

    let model_matrix_3x3: Mat3 = Mat3::new(
        instance.model_matrix_0.xyz(),
        instance.model_matrix_1.xyz(),
        instance.model_matrix_2.xyz()
    );

    let uvs: Vec2 = *model.uvs;
    let color: Vec4 = test_sample(uvs);
    let world_normal: Vec3 = normalize_vec3(model_matrix_3x3 * *model.normal);
    let world_position: Vec4 = model_matrix * Vec4::from_vec3_w(*model.position, 1.0);
    let clip_position: Vec4 = *camera.camera_vp * world_position;

    return VertexOutput { 
        clip_position: BuiltIn::new(clip_position),
        uvs: Location::new(uvs),
        color: Location::new(color),
        world_normal: Location::new(world_normal),
        world_position: Location::new(world_position.xyz()),
        mat_id: Location::new(*instance.mat_id)
    }
}
