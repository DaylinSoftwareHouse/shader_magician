use anyhow::Context;
use getset::{Getters, MutGetters};
use magician_rust::glam::Vec4;

use crate::{PassAttachment, SinglePass, VirtualGpu};

/// What a [`RenderFrame`] renders into.
enum FrameTarget {
    /// The window surface's next texture, presented on submit.
    Surface(wgpu::SurfaceTexture),
    /// A headless `VirtualGpu`'s offscreen texture.
    Offscreen(wgpu::Texture)
}

/// The pixels of a captured frame, see [`RenderFrame::submit_and_capture`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameCapture {
    pub width: u32,
    pub height: u32,
    /// Tightly packed rows of RGBA8 pixels, top row first.
    pub rgba: Vec<u8>
}

impl FrameCapture {
    /// The RGBA value of the pixel at `x`, `y`, or `None` if it is out of bounds.
    pub fn pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height { return None }
        let i = ((y * self.width + x) * 4) as usize;
        self.rgba.get(i..i + 4).map(|p| [p[0], p[1], p[2], p[3]])
    }
}

#[derive(Getters, MutGetters)]
pub struct RenderFrame {
    target: FrameTarget,
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
    ///
    /// The next redraw is requested when a frame is created, or when the
    /// surface was outdated and has been reconfigured.  Otherwise (zero sized,
    /// occluded, timed out) no redraw is requested, so the caller decides when
    /// to try again rather than spinning while the window can't be drawn to.
    ///
    /// A headless `VirtualGpu` always gets a frame, rendering into its
    /// offscreen texture.
    pub fn begin(vgpu: &VirtualGpu) -> anyhow::Result<Option<Self>> {
        if vgpu.config().width < 1 || vgpu.config().height < 1 { return Ok(None); }

        let target = match vgpu.surface() {
            Some(surface) => match surface.get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(surface_texture) => FrameTarget::Surface(surface_texture),
                wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => {
                    surface.configure(&vgpu.device(), &vgpu.config());
                    FrameTarget::Surface(surface_texture)
                }
                wgpu::CurrentSurfaceTexture::Timeout
                | wgpu::CurrentSurfaceTexture::Occluded
                | wgpu::CurrentSurfaceTexture::Validation => {
                    // Skip this frame
                    return Ok(None);
                }
                wgpu::CurrentSurfaceTexture::Outdated => {
                    surface.configure(&vgpu.device(), &vgpu.config());
                    vgpu.request_redraw();
                    return Ok(None);
                }
                wgpu::CurrentSurfaceTexture::Lost => {
                    // You could recreate the devices and all resources
                    // created with it here, but we'll just bail
                    anyhow::bail!("Lost device");
                }
            },
            None => FrameTarget::Offscreen(
                vgpu.offscreen().clone().context("Headless VirtualGpu has no offscreen target")?
            ),
        };
        vgpu.request_redraw();
        let texture = match &target {
            FrameTarget::Surface(surface_texture) => &surface_texture.texture,
            FrameTarget::Offscreen(texture) => texture,
        };
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let encoder = vgpu
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        
        Ok(Some(Self { target, view, encoder }))
    }

    /// The texture this frame renders into.
    pub fn texture(&self) -> &wgpu::Texture {
        match &self.target {
            FrameTarget::Surface(surface_texture) => &surface_texture.texture,
            FrameTarget::Offscreen(texture) => texture,
        }
    }

    pub fn init_pass(
        &mut self,
        color_attachments: &[PassAttachment<Vec4>],
        depth_attachment: Option<PassAttachment<f32>>
    ) -> SinglePass {
        let frame_output = self.view().clone();
        let color_attachments = 
            color_attachments.into_iter()
            .map(|a| Some(a.as_color_attachment(&frame_output)))
            .collect::<Vec<_>>();

        let depth_stencil_attachment = 
            depth_attachment.as_ref().map(|a| a.as_depth_attachment(&frame_output));

        let pass = self
            .encoder_mut()
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &color_attachments,
                depth_stencil_attachment,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
        SinglePass::new(pass)
    }

    /// Submit this frame for rendering/use by the GPU.  This will consume
    /// this frame, effectively ending it.
    pub fn submit(self, vgpu: &VirtualGpu) {
        vgpu.queue().submit(std::iter::once(self.encoder.finish()));
        if let FrameTarget::Surface(surface_texture) = self.target {
            surface_texture.present();
        }
    }

    /// Submit this frame like [`submit`](Self::submit), then wait for the GPU and read
    /// the rendered pixels back.  Blocks until the frame has finished rendering.
    ///
    /// # Errors
    ///
    /// Fails if the frame's texture can't be copied from (a surface without
    /// `COPY_SRC` support), isn't an 8 bit RGBA or BGRA format, or the read back fails.
    pub fn submit_and_capture(mut self, vgpu: &VirtualGpu) -> anyhow::Result<FrameCapture> {
        let texture = self.texture().clone();
        let (width, height) = (texture.width(), texture.height());
        let bgra = match texture.format() {
            wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb => false,
            wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb => true,
            format => anyhow::bail!("Can't capture a frame in {format:?}"),
        };
        if !texture.usage().contains(wgpu::TextureUsages::COPY_SRC) {
            anyhow::bail!("This frame's texture doesn't allow copies, so it can't be captured");
        }

        let padded_row = padded_bytes_per_row(width);
        let buffer = vgpu.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("frame_capture"),
            size: padded_row as u64 * height as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        self.encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        );
        self.submit(vgpu);

        let (sender, receiver) = std::sync::mpsc::channel();
        buffer.map_async(wgpu::MapMode::Read, .., move |result| { let _ = sender.send(result); });
        vgpu.device().poll(wgpu::PollType::wait_indefinitely())
            .map_err(|err| anyhow::anyhow!("Failed waiting for the captured frame: {err}"))?;
        receiver.recv()?.map_err(|err| anyhow::anyhow!("Failed to map the captured frame: {err}"))?;

        let rgba = unpad_rows(&buffer.get_mapped_range(..), width, height, padded_row, bgra);
        buffer.unmap();
        Ok(FrameCapture { width, height, rgba })
    }
}

