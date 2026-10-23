//! `hud::draw_marker` (0x4B55AB): a marker sprite (rectangle 0x5C of `dword_75B940`) along
//! the top at the share +0x6C over +0x68 of the HUD's record, times `flt_75C0A8` plus
//! `flt_75C0AC` (`fistp` through `dword_75C060`), coloured `dword_75BF50`.
//!
//! In:  ebp the HUD's record
//! Out: eax 0x5C0, ebx the x, ecx 0, esi the rectangle (as `add_sprite` keeps them)
//!
//! Caller: layout 0's drawer (0x4B1F0C). The sprite through `part` (1), in the game
//! `add_sprite`'s Rust version. Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose call goes to a stand-in, against the Rust version, shares by
//! lot, at both test control words.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::hud::add_quad::QUAD_RVA;
use rc2k_core::hud::quads::marker_x;

const COLOUR_RVA: usize = rva(0x75BF50); // dword_75BF50
const K_RVA: [usize; 2] = [rva(0x75C0A8), rva(0x75C0AC)];
const SCRATCH_RVA: usize = rva(0x75C060); // dword_75C060
pub const RECTS_RVA: usize = rva(0x75B940); // dword_75B940: 0x10 a rectangle
const RECT: u32 = 0x5C;

/// A call by place: 1 `add_sprite`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::hud::quads::marker_x`
pub unsafe fn draw_marker(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        game_write(QUAD_RVA + 0x14, game_read::<u32>(COLOUR_RVA));
        let get = |o: u32| ((g.ebp + o) as *const u32).read_unaligned();
        let f = f32::from_bits;
        let x = marker_x(
            f(get(0x6C)),
            f(get(0x68)),
            f(game_read(K_RVA[0])),
            f(game_read(K_RVA[1])),
        );
        game_write(SCRATCH_RVA, x);
        g.eax = RECT * 0x10;
        g.ebx = x;
        g.ecx = 0;
        g.esi = game_addr(RECTS_RVA) as u32 + g.eax;
        part(1, &mut g)?;
    }
    Ok(g)
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { draw_marker(r, &mut crate::hud::draw_marker::game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}

/// In the game: `add_sprite`'s Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::hud::add_sprite as s;
    *r = unsafe { s::add_sprite(r) }?;
    Ok(())
}
