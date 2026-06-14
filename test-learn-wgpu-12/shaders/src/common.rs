use bytemuck::{Pod, Zeroable};
use magician_macros::ShaderGroup;
use magician_rust::{Mat4, Vec3, Vec4};
use magician_vgpu::macros::{BindableObject, BufferObject};

#[derive(ShaderGroup, BindableObject)]
pub struct CameraInput {
    #[uniform] pub camera: Camera
}

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy)]
pub struct Camera {
    pub view_pos: Vec4,
    pub view_proj: Mat4
}


#[derive(ShaderGroup, BindableObject)]
pub struct LightInput {
    #[uniform] pub light: Light
}

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy)]
pub struct Light {
    pub position: Vec3,
    pub _pad0: u32,
    pub color: Vec3,
    pub _pad1: u32
}