//! `hud::add_triangle` (0x4B6DA7): `g_polygonVertexCount` 3 and `hud::add_polygon`.
//!
//! In:  nothing
//! Out: all kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4B6D10` (0x4B6D03). The polygon through `part` (1), in the game its Rust
//! version. Shadow mode runs the original alone (it draws). Self-test: a copy whose call
//! goes to a stand-in, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_write};
use crate::hud::batch::POLYGON_COUNT_RVA;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn add_triangle(entry: &Regs) -> Result<Regs, Refused> {
    unsafe { game_write(POLYGON_COUNT_RVA, 3u32) };
    let mut r = *entry;
    crate::hud::add_quad::game_part(1, &mut r)?;
    Ok(*entry)
}

/// In the game, for the quads' callers: `add_quad`'s Rust version.
pub fn game_part_quad(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::hud::add_quad as aq;
    *r = unsafe { aq::add_quad(r) }?;
    Ok(())
}

/// In the game, for `draw_disc`: this function's Rust version.
pub fn game_part_triangle(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    *r = unsafe { add_triangle(r) }?;
    Ok(())
}

/// In the game, for the dials' set-ups: `draw_disc`'s Rust version.
pub fn game_part_disc(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    *r = unsafe { crate::hud::draw_disc::draw_disc(r) }?;
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { add_triangle(r) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
