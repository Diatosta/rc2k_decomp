//! `marks::make_fade_colours` (0x4C37EF): 0x11 colours from `dword_6415E4` toward white:
//! each component's inverse (0xFF less it) times a step that grows from 0 by
//! `flt_7651C4` (`dword_8DB4A8`, a float), rounded, made a pixel (alpha 0xFF) and added to
//! the colour (as one dword, carries crossing components), into `dword_8DB464`.
//!
//! In:  nothing
//! Out: eax the last entry, ebx, ecx the green and blue inverses, esi past the entries
//!
//! One caller, `sub_4C25F6`. Self-test: the original and the Rust version on several
//! colours and steps, at the test precisions; put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::math::x87;

const COLOUR_RVA: usize = rva(0x6415E4); // dword_6415E4
const FADES_RVA: usize = rva(0x8DB464); // g_marksFadeColours: 0x11 dwords
const FADES: u32 = 0x11;
const STEP_RVA: usize = rva(0x8DB4A8); // dword_8DB4A8 (a float)
const INVERSES_RVA: usize = rva(0x8DB4AC); // three dwords
const SCALED_RVA: usize = rva(0x8DB4B8); // three dwords
// the entries to the scaled three
const GROWTH_RVA: usize = rva(0x7651C4); // flt_7651C4

/// The Rust version (see the header).
pub unsafe fn make_fade_colours(entry: &Regs) -> Regs {
    let mut g = *entry;
    unsafe {
        let colour: u32 = game_read(COLOUR_RVA);
        g.eax = ((colour >> 16) & 0xFF) ^ 0xFF;
        g.ebx = ((colour >> 8) & 0xFF) ^ 0xFF;
        g.ecx = (colour & 0xFF) ^ 0xFF;
        game_write(INVERSES_RVA, [g.eax, g.ebx, g.ecx]);
        game_write(STEP_RVA, 0u32);
        g.esi = game_addr(FADES_RVA) as u32;
        for _ in 0..FADES {
            let step: u32 = game_read(STEP_RVA);
            let inverses: [i32; 3] = game_read(INVERSES_RVA);
            let scaled = inverses.map(|v| x87::int_times_to_int(v, step));
            game_write(SCALED_RVA, scaled);
            let [r, gr, b] = scaled.map(|v| v as u32 & 0xFF);
            g.eax = (0xFF00_0000 | r << 16 | gr << 8 | b).wrapping_add(game_read(COLOUR_RVA));
            (g.esi as *mut u32).write_unaligned(g.eax);
            g.esi += 4;
            game_write(STEP_RVA, x87::add(step, game_read(GROWTH_RVA)));
        }
    }
    g
}
