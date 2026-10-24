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
//! Callers: `sub_438BF4`, `sub_438CEC` (three). Refuses what `find_ground` refuses.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{game_read, game_write, rva};
use rc2k_core::physics::walls::ProbeCounts;

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

/// The Rust version. The eax it leaves (`dword_5CCC78` when found) is read by none of its
/// callers.
// Converted: `rc2k_core::physics::walls::ProbeCounts`.
pub unsafe fn count_ground_probe() -> Result<(), Refused> {
    let mut counts = ProbeCounts {
        tries: g(0x608C3C),
        found: g(0x608C40),
        same: g(0x608C44),
    };
    p(0x620470, 0x40);
    counts.tried();
    p(0x608C3C, counts.tries);
    unsafe { crate::physics::find_ground::find_ground() }?;
    if g(0x639830) != 0 {
        counts.searched(true, g(0x5CCC78), g(0x5CCC74));
        p(0x608C40, counts.found);
        p(0x608C44, counts.same);
    }
    Ok(())
}
