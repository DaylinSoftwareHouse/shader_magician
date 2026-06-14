use std::marker::PhantomData;

use getset::{Getters, MutGetters};

use crate::{VirtualGpu};

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

    pub fn from_inputs(
        vgpu: &VirtualGpu, 
        inputs: &G::Input
    ) -> Self {
        let layout = G::layout(vgpu, wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX);
        let group = G::group(vgpu, &layout, inputs);
        return Self::new(group, layout);
    }
}


pub trait BindGroupPart {
    type PartInput;

    /// Create an entry for a `wgpu::BindGroupLayout` for this part of the
    /// bind group.
    fn layout_entry(
        vgpu: &VirtualGpu,
        binding: u32, 
        visibility: wgpu::ShaderStages
    ) -> wgpu::BindGroupLayoutEntry;

    /// Create a single `wgpu::BindGroupEntry` for a `wgpu::BindGroup` for
    /// this part of the bind group.
    fn group_entry<'a>(
        vgpu: &'a VirtualGpu,
        binding: u32,
        input: &'a Self::PartInput
    ) -> wgpu::BindGroupEntry<'a>;
}


pub trait BindGroupProvider {
    type Input;

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
        vgpu: &'a VirtualGpu,
        layout: &'a wgpu::BindGroupLayout,
        input: &'a Self::Input
    ) -> wgpu::BindGroup;
}
