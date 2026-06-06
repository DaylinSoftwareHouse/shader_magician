use magician_rust::glam::Vec4;

/// Allows for attaching a texture to the render pass like `SinglePass`.
/// This contains information about what texture view to bind as well as
/// what load and store operations to use.
pub struct PassAttachment<V: Clone + Copy> {
    pub view: wgpu::TextureView,
    pub load_op: LoadOp<V>,
    pub store_op: StoreOp
}

impl PassAttachment<Vec4> {
    pub(crate) fn as_color_attachment<'a>(&'a self) -> wgpu::RenderPassColorAttachment<'a> {
        wgpu::RenderPassColorAttachment {
            view: &self.view,
            ops: wgpu::Operations {
                load: self.load_op.into(),
                store: self.store_op.into()
            },
            resolve_target: None,
            depth_slice: None
        }
    }
}

impl PassAttachment<f32> {
    pub(crate) fn as_depth_attachment<'a>(&'a self) -> wgpu::RenderPassDepthStencilAttachment<'a> {
        wgpu::RenderPassDepthStencilAttachment {
            view: &self.view,
            depth_ops: Some(wgpu::Operations {
                load: self.load_op.into(),
                store: self.store_op.into()
            }),
            stencil_ops: None
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
