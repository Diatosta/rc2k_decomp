//! `marks::set_quad_uvs` (0x4CFD6A): writing to the depth buffer off, and the four texture
//! places of the 0x20-byte set at `dword_920738` + 0xC60 put into the four vertices'
//! texture places at `flt_9605B8` (0x20 bytes apart: +0x18 of Direct3D vertices from
//! 0x9605A0, a reading).
//!
//! In:  nothing
//! Out: eax 0xC60, ebx 0xE, ebp and edi the last pair; the rest kept
//!
//! One caller, `sub_4CFB49` (the marks' frame). Self-test: the original and the Rust
//! version recorded on the layer (nothing drawn), the set filled by a pattern; put back
//! afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

const SETS_RVA: usize = rva(0x920738); // dword_920738: sets of four texture places
const SET: u32 = 0xC60;
const VERTICES_RVA: usize = rva(0x9605B8); // flt_9605B8: the first vertex's u, v
const VERTEX: usize = 0x20;

/// The Rust version (see the header).
pub unsafe fn set_quad_uvs(entry: &Regs) -> Regs {
    let mut g = *entry;
    unsafe {
        crate::scene::depth_test_off::depth_test_off(&mut g);
        g.eax = SET;
        for n in 0..4 {
            let from = SETS_RVA + SET as usize + 8 * n;
            g.ebp = game_read(from);
            g.edi = game_read(from + 4);
            game_write(VERTICES_RVA + VERTEX * n, [g.ebp, g.edi]);
        }
    }
    g
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { set_quad_uvs(r) }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
