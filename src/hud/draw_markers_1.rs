//! `hud::draw_markers_1` (0x4B56E3): the two players' markers along the top in a split
//! layout: rectangle 0x5C of `dword_75B940` at the record's +0x6C (then +0x70) over +0x68
//! times `flt_75C0A8` plus `flt_75C0AC` (`fistp` through `dword_75C060`), coloured
//! `dword_75BF50` (then `dword_75BF54`), at rows 1 and 0xE, scaled 0x100 by 0x78 over
//! `flt_75C00C` (`hud::add_scaled_sprite`, the scales taken off after).
//!
//! In:  ebp the HUD's record
//! Out: eax 0x5C0, ebx the second's x, ecx its row, esi the rectangle
//!
//! Caller: layout 1's drawer (0x4B2BF4); `hud::draw_markers_2` is its twin (rows 0x11F, 0x12C).

use crate::errors::Refused;
use crate::hud::store::{HUD, rect_addr};
use crate::pointers::GamePtr;
use rc2k_core::hud::quads;
use rc2k_core::math::extended::{from_f64, to_f64};

const RECT: u32 = 0x5C;

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
pub fn markers_with(record: GamePtr, rows: [u32; 2]) -> Result<(), Refused> {
    {
        let get = |o: u32| record.offset(o).read::<u32>("a hud record's field");
        let k = HUD.uv_scale_x.bits();
        for (n, field) in [0x6Cu32, 0x70].into_iter().enumerate() {
            HUD.quad[5].set(HUD.marker_colours[n].get());
            let f = f32::from_bits;
            let x = quads::marker_x(
                f(get(field)?),
                f(get(0x68)?),
                HUD.marker_k[0].get(),
                HUD.marker_k[1].get(),
            );
            HUD.scratch.set(x);
            let rect = rect_addr(RECT);
            HUD.scratch.set(0x78u32);
            let st1 = int_over_80(0x78, k);
            HUD.scratch.set(0x100u32);
            let st0 = int_over_80(0x100, k);
            crate::hud::add_scaled_sprite::add_scaled_sprite(x, rows[n], rect, [st0, st1])?;
            // the two scales taken off into dword_75C060 (st(0), then st(1))
            HUD.scratch.set(taken_off(st1));
        }
    }
    Ok(())
}

/// The Rust version (see the header).
// Converted: `rc2k_core::hud::quads::{marker_x, int_over}` (through `markers_with`)
pub fn draw_markers_1(record: GamePtr) -> Result<(), Refused> {
    markers_with(record, [1, 0xE])
}
