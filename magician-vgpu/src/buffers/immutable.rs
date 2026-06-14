use std::marker::PhantomData;

use wgpu::util::DeviceExt;

use crate::{Buffer, BufferContent, VirtualGpu};

/// An immutable buffer that cannot be written too.
pub struct ImmutableBuffer<T: BufferContent + ?Sized> {
    buffer: wgpu::Buffer,
    _phantom: PhantomData<T>
}

impl <T: BufferContent + ?Sized> ImmutableBuffer<T> {
    /// Create a new `ImmutableBuffer` from a `VirtualGPU` reference and some data.
    pub fn new(vgpu: &VirtualGpu, data: &T, usage: wgpu::BufferUsages) -> Self {
        let buffer = vgpu.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light VB"),
            contents: data.as_bytes(),
            usage
        });

        Self { buffer, _phantom: PhantomData::default() }
    }

    /// Create a new `ImmutableBuffer` from a raw buffer.
    pub fn from_raw(buffer: wgpu::Buffer) -> Self {
        Self { buffer, _phantom: PhantomData::default() }
    }
}

impl <T: BufferContent + ?Sized> Buffer for ImmutableBuffer<T> {
    type Type = T;
    fn buffer(&self) -> &wgpu::Buffer { &self.buffer }
}
