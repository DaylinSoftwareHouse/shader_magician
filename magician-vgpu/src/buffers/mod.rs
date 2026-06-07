use crate::VirtualGpu;

pub mod dynamic;
pub mod immutable;
pub mod mutable;

pub use dynamic::*;
pub use immutable::*;
pub use mutable::*;

/// A trait representing a buffer that could be used by the `VirtualGPU`.
pub trait Buffer {
    /// Defines what data type this Buffer contains.
    type Type;

    /// Returns a reference the internal wgpu Buffer
    /// for this buffer.
    fn buffer(&self) -> &wgpu::Buffer;
}

pub trait WritableBuffer: Buffer {
    /// Write an instance of `Type` to this buffer.
    fn write(&self, vgpu: &VirtualGpu, data: Self::Type) -> anyhow::Result<()>;
}
