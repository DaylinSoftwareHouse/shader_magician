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

/// A GPU device and queue, plus what frames render into: a window's surface, or an
/// offscreen texture for a headless `VirtualGpu` (see [`VirtualGpu::new_headless`]).
#[derive(Getters, MutGetters)]
pub struct VirtualGpu {
    /// The window being drawn to, `None` when headless.
    window: Option<Arc<winit::window::Window>>,
    /// The window's surface, `None` when headless.
    surface: Option<wgpu::Surface<'static>>,
    #[getset(get = "pub", get_mut = "pub")]
    device: wgpu::Device,
    #[getset(get = "pub", get_mut = "pub")]
    queue: wgpu::Queue,
    /// The surface configuration.  A headless `VirtualGpu` uses its size and format for
    /// the offscreen target.
    #[getset(get = "pub", get_mut = "pub")]
    config: wgpu::SurfaceConfiguration,
    /// Whether the adapter this `VirtualGpu` was created on supports `wgpu::Features::TEXTURE_BINDING_ARRAY`
    /// and `PARTIALLY_BOUND_BINDING_ARRAY` (true bindless texture arrays, populated incrementally as
    /// textures load rather than all 128 slots at once). GL/WebGL2 backends never support this; callers
    /// should fall back to a non-bindless rendering path (e.g. a texture atlas) when this is `false`.
    #[getset(get = "pub")]
    supports_bindless_arrays: bool,
    /// The texture frames render into when headless, matching `config`'s size and format.
    #[getset(get = "pub")]
    offscreen: Option<wgpu::Texture>,
}

/// The `wgpu` features a texture binding array needs: the array binding itself, plus the
/// ability to bind fewer than the declared array size (gearbox's bindless vault grows its
/// array from 0 as textures load, so a fixed-size-only array would never validate until all
/// 128 slots were filled).
const BINDLESS_ARRAY_FEATURES: wgpu::Features = wgpu::Features::TEXTURE_BINDING_ARRAY.union(wgpu::Features::PARTIALLY_BOUND_BINDING_ARRAY)
    .union(wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING);

/// The color format of a headless `VirtualGpu`'s offscreen target.
pub const HEADLESS_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

impl VirtualGpu {
    /// Creates a new `VirtualGpu` from a window instance.
    pub async fn new(window: Arc<winit::window::Window>) -> Self {
        let size = window.inner_size();
        let instance = create_instance();

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
        let (device, queue, supports_bindless_arrays) = request_device(&adapter).await.unwrap();

        // pull surface capbilities and format
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        // configure surface, allowing copies out of it where supported so frames can be captured
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | (surface_caps.usages & wgpu::TextureUsages::COPY_SRC),
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: pick_alpha_mode(&surface_caps.alpha_modes),
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        Self { window: Some(window), surface: Some(surface), device, queue, config, supports_bindless_arrays, offscreen: None }
    }

    /// Creates a `VirtualGpu` with no window, whose frames render into a `width` by `height`
    /// offscreen texture in [`HEADLESS_FORMAT`].  Capture them with
    /// [`RenderFrame::submit_and_capture`].
    ///
    /// # Errors
    ///
    /// Fails if there is no usable GPU adapter or device.
    pub async fn new_headless(width: u32, height: u32) -> anyhow::Result<Self> {
        let adapter = create_instance()
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .map_err(|err| anyhow::anyhow!("No GPU adapter for a headless VirtualGpu: {err}"))?;
        let (device, queue, supports_bindless_arrays) = request_device(&adapter).await?;

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            format: HEADLESS_FORMAT,
            width: width.max(1),
            height: height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        let offscreen = Some(create_offscreen(&device, &config));

        Ok(Self { window: None, surface: None, device, queue, config, supports_bindless_arrays, offscreen })
    }

    /// The window being drawn to, `None` when headless.
    pub fn window(&self) -> Option<&Arc<winit::window::Window>> {
        self.window.as_ref()
    }

