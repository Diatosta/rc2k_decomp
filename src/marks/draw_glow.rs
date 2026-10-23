//! `marks::draw_glow` (0x4D2DED): unless `dword_93CC70` is 1, render states 0x15 = 4, 0x13 =
//! 5, 0x14 = 6, 0x1C = 0, 0x1B = 1 (`scene::set_render_state`), `scene::set_texture` with
//! `dword_5D1E34`, and `marks::draw_turned_sprite` at (`flt_93CC7C` + `flt_93CCA8`,
//! `flt_93CC78` + `flt_93CCAC`, `flt_93CC74`) (st(0) first, the sums unrounded), angle
//! `dword_93CCA4`, colour -1, size 0x10000. A glow over the scene, by the look of it (a
//! reading). (Its first lines, a developer's key for the switch, are jumped over.)
//!
//! In:  nothing
//! Out: the registers as the sprite leaves them (eax `dword_5D1E34` going in)
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
use crate::framework::{Regs, game_read, rva};
use crate::math::x87;
use crate::scene::matrix3::extended;

const OFF_RVA: usize = rva(0x93CC70); // dword_93CC70: 1 = none
const TEXTURE_RVA: usize = rva(0x5D1E34); // dword_5D1E34
/// flt_93CC74 .. flt_93CC7C, flt_93CCA8, flt_93CCAC, dword_93CCA4.
const PLACE: [usize; 3] = [rva(0x93CC74), rva(0x93CC78), rva(0x93CC7C)];
const OFFSET: [usize; 2] = [rva(0x93CCA8), rva(0x93CCAC)];
const ANGLE_RVA: usize = rva(0x93CCA4);
/// The render states set (value, state).
const STATES: [(u32, u32); 5] = [(4, 0x15), (5, 0x13), (6, 0x14), (0, 0x1C), (1, 0x1B)];

/// A call by place (see the header), with the x87 values it takes (st(0) first).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs, &[[u8; 10]]) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn draw_glow(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        if game_read::<u32>(OFF_RVA) == 1 {
            return Ok(g);
        }
        for (value, state) in STATES {
            g.eax = value;
            g.ebx = state;
            part(1, &mut g, &[])?;
        }
        g.eax = game_read(TEXTURE_RVA);
        part(2, &mut g, &[])?;
        let place = [
            x87::add_80(game_read(PLACE[2]), game_read(OFFSET[0])),
            x87::add_80(game_read(PLACE[1]), game_read(OFFSET[1])),
            extended(game_read(PLACE[0])),
        ];
        g.ebx = game_read(ANGLE_RVA);
        g.ecx = 0xFFFF_FFFF;
        g.edx = 0x1_0000;
        part(3, &mut g, &place)?;
    }
    Ok(g)
}

/// In the game: the calls' Rust versions.
pub fn game_part(place: u32, r: &mut Regs, v: &[[u8; 10]]) -> Result<(), Refused> {
    unsafe {
        match place {
            1 => {
                let (value, state) = (r.eax, r.ebx);
                crate::scene::set_render_state::set_render_state(r, value, state);
            }
            2 => crate::scene::set_texture::set_texture_in_game(r),
            _ => {
                use crate::marks::draw_turned_sprite as sprite;
                let left = sprite::draw_turned_sprite(r, [v[0], v[1], v[2]])?;
                *r = Regs { esp: r.esp, ..left };
            }
        }
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { draw_glow(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
