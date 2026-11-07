//! `race::flush_draw_batch` (0x45FD1D): the scene's draw batch handed to the device: with
//! indices in it (`dword_6D3CEC`, edx), `sub_41ADF5` (the platform's `DrawIndexedPrimitive`:
//! eax the vertex count `dword_6D3CF0`, ebx the vertices `unk_6D3CF4`, ecx the indices
//! `unk_6D7CF4`); a failure (eax not 0) logged as "drawtri" and its code (level 1, each
//! inside `pusha`); then both counts 0.

use crate::errors::Refused;
use crate::race::owner::Race;
const DRAWTRI_RVA: &std::ffi::CStr = c"drawtri"; // "drawtri"

/// The Rust version (see the header).
pub fn flush_draw_batch(race: &Race) -> Result<(), Refused> {
    let indices: u32 = race.physics.contact.batch_index_count.bits();
    if indices != 0 {
        let result = crate::platform_3d::draw_indexed(
            &race.frame.draw_batches.scene_vertices,
            race.physics.contact.batch_vertex_count.bits(),
            &race.frame.draw_batches.scene_indices,
            indices,
        );
        if result != 0 {
            crate::logfile::log_write::log_nul(&DRAWTRI_RVA, false, 1);
            crate::logfile::log_hex8::log_hex8(result, false, 1);
        }
    }
    race.physics.contact.batch_index_count.set_bits(0u32);
    race.physics.contact.batch_vertex_count.set_bits(0u32);
    Ok(())
}
