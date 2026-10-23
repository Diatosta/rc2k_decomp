//! `physics::count_ground_probe` (0x438E22): one ground probe, counted: the face mask
//! `dword_620470` 0x40, `dword_608C3C` up by one, `physics::find_ground`; when found
//! `dword_608C40` up by one, and when the face is the last one again (`dword_5CCC78` =
//! `dword_5CCC74`) `dword_608C44` too. A reading: the camera's (or a test drive's) ground
//! statistics.
//!
//! In:  the point (`dword_639824`, `dword_63982C`; the callers set it)
//! Out: as `find_ground` leaves them, eax `dword_5CCC78` when found; no flags read (the
//!      callers compare `g_groundFound`)
//!
//! Callers: `sub_438BF4`, `sub_438CEC` (three).
//! Refuses what `find_ground` refuses. Shadow mode runs the original alone
//! (`physics/reset_parts.rs`).
//! Self-test: a copy whose call goes to a stand-in (place 18), against the Rust version
//! with the same: found or not, the same face or not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::physics::reset_parts::{FIND_GROUND, reset_entry};
use crate::race::car_starts::Part;
use rc2k_core::physics::walls::ProbeCounts;

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

/// The Rust version, `find_ground` through `part` (place 18).
// Converted: `rc2k_core::physics::walls::ProbeCounts`.
pub unsafe fn count_ground_probe(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let mut counts = ProbeCounts {
        tries: g(0x608C3C),
        found: g(0x608C40),
        same: g(0x608C44),
    };
    p(0x620470, 0x40);
    counts.tried();
    p(0x608C3C, counts.tries);
    part(FIND_GROUND, &mut r)?;
    let found = g(0x639830) != 0;
    if found {
        // the original's `mov eax, dword_5CCC78` for its compare
        r.eax = g(0x5CCC78);
        counts.searched(true, r.eax, g(0x5CCC74));
        p(0x608C40, counts.found);
        p(0x608C44, counts.same);
    }
    Ok(r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    reset_entry(r, count_ground_probe)
}
