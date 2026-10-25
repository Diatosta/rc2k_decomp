//! `marks::ring_frame` (0x4CB5B9): the emitters' ring drawn: render states 0x15 = 4, 0x13 =
//! 5, 0x14 = 6, 0x1C = 1, 0x1B = 1, `scene::set_texture` (`dword_5D1E34`),
//! `scene::depth_test_off`; `dword_939BA6` the settings' +0x7C's low 24 bits,
//! `g_marksKind1Choice` `dword_8BC50D` (the rain's drops); `marks::run_ring`; render state
//! 0x14 = 2; `marks::run_ribbons`.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (no caller reads a register it leaves)
//!
//! Caller: `marks::run_frame` (0x4D160B). Refuses null settings, before anything.

use crate::errors::{BadInput, Refused};
use crate::marks::emit_kind_1::KIND1_CHOICE;
use crate::marks::surface_store::SURFACE;
use crate::scene::kind_texture_store::KIND_TEXTURES;
/// The render states set (value, state).
const STATES: [(u32, u32); 5] = [(4, 0x15), (5, 0x13), (6, 0x14), (1, 0x1C), (1, 0x1B)];

/// The Rust version (see the header): the render states (`scene::set_render_state`), the
/// texture (`scene::set_texture`), `scene::depth_test_off`, `run_ring`, `run_ribbons`.
pub unsafe fn ring_frame() -> Result<(), Refused> {
    use crate::scene::set_render_state::set_render_state;
    unsafe {
        let s: u32 = crate::marks::setup::SETTINGS.pointer.get();
        if s == 0 {
            return Err(BadInput("no marks settings").into());
        }
        for (value, state) in STATES {
            set_render_state(value, state);
        }
        crate::scene::set_texture::set_texture(KIND_TEXTURES.kind_first_slots[211].bits());
        crate::scene::depth_test_off::depth_test_off();
        let settings: u32 = crate::marks::setup::SETTINGS.pointer.get();
        // dword_939BA6: the ninth of `lives_by_level`.
        SURFACE.lives_by_level[8]
            .set(((settings + 0x7C) as *const u32).read_unaligned() & 0xFF_FFFF);
        KIND1_CHOICE
            .choice
            .set(crate::marks::rain::RAIN.drops.get());
        crate::marks::run_ring::run_ring()?;
        set_render_state(2, 0x14);
        crate::marks::run_ribbons::run_ribbons()?;
    }
    Ok(())
}
