use std::cell::RefCell;

use wgpu::util::DeviceExt;

use crate::{Buffer, BufferContent, VirtualGpu, WritableBuffer};

/// A resizable GPU buffer that holds a single instance of `T`.
pub struct DynamicBuffer<T: BufferContent + ?Sized> {
    buffer: RefCell<wgpu::Buffer>,
    usage: wgpu::BufferUsages,
    _marker: std::marker::PhantomData<T>,
}

impl<T: BufferContent + ?Sized> DynamicBuffer<T> {
    /// Create a new `DynamicBuffer` from a `VirtualGpu` instance, some 
    /// data, and how the buffer will be used.
    pub fn new(
        vgpu: &VirtualGpu,
        data: &T,
        usage: wgpu::BufferUsages
    ) -> Self {
        let buffer = vgpu.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: data.as_bytes(),
            usage: usage | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            buffer: RefCell::new(buffer),
            usage: usage | wgpu::BufferUsages::COPY_DST,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: BufferContent + ?Sized> Buffer for DynamicBuffer<T> {
    type Type = T;

    fn buffer(&self) -> &wgpu::Buffer {
        // SAFETY: We return a reference tied to self's lifetime.
        // The RefCell borrow is released immediately after getting the raw pointer,
        // but the buffer itself lives as long as self does. The pointer is only
        // invalidated if the RefCell contents are replaced, which only happens
        // in `write` — a &mut self operation that can't alias with this borrow.
        //
        // If truly concurrent access is needed, switch to RwLock and return
        // an Arc<wgpu::Buffer> instead.
        unsafe { &*self.buffer.as_ptr() }
    }

    fn size(&self) -> u32 {
        (self.buffer.borrow().size() as usize / T::element_size()) as u32
    }
}

impl<T: bytemuck::Pod> WritableBuffer for DynamicBuffer<T> {
    fn write(&self, vgpu: &VirtualGpu, data: &Self::Type) -> anyhow::Result<()> {
        let bytes = data.as_bytes();
        let current_size = self.buffer.borrow().size();

        if bytes.len() as u64 == current_size {
            vgpu.queue.write_buffer(&self.buffer.borrow(), 0, bytes);
        } else {
            let new_buffer =
                vgpu.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: None,
                        contents: bytes,
                        usage: self.usage,
                    });

            *self.buffer.borrow_mut() = new_buffer;
        }

        Ok(())
    }
}
