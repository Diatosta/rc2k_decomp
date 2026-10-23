//! `d3d::draw_indexed` (0x41ADF5): draws an indexed list of triangles.
//!
//! In:  ebx = the vertices, eax = their number, ecx = the indices (16 bits), edx = their
//!      number
//! Out: eax = Direct3D's result (0: drawn); ecx, edx as Direct3D leaves them
//! The device's DrawIndexedPrimitive with a triangle list (4) of transformed and lit
//! vertices (3) and the flags 0x1C (not to be clipped, lit or measured). Callers: the
//! scene's batches (`race::flush_draw_batch`, `race/frame_flush.rs`).
//! Platform: the original always runs. In the standalone program the rasteriser draws
//! each triangle (`platform_3d::draw_indexed_list`) and the result is 0. No self-test.

use crate::framework::Regs;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { crate::platform_3d::draw_indexed_list(r.ebx, r.eax, r.ecx, r.edx) };
    r.eax = 0;
    Ok(())
}
