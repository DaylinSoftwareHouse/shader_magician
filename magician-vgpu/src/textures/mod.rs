pub mod desc;

pub use desc::*;

use crate::VirtualGpu;

/// Trait representing anything that could be represented
/// as a texture to wgpu.  This trait gives access to the
/// required information for anything textures in WGPU.
pub trait Texture {
    fn descriptor(&self) -> &TextureDescriptor;

    /// Returns a reference to the textures underlying
    /// wgpu `Texture`.
    fn texture(&self) -> &wgpu::Texture;

    /// Returns a reference to the wgpu `TextureView`
    /// for this texture.
    fn view(&self) -> &wgpu::TextureView;

    /// Returns a reference to the textures underlying
    /// wgpu `Sampler`.
    fn sampler(&self) -> &wgpu::Sampler;
}

/// A static texture that only contains raw wgpu
/// information for use in wgpu rendering.
pub struct StaticTexture {
    pub descriptor: TextureDescriptor,
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler
}

impl StaticTexture {
    /// Create a new `StaticTexture` from a
    /// wgpu `Texture`, `TextureView`, and `Sampler`.
    pub fn new(
        descriptor: TextureDescriptor,
        texture: wgpu::Texture,
        view: wgpu::TextureView,
        sampler: wgpu::Sampler
    ) -> Self { Self { descriptor, texture, view, sampler } }

    /// Create an empty texture setup to be a framebuffer.  The size of
    /// the returned texture will be that of the current windows size.
    pub fn framebuffer(
        vgpu: &VirtualGpu,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages
    ) -> Self {
        return Self::empty_texure(
            vgpu, 
            TextureDescriptor { 
                format, usage, 
                dimension: wgpu::TextureDimension::D2, 
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            }, 
            vgpu.config.width.max(1), 
            vgpu.config.height.max(1)
        );
    }

    /// Create an empty texture setup with the given descriptor,
    /// width and height.  There will be NO data in the texture
    /// so you need to write to this before using.
    pub fn empty_texure(
        vgpu: &VirtualGpu,
        descriptor: TextureDescriptor,
        width: u32,
        height: u32
    ) -> Self {
        let size = wgpu::Extent3d { width, height, depth_or_array_layers: 1 };
        
        let texture = vgpu.device().create_texture(
            &wgpu::TextureDescriptor {
                label: None, size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: descriptor.dimension,
                format: descriptor.format,
                usage: descriptor.usage,
                view_formats: &[]
            }
        );

        let view = texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = vgpu.device()
            .create_sampler(&descriptor.as_sampler_descriptor());

        Self { descriptor, texture, view, sampler }
    }

    /// Create a texture setup with the given descriptor,
    /// width and height.  This texture will be created with the
    /// given texture data (`bytes`).
    pub fn from_raw(
        vgpu: &VirtualGpu,
        descriptor: TextureDescriptor,
        bytes: &[u8],
        width: u32,
        height: u32
    ) -> Self {
        let texture = Self::empty_texure(vgpu, descriptor, width, height);

        vgpu.queue().write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: texture.texture(),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            texture.texture().size()
        );

        return texture;
    }
}

impl Texture for StaticTexture {
    fn descriptor(&self) -> &TextureDescriptor { &self.descriptor }
    fn texture(&self) -> &wgpu::Texture { &self.texture }
    fn view(&self) -> &wgpu::TextureView { &self.view }
    fn sampler(&self) -> &wgpu::Sampler { &self.sampler }
}
