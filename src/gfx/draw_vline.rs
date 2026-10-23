//! `gfx::draw_vline` (0x4067AF): ebx pixels from (ecx, edx) downwards in colour eax, clipped (see `gfx/rect.rs`).
//!
//! In:  ecx, edx = the start, ebx = the length, eax = the colour, esi = the palette
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Callers: `gfx::frame_rect` (two). Shadow mode: by state (the surface's pixels it may touch are
//! not compared: the work variables are). Self-test: a test surface at 16 and 32 bits,
//! clips and lines in, across and out of the clip.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::gfx::rect::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { draw_vline(r.ecx as i32, r.edx as i32, r.ebx as i32, r.eax, r.esi) };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
