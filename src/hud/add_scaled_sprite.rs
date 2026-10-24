//! `hud::add_scaled_sprite` (0x4B716A): as `hud::add_sprite` with the width times st(0)
//! and the height times st(1) (`fistp`ed through `dword_75C060`) before they are made
//! positive; st(0) and st(1) are read, not popped.
//!
//! In:  esi the rectangle, ebx x, ecx y, st(0) / st(1) the scales (kept)
//! Out: all kept (`pusha` .. `popa`)
//!
//! Callers: the HUD's drawers (code at 0x4B28B0, ...).

use crate::errors::Refused;
use crate::hud::add_sprite::{put_corners, put_places};
use crate::hud::store::Slot;
use crate::hud::store::game_write;
use rc2k_core::hud::quads;
use rc2k_core::math::extended::to_f64;

const SCRATCH_RVA: Slot = Slot::at(0x75C060); // dword_75C060

/// The Rust version (see the header), the scales as `f64`s (what the x87 holds at the
/// game's double precision): the sprite at `rect` with its corner at `x`, `y` (ebx, ecx,
/// esi; every register kept).
// Converted: `rc2k_core::hud::quads::{sprite_places, scaled_size, sprite_corners}`
pub unsafe fn add_scaled_sprite_plain(
    x: u32,
    y: u32,
    rect: u32,
    scales: [f64; 2],
) -> Result<(), Refused> {
    unsafe {
        let [_, _, w, h] = put_places(rect);
        let w = quads::scaled_size(w, scales[0]);
        let h = quads::scaled_size(h, scales[1]);
        game_write(SCRATCH_RVA, h);
        put_corners(x, y, w, h);
        crate::hud::add_quad::add_quad()
    }
}

/// `add_scaled_sprite_plain` with the scales as the x87 holds them.
pub unsafe fn add_scaled_sprite(
    x: u32,
    y: u32,
    rect: u32,
    scales: [[u8; 10]; 2],
) -> Result<(), Refused> {
    unsafe { add_scaled_sprite_plain(x, y, rect, scales.map(to_f64)) }
}
