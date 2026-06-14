use magician_rust::{Mat4, Vec2, Vec3, Vec4, macros::{ShaderGroup, ShaderLayout}};

#[derive(ShaderGroup)]
pub struct CameraInput {
    #[uniform] pub camera_vp: Mat4
}

#[derive(ShaderLayout)]
pub struct VertexInput {
    #[location = 0] pub position: Vec3,
    #[location = 1] pub uvs: Vec2,
    #[location = 2] pub normal: Vec3
}

#[derive(ShaderLayout)]
pub struct InstanceInput {
    #[location = 5] pub model_matrix_0: Vec4,
    #[location = 6] pub model_matrix_1: Vec4,
    #[location = 7] pub model_matrix_2: Vec4,
    #[location = 8] pub model_matrix_3: Vec4,
    #[location = 9] pub mat_id: u32
}

#[derive(ShaderLayout)]
pub struct VertexOutput {
    #[builtin = "position"] pub clip_position: Vec4,
    #[location = 0] pub uvs: Vec2,
    #[location = 1] pub color: Vec4,
    #[location = 2] pub world_normal: Vec3,
    #[location = 3] pub world_position: Vec3,
    #[location = 4] pub mat_id: u32
}

// #[shader("./shader_out", vertex)]
// pub fn static_vertex_main(
//     camera: CameraInput,
//     model: VertexInput,
//     instance: InstanceInput
// ) -> VertexOutput {
//     let model_matrix: Mat4 = Mat4::new(
//         instance.model_matrix_0, 
//         instance.model_matrix_1, 
//         instance.model_matrix_2, 
//         instance.model_matrix_3
//     );

//     let model_matrix_3x3: Mat3 = Mat3::new(
//         instance.model_matrix_0.xyz(),
//         instance.model_matrix_1.xyz(),
//         instance.model_matrix_2.xyz()
//     );

//     let uvs: Vec2 = model.uvs;
//     let color: Vec4 = test_sample(uvs);
//     let world_normal: Vec3 = normalize_vec3(model_matrix_3x3 * model.normal);
//     let world_position: Vec4 = model_matrix * Vec4::from_vec3_w(model.position, 1.0);
//     let clip_position: Vec4 = camera.camera_vp * world_position;

//     return VertexOutput { 
//         clip_position: clip_position,
//         uvs: uvs,
//         color: color,
//         world_normal: world_normal,
//         world_position: world_position.xyz(),
//         mat_id: instance.mat_id
//     }
// }
