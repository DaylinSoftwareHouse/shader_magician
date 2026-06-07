use std::marker::PhantomData;

use anyhow::anyhow;
use bytemuck::NoUninit;
use wgpu::util::DeviceExt;

use crate::{Buffer, VirtualGpu, WritableBuffer};

/// A simple mutable buffer that allows for writing
/// to a statically sized buffer
pub struct MutableBuffer<T: NoUninit> {
    buffer: wgpu::Buffer,
    size: usize,
    _phantom: PhantomData<T>
}

impl <T: NoUninit> MutableBuffer<T> {
    /// Create a new `MutableBuffer` from a `VirtualGPU` reference and some data.
    pub fn new(vgpu: &VirtualGpu, data: T, usage: wgpu::BufferUsages) -> Self {
        let binding = [data];
        let contents = bytemuck::cast_slice(&binding);
        let buffer = vgpu.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents, usage
        });

        Self { buffer, size: contents.len(), _phantom: PhantomData::default() }
    }

    /// Create a new `MutableBuffer` from a raw buffer.
    pub fn from_raw(buffer: wgpu::Buffer, size: usize) -> Self {
        Self { buffer, size, _phantom: PhantomData::default() }
    }
}

impl <T: NoUninit> Buffer for MutableBuffer<T> {
    type Type = T;
    fn buffer(&self) -> &wgpu::Buffer { &self.buffer }
}

impl <T: NoUninit> WritableBuffer for MutableBuffer<T> {
    fn write(&self, vgpu: &VirtualGpu, data: Self::Type) -> anyhow::Result<()> {
        let binding = [data];
        let bytes = bytemuck::cast_slice(&binding);
        let new_size = bytes.len();

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
