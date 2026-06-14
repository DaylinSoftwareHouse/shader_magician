use std::marker::PhantomData;

use getset::{Getters, MutGetters};

use crate::{Buffer, VirtualGpu};

#[derive(Getters, MutGetters)]
pub struct BindableObject<A> {
    #[getset(get = "pub", get_mut = "pub")]
    bind_group: wgpu::BindGroup,
    #[getset(get = "pub", get_mut = "pub")]
    layout: wgpu::BindGroupLayout,
    _phantom: PhantomData<A>
}

impl <A> BindableObject<A> {
    pub fn new(bind_group: wgpu::BindGroup, layout: wgpu::BindGroupLayout) -> Self {
        Self { bind_group, layout, _phantom: PhantomData::default() }
    }

    pub fn from_inputs<G: BindGroupProvider<A>>(
        vgpu: &VirtualGpu, 
        inputs: &A
    ) -> Self {
        let layout = G::layout(vgpu, wgpu::ShaderStages::all());
        let group = G::group(vgpu, &layout, inputs);
        return Self::new(group, layout);
    }
}


pub trait BindGroupPart<I> {

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
        input: &'a I
    ) -> wgpu::BindGroupEntry<'a>;
}


pub trait BindGroupProvider<I> {
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
        input: &'a I
    ) -> wgpu::BindGroup;
}


impl <A: BindGroupPart<B>, B> BindGroupProvider<B> for A {
    fn layout(
        vgpu: &VirtualGpu,
        visibility: wgpu::ShaderStages
    ) -> wgpu::BindGroupLayout {
        vgpu.device().create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    A::layout_entry(vgpu, 0, visibility)
                ]
            }
        )
    }

    fn group(
        vgpu: &VirtualGpu,
        layout: &wgpu::BindGroupLayout,
        input: &B
    ) -> wgpu::BindGroup {
        vgpu.device().create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &layout,
                label: None,
                entries: &[
                    A::group_entry(vgpu, 0, input)
                ]
            }
        )
    }
}
