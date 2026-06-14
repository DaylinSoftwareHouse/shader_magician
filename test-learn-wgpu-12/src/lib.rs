use std::{convert::TryInto, sync::Arc};

use magician_vgpu::glam::{Quat, Vec3, Vec4};
use magician_vgpu::{BindGroupProvider, BindableObject, Buffer, ImmutableBuffer, LoadOp, MutableBuffer, PassAttachment, PassTarget, Pipeline, RenderFrame, ShaderSource, StoreOp, VirtualGpu, WritableBuffer};
use model::Vertex;
use shaders::common::{Camera, CameraInput, Light, LightInput, Material};
use winit::application::ApplicationHandler;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::{event::*, event_loop::EventLoop, keyboard::PhysicalKey, window::Window};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use winit::platform::web::EventLoopExtWebSys;

use crate::instance::*;

mod camera;
mod instance;
mod model;
mod resources;
mod texture;

pub struct State {
    vgpu: VirtualGpu,

    render_pipeline: Pipeline,
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
    light_object: BindableObject<LightInput>,
    light_render_pipeline: Pipeline,
    #[allow(dead_code)]
    debug_material: model::Material,
    mouse_pressed: bool,
}

impl State {
    async fn new(window: Arc<Window>) -> anyhow::Result<State> {
        let vgpu = VirtualGpu::new(window).await;

        let material_bgl = Material::layout(&vgpu, wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX);

        // create camera
        let camera = camera::Camera::new((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        let projection =
            camera::Projection::new(vgpu.config().width, vgpu.config().height, cgmath::Deg(45.0), 0.1, 100.0);
        let camera_controller = camera::CameraController::new(4.0, 0.4);
        let camera_uniform = build_camera_from_projection(&camera, &projection);
        let camera_buffer = MutableBuffer
            ::<Camera>
            ::new(&vgpu, camera_uniform, wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST);
        let camera_object = BindableObject::<CameraInput>
            ::from_inputs(&vgpu, camera_buffer.buffer());

        
        // create instances
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

        let obj_model =
            resources::load_model("cube.obj", &vgpu)
                .await
                .unwrap();

        // load light
        let light_uniform = Light {
            position: Vec3::new(2.0, 2.0, 2.0).into(),
            _pad0: 0,
            color: Vec3::new(1.0, 1.0, 1.0).into(),
            _pad1: 0
        };
        let light_buffer = MutableBuffer
            ::new(&vgpu, light_uniform, wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST);
        let light_object = BindableObject::<LightInput>
            ::from_inputs(&vgpu, light_buffer.buffer());

        // load depth texture
        let depth_texture = texture::Texture::create_depth_texture(
            vgpu.device(), 
            vgpu.config(), 
            "depth_texture"
        );
        let depth_texture = magician_vgpu::StaticTexture::new(depth_texture.texture, depth_texture.view, depth_texture.sampler);

        let render_pipeline = Pipeline::builder("Normal Shader")
            .shader(ShaderSource::Independent { 
                vertex: include_str!("../shaders/shader_out/primary_vs_main.wgsl").into(), 
                vertex_main_function: "primary_vs_main".into(), 
                fragment: include_str!("../shaders/shader_out/primary_fs_main.wgsl").into(), 
                fragment_main_function: "primary_fs_main".into()
            })
            .depth_format(texture::Texture::DEPTH_FORMAT)
            .vertex(model::ModelVertex::desc())
            .vertex(InstanceRaw::desc())
            .layout_raw::<Material>(&material_bgl)
            .layout(&camera_object)
            .layout(&light_object)
            .build(&vgpu);

        let light_render_pipeline = Pipeline::builder("Light Shader")
            .shader(ShaderSource::Independent { 
                vertex: include_str!("../shaders/shader_out/light_vs_main.wgsl").into(), 
                vertex_main_function: "light_vs_main".into(), 
                fragment: include_str!("../shaders/shader_out/light_fs_main.wgsl").into(), 
                fragment_main_function: "light_fs_main".into()
            })
            .depth_format(texture::Texture::DEPTH_FORMAT)
            .vertex(model::ModelVertex::desc())
            .layout(&camera_object)
            .layout(&light_object)
            .build(&vgpu);

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
                &vgpu,
                "alt-material",
                diffuse_texture,
                normal_texture
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
            light_object,
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

            pass.pass_mut().set_pipeline(&self.light_render_pipeline.pipeline());
            pass.pass_mut().set_vertex_buffer(1, self.instance_buffer.buffer().slice(..));
            pass.pass_mut().set_bind_group(0, self.camera_object.bind_group(), &[]);
            pass.pass_mut().set_bind_group(1, self.light_object.bind_group(), &[]);
            for mesh in &self.obj_model.meshes {
                pass.pass_mut().set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                pass.pass_mut().set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                pass.pass_mut().draw_indexed(0..mesh.num_elements, 0, 0 .. 1);
            }

            pass.pass_mut().set_pipeline(&self.render_pipeline.pipeline());
            pass.pass_mut().set_bind_group(1, self.camera_object.bind_group(), &[]);
            pass.pass_mut().set_bind_group(2, self.light_object.bind_group(), &[]);    
            for mesh in &self.obj_model.meshes {
                let material = &self.obj_model.materials[mesh.material];
                pass.pass_mut().set_bind_group(0, Some(material.bindable.bind_group()), &[]);
                pass.pass_mut().set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                pass.pass_mut().set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                pass.pass_mut().draw_indexed(0..mesh.num_elements, 0, 0..self.instances.len() as u32);
            }
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
        let Some(state) = self.state.as_mut() else { return };

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
