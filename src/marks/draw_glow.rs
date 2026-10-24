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
//! Caller: `marks::run_frame` (0x4D1693). Its calls through `part` (1 a render state, 2
//! the texture, 3 the sprite with its three values); in the game their Rust versions. Its
//! first instruction is a short jump: private copy (seven calls relocated), manual
//! patch. Shadow mode runs the original alone (it draws). Self-test: a copy whose calls go
//! to stand-ins (the sprite's taking its values off), against the Rust version on the
//! same, the switch on and off.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::rva;
use crate::marks::store::{Slot, game_read};
use crate::math::x87;
use crate::scene::matrix3::extended;

const OFF_RVA: Slot = Slot::at(0x93CC70); // dword_93CC70: 1 = none
const TEXTURE_RVA: usize = rva(0x5D1E34); // dword_5D1E34
/// flt_93CC74 .. flt_93CC7C, flt_93CCA8, flt_93CCAC, dword_93CCA4.
const PLACE: [Slot; 3] = [Slot::at(0x93CC74), Slot::at(0x93CC78), Slot::at(0x93CC7C)];
const OFFSET: [Slot; 2] = [Slot::at(0x93CCA8), Slot::at(0x93CCAC)];
const ANGLE_RVA: Slot = Slot::at(0x93CCA4);
/// The render states set (value, state).
const STATES: [(u32, u32); 5] = [(4, 0x15), (5, 0x13), (6, 0x14), (0, 0x1C), (1, 0x1B)];

/// The Rust version (see the header): the render states (`scene::set_render_state`), the
/// texture (`scene::set_texture`), `draw_turned_sprite`.
pub unsafe fn draw_glow() -> Result<(), Refused> {
    unsafe {
        if game_read::<u32>(OFF_RVA) == 1 {
            return Ok(());
        }
        for (value, state) in STATES {
            crate::scene::set_render_state::set_render_state(value, state);
        }
        crate::scene::set_texture::set_texture(game_read(TEXTURE_RVA));
        let place = [
            x87::add_80(game_read(PLACE[2]), game_read(OFFSET[0])),
            x87::add_80(game_read(PLACE[1]), game_read(OFFSET[1])),
            extended(game_read(PLACE[0])),
        ];
        crate::marks::draw_turned_sprite::draw_turned_sprite(
            game_read(ANGLE_RVA),
            0xFFFF_FFFF,
            0x1_0000,
            place,
        )?;
    }
    Ok(())
}
