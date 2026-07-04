use std::any::TypeId;

use ahash::AHashMap;
use getset::{Getters, MutGetters};

use crate::{BindGroupProvider, BindableObject, VirtualGpu};

/// A wrapper around `wgpu::RenderPipeline` that ensures type safety and ease
/// of use for Pipelines.
#[derive(Getters, MutGetters)]
pub struct Pipeline {
    #[getset(get = "pub", get_mut = "pub")]
    name: String,
    #[getset(get = "pub", get_mut = "pub")]
    pipeline: wgpu::RenderPipeline,
    #[getset(get = "pub", get_mut = "pub")]
    layout: wgpu::PipelineLayout,
    #[getset(get = "pub", get_mut = "pub")]
    slot_map: AHashMap<TypeId, u32>
}

impl Pipeline {
    /// Create a `PipelineBuilder` to build a new `Pipeline`.
    pub fn builder<'a>(label: impl Into<String>) -> PipelineBuilder<'a> {
        PipelineBuilder {
            label: label.into(),
            ..Default::default()
        }
    }
}


#[derive(Debug, Clone)]
pub struct ShaderSource {
    pub source: String,
    pub main_function: String
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ShaderType {
    Fragment,
    Vertex
}

/// A simple builder for creating `Pipeline`s in a simple and easy
/// to use way without the bulky boiler plate of normal WGPU
/// pipeline creation.
#[derive(Default)]
pub struct PipelineBuilder<'a> {
    label: String,
    shader_srcs: AHashMap<ShaderType, ShaderSource>,
    vertex_layouts: Vec<wgpu::VertexBufferLayout<'a>>,
    depth_format: Option<wgpu::TextureFormat>,
    slot_map: AHashMap<TypeId, (u32, wgpu::BindGroupLayout)>
}

impl <'a> PipelineBuilder<'a> {
    /// Add a shader source of the given type to this builder.  If a source
    /// of the same type has already been added, it will be overriden.
    pub fn source(mut self, ty: ShaderType, source: ShaderSource) -> Self {
        self.shader_srcs.insert(ty, source);
        return self;
    }

    /// Add a vertex layout to this builder.
    pub fn vertex(mut self, layout: wgpu::VertexBufferLayout<'a>) -> Self {
        self.vertex_layouts.push(layout);
        return self;
    }

    /// Add a bind group layout from a bindable object to this builder.
    pub fn layout<T: BindGroupProvider + 'static>(mut self, idx: usize, object: &'a BindableObject<T>) -> Self {
        let type_id = TypeId::of::<T>();
        self.slot_map.insert(type_id, (idx as u32, object.layout().clone()));
        return self;
    }

    /// Add a bind group layout from a bindable object to this builder.
    pub fn layout_raw<T: 'static>(mut self, idx: usize, layout: wgpu::BindGroupLayout) -> Self {
        self.slot_map.insert(TypeId::of::<T>(), (idx as u32, layout));
        return self;
    }

    /// Add a depth format to this builder.
    pub fn depth_format(mut self, format: wgpu::TextureFormat) -> Self {
        self.depth_format = Some(format);
        return self;
    }

    /// Merges PipelineBuilder `other` into this builder.  If some data is present
    /// in both (for example, from the internal shader sources map), the data from 
    /// `other` will be taken over those in `self`.
    pub fn merge(mut self, other: PipelineBuilder<'a>) -> Self {
        other.shader_srcs.into_iter()
            .for_each(|(k, v)| { self.shader_srcs.insert(k, v); });
        self.vertex_layouts.extend(other.vertex_layouts);
        if let Some(depth_format) = other.depth_format { self.depth_format = Some(depth_format) }
        other.slot_map.into_iter()
            .for_each(|(k, v)| { self.slot_map.insert(k, v); });

        return self;
    }

    /// Build this builder into a Pipeline.
    pub fn build(self, vgpu: &VirtualGpu) -> Pipeline {
        // sort and compose bind group layouts
        let mut bgls_sorted = self.slot_map
            .into_iter()
            .collect::<Vec<_>>();
        bgls_sorted.sort_by_key(|a| a.1.0);
        let bgls = bgls_sorted
            .iter()
            .map(|a| Some(&a.1.1))
            .collect::<Vec<_>>();
        let slot_map = bgls_sorted
            .iter()
            .map(|a| (a.0, a.1.0))
            .collect::<AHashMap<TypeId, u32>>();

        // build pipeline layout
        let layout = vgpu.device().create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some(&format!("{}_layout", self.label)),
                immediate_size: 0,
                bind_group_layouts: &bgls
            }
        );

        // create shader module
        let vs_source = self.shader_srcs
            .get(&ShaderType::Vertex)
            .cloned()
            .expect("Failed to find vertex shader source");
        let vertex_shader = vgpu.device().create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: Some(&self.label),
                source: wgpu::ShaderSource::Wgsl(vs_source.source.into())
            }
        );
        let fs_source = self.shader_srcs
            .get(&ShaderType::Fragment)
            .cloned()
            .expect("Failed to find fragment shader source");
        let fragment_shader = vgpu.device().create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: Some(&self.label),
                source: wgpu::ShaderSource::Wgsl(fs_source.source.into())
            }
        );

        let pipeline = vgpu.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&self.label),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &vertex_shader,
                entry_point: Some(vs_source.main_function.as_str()),
                buffers: &self.vertex_layouts,
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &fragment_shader,
                entry_point: Some(fs_source.main_function.as_str()),
                targets: &[Some(wgpu::ColorTargetState {
                    format: vgpu.config().format,
                    blend: Some(wgpu::BlendState {
                        alpha: wgpu::BlendComponent::REPLACE,
                        color: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: self.depth_format
                .map(|format| wgpu::DepthStencilState {
                    format,
                    depth_write_enabled: Some(true),
                    depth_compare: Some(wgpu::CompareFunction::Less),
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        Pipeline { name: self.label, pipeline, layout, slot_map }
    }
}
