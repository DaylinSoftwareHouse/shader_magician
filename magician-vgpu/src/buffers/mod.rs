use crate::VirtualGpu;

pub mod dynamic;
pub mod immutable;
pub mod mutable;

use bytemuck::NoUninit;
pub use dynamic::*;
pub use immutable::*;
pub use mutable::*;

/// A trait representing a buffer that could be used by the `VirtualGPU`.
pub trait Buffer {
    /// Defines what data type this Buffer contains.
    type Type: BufferContent + ?Sized;

    /// Returns a reference the internal wgpu Buffer
    /// for this buffer.
    fn buffer(&self) -> &wgpu::Buffer;
}

pub trait WritableBuffer: Buffer {
    /// Write an instance of `Type` to this buffer.
    fn write(&self, vgpu: &VirtualGpu, data: Self::Type) -> anyhow::Result<()>;
}

/// Defines an type that may be added to a buffer.
/// This should be automatically implemented for 
/// all `Sized` types.  This also allows us to add
/// unsized types to buffers like [T].
pub trait BufferContent {
    fn as_bytes(&self) -> &[u8];
}

impl<T: NoUninit> BufferContent for T {
    fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl<T: NoUninit> BufferContent for [T] {
    fn as_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(self)
    }
}
