//! `race::flush_draw_batch` (0x45FD1D): the scene's draw batch handed to the device: with
//! indices in it (`dword_6D3CEC`, edx), `sub_41ADF5` (the platform's `DrawIndexedPrimitive`:
//! eax the vertex count `dword_6D3CF0`, ebx the vertices `unk_6D3CF4`, ecx the indices
//! `unk_6D7CF4`); a failure (eax not 0) logged as "drawtri" and its code (level 1, each
//! inside `pusha`); then both counts 0.
//!
//! In:  nothing
//! Out: nothing its callers (`race::draw_sky`, `race::draw_view_scene` twice,
//!      `race::draw_view_overlays`) read

use crate::errors::Refused;
use crate::framework::{game_addr, rva};
use crate::physics::contact_store::CONTACT_WORK;
const VERTICES_RVA: usize = rva(0x6D3CF4); // unk_6D3CF4
const INDICES_RVA: usize = rva(0x6D7CF4); // unk_6D7CF4
const DRAWTRI_RVA: &std::ffi::CStr = c"drawtri"; // "drawtri"

/// The Rust version (see the header).
pub unsafe fn flush_draw_batch() -> Result<(), Refused> {
    unsafe {
        let indices: u32 = CONTACT_WORK.batch_index_count.bits();
        if indices != 0 {
            let result = crate::platform_3d::draw_indexed(
                game_addr(VERTICES_RVA) as u32,
                CONTACT_WORK.batch_vertex_count.bits(),
                game_addr(INDICES_RVA) as u32,
                indices,
            );
            if result != 0 {
                crate::logfile::log_write::log_write(
                    DRAWTRI_RVA.as_ptr() as usize as *const u8,
                    false,
                    1,
                );
                crate::logfile::log_hex8::log_hex8(result, false, 1);
            }
        }
        CONTACT_WORK.batch_index_count.set_bits(0u32);
        CONTACT_WORK.batch_vertex_count.set_bits(0u32);
    }
    Ok(())
}
