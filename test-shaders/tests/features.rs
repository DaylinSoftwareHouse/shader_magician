use magician_vgpu::{LoadOp, PassAttachment, PassTarget, Pipeline, RenderFrame, ShaderSource, ShaderType, StoreOp, VirtualGpu};
use test_shaders::features::{SHADER_features_fs_main, SHADER_fullscreen_vs_main};

/// Renders `features_fs_main` over the whole target and checks every pixel is magenta.
/// Skips if there is no GPU adapter.
#[test]
fn transpiled_features_render_magenta() {
    let vgpu = match pollster::block_on(VirtualGpu::new_headless(4, 4)) {
        Ok(vgpu) => vgpu,
        Err(err) => { eprintln!("skipping, no GPU: {err}"); return }
    };

    let pipeline = Pipeline::builder("features")
        .source(ShaderType::Vertex, ShaderSource { source: SHADER_fullscreen_vs_main.into(), main_function: "fullscreen_vs_main".into() })
        .source(ShaderType::Fragment, ShaderSource { source: SHADER_features_fs_main.into(), main_function: "features_fs_main".into() })
        .build(&vgpu);

    let mut frame = RenderFrame::begin(&vgpu).unwrap().expect("headless frames always begin");
    {
        let mut pass = frame.init_pass(&[PassAttachment {
            target: PassTarget::PassOutput,
            load_op: LoadOp::Clear(magician_vgpu::glam::Vec4::ZERO),
            store_op: StoreOp::Store,
        }], None);
        pass.pass_mut().set_pipeline(pipeline.pipeline());
        pass.pass_mut().draw(0 .. 3, 0 .. 1);
    }
    let capture = frame.submit_and_capture(&vgpu).unwrap();

    for y in 0 .. capture.height {
        for x in 0 .. capture.width {
            assert_eq!(capture.pixel(x, y), Some([255, 0, 255, 255]), "pixel {x}, {y}");
        }
    }
}
