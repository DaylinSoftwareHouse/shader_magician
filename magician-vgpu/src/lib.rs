use std::sync::Arc;

use getset::{Getters, MutGetters};

pub mod attachments;
pub mod frame;
pub mod pass;
pub mod textures;

pub use attachments::*;
pub use frame::*;
pub use pass::*;
pub use textures::*;

pub use magician_rust as magician_rust;
pub use magician_rust::glam as glam;

#[derive(Getters, MutGetters)]
pub struct VirtualGpu {
    #[getset(get = "pub", get_mut = "pub")]
    window: Arc<winit::window::Window>,
    #[getset(get = "pub", get_mut = "pub")]
    surface: wgpu::Surface<'static>,
    #[getset(get = "pub", get_mut = "pub")]
    device: wgpu::Device,
    #[getset(get = "pub", get_mut = "pub")]
    queue: wgpu::Queue,
    #[getset(get = "pub", get_mut = "pub")]
    config: wgpu::SurfaceConfiguration,
}

impl VirtualGpu {
    /// Creates a new `VirtualGpu` from a window instance.
    pub async fn new(window: Arc<winit::window::Window>) -> Self {
        let size = window.inner_size();

        // create wgpu instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });

        // create wgpu surface
        let surface = instance.create_surface(window.clone()).unwrap();

        // create wgpu adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        
        // create device and queue through adapter
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off, // Trace path
            })
            .await
            .unwrap();

        // pull surface capbilities and format
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        // configure surface
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        Self { window, surface, device, queue, config }
    }
}