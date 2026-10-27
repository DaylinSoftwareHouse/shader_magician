//! Compute shaders, run by `tests/compute.rs`.

use magician_rust::*;
use magician_rust::macros::*;

#[allow(dead_code)]
#[derive(ShaderLayout)]
pub struct MatvecIds {
    #[builtin = "workgroup_id"] wg: UVec3,
    #[builtin = "num_workgroups"] nwg: UVec3,
    #[builtin = "local_invocation_index"] lid: u32,
}

#[allow(dead_code)]
#[derive(ShaderGroup)]
pub struct MatvecBindings {
    /// rows, cols, accumulate
    #[uniform] dims: UVec4,
    /// Row-major bf16 weights, two per u32.
    #[read] w: RuntimeArray<u32>,
    #[read] x: RuntimeArray<f32>,
    #[write] y: RuntimeArray<f32>,
    #[workgroup] partial: [f32; 64],
}

/// y[row] (+)= dot(W[row, :], x), one 64 wide workgroup per row.  Rows past the
/// per-dimension dispatch limit spill into the grid's y dimension.
#[shader("./shader_out", compute(64))]
pub fn matvec_main(#[group = 0] mut b: MatvecBindings, ids: MatvecIds) {
    let row = ids.wg.x() + ids.wg.y() * ids.nwg.x();
    if row >= b.dims.x() { return; }

    let pairs = b.dims.y() / 2u32;
    let base = row * pairs;
    let mut acc = 0.0f32;
    let mut i = ids.lid;
    while i < pairs {
        let packed = b.w[(base + i) as usize];
        acc += bitcast_f32(packed << 16u32) * b.x[(2u32 * i) as usize]
            + bitcast_f32(packed & 0xffff0000u32) * b.x[(2u32 * i + 1u32) as usize];
        i += 64u32;
    }

    b.partial[ids.lid as usize] = acc;
    workgroup_barrier();
    let mut s = 32u32;
    while s > 0u32 {
        if ids.lid < s { b.partial[ids.lid as usize] += b.partial[(ids.lid + s) as usize]; }
        workgroup_barrier();
        s = s >> 1u32;
    }

    if ids.lid == 0u32 {
        if b.dims.z() != 0u32 { b.y[row as usize] += b.partial[0]; }
        else { b.y[row as usize] = b.partial[0]; }
    }
}
