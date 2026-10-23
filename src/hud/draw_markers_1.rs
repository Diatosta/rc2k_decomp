//! `hud::draw_markers_1` (0x4B56E3): the two players' markers along the top in a split
//! layout: rectangle 0x5C of `dword_75B940` at the record's +0x6C (then +0x70) over +0x68
//! times `flt_75C0A8` plus `flt_75C0AC` (`fistp` through `dword_75C060`), coloured
//! `dword_75BF50` (then `dword_75BF54`), at rows 1 and 0xE, scaled 0x100 by 0x78 over
//! `flt_75C00C` (`hud::add_scaled_sprite`, the scales taken off after).
//!
//! In:  ebp the HUD's record
//! Out: eax 0x5C0, ebx the second's x, ecx its row, esi the rectangle
//!
//! Caller: layout 1's drawer (0x4B2BF4); `hud::draw_markers_2` is its twin (rows 0x11F,
//! 0x12C). The sprites through `part` (1 with the two scales), in the game
//! `add_scaled_sprite`'s Rust version. Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose calls go to a stand-in reading the scales, against the Rust
//! version, shares by lot, at both test control words.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::hud::add_quad::QUAD_RVA;
use crate::hud::draw_marker::RECTS_RVA;
use rc2k_core::hud::quads;
use rc2k_core::math::extended::{from_f64, to_f64};

const COLOURS_RVA: [usize; 2] = [rva(0x75BF50), rva(0x75BF54)];
const K_RVA: [usize; 2] = [rva(0x75C0A8), rva(0x75C0AC)];
const SCRATCH_RVA: usize = rva(0x75C060); // dword_75C060
const UV_SCALE_RVA: usize = rva(0x75C00C); // flt_75C00C
const RECT: u32 = 0x5C;

/// A call by place: 1 `add_scaled_sprite`, with its two scales (st(0) first).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs, &[[u8; 10]]) -> Result<(), Refused>;

/// `fild i; fdiv k`, kept extended (`quads::int_over`: exact at the game's double
/// precision).
pub fn int_over_80(i: u32, k: u32) -> [u8; 10] {
    from_f64(quads::int_over(i, f32::from_bits(k)))
}

/// An 80-bit value taken off into a float (`fstp dword`).
pub fn taken_off(v: [u8; 10]) -> u32 {
    (to_f64(v) as f32).to_bits()
}

/// The two markers at rows `rows` (see the header).
// Converted: `rc2k_core::hud::quads::{marker_x, int_over}`
pub unsafe fn markers_with(entry: &Regs, rows: [u32; 2], part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let get = |o: u32| ((entry.ebp + o) as *const u32).read_unaligned();
        let k: u32 = game_read(UV_SCALE_RVA);
        for (n, field) in [0x6Cu32, 0x70].into_iter().enumerate() {
            game_write(QUAD_RVA + 0x14, game_read::<u32>(COLOURS_RVA[n]));
            let f = f32::from_bits;
            let x = quads::marker_x(
                f(get(field)),
                f(get(0x68)),
                f(game_read(K_RVA[0])),
                f(game_read(K_RVA[1])),
            );
            game_write(SCRATCH_RVA, x);
            g.eax = RECT * 0x10;
            g.ebx = x;
            g.ecx = rows[n];
            g.esi = game_addr(RECTS_RVA) as u32 + g.eax;
            game_write(SCRATCH_RVA, 0x78u32);
            let st1 = int_over_80(0x78, k);
            game_write(SCRATCH_RVA, 0x100u32);
            let st0 = int_over_80(0x100, k);
            part(1, &mut g, &[st0, st1])?;
            // the two scales taken off into dword_75C060 (st(0), then st(1))
            game_write(SCRATCH_RVA, taken_off(st1));
        }
    }
    Ok(g)
}

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::hud::quads::{marker_x, int_over}` (through `markers_with`)
pub unsafe fn draw_markers_1(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    unsafe { markers_with(entry, [1, 0xE], part) }
}

/// In the game: `add_scaled_sprite`'s Rust version.
pub fn game_part(_place: u32, r: &mut Regs, v: &[[u8; 10]]) -> Result<(), Refused> {
    use crate::hud::add_scaled_sprite as s;
    *r = unsafe {
        s::add_scaled_sprite(
            r,
            [v[0], v[1]],
            &mut crate::hud::add_triangle::game_part_quad,
        )
    }?;
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { draw_markers_1(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
