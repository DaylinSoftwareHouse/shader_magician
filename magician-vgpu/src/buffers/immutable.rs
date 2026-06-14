use std::marker::PhantomData;

use wgpu::util::DeviceExt;

use crate::{Buffer, BufferContent, VirtualGpu};

/// An immutable buffer that cannot be written too.
pub struct ImmutableBuffer<T: BufferContent + ?Sized> {
    buffer: wgpu::Buffer,
    size: u32,
    _phantom: PhantomData<T>
}

impl <T: BufferContent + ?Sized> ImmutableBuffer<T> {
    /// Create a new `ImmutableBuffer` from a `VirtualGPU` reference and some data.
    pub fn new(vgpu: &VirtualGpu, data: &T, usage: wgpu::BufferUsages) -> Self {
        let bytes = data.as_bytes();
        let size = bytes.len() / T::element_size();
        let buffer = vgpu.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytes,
            usage
        });

        Self { buffer, size: size as u32, _phantom: PhantomData::default() }
    }

    /// Create a new `ImmutableBuffer` from a raw buffer.
    pub fn from_raw(size: u32, buffer: wgpu::Buffer) -> Self {
        Self { buffer, size, _phantom: PhantomData::default() }
    }
}

impl <T: BufferContent + ?Sized> Buffer for ImmutableBuffer<T> {
    type Type = T;
    fn buffer(&self) -> &wgpu::Buffer { &self.buffer }
    fn size(&self) -> u32 { self.size }
}
