//! `effects::ring_frame` (0x4CB5B9): the emitters' ring drawn: render states 0x15 = 4, 0x13 =
//! 5, 0x14 = 6, 0x1C = 1, 0x1B = 1, `scene::set_texture` (`dword_5D1E34`),
//! `scene::depth_test_off`; `dword_939BA6` the settings' +0x7C's low 24 bits,
//! `g_marksKind1Choice` `dword_8BC50D` (the rain's drops); `effects::run_ring`; render state
//! 0x14 = 2; `effects::run_ribbons`.
//!
//! Screen: the dust and the water splashes (owner, 2026-10-29: both gone with the ring's sprites cut).
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (no caller reads a register it leaves)
//!
//! Caller: `effects::run_frame` (0x4D160B). Refuses null settings, before anything.

use crate::effects::emit_kind_1::KIND1_CHOICE;
use crate::effects::surface_store::SURFACE;
use crate::errors::Refused;
use crate::scene::kind_texture_store::KIND_TEXTURES;
/// The render states set (value, state).
const STATES: [(u32, u32); 5] = [(4, 0x15), (5, 0x13), (6, 0x14), (1, 0x1C), (1, 0x1B)];

/// The Rust version (see the header): the render states (`scene::set_render_state`), the
/// texture (`scene::set_texture`), `scene::depth_test_off`, `run_ring`, `run_ribbons`.
pub fn ring_frame() -> Result<(), Refused> {
    use crate::scene::set_render_state::set_render_state;
    {
        let marks = crate::effects::setup::marks()?;
        for (value, state) in STATES {
            set_render_state(value, state);
        }
        crate::scene::set_texture::set_texture(KIND_TEXTURES.kind_first_slots[211].bits());
        crate::scene::depth_test_off::depth_test_off();
        // dword_939BA6: the ninth of `lives_by_level`.
        // The settings' +0x7C: the first car record's +0x1C.
        SURFACE.lives_by_level[8].set(marks.cars[0].word(0x1C)? & 0xFF_FFFF);
        KIND1_CHOICE
            .choice
            .set(crate::effects::rain::RAIN.drops.get());
        crate::effects::run_ring::run_ring()?;
        set_render_state(2, 0x14);
        crate::effects::run_ribbons::run_ribbons()?;
    }
    Ok(())
}
