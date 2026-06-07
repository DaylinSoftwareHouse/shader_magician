use std::{convert::TryInto, sync::Arc};

use magician_vgpu::glam::{Mat3A, Mat4, Quat, Vec3, Vec4};
use magician_vgpu::{BindableObject, BindableObjectCreator, Buffer, ImmutableBuffer, LoadOp, MutableBuffer, PassAttachment, PassTarget, RenderFrame, StoreOp, VirtualGpu, WritableBuffer};
use model::{DrawLight, DrawModel, Vertex};
use shaders::common::{Camera, CameraInput, Light};
use winit::application::ApplicationHandler;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::{event::*, event_loop::EventLoop, keyboard::PhysicalKey, window::Window};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use winit::platform::web::EventLoopExtWebSys;


mod camera;
mod model;
mod resources;
mod texture;


const NUM_INSTANCES_PER_ROW: usize = 10;


fn build_camera_from_projection(camera: &camera::Camera, projection: &camera::Projection) -> Camera {
    let view_position = camera.position.to_homogeneous().into();
    let view_proj: [[f32; 4]; 4] = (projection.calc_matrix() * camera.calc_matrix()).into();

    Camera { view_pos: Vec4::from_array(view_position).into(), view_proj: Mat4::from_cols_array_2d(&view_proj).into() }
}

struct Instance {
    position: Vec3,
    rotation: Quat,
}

impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: Mat4::from_rotation_translation(self.rotation, self.position),
            normal: Mat3A::from_quat(self.rotation)
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[allow(dead_code)]
struct InstanceRaw {
    model: Mat4,
    normal: Mat3A
}

impl model::Vertex for InstanceRaw {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5 not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We don't have to do this in code though.
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 19]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 22]>() as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub struct State {
    vgpu: VirtualGpu,

    render_pipeline: wgpu::RenderPipeline,
    obj_model: model::Model,
    camera: camera::Camera,                      
    projection: camera::Projection,              
    camera_controller: camera::CameraController, 
    camera_uniform: Camera,
    camera_buffer: MutableBuffer<Camera>,
    camera_object: BindableObject<CameraInput>,
    instances: Vec<Instance>,
    #[allow(dead_code)]
    instance_buffer: ImmutableBuffer<[InstanceRaw; NUM_INSTANCES_PER_ROW * NUM_INSTANCES_PER_ROW]>,
    depth_texture: magician_vgpu::StaticTexture,
    is_surface_configured: bool,
    light_uniform: Light,
    light_buffer: MutableBuffer<Light>,
    light_bind_group: wgpu::BindGroup,
    light_render_pipeline: wgpu::RenderPipeline,
    #[allow(dead_code)]
    debug_material: model::Material,
    mouse_pressed: bool,
}

fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    shader: wgpu::ShaderModuleDescriptor,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(shader);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(&format!("{:?}", shader)),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: vertex_layouts,
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
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
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
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
        // If the pipeline will be used with a multiview render pass, this
        // tells wgpu to render to just specific texture layers.
        multiview_mask: None,
        cache: None,
    })
}

