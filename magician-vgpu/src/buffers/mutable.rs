use std::marker::PhantomData;

use anyhow::anyhow;
use wgpu::util::DeviceExt;

use crate::{Buffer, BufferContent, VirtualGpu, WritableBuffer};

/// A simple mutable buffer that allows for writing
/// to a statically sized buffer
pub struct MutableBuffer<T: BufferContent + ?Sized> {
    buffer: wgpu::Buffer,
    size: usize,
    _phantom: PhantomData<T>
}

impl <T: BufferContent + ?Sized> MutableBuffer<T> {
    /// Create a new `MutableBuffer` from a `VirtualGPU` reference and some data.
    pub fn new(vgpu: &VirtualGpu, data: &T, usage: wgpu::BufferUsages) -> Self {
        let contents = data.as_bytes();
        let buffer = vgpu.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents, usage
        });

        Self { buffer, size: contents.len() / T::element_size(), _phantom: PhantomData::default() }
    }

    /// Create a new `MutableBuffer` from a raw buffer.
    pub fn from_raw(buffer: wgpu::Buffer, size: usize) -> Self {
        Self { buffer, size, _phantom: PhantomData::default() }
    }
}

impl <T: BufferContent + ?Sized> Buffer for MutableBuffer<T> {
    type Type = T;
    fn buffer(&self) -> &wgpu::Buffer { &self.buffer }
    fn size(&self) -> u32 { self.size as u32 }
}

impl <T: BufferContent + ?Sized> WritableBuffer for MutableBuffer<T> {
    fn write(&self, vgpu: &VirtualGpu, data: &Self::Type) -> anyhow::Result<()> {
        let bytes = data.as_bytes();
        let new_size = bytes.len() / T::element_size();

        if new_size != self.size {
            return Err(anyhow!("Cannot change of statically sized buffer"));
        }

        vgpu.queue().write_buffer(
            &self.buffer(),
            0,
            bytes,
        );

        Ok(())
    }
}