/// Bytes per row of a `width` pixel, 4 byte per pixel image, padded to wgpu's copy alignment.
fn padded_bytes_per_row(width: u32) -> u32 {
    let unpadded = width * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    unpadded.div_ceil(align) * align
}

/// Strip the row padding from copied pixel data, swapping BGRA to RGBA if `bgra`.
fn unpad_rows(data: &[u8], width: u32, height: u32, padded_row: u32, bgra: bool) -> Vec<u8> {
    let row = (width * 4) as usize;
    let mut rgba = Vec::with_capacity(row * height as usize);
    for y in 0..height as usize {
        let start = y * padded_row as usize;
        rgba.extend_from_slice(&data[start..start + row]);
    }
    if bgra {
        rgba.chunks_exact_mut(4).for_each(|pixel| pixel.swap(0, 2));
    }
    rgba
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_pad_to_the_copy_alignment() {
        assert_eq!(padded_bytes_per_row(1), 256);
        assert_eq!(padded_bytes_per_row(64), 256);
        assert_eq!(padded_bytes_per_row(65), 512);
    }

    #[test]
    fn unpad_drops_padding_and_swaps_bgra() {
        // 2x2 image, rows padded to 12 bytes
        let data = [
            1, 2, 3, 4,  5, 6, 7, 8,  0, 0, 0, 0,
            9, 10, 11, 12,  13, 14, 15, 16,  0, 0, 0, 0,
        ];
        assert_eq!(unpad_rows(&data, 2, 2, 12, false), (1..=16).collect::<Vec<u8>>());
        assert_eq!(unpad_rows(&data, 2, 2, 12, true)[..4], [3, 2, 1, 4]);
    }

    #[test]
    fn capture_pixel_lookup() {
        let capture = FrameCapture { width: 2, height: 1, rgba: vec![1, 2, 3, 4, 5, 6, 7, 8] };
        assert_eq!(capture.pixel(1, 0), Some([5, 6, 7, 8]));
        assert_eq!(capture.pixel(2, 0), None);
    }

    /// Clears a headless frame and reads it back.  Skips if there is no GPU adapter.
    #[test]
    fn headless_frame_round_trips_through_capture() {
        let vgpu = match pollster::block_on(VirtualGpu::new_headless(8, 4)) {
            Ok(vgpu) => vgpu,
            Err(err) => { eprintln!("skipping, no GPU: {err}"); return }
        };
        let mut frame = RenderFrame::begin(&vgpu).unwrap().expect("headless frames always begin");
        drop(frame.init_pass(&[PassAttachment {
            target: crate::PassTarget::PassOutput,
            load_op: crate::LoadOp::Clear(Vec4::new(1.0, 0.0, 0.0, 1.0)),
            store_op: crate::StoreOp::Store,
        }], None));
        let capture = frame.submit_and_capture(&vgpu).unwrap();
        assert_eq!((capture.width, capture.height), (8, 4));
        assert_eq!(capture.pixel(7, 3), Some([255, 0, 0, 255]));
    }
}
