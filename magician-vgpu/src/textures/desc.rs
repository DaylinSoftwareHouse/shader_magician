use getset::Getters;

/// Standard texture descriptor that contains some
/// information on how the texture should act relative
/// to the GPU.
#[derive(Clone, Copy, Debug, Getters)]
pub struct TextureDescriptor {
    pub format: wgpu::TextureFormat,
    pub usage: wgpu::TextureUsages,
    pub dimension: wgpu::TextureDimension,
    pub address_mode_u: wgpu::AddressMode,
    pub address_mode_v: wgpu::AddressMode,
    pub address_mode_w: wgpu::AddressMode,
    pub mag_filter: wgpu::FilterMode,
    pub min_filter: wgpu::FilterMode,
    pub mipmap_filter: wgpu::MipmapFilterMode,
}

impl Default for TextureDescriptor {
    fn default() -> Self {
        Self {
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            dimension: wgpu::TextureDimension::D2,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest
        }
    }
}

impl TextureDescriptor {
    /// Convert this texture descriptor to a WGPU sampler descriptor.
    pub fn as_sampler_descriptor<'a>(&'a self) -> wgpu::SamplerDescriptor<'a> {
        wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: self.address_mode_u,
            address_mode_v: self.address_mode_v,
            address_mode_w: self.address_mode_w,
            mag_filter: self.mag_filter,
            min_filter: self.min_filter,
            mipmap_filter: self.mipmap_filter,
            ..Default::default()
        }
    }
}
