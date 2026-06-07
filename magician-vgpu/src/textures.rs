/// Trait representing anything that could be represented
/// as a texture to wgpu.  This trait gives access to the
/// required information for anything textures in WGPU.
pub trait Texture {
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
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler
}

impl StaticTexture {
    /// Create a new `StaticTexture` from a
    /// wgpu `Texture`, `TextureView`, and `Sampler`.
    pub fn new(
        texture: wgpu::Texture,
        view: wgpu::TextureView,
        sampler: wgpu::Sampler
    ) -> Self { Self { texture, view, sampler } }
}

impl Texture for StaticTexture {
    fn texture(&self) -> &wgpu::Texture { &self.texture }
    fn view(&self) -> &wgpu::TextureView { &self.view }
    fn sampler(&self) -> &wgpu::Sampler { &self.sampler }
}