impl State {
    async fn new(window: Arc<Window>) -> anyhow::Result<State> {
        let vgpu = VirtualGpu::new(window).await;

        let texture_bind_group_layout =
            vgpu.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    // normal map
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        // create camera
        let camera = camera::Camera::new((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        let projection =
            camera::Projection::new(vgpu.config().width, vgpu.config().height, cgmath::Deg(45.0), 0.1, 100.0);
        let camera_controller = camera::CameraController::new(4.0, 0.4);
        // let mut camera_uniform = Camera::new();
        // camera_uniform.update_view_proj(&camera, &projection);
        let camera_uniform = build_camera_from_projection(&camera, &projection);
        let camera_buffer = MutableBuffer
            ::<Camera>
            ::new(&vgpu, camera_uniform, wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST);

        const SPACE_BETWEEN: f32 = 3.0;
        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                    let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

                    let position = Vec3::new(x, 0.0, z);

                    let rotation = if position.length() == 0.0 {
                        Quat::from_axis_angle(Vec3::Z, 0.0)
                    } else {
                        Quat::from_axis_angle(position.normalize(), 0.785398)
                    };

                    Instance { position, rotation }
                })
            })
            .collect::<Vec<_>>();

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = ImmutableBuffer
            ::<[InstanceRaw; NUM_INSTANCES_PER_ROW * NUM_INSTANCES_PER_ROW]>
            ::new(
                &vgpu, 
                instance_data.try_into().unwrap(), 
                wgpu::BufferUsages::VERTEX
            );

        let camera_object = CameraInput::create_object(&vgpu, &camera_buffer);

        let obj_model =
            resources::load_model("cube.obj", vgpu.device(), vgpu.queue(), &texture_bind_group_layout)
                .await
                .unwrap();

        let light_uniform = Light {
            position: Vec3::new(2.0, 2.0, 2.0).into(),
            _pad0: 0,
            color: Vec3::new(1.0, 1.0, 1.0).into(),
            _pad1: 0
        };
        let light_buffer = MutableBuffer
            ::new(&vgpu, light_uniform, wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST);

        let light_bind_group_layout =
            vgpu.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: None,
            });

        let light_bind_group = vgpu.device().create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.buffer().as_entire_binding(),
            }],
            label: None,
        });

        let depth_texture = texture::Texture::create_depth_texture(
            vgpu.device(), 
            vgpu.config(), 
            "depth_texture"
        );
        let depth_texture = magician_vgpu::StaticTexture::new(depth_texture.texture, depth_texture.view, depth_texture.sampler);

        let render_pipeline_layout =
            vgpu.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    Some(&texture_bind_group_layout),
                    Some(camera_object.layout()),
                    Some(&light_bind_group_layout),
                ],
                immediate_size: 0,
            });

        let render_pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Normal Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/src/shader.wgsl").into()),
            };
            create_render_pipeline(
                vgpu.device(),
                &render_pipeline_layout,
                vgpu.config().format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc(), InstanceRaw::desc()],
                shader,
            )
        };

        let light_render_pipeline = {
            let layout = vgpu.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Light Pipeline Layout"),
                bind_group_layouts: &[
                    Some(camera_object.layout()),
                    Some(&light_bind_group_layout),
                ],
                immediate_size: 0,
            });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Light Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/src/light.wgsl").into()),
            };
            create_render_pipeline(
                vgpu.device(),
                &layout,
                vgpu.config().format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc()],
                shader,
            )
        };

        let debug_material = {
            let diffuse_bytes = include_bytes!("../res/cobble-diffuse.png");
            let normal_bytes = include_bytes!("../res/cobble-normal.png");

            let diffuse_texture = texture::Texture::from_bytes(
                vgpu.device(),
                vgpu.queue(),
                diffuse_bytes,
                "res/alt-diffuse.png",
                false,
            )
            .unwrap();
            let normal_texture = texture::Texture::from_bytes(
                vgpu.device(),
                vgpu.queue(),
                normal_bytes,
                "res/alt-normal.png",
                true,
            )
            .unwrap();

            model::Material::new(
                vgpu.device(),
                "alt-material",
                diffuse_texture,
                normal_texture,
                &texture_bind_group_layout,
            )
        };

        Ok(Self {
            vgpu,
            render_pipeline,
            obj_model,
            camera,
            projection,
            camera_controller,
            camera_buffer,
            camera_object,
            camera_uniform,
            instances,
            instance_buffer,
            depth_texture,
            is_surface_configured: false,
            light_uniform,
            light_buffer,
            light_bind_group,
            light_render_pipeline,
            #[allow(dead_code)]
            debug_material,
            
            mouse_pressed: false,
        })
    }

    fn resize(&mut self, width: u32, height: u32) {
        
        if width > 0 && height > 0 {
            self.projection.resize(width, height);
            self.is_surface_configured = true;
            self.vgpu.config_mut().width = width;
            self.vgpu.config_mut().height = height;
            self.vgpu.surface().configure(self.vgpu.device(), self.vgpu.config());
            
            let depth_texture = texture::Texture::create_depth_texture(
                self.vgpu.device(), 
                self.vgpu.config(), 
                "depth_texture"
            );
            self.depth_texture = magician_vgpu::StaticTexture::new(depth_texture.texture, depth_texture.view, depth_texture.sampler);
        }
    }

    
    fn handle_key(&mut self, event_loop: &ActiveEventLoop, key: KeyCode, pressed: bool) {
        if !self.camera_controller.handle_key(key, pressed) {
            match (key, pressed) {
                (KeyCode::Escape, true) => event_loop.exit(),
                _ => {}
            }
        }
    }

    
    fn handle_mouse_button(&mut self, button: MouseButton, pressed: bool) {
        match button {
            MouseButton::Left => self.mouse_pressed = pressed,
            _ => {}
        }
    }

    
    fn handle_mouse_scroll(&mut self, delta: &MouseScrollDelta) {
        self.camera_controller.handle_scroll(delta);
    }

    fn update(&mut self, dt: std::time::Duration) {
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.camera_uniform = build_camera_from_projection(&self.camera, &self.projection);
        self.camera_buffer.write(&self.vgpu, self.camera_uniform)
            .expect("Failed to update camera buffer");

        // Update the light
        let old_position: Vec3 = self.light_uniform.position.into();
        self.light_uniform.position = (
            Quat::from_axis_angle(
                Vec3::new(0.0, 1.0, 0.0), 
                dt.as_secs_f32()
            ) * old_position
        ).into();
        self.light_buffer.write(&self.vgpu, self.light_uniform)
            .expect("Failed to write light buffer");
    }

    fn render(&mut self) -> anyhow::Result<()> {
        let Some(mut frame) = RenderFrame::begin(&self.vgpu)?
            else { return Ok(()) };

        {
            let mut pass = frame.init_pass(
                &[
                    PassAttachment {
                        target: PassTarget::PassOutput,
                        load_op: LoadOp::Clear(Vec4::new(0.1, 0.2, 0.3, 1.0)),
                        store_op: StoreOp::Store
                    }
                ], 
                Some(PassAttachment { 
                    target: PassTarget::Texture(&self.depth_texture),
                    load_op: LoadOp::Clear(1.0), 
                    store_op: StoreOp::Store
                })
            );

            pass.pass_mut().set_vertex_buffer(1, self.instance_buffer.buffer().slice(..));
            pass.pass_mut().set_pipeline(&self.light_render_pipeline);
            pass.pass_mut().draw_light_model(
                &self.obj_model,
                self.camera_object.bind_group(),
                &self.light_bind_group,
            );

            pass.pass_mut().set_pipeline(&self.render_pipeline);
            pass.pass_mut().draw_model_instanced(
                &self.obj_model,
                0..self.instances.len() as u32,
                self.camera_object.bind_group(),
                &self.light_bind_group,
            );
        }

        frame.submit();

        Ok(())
    }
}

