use std::ops::Range;

use bytemuck::NoUninit;
use getset::{Getters, MutGetters};

use crate::{BindGroupProvider, BindableObject, Buffer, Pipeline};

/// Represents a render pass that may be used to render
/// one pass of a frame before being dropped.  These are
/// created solely through the `RenderFrame` struct.
#[derive(Getters, MutGetters)]
pub struct SinglePass<'a> {
    #[getset(get = "pub", get_mut = "pub")]
    pass: wgpu::RenderPass<'a>,
    last_instances_size: u32
}

impl <'a> SinglePass<'a> {
    /// Create from a wgpu `RenderPass`.
    pub(crate) fn new(pass: wgpu::RenderPass<'a>) -> Self {
        Self { 
            pass,
            last_instances_size: 1
        }
    }

    /// Use a specific pipeline for rendering.
    /// This resets the last instances size tracker
    /// to prevent drawing instances that do not exist.
    pub fn use_pipeline(
        &mut self,
        pipeline: &Pipeline
    ) {
        self.pass_mut().set_pipeline(&pipeline.pipeline());
        self.last_instances_size = 1;
    }

    /// Bind a bindable object to a specific index.
    pub fn bind<T: BindGroupProvider>(
        &mut self,
        index: u32,
        bindable: &BindableObject<T>
    ) {
        self.pass_mut().set_bind_group(index, bindable.bind_group(), &[]);
    }

    /// Bind an instances buffer to vertex buffer slot 1.
    pub fn bind_instances<T: NoUninit>(
        &mut self,
        buffer: &dyn Buffer<Type = [T]>
    ) {
        self.pass_mut().set_vertex_buffer(1, buffer.buffer().slice(..));
        self.last_instances_size = buffer.size();
    }

    /// Draw some vertices and indices to the screen.  The given settings
    /// can be used to define the index format, base vertex, indices range, 
    /// and instances range to draw while allowing for standard defaults.
    /// 
    /// Vertices are bound to slot 0.
    pub fn draw<V: NoUninit, I: NoUninit>(
        &mut self,
        vertices: &dyn Buffer<Type = [V]>,
        indices: &dyn Buffer<Type = [I]>,
        settings: DrawSettings
    ) {
        // convert index format
        let index_fmt = match settings.index_fmt {
            IndexFmt::Fat => wgpu::IndexFormat::Uint32,
            IndexFmt::Light => wgpu::IndexFormat::Uint16,
        };

        // unpack indices and instances range
        let indices_range = settings.indices
            .unwrap_or_else(|| 0 .. indices.size());
        let instances_range = settings.instances
            .unwrap_or_else(|| 0 .. self.last_instances_size);

        // perform draw
        self.pass_mut().set_vertex_buffer(0, vertices.buffer().slice(..));
        self.pass_mut().set_index_buffer(indices.buffer().slice(..), index_fmt);
        self.pass_mut().draw_indexed(indices_range, settings.base_vertex, instances_range);
    }
}

/// Settings for how some vertices and indices should be drawn.
/// The `index_fmt` field defines what kind of indices are being used.
/// The `base_vertex` field defines which vertex to start the draw from.
/// The `indices` field defines what range of indices to draw.
///     This will default to the size of the instances buffer.
/// The `instances` field defines what range instances to draw.
///     This will default to the size of the previous bound instance 
///     buffer if one is bound.
#[derive(Default, Debug, Clone)]
pub struct DrawSettings {
    pub index_fmt: IndexFmt,
    pub base_vertex: i32,
    pub indices: Option<Range<u32>>,
    pub instances: Option<Range<u32>>
}

/// Represents what formats an indicies buffer may be in.
#[derive(Default, Debug, Clone, Copy)]
pub enum IndexFmt {
    /// Is the indices a U32 (default)
    #[default]
    Fat,
    // Is the indices a U16
    Light
}