    /// The window's surface, `None` when headless.
    pub fn surface(&self) -> Option<&wgpu::Surface<'static>> {
        self.surface.as_ref()
    }

    /// Returns true if this `VirtualGpu` renders offscreen rather than to a window.
    pub fn is_headless(&self) -> bool {
        self.surface.is_none()
    }

    /// Ask the window for another redraw.  Does nothing when headless.
    pub fn request_redraw(&self) {
        if let Some(window) = &self.window { window.request_redraw(); }
    }

    /// Resize what frames render into: reconfigures the surface, or recreates the
    /// offscreen target when headless.  Does nothing if either size is 0.
    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 { return }
        self.config.width = width;
        self.config.height = height;
        match &self.surface {
            Some(surface) => surface.configure(&self.device, &self.config),
            None => self.offscreen = Some(create_offscreen(&self.device, &self.config)),
        }
    }
}

/// Creates the `wgpu` instance for this platform's backends.
fn create_instance() -> wgpu::Instance {
    wgpu::Instance::new(wgpu::InstanceDescriptor {
        // windows drivers' vulkan surfaces are opaque only, so use dx12 there
        #[cfg(target_os = "windows")]
        backends: wgpu::Backends::DX12,
        #[cfg(not(any(target_os = "windows", target_arch = "wasm32")))]
        backends: wgpu::Backends::PRIMARY,
        #[cfg(target_arch = "wasm32")]
        backends: wgpu::Backends::GL,
        flags: Default::default(),
        memory_budget_thresholds: Default::default(),
        backend_options: wgpu::BackendOptions {
            // present through DirectComposition, the only dx12 swapchain that supports
            // transparent windows. `WGPU_DX12_PRESENTATION_SYSTEM=Hwnd` switches back, e.g. for RenderDoc
            dx12: wgpu::Dx12BackendOptions {
                presentation_system: wgpu::Dx12SwapchainKind::DxgiFromVisual.with_env(),
                ..wgpu::Dx12BackendOptions::from_env_or_default()
            },
            ..wgpu::BackendOptions::from_env_or_default()
        },
        display: None,
    })
}

/// Picks a composite alpha mode that blends the surface with what's behind the window, so
/// transparent clear colors and pixels show through, falling back to the first supported.
/// Colors are premultiplied when the surface uses [`wgpu::CompositeAlphaMode::PreMultiplied`].
fn pick_alpha_mode(supported: &[wgpu::CompositeAlphaMode]) -> wgpu::CompositeAlphaMode {
    [wgpu::CompositeAlphaMode::PreMultiplied, wgpu::CompositeAlphaMode::PostMultiplied]
        .into_iter()
        .find(|mode| supported.contains(mode))
        .unwrap_or(supported[0])
}

/// Requests a device and queue from `adapter`, with bindless texture arrays if supported.
/// Returns whether they were.
async fn request_device(adapter: &wgpu::Adapter) -> anyhow::Result<(wgpu::Device, wgpu::Queue, bool)> {
    // a texture-binding-array (true bindless textures) is only usable if the adapter
    // actually supports it; GL/WebGL2 adapters never do, so callers need to know this to
    // pick a fallback (atlas-based) rendering path instead
    let supports_bindless_arrays = adapter.features().contains(BINDLESS_ARRAY_FEATURES);

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: if supports_bindless_arrays { BINDLESS_ARRAY_FEATURES } else { wgpu::Features::empty() },
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            required_limits: if cfg!(target_arch = "wasm32") {
                wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits())
            } else if supports_bindless_arrays {
                wgpu::Limits {
                    max_binding_array_elements_per_shader_stage: 128,
                    ..adapter.limits()
                }
            } else {
                wgpu::Limits::default()
            },
            memory_hints: Default::default(),
            trace: wgpu::Trace::Off, // Trace path
        })
        .await?;
    Ok((device, queue, supports_bindless_arrays))
}

/// Creates the offscreen render target a headless `VirtualGpu` draws into.
fn create_offscreen(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("headless_frame"),
        size: wgpu::Extent3d { width: config.width, height: config.height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: config.format,
        usage: config.usage,
        view_formats: &[],
    })
}
