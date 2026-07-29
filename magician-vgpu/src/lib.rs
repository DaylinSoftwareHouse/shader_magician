use std::sync::Arc;

use getset::{Getters, MutGetters};

pub mod attachments;
pub mod bindable;
pub mod buffers;
pub mod frame;
pub mod pass;
pub mod pipeline;
pub mod textures;

pub use attachments::*;
pub use bindable::*;
pub use buffers::*;
pub use frame::*;
pub use pass::*;
pub use pipeline::*;
pub use textures::*;

pub use magician_rust as rust;
pub use magician_rust::glam as glam;
pub use magician_vgpu_macros as macros;

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
    /// Whether the adapter this `VirtualGpu` was created on supports `wgpu::Features::TEXTURE_BINDING_ARRAY`
    /// and `PARTIALLY_BOUND_BINDING_ARRAY` (true bindless texture arrays, populated incrementally as
    /// textures load rather than all 128 slots at once). GL/WebGL2 backends never support this; callers
    /// should fall back to a non-bindless rendering path (e.g. a texture atlas) when this is `false`.
    #[getset(get = "pub")]
    supports_bindless_arrays: bool,
}

/// The `wgpu` features a texture binding array needs: the array binding itself, plus the
/// ability to bind fewer than the declared array size (gearbox's bindless vault grows its
/// array from 0 as textures load, so a fixed-size-only array would never validate until all
/// 128 slots were filled).
const BINDLESS_ARRAY_FEATURES: wgpu::Features = wgpu::Features::TEXTURE_BINDING_ARRAY.union(wgpu::Features::PARTIALLY_BOUND_BINDING_ARRAY);

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
        
        // a texture-binding-array (true bindless textures) is only usable if the adapter
        // actually supports it; GL/WebGL2 adapters never do, so callers need to know this to
        // pick a fallback (atlas-based) rendering path instead
        let supports_bindless_arrays = adapter.features().contains(BINDLESS_ARRAY_FEATURES);

        // create device and queue through adapter
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: if supports_bindless_arrays { BINDLESS_ARRAY_FEATURES } else { wgpu::Features::empty() },
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else if supports_bindless_arrays {
                    wgpu::Limits {
                        max_binding_array_elements_per_shader_stage: 128,
                        ..Default::default()
                    }
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

        Self { window, surface, device, queue, config, supports_bindless_arrays }
    }
}