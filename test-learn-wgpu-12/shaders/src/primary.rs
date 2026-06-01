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
    #[location = 0] position: Vec3,
    #[location = 1] tex_coords: Vec2,
    #[location = 2] normal: Vec3,
    #[location = 3] tangent: Vec3,
    #[location = 4] bitangent: Vec3
}


#[derive(ShaderLayout)]
pub struct InstanceInput {
    #[location = 5] mm0: Vec4,
    #[location = 6] mm1: Vec4,
    #[location = 7] mm2: Vec4,
    #[location = 8] mm3: Vec4,
    #[location = 9] nm0: Vec3,
    #[location = 10] nm1: Vec3,
    #[location = 11] nm2: Vec3
}


#[allow(unused)]
#[derive(ShaderLayout)]
pub struct VertexOutput {
    #[builtin = "position"] clip_position: Vec4,
    #[location = 0] tex_coords: Vec2,
    #[location = 1] tangent_position: Vec3,
    #[location = 2] tangent_light_position: Vec3,
    #[location = 3] tangent_view_position: Vec3
}


#[shader("./shader_out")]
pub fn primary_vs_main(
    cam_in: CameraInput,
    light_in: LightInput,
    model: VertexInput,
    instance: InstanceInput
) -> VertexOutput {
    let mm = Mat4::new(instance.mm0, instance.mm1, instance.mm2, instance.mm3);
    let nm = Mat3::new(instance.nm0, instance.nm1, instance.nm2);

    let world_normal = normalize_vec3(nm * model.normal);
    let world_tangent = normalize_vec3(nm * model.tangent);
    let world_bitangent = normalize_vec3(nm * model.bitangent);

    let tangent_matrix = transpose_mat3(Mat3::new(world_tangent, world_bitangent, world_normal));
    let world_position = mm * Vec4::from_vec3_w(model.position, 1.0);

    return VertexOutput { 
        clip_position: cam_in.camera.view_proj * world_position, 
        tex_coords: model.tex_coords, 
        tangent_position: tangent_matrix * world_position.xyz(), 
        tangent_light_position: tangent_matrix * cam_in.camera.view_pos.xyz(), 
        tangent_view_position: tangent_matrix * light_in.light.position 
    };
}


#[derive(ShaderGroup)]
pub struct Material {
    t_diffuse: Texture2D,
    s_diffuse: Sampler,
    t_normal: Texture2D,
    s_normal: Sampler
}

#[allow(unused)]
#[derive(ShaderLayout)]
pub struct FragmentOutput {
    #[location = 0] color: Vec4
}

#[shader("./shader_out")]
pub fn primary_fs_main(
    material: Material,
    light_in: LightInput,
    input: VertexOutput
) -> FragmentOutput {
    let object_color = textureSample(material.t_diffuse, material.s_diffuse, input.tex_coords);
    let object_normal = textureSample(material.t_normal, material.s_normal, input.tex_coords);

    let ambient_strength = 0.1;
    let ambient_color = light_in.light.color * ambient_strength;

    let tangent_normal = object_normal.xyz() * 2.0 - Vec3::new(1.0, 1.0, 1.0);
    let light_dir = normalize_vec3(input.tangent_light_position - input.tangent_position);
    let view_dir = normalize_vec3(input.tangent_view_position - input.tangent_position);
    let half_dir = normalize_vec3(view_dir + light_dir);

    let diffuse_strength = max_f32(dot_vec3(tangent_normal, light_dir), 0.0);
    let diffuse_color = light_in.light.color * diffuse_strength;

    let spec_strength = pow(max_f32(dot_vec3(tangent_normal, half_dir), 0.0), 32.0);
    let spec_color = spec_strength * light_in.light.color;

    let color_sum = ambient_color + diffuse_color + spec_color;
    let result = color_sum * object_color.xyz();

    return FragmentOutput { color: Vec4::from_vec3_w(result, object_color.w()) };
}
