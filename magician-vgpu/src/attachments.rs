use magician_rust::glam::Vec4;

use crate::Texture;

/// Allows for attaching a texture to the render pass like `SinglePass`.
/// This contains information about what texture view to bind as well as
/// what load and store operations to use.
pub struct PassAttachment<'a, V: Clone + Copy> {
    pub target: PassTarget<'a>,
    pub load_op: LoadOp<V>,
    pub store_op: StoreOp
}

impl <'a> PassAttachment<'a, Vec4> {
    pub(crate) fn as_color_attachment<'b>(
        &'b self, 
        frame_output: &'a wgpu::TextureView
    ) -> wgpu::RenderPassColorAttachment<'a> {
        wgpu::RenderPassColorAttachment {
            view: &self.target.to_view(frame_output),
            ops: wgpu::Operations {
                load: self.load_op.into(),
                store: self.store_op.into()
            },
            resolve_target: None,
            depth_slice: None
        }
    }
}

impl <'a> PassAttachment<'a, f32> {
    pub(crate) fn as_depth_attachment<'b>(
        &'b self, 
        frame_output: &'a wgpu::TextureView
    ) -> wgpu::RenderPassDepthStencilAttachment<'a> {
        wgpu::RenderPassDepthStencilAttachment {
            view: &self.target.to_view(frame_output),
            depth_ops: Some(wgpu::Operations {
                load: self.load_op.into(),
                store: self.store_op.into()
            }),
            stencil_ops: None
        }
    }
}


/// Instructs a `PassAttachment` where to find its render target.
/// If `PassOutput`, the passes output texture will be used.
/// If `Texture`, the inner `Texture` reference will be used.
pub enum PassTarget<'a> {
    PassOutput,
    Texture(&'a dyn Texture)
}

impl <'a> PassTarget<'a> {
    pub fn to_view(&self, frame_output: &'a wgpu::TextureView) -> &'a wgpu::TextureView {
        match &self {
            PassTarget::PassOutput => frame_output,
            PassTarget::Texture(texture) => 
                texture.view()
        }
    }
}


/// Determines how a texture view should be loaded for a `PassAttachment`.
/// If `Clear` is given, the texture will be cleared to the given output
/// before rendering.  If `Load` is given, the textures previous data will
/// be loaded instead of cleared.
#[derive(Clone, Copy)]
pub enum LoadOp<V: Clone + Copy> {
    Clear(V),
    Load
}

impl Into<wgpu::LoadOp<wgpu::Color>> for LoadOp<Vec4> {
    fn into(self) -> wgpu::LoadOp<wgpu::Color> {
        match self {
            LoadOp::Clear(color) => wgpu::LoadOp::Clear(wgpu::Color {
                r: color.x as f64, g: color.y as f64,
                b: color.z as f64, a: color.w as f64
            }),
            LoadOp::Load => wgpu::LoadOp::Load
        }
    }
}

impl Into<wgpu::LoadOp<f32>> for LoadOp<f32> {
    fn into(self) -> wgpu::LoadOp<f32> {
        match self {
            LoadOp::Clear(color) => wgpu::LoadOp::Clear(color),
            LoadOp::Load => wgpu::LoadOp::Load
        }
    }
}


/// Determines what a `PassAttachment` should do with the results of rendering.
/// If `Store` is given, the rendered data will be saved.  If `Discard` is given,
/// the rendered data will be discarded after rendering.
#[derive(Clone, Copy, Debug)]
pub enum StoreOp {
    Store,
    Discard
}

impl Into<wgpu::StoreOp> for StoreOp {
    fn into(self) -> wgpu::StoreOp {
        match self {
            StoreOp::Store => wgpu::StoreOp::Store,
            StoreOp::Discard => wgpu::StoreOp::Discard
        }
    }
}
