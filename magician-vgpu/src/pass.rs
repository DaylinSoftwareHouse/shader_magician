use getset::{Getters, MutGetters};

/// Represents a render pass that may be used to render
/// one pass of a frame before being dropped.  These are
/// created solely through the `RenderFrame` struct.
#[derive(Getters, MutGetters)]
pub struct SinglePass<'a> {
    #[getset(get = "pub", get_mut = "pub")]
    pass: wgpu::RenderPass<'a>
}

impl <'a> SinglePass<'a> {
    /// Create from a wgpu `RenderPass`.
    pub(crate) fn new(pass: wgpu::RenderPass<'a>) -> Self {
        Self { pass }
    }
}
