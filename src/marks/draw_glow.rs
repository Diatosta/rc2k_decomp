//! `marks::draw_glow` (0x4D2DED): unless `dword_93CC70` is 1, render states 0x15 = 4, 0x13 =
//! 5, 0x14 = 6, 0x1C = 0, 0x1B = 1 (`scene::set_render_state`), `scene::set_texture` with
//! `dword_5D1E34`, and `marks::draw_turned_sprite` at (`flt_93CC7C` + `flt_93CCA8`,
//! `flt_93CC78` + `flt_93CCAC`, `flt_93CC74`) (st(0) first, the sums unrounded), angle
//! `dword_93CCA4`, colour -1, size 0x10000. A glow over the scene, by the look of it (a
//! reading). (Its first lines, a developer's key for the switch, are jumped over.)
//!
//! In:  nothing
//! Out: nothing (no caller reads a register it leaves)
//!
//! Caller: `marks::run_frame` (0x4D1693).

use crate::errors::Refused;
use crate::framework::rva;
use crate::marks::clip::GLOW;
use crate::marks::store::game_read;
use crate::math::x87;
use crate::scene::matrix3::extended;

const TEXTURE_RVA: usize = rva(0x5D1E34); // dword_5D1E34
/// The render states set (value, state).
const STATES: [(u32, u32); 5] = [(4, 0x15), (5, 0x13), (6, 0x14), (0, 0x1C), (1, 0x1B)];

/// The Rust version (see the header): the render states (`scene::set_render_state`), the
/// texture (`scene::set_texture`), `draw_turned_sprite`.
pub unsafe fn draw_glow() -> Result<(), Refused> {
    unsafe {
        if GLOW.off.get() == 1 {
            return Ok(());
        }
        for (value, state) in STATES {
            crate::scene::set_render_state::set_render_state(value, state);
        }
        crate::scene::set_texture::set_texture(game_read(TEXTURE_RVA));
        let place = [
            x87::add_80(GLOW.place[2].bits(), GLOW.offset[0].bits()),
            x87::add_80(GLOW.place[1].bits(), GLOW.offset[1].bits()),
            extended(GLOW.place[0].bits()),
        ];
        crate::marks::draw_turned_sprite::draw_turned_sprite(
            GLOW.angle.get(),
            0xFFFF_FFFF,
            0x1_0000,
            place,
        )?;
    }
    Ok(())
}
