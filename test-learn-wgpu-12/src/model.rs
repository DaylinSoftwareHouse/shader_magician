use magician_vgpu::{BindableObject, ImmutableBuffer, VirtualGpu};

use crate::texture;

pub trait Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
}

impl Vertex for ModelVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Tangent and bitangent
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 11]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub struct Material {
    #[allow(unused)]
    pub name: String,
    #[allow(unused)]
    pub diffuse_texture: texture::Texture,
    #[allow(unused)]
    pub normal_texture: texture::Texture,
    pub bindable: BindableObject<shaders::common::Material>,
}

impl Material {
    pub fn new(
        vgpu: &VirtualGpu,
        name: &str,
        diffuse_texture: texture::Texture,
        normal_texture: texture::Texture,
    ) -> Self {
        let bindable = BindableObject
            ::<shaders::common::Material>
            ::from_inputs(vgpu, &(
                diffuse_texture.view.clone(), 
                diffuse_texture.sampler.clone(), 
                normal_texture.view.clone(), 
                normal_texture.sampler.clone()
            ));

        Self {
            name: String::from(name),
            diffuse_texture,
            normal_texture,
            bindable,
        }
    }
}

pub struct Mesh {
    #[allow(unused)]
    pub name: String,
    pub vertex_buffer: ImmutableBuffer<[ModelVertex]>,
    pub index_buffer: ImmutableBuffer<[u32]>,
    pub num_elements: u32,
    pub material: usize,
}

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}
