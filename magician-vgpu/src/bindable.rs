use std::marker::PhantomData;

use getset::{Getters, MutGetters};

use crate::VirtualGpu;

#[derive(Getters, MutGetters)]
pub struct BindableObject<I> {
    #[getset(get = "pub", get_mut = "pub")]
    bind_group: wgpu::BindGroup,
    #[getset(get = "pub", get_mut = "pub")]
    layout: wgpu::BindGroupLayout,
    _phantom: PhantomData<I>
}

impl <I> BindableObject<I> {
    pub fn new(bind_group: wgpu::BindGroup, layout: wgpu::BindGroupLayout) -> Self {
        Self { bind_group, layout, _phantom: PhantomData::default() }
    }
}

pub trait BindableObjectCreator<'a> {
    type Inputs;
    fn create_object(
        vgpu: &VirtualGpu, 
        inputs: Self::Inputs
    ) -> BindableObject<Self> where Self: Sized;
}
