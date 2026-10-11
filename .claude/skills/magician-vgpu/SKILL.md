---
name: magician-vgpu
description: Use magician-vgpu's VirtualGpu to render with wgpu without the boilerplate. Covers windowed and headless setup, buffers (Immutable/Mutable/Dynamic/Chunked/Tree), StaticTexture, BindableObject bind groups, the Pipeline builder, and RenderFrame/SinglePass drawing and frame capture. Use when writing or changing rendering code that imports `magician_vgpu`, wiring transpiled shaders into pipelines, or adding a headless GPU test.
---

# magician-vgpu: rendering with VirtualGpu

`magician_vgpu` wraps a wgpu device, queue, and surface (`VirtualGpu`) and provides typed helpers. The crate re-exports `magician_rust` as `rust`, `glam`, and its derive macros as `macros`. Your crate still needs `wgpu = "29.0"` (the derives expand to `wgpu::` paths). A windowed app also needs `winit = "0.30"`, and anything async needs `pollster` or similar.

The working reference app is `test-learn-wgpu-12/src/lib.rs` (`State::new` and `render`). Copy its patterns.

## 1. Create the GPU

```rust
// windowed (winit 0.30): after the window exists
let vgpu = VirtualGpu::new(Arc::new(window)).await;
// on WindowEvent::Resized(size):
vgpu.resize(size.width, size.height);           // ignores 0 sizes; recreate depth textures yourself

// headless (tests, offscreen rendering): format is HEADLESS_FORMAT (Rgba8UnormSrgb)
let vgpu = pollster::block_on(VirtualGpu::new_headless(256, 256))?;
```

Accessors: `device()`, `queue()`, `config()` (width, height, format), `window()`, `surface()`, `is_headless()`, `request_redraw()`, and `supports_bindless_arrays()`. On GL/WebGL the last one is false, so fall back to an atlas instead of `BindlessArray`.

## 2. Buffers (`T: bytemuck::Pod`, or `[T]` for arrays)

| Type | Use | Notes |
|---|---|---|
| `ImmutableBuffer::<[V]>::new(&vgpu, &data[..], BufferUsages::VERTEX)` | vertex, index, and instance data that never changes | |
| `MutableBuffer::new(&vgpu, &value, UNIFORM \| COPY_DST)` | uniforms you update every frame | `.write(&vgpu, &new)` needs the `WritableBuffer` trait in scope and the **same element count** |
| `DynamicBuffer::new(&vgpu, &data, usage)` | data that changes size | `write` reallocates when the size changes, so **rebuild any bind group** that references it |
| `ChunkedBuffer`, `TreeBuffer` | allocator-style storage for chunk lists and flattened trees | see `magician-vgpu/src/buffers/{chunked,tree}.rs` docs |

All of them implement `Buffer` (`buffer() -> &wgpu::Buffer`, `size()` = element count).

## 3. Textures

```rust
let tex = StaticTexture::from_raw(&vgpu, TextureDescriptor {
    usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
    mag_filter: FilterMode::Linear, ..Default::default()        // default: Rgba8UnormSrgb, nearest, clamp
}, &rgba_bytes, width, height);
tex.write_region(&vgpu, &bytes, UVec2::new(x, y), UVec2::new(w, h));  // sub-image update (atlas)

let depth = StaticTexture::framebuffer(&vgpu, TextureFormat::Depth32Float,
    TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING);   // sized to config()
```

`from_raw` and `write_region` assume 4 bytes per pixel.

## 4. Bind groups: one struct for the shader and the CPU

Define the group once, in the shader crate:

```rust
#[derive(ShaderGroup, BindableObject)]           // magician_macros::ShaderGroup + magician_vgpu::macros::BindableObject
pub struct Material { pub t_diffuse: Texture2D, pub s_diffuse: Sampler }

#[derive(ShaderGroup, BindableObject)]
pub struct CameraInput { #[uniform] pub camera: Camera }

#[repr(C)] #[derive(Pod, Zeroable, Clone, Copy, UniformBufferObject)]
pub struct Camera { pub view_pos: Vec4, pub view_proj: Mat4 }
```

Binding `i` is field `i` in both the WGSL and the layout. `from_inputs` takes **one input per field, in field order**: the input itself when there is one field, or a tuple when there are several.

