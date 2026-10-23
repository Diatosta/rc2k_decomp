//! `race::place_car_on_line` (0x44666F): the work car set between checkpoint
//! `dword_6365E4` and the next, `dword_6365EC` / 65536 of the way: their faces' first
//! points blended into `dword_5CCDA0` / `C0` (+ `dword_613FC4`) / `E0`, their headings
//! (`dword_9ED808`) into `dword_5CCD60`, then the car's x, z, heading and y (+0x134,
//! +0x13C, +0x9C0, +0x138); the first face's number in `dword_5CCD28`. Both checkpoints
//! held below `g_checkpointCount` (unsigned).
//!
//! In:  nothing
//! Out: eax = y, ecx = z, edx = the heading product's high half, ebx and esi = the second
//!      point's number and address, ebp = the first point's address; edi preserved
//!
//! See `race/start_place.rs`. Callers: `sub_4464AD` (race init's camera start) and four
//! more (0x429479, 0x437687, 0x43F0CA, 0x43FB1E). Refuses no checkpoints (the original
//! reads entry -1) or more than the lists hold. Shadow mode: the Rust version, the
//! variables put back, the original's, compared. Self-test: faces, points and checkpoints
//! of the test's (`TestLine`), headings of the test's in the game's table, places and
//! fractions inside and past the line; the refusal on the Rust version alone; all put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::start_place::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { place_car_on_line(r) }?;
    Ok(left_changed(r, &g))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
