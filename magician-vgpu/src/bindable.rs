use std::marker::PhantomData;

use getset::{Getters, MutGetters};
use magician_rust::{BindlessArray, Sampler, Texture2D, Vec4};

use crate::{Buffer, MutableBuffer, VirtualGpu};

#[derive(Getters, MutGetters)]
pub struct BindableObject<G: BindGroupProvider> {
    #[getset(get = "pub", get_mut = "pub")]
    bind_group: wgpu::BindGroup,
    #[getset(get = "pub", get_mut = "pub")]
    layout: wgpu::BindGroupLayout,
    _phantom: PhantomData<G>
}

impl <G: BindGroupProvider> BindableObject<G> {
    pub fn new(bind_group: wgpu::BindGroup, layout: wgpu::BindGroupLayout) -> Self {
        Self { bind_group, layout, _phantom: PhantomData::default() }
    }

    pub fn from_inputs<'a>(
        vgpu: &'a VirtualGpu, 
        inputs: &'a G::Input<'a>
    ) -> Self {
        let layout = G::layout(vgpu, wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX);
        return Self::new(G::group(vgpu, &layout, inputs), layout);
    }
}

pub trait BindGroupPart {
    type PartInput<'a>: ?Sized;

    /// Create an entry for a `wgpu::BindGroupLayout` for this part of the
    /// bind group.
    fn layout_entry(
        _vgpu: &VirtualGpu,
        binding: u32, 
        visibility: wgpu::ShaderStages
    ) -> wgpu::BindGroupLayoutEntry;

    /// Create a single `wgpu::BindGroupEntry` for a `wgpu::BindGroup` for
    /// this part of the bind group.
    fn group_entry<'a>(
        vgpu: &'a VirtualGpu,
        binding: u32,
        input: &'a Self::PartInput<'a>
    ) -> wgpu::BindGroupEntry<'a>;
}


pub trait BindGroupProvider {
    type Input<'a>;

    /// Create a `wgpu::BindGroupLayout` from a `VirtualGpu` ref, and some visibility flags.
    /// These visibilty flags tell WGPU what shaders should have access to this layout.
    fn layout(
        vgpu: &VirtualGpu,
        visibility: wgpu::ShaderStages
    ) -> wgpu::BindGroupLayout;

    /// Create a `wgpu::BindGroup` from a `VirtualGpu` ref, a layout and some inputs.
    /// The layout must correspond to one built by this provider or one similar
    /// to it.
    fn group<'a>(
        vgpu: &VirtualGpu,
        layout: &wgpu::BindGroupLayout,
        input: &'a Self::Input<'a>
    ) -> wgpu::BindGroup;
}

impl BindGroupPart for u32 {
    type PartInput<'a> = MutableBuffer<u32>;

    fn layout_entry(
        _vgpu: &VirtualGpu,
        binding: u32, 
        visibility: wgpu::ShaderStages
    ) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding, visibility,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None
            },
            count: None,
        }
    }

    fn group_entry<'a>(
        _vgpu: &VirtualGpu,
        binding: u32,
        input: &'a Self::PartInput<'a>
    ) -> wgpu::BindGroupEntry<'a> {
        wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: input.buffer(),
                offset: 0,
                size: None,
            }),
        }
    }
}

impl BindGroupPart for Vec4 {
    type PartInput<'a> = MutableBuffer<Vec4>;

    fn layout_entry(
        _vgpu: &VirtualGpu,
        binding: u32, 
        visibility: wgpu::ShaderStages
    ) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding, visibility,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None, // or Some(NonZeroU64::new(16).unwrap()) for vec4 (4 * 4 bytes)
            },
            count: None,
        }
    }

    fn group_entry<'a>(
        _vgpu: &VirtualGpu,
        binding: u32,
        input: &'a Self::PartInput<'a>
    ) -> wgpu::BindGroupEntry<'a> {
        wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: input.buffer(), // a wgpu::Buffer created with BufferUsages::UNIFORM
                offset: 0,
                size: None, // or Some(NonZeroU64::new(16).unwrap()) to be explicit
            }),
        }
    }
}

impl BindGroupPart for Texture2D {
    type PartInput<'a> = wgpu::TextureView;

    fn layout_entry(
        _vgpu: &VirtualGpu,
        binding: u32, 
        visibility: wgpu::ShaderStages
    ) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding, visibility,
            count: None,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
            }
        }
    }

    fn group_entry<'a>(
        _vgpu: &VirtualGpu,
        binding: u32,
        input: &'a Self::PartInput<'a>
    ) -> wgpu::BindGroupEntry<'a> {
        wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::TextureView(input),
        }
    }
}

impl BindGroupPart for Sampler {
    type PartInput<'a> = wgpu::Sampler;

    fn layout_entry(
        _vgpu: &VirtualGpu,
        binding: u32, 
        visibility: wgpu::ShaderStages
    ) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding, visibility,
            count: None,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        }
    }

    fn group_entry<'a>(
        _vgpu: &'a VirtualGpu,
        binding: u32,
        input: &'a Self::PartInput<'a>
    ) -> wgpu::BindGroupEntry<'a> {
        wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::Sampler(input),
        }
    }
}

impl BindGroupPart for BindlessArray<Texture2D> {
    type PartInput<'a> = Vec<&'a wgpu::TextureView>;

    fn layout_entry(
        _vgpu: &VirtualGpu,
        binding: u32, 
        visibility: wgpu::ShaderStages
    ) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding, visibility,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: std::num::NonZeroU32::new(128)
        }
    }

    fn group_entry<'a>(
        _vgpu: &'a VirtualGpu,
        binding: u32,
        input: &'a Self::PartInput<'a>
    ) -> wgpu::BindGroupEntry<'a> {
        wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::TextureViewArray(input)
        }
    }
}