| Field type | Input |
|---|---|
| struct deriving `UniformBufferObject` | `wgpu::Buffer` (pass `buf.buffer()`) |
| `Texture2D` | `wgpu::TextureView` |
| `Sampler` | `wgpu::Sampler` |
| `BindlessArray<Texture2D>` | `Vec<&wgpu::TextureView>` (requires `supports_bindless_arrays()`) |
| `Vec4`, `u32` | `MutableBuffer<Vec4>`, `MutableBuffer<u32>` |

```rust
let camera_buf = MutableBuffer::<Camera>::new(&vgpu, &cam, BufferUsages::UNIFORM | BufferUsages::COPY_DST);
let camera_obj = BindableObject::<CameraInput>::from_inputs(&vgpu, camera_buf.buffer());
let mat_obj = BindableObject::<Material>::from_inputs(&vgpu, &(tex.view.clone(), tex.sampler.clone()));
```

Visibility is always `VERTEX | FRAGMENT`. For a layout without a bind group (for example a material whose bind groups are made later), call `Material::layout(&vgpu, stages)` directly; it needs the `BindGroupProvider` trait in scope.

## 5. Pipelines

```rust
let pipeline = Pipeline::builder("main")
    .source(ShaderType::Vertex,   ShaderSource { source: SHADER_my_vs_main.into(), main_function: "my_vs_main".into() })
    .source(ShaderType::Fragment, ShaderSource { source: SHADER_my_fs_main.into(), main_function: "my_fs_main".into() })
    .vertex(ModelVertex::desc())          // slot 0: per-vertex wgpu::VertexBufferLayout
    .vertex(InstanceRaw::desc())          // slot 1: per-instance (step_mode Instance)
    .layout(0, &material_obj)             // group index must match #[group = N] in the shader
    .layout(1, &camera_obj)
    .layout_raw::<Material>(0, material_bgl)  // alternative when you only have a layout
    .depth_format(TextureFormat::Depth32Float)
    .build(&vgpu);
```

Fixed defaults: TriangleList, CCW, back-face culling, alpha blending, one color target in `config().format`, and depth `Less` with writes when `depth_format` is set. Anything else needs a builder change in `pipeline.rs`. Vertex `#[location]`s in the shader must match the `VertexBufferLayout` attributes.

Bind slots are keyed by the **group's Rust type**, so a pipeline can't hold two groups of the same type.

## 6. Rendering a frame

```rust
fn render(&mut self) -> anyhow::Result<()> {
    let Some(mut frame) = RenderFrame::begin(&self.vgpu)? else { return Ok(()) }; // None means skip this frame
    {
        let mut pass = frame.init_pass(
            &[PassAttachment { target: PassTarget::PassOutput, load_op: LoadOp::Clear(Vec4::new(0.1, 0.2, 0.3, 1.0)), store_op: StoreOp::Store }],
            Some(PassAttachment { target: PassTarget::Texture(&self.depth), load_op: LoadOp::Clear(1.0), store_op: StoreOp::Store }),
        );
        pass.use_pipeline(self.pipeline.get_ref());   // pipeline held as mutual::CowData<Pipeline>
        pass.bind(&self.camera_obj);                  // returns false if the pipeline has no slot for this type
        pass.bind_instances(&self.instances);         // vertex slot 1; sets the default instance count
        pass.draw(&mesh.vertices, &mesh.indices, DrawSettings::default()); // u32 indices; IndexFmt::Light for u16
    } // the pass must drop before submit
    frame.submit(&self.vgpu);
    Ok(())
}
```

- `Vec4` here is `magician_vgpu::glam::Vec4`, not `magician_rust::Vec4`.
- `use_pipeline` resets the instance count to 1, so call `bind_instances` *after* it.
- Check the `bool` from `bind`. A `false` means the group type wasn't registered on that pipeline, and the draw will fail validation.

## 7. Headless capture and tests

```rust
let mut frame = RenderFrame::begin(&vgpu)?.expect("headless frames always begin");
drop(frame.init_pass(&[/* ... */], None));
let img: FrameCapture = frame.submit_and_capture(&vgpu)?;  // RGBA8, top row first
assert_eq!(img.pixel(0, 0), Some([255, 0, 0, 255]));
```

Copy the test pattern from `magician-vgpu/src/frame.rs`: if `new_headless` returns `Err`, print `skipping, no GPU` and return, so CI without a GPU still passes. Windowed frames can be captured too when the surface supports `COPY_SRC`.
