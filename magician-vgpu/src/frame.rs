use anyhow::Ok;
use getset::{Getters, MutGetters};

use crate::VirtualGpu;

#[derive(Getters, MutGetters)]
pub struct RenderFrame {
    #[getset(get = "pub", get_mut = "pub")]
    output: wgpu::SurfaceTexture,
    #[getset(get = "pub", get_mut = "pub")]
    view: wgpu::TextureView,
    #[getset(get = "pub", get_mut = "pub")]
    encoder: wgpu::CommandEncoder
}

impl RenderFrame {
    /// Start a render frame from a reference to a virtual GPU.
    /// This creates a new `RenderFrame` instance that may be used
    /// for rendering your next frame.
    /// 
    /// This returns one of three possible states:
    ///   - A Ok and Some wrapped self instance indicating a successful
    ///         frame creation.
    ///   - A Ok wrapping a None value if the frame could not be created
    ///         due to the VirtualGpu not being ready yet, but the state
    ///         the VirtualGpu is in is not fatal, for example, if the 
    ///         VirtualGpu is still loading asychronously in the background.
    ///   - An error value if the VirtualGpu is in an illegal state that
    ///         does not allow the frame to be created.
    pub fn begin(vgpu: &VirtualGpu) -> anyhow::Result<Option<Self>> {
        vgpu.window().request_redraw();
        if !vgpu.config().width < 1 { return Ok(None); }

        let output = match vgpu.surface().get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => {
                vgpu.surface().configure(&vgpu.device(), &vgpu.config());
                surface_texture
            }
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => {
                // Skip this frame
                return Ok(None);
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                vgpu.surface().configure(&vgpu.device(), &vgpu.config());
                return Ok(None);
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                // You could recreate the devices and all resources
                // created with it here, but we'll just bail
                anyhow::bail!("Lost device");
            }
        };
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let encoder = vgpu
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        
        Ok(Some(Self { output, view, encoder }))
    }

    /// Submit this frame for rendering/use by the GPU.  This will consume
    /// this frame, effectively ending it.
    pub fn submit(self, vgpu: &VirtualGpu) {
        vgpu.queue().submit(std::iter::once(self.encoder.finish()));
        self.output.present();
    }
}