pub struct App {
    #[cfg(target_arch = "wasm32")]
    proxy: Option<winit::event_loop::EventLoopProxy<State>>,
    state: Option<State>,
    last_time: instant::Instant,
}

impl App {
    pub fn new(#[cfg(target_arch = "wasm32")] event_loop: &EventLoop<State>) -> Self {
        #[cfg(target_arch = "wasm32")]
        let proxy = Some(event_loop.create_proxy());
        Self {
            state: None,
            #[cfg(target_arch = "wasm32")]
            proxy,
            last_time: instant::Instant::now(),
        }
    }
}

impl ApplicationHandler<State> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes();

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;

            const CANVAS_ID: &str = "canvas";

            let window = wgpu::web_sys::window().unwrap_throw();
            let document = window.document().unwrap_throw();
            let canvas = document.get_element_by_id(CANVAS_ID).unwrap_throw();
            let html_canvas_element = canvas.unchecked_into();
            window_attributes = window_attributes.with_canvas(Some(html_canvas_element));
        }

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        #[cfg(not(target_arch = "wasm32"))]
        {
            // If we are not on web we can use pollster to
            // await the
            self.state = Some(pollster::block_on(State::new(window)).unwrap());
        }

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(proxy) = self.proxy.take() {
                wasm_bindgen_futures::spawn_local(async move {
                    assert!(proxy
                        .send_event(
                            State::new(window)
                                .await
                                .expect("Unable to create canvas!!!")
                        )
                        .is_ok())
                });
            }
        }
    }

    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: State) {
        #[cfg(target_arch = "wasm32")]
        {
            event.window.request_redraw();
            event.resize(
                event.window.inner_size().width,
                event.window.inner_size().height,
            );
        }
        self.state = Some(event);
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        let state = if let Some(state) = &mut self.state {
            state
        } else {
            return;
        };
        match event {
            DeviceEvent::MouseMotion { delta: (dx, dy) } => {
                if state.mouse_pressed {
                    state.camera_controller.handle_mouse(dx, dy);
                }
            }
            _ => {}
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                let dt = self.last_time.elapsed();
                self.last_time = instant::Instant::now();
                state.update(dt);
                match state.render() {
                    Ok(_) => {}
                    Err(e) => {
                        // Log the error and exit gracefully
                        log::error!("{e}");
                        event_loop.exit();
                    }
                }
            }
            WindowEvent::MouseInput {
                state: btn_state,
                button,
                ..
            } => state.handle_mouse_button(button, btn_state.is_pressed()),
            WindowEvent::MouseWheel { delta, .. } => {
                state.handle_mouse_scroll(&delta);
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => state.handle_key(event_loop, code, key_state.is_pressed()),
            _ => {}
        }
    }
}

pub fn run() -> anyhow::Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Info).unwrap_throw();
    }

    let event_loop = EventLoop::with_user_event().build()?;
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut app = App::new();
        event_loop.run_app(&mut app)?;
    }
    #[cfg(target_arch = "wasm32")]
    {
        let app = App::new(&event_loop);
        event_loop.spawn_app(app);
    }

    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run_web() -> Result<(), wasm_bindgen::JsValue> {
    console_error_panic_hook::set_once();
    run().unwrap_throw();

    Ok(())
}
