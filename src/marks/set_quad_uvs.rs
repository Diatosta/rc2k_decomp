//! `marks::set_quad_uvs` (0x4CFD6A): writing to the depth buffer off, and the four texture
//! places of the 0x20-byte set at `dword_920738` + 0xC60 put into the four vertices'
//! texture places at `flt_9605B8` (0x20 bytes apart: +0x18 of Direct3D vertices from
//! 0x9605A0, a reading).
//!
//! In:  nothing
//! Out: nothing (ebp, the last place's v, is read only by the original `jitter_point`
//!      under `trail_particles`; the Rust version reads no register)
//!
//! One caller, `sub_4CFB49` (the marks' frame).

use crate::d3d::vertices_store::VERTICES;
use crate::marks::blocks::CORNER_SLOPES;

/// The record of `g_marksCornerSlopes` whose eight values are the set (0xC60 bytes on).
const SET_RECORD: usize = 0xC60 / 0x24;

/// The Rust version (see the header).
pub unsafe fn set_quad_uvs() {
    {
        crate::scene::depth_test_off::depth_test_off();
        for n in 0..4 {
            let values = &CORNER_SLOPES.slopes[SET_RECORD].values;
            let uv: [u32; 2] = [values[2 * n].bits(), values[2 * n + 1].bits()];
            VERTICES.vertices[n].u.set_bits(uv[0]);
            VERTICES.vertices[n].v.set_bits(uv[1]);
        }
    }
}
