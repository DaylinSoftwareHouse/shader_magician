use anyhow::*;
use derive_more::{Deref, DerefMut};
use image::GenericImageView;
use magician_vgpu::{StaticTexture, TextureDescriptor, VirtualGpu};

#[derive(Deref, DerefMut)]
pub struct Texture(pub StaticTexture);

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn create_depth_texture(vgpu: &VirtualGpu) -> Self {
        Self(StaticTexture::framebuffer(
            vgpu, 
            Self::DEPTH_FORMAT, 
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING
        ))
    }

    #[allow(dead_code)]
    pub fn from_bytes(
        vgpu: &VirtualGpu,
        bytes: &[u8],
        is_normal_map: bool,
    ) -> Result<Self> {
        let img = image::load_from_memory(bytes)?;
        Self::from_image(vgpu, &img, is_normal_map)
    }

    pub fn from_image(
        vgpu: &VirtualGpu,
        img: &image::DynamicImage,
        is_normal_map: bool,
    ) -> Result<Self> {
        let dimensions = img.dimensions();
        let rgba = img.to_rgba8();

        let format = if is_normal_map {
            wgpu::TextureFormat::Rgba8Unorm
        } else {
            wgpu::TextureFormat::Rgba8UnormSrgb
        };

        Ok(Self(StaticTexture::from_raw(
            vgpu, 
            TextureDescriptor {
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                ..Default::default()
            }, 
            &rgba, 
            dimensions.0, 
            dimensions.1
        )))
    }
}
