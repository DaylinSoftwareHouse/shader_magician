use bytemuck::{Pod, Zeroable};
use magician_macros::ShaderGroup;
use magician_rust::{Mat4, Vec3, Vec4};
use magician_vgpu::{BindableObject, BindableObjectCreator, Buffer};

#[derive(ShaderGroup)]
pub struct CameraInput {
    #[uniform] pub camera: Camera
}

impl <'a> BindableObjectCreator<'a> for CameraInput {
    type Inputs = &'a dyn Buffer<Type = Camera>;

    fn create_object(
        vgpu: &magician_vgpu::VirtualGpu, 
        inputs: Self::Inputs
    ) -> magician_vgpu::BindableObject<Self> where Self: Sized
    {
        let layout = vgpu.device().create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        count: None,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None
                        }
                    }
                ],
                label: None
            }
        );

        let bind_group = vgpu.device().create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &layout,
                label: None,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: inputs.buffer().as_entire_binding()
                    }
                ]
            }
        );

        BindableObject::new(bind_group, layout)
    }
}

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy)]
pub struct Camera {
    pub view_pos: Vec4,
    pub view_proj: Mat4
}


#[derive(ShaderGroup)]
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