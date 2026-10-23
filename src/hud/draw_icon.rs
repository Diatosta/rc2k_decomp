//! `hud::draw_icon` (0x4B587E): a square icon of the HUD record's entry at ebp + edi
//! (none when its +0x84 is -1): its width (+0x88) and drop (+0x90) times `dword_75C06C`
//! (`fistp` into `dword_75C064` / `dword_75C068`), centred on 0x320 / 2 plus
//! `dword_75C070`, its top ebx plus the drop, as tall as wide; texture places 0 / 1;
//! colour white under its +0x8C; `hud::add_polygon` on four (inside `pusha` .. `popa`).
//!
//! In:  ebp the record, edi the entry's offset, ebx the top
//! Out: ebx the bottom, edx the colour, esi the left, edi the right (kept when none)
//!
//! Callers: `hud::icons_0`, `hud::icons_1`, `hud::icons_2`. The polygon through `part`
//! (1), in the game its Rust version. Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose call goes to a stand-in, against the Rust version, entries by
//! lot, at both test control words.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::hud::batch::{POLYGON_COUNT_RVA, POLYGON_RVA};
use rc2k_core::hud::icons;

pub const SIZE_RVA: [usize; 2] = [rva(0x75C064), rva(0x75C068)]; // dword_75C064, dword_75C068
pub const SCALE_RVA: usize = rva(0x75C06C); // dword_75C06C
pub const SHIFT_RVA: usize = rva(0x75C070); // dword_75C070
const SCRATCH_RVA: usize = rva(0x75C060); // dword_75C060

/// A call by place: 1 `add_polygon`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::hud::icons`
pub unsafe fn draw_icon(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let e = g.ebp.wrapping_add(g.edi);
        let get = |o: u32| ((e + o) as *const u32).read_unaligned();
        if !icons::shown(get(0x84)) {
            return Ok(g);
        }
        let scale = f32::from_bits(game_read(SCALE_RVA));
        let width = get(0x88);
        game_write(SIZE_RVA[0], icons::scaled(width, scale));
        let drop = get(0x90);
        game_write(SIZE_RVA[1], icons::scaled(drop, scale));
        let i = icons::icon(width, get(0x8C), drop, scale, game_read(SHIFT_RVA), g.ebx);
        [g.ebx, g.edx, g.esi, g.edi] = [i.bottom, i.colour, i.left, i.right];
        for (n, u, v, x, y) in icons::corners(&i) {
            let at = POLYGON_RVA + 0x20 * n;
            game_write(at + 0x18, [u, v].map(|k| (k as i32 as f32).to_bits()));
            game_write(at, [x, y].map(|k| (k as i32 as f32).to_bits()));
            game_write(at + 0x10, [i.colour, 0]);
            // `fild` through `dword_75C060`: it keeps the last.
            game_write(SCRATCH_RVA, y);
        }
        game_write(POLYGON_COUNT_RVA, 4u32);
        let mut r = g;
        part(1, &mut r)?;
    }
    Ok(g)
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { draw_icon(r, &mut crate::hud::add_quad::game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
