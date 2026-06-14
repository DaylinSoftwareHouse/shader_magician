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

#[derive(Default, Debug, Clone)]
pub enum ShaderSource {
    #[default]
    Empty,
    Combined {
        source: String,
        vertex_main_function: String,
        fragment_main_function: String
    },
    Independent {
        vertex: String,
        vertex_main_function: String,
        fragment: String,
        fragment_main_function: String
    }
}

/// A simple builder for creating `Pipeline`s in a simple and easy
/// to use way without the bulky boiler plate of normal WGPU
/// pipeline creation.
#[derive(Default)]
pub struct PipelineBuilder<'a> {
    label: String,
    shader_src: ShaderSource,
    vertex_layouts: Vec<wgpu::VertexBufferLayout<'a>>,
    depth_format: Option<wgpu::TextureFormat>,
    slot_map: AHashMap<TypeId, (u32, &'a wgpu::BindGroupLayout)>
}

impl <'a> PipelineBuilder<'a> {
    /// Sets the shader source WGSL text in the builder.
    pub fn shader(mut self, shader_src: ShaderSource) -> Self {
        self.shader_src = shader_src;
        return self;
    }

    /// Add a vertex layout to this builder.
    pub fn vertex(mut self, layout: wgpu::VertexBufferLayout<'a>) -> Self {
        self.vertex_layouts.push(layout);
        return self;
    }

    /// Add a bind group layout from a bindable object to this builder.
    pub fn layout<T: BindGroupProvider + 'static>(mut self, object: &'a BindableObject<T>) -> Self {
        let type_id = TypeId::of::<T>();
        let id = self.slot_map.len();
        self.slot_map.insert(type_id, (id as u32, object.layout()));
        return self;
    }

    /// Add a bind group layout from a bindable object to this builder.
    pub fn layout_raw(mut self, type_id: TypeId, layout: &'a wgpu::BindGroupLayout) -> Self {
        let id = self.slot_map.len();
        self.slot_map.insert(type_id, (id as u32, layout));
        return self;
    }

    /// Add a depth format to this builder.
    pub fn depth_format(mut self, format: wgpu::TextureFormat) -> Self {
        self.depth_format = Some(format);
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
            .map(|a| Some(a.1.1))
            .collect::<Vec<_>>();
        let slot_map = bgls_sorted
            .into_iter()
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
        let (vs_main, vs_shader) = match &self.shader_src {
            ShaderSource::Empty => panic!("No shader source given"),
            ShaderSource::Combined { source, vertex_main_function, fragment_main_function: _ } =>
                (vertex_main_function, source),
            ShaderSource::Independent { vertex, vertex_main_function, fragment: _, fragment_main_function: _ } =>
                (vertex_main_function, vertex)
        };
        let vertex_shader = vgpu.device().create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: Some(&self.label),
                source: wgpu::ShaderSource::Wgsl(vs_shader.into())
            }
        );
        let (fs_main, fs_shader) = match &self.shader_src {
            ShaderSource::Empty => panic!("No shader source given"),
            ShaderSource::Combined { source, vertex_main_function: _, fragment_main_function } =>
                (fragment_main_function, source),
            ShaderSource::Independent { vertex: _, vertex_main_function: _, fragment, fragment_main_function } =>
                (fragment_main_function, fragment)
        };
        let fragment_shader = vgpu.device().create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: Some(&self.label),
                source: wgpu::ShaderSource::Wgsl(fs_shader.into())
            }
        );

        let pipeline = vgpu.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&self.label),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &vertex_shader,
                entry_point: Some(vs_main.as_str()),
                buffers: &self.vertex_layouts,
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &fragment_shader,
                entry_point: Some(fs_main.as_str()),
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
