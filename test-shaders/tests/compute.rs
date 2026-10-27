use test_shaders::compute::SHADER_matvec_main;
use wgpu::util::DeviceExt;

/// Runs the transpiled `matvec_main` in accumulate mode against bf16 weights and checks
/// it against a CPU reference.  Skips if there is no GPU adapter.
#[test]
fn transpiled_matvec_matches_cpu() {
    let (rows, cols) = (300usize, 200usize);

    // deterministic bf16 weights in [-1, 1) and f32 inputs
    let mut seed = 0x1234_5678u32;
    let mut next = || { seed ^= seed << 13; seed ^= seed >> 17; seed ^= seed << 5; (seed >> 8) as f32 / (1u32 << 23) as f32 * 2.0 - 1.0 };
    let weights: Vec<u16> = (0 .. rows * cols).map(|_| (next().to_bits() >> 16) as u16).collect();
    let x: Vec<f32> = (0 .. cols).map(|_| next()).collect();
    let expected: Vec<f32> = (0 .. rows)
        .map(|r| 1.0 + (0 .. cols).map(|c| f32::from_bits((weights[r * cols + c] as u32) << 16) * x[c]).sum::<f32>())
        .collect();

    let instance = wgpu::Instance::default();
    let adapter = match pollster::block_on(instance.request_adapter(&Default::default())) {
        Ok(adapter) => adapter,
        Err(err) => { eprintln!("skipping, no GPU: {err}"); return }
    };
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();

    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("matvec"),
        source: wgpu::ShaderSource::Wgsl(SHADER_matvec_main.into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("matvec"),
        layout: None,
        module: &module,
        entry_point: Some("matvec_main"),
        compilation_options: Default::default(),
        cache: None,
    });

    let init = |contents: &[u8], usage| device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents, usage });
    let dims = init(bytemuck::cast_slice(&[rows as u32, cols as u32, 1, 0]), wgpu::BufferUsages::UNIFORM);
    let w = init(bytemuck::cast_slice(&weights), wgpu::BufferUsages::STORAGE);
    let xb = init(bytemuck::cast_slice(&x), wgpu::BufferUsages::STORAGE);
    let y = init(bytemuck::cast_slice(&vec![1.0f32; rows]), wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC);
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (rows * 4) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[&dims, &w, &xb, &y].iter().enumerate()
            .map(|(i, b)| wgpu::BindGroupEntry { binding: i as u32, resource: b.as_entire_binding() })
            .collect::<Vec<_>>(),
    });

    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_compute_pass(&Default::default());
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(rows as u32, 1, 1);
    }
    encoder.copy_buffer_to_buffer(&y, 0, &readback, 0, (rows * 4) as u64);
    queue.submit([encoder.finish()]);

    readback.slice(..).map_async(wgpu::MapMode::Read, |r| r.unwrap());
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    let result: Vec<f32> = bytemuck::cast_slice(&readback.slice(..).get_mapped_range()).to_vec();

    for (row, (got, want)) in result.iter().zip(&expected).enumerate() {
        assert!((got - want).abs() <= 1e-4 * want.abs().max(1.0), "row {row}: got {got}, want {want}");
    }
}
