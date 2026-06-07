/// Trait representing anything that could be represented
/// as a texture to wgpu.  This trait gives access to the
/// required information for anything textures in WGPU.
pub trait Texture {
    /// Returns a reference to the wgpu `TextureView`
    /// for this texture.
    fn view(&self) -> &wgpu::TextureView;
}

/// A static texture that only contains raw wgpu
/// information for use in wgpu rendering.
pub struct StaticTexture {
    view: wgpu::TextureView
}

impl Texture for StaticTexture {
    fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
}
